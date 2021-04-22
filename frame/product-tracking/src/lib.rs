// This file is part of Nuchain.
//
// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::alloc::string::ToString;
use core::convert::TryInto;
use frame_support::{
    debug, ensure,
    sp_runtime::offchain::{
        self as rt_offchain,
        storage::StorageValueRef,
        storage_lock::{StorageLock, Time},
    },
    sp_std::prelude::*,
    traits::EnsureOrigin,
};
use frame_system::{self, ensure_signed, offchain::SendTransactionTypes};
use pallet_did::Did;
use pallet_product_registry::{self as product_registry};
use product_registry::ProductId;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
use crate::types::*;

mod builders;
use crate::builders::*;

// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const IDENTIFIER_MAX_LENGTH: usize = 36;
pub const SHIPMENT_MAX_PRODUCTS: usize = 10;
pub const LISTENER_ENDPOINT: &str = "http://localhost:3005";
pub const LOCK_TIMEOUT_EXPIRATION: u64 = 3000; // in milli-seconds

pub type Year = u32;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_timestamp::Config
        + pallet_organization::Config
        + SendTransactionTypes<Call<Self>>
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::storage]
    #[pallet::getter(fn tracking)]
    pub type Tracking<T: Config> =
        StorageMap<_, Blake2_128Concat, TrackingId, Track<T::AccountId, T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn trackings_of_org)]
    pub type TrackingOfOrganization<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        Year,
        Vec<TrackingId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn event_count)]
    pub type EventCount<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn event_by_idx)]
    pub type AllEvents<T: Config> =
        StorageMap<_, Twox64Concat, TrackingEventIndex, TrackingEvent<T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn events_of_tracking)]
    pub type EventsOfTracking<T: Config> =
        StorageMap<_, Blake2_128Concat, TrackingId, Vec<TrackingEventIndex>>;

    #[pallet::storage]
    #[pallet::getter(fn ocw_notifications)]
    pub type OcwNotifications<T: Config> =
        StorageMap<_, Identity, T::BlockNumber, Vec<TrackingEventIndex>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TrackingRegistered(T::AccountId, TrackingId, T::AccountId),
        TrackingStatusUpdated(T::AccountId, TrackingId, TrackingEventIndex, TrackingStatus),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidOrMissingIdentifier,
        TrackingAlreadyExists,
        TrackingHasBeenDelivered,
        TrackingIsInTransit,
        TrackingIsUnknown,
        TrackingHasTooManyProducts,
        TrackingStatusNotChanged,
        TrackingEventAlreadyExists,
        TrackingEventMaxExceeded,
        OffchainWorkerAlreadyBusy,
        PermissionDenied,
        Overflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register product for tracking.
        ///
        /// The caller of this function must be _signed_.
        ///
        /// * `id` - Tracking ID.
        /// * `org_id` - ID of organization associated with the product.
        /// * `year` - Year of the product registered.
        /// * `products` - List of product IDs.
        #[pallet::weight(100_000)]
        pub fn register(
            origin: OriginFor<T>,
            id: TrackingId,
            org_id: T::AccountId,
            year: Year,
            products: Vec<ProductId>,
        ) -> DispatchResultWithPostInfo {
            // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate format of tracking ID
            Self::validate_identifier(&id)?;

            // Validate tracking products
            Self::validate_tracking_products(&products)?;

            // Check tracking doesn't exist yet (1 DB read)
            Self::validate_new_tracking(&id)?;

            // Pastikan origin memiliki akses organisasi
            <pallet_organization::Module<T>>::ensure_access_active_id(&who, &org_id)?;

            // Create a tracking instance
            let tracking = Self::new_tracking()
                .identified_by(id.clone())
                .owned_by(org_id.clone())
                .registered_at(<pallet_timestamp::Module<T>>::now())
                .with_products(products)
                .build();

            // Create shipping event
            let event = Self::new_tracking_event()
                .of_type(TrackingEventType::TrackingRegistration)
                .for_tracking(id.clone())
                .at_location(None)
                .with_readings(vec![])
                .at_time(tracking.registered)
                .build();

            // Storage writes
            // --------------
            // Add track (2 DB write)
            <Tracking<T>>::insert(&id, tracking);
            <TrackingOfOrganization<T>>::append(&org_id, year, &id);
            // Store shipping event (1 DB read, 3 DB writes)
            let event_idx = Self::store_event(event)?;
            // Update offchain notifications (1 DB write)
            <OcwNotifications<T>>::append(<frame_system::Module<T>>::block_number(), event_idx);

            // Raise events
            Self::deposit_event(Event::TrackingRegistered(who.clone(), id.clone(), org_id));

            Ok(().into())
        }

        /// Update tracking data.
        ///
        /// Dispatcher of this function must be _signed_ and match `T::CreateRoleOrigin`.
        ///
        #[pallet::weight(100_000)]
        pub fn update_status(
            origin: OriginFor<T>,
            id: TrackingId,
            status: TrackingStatus,
            #[pallet::compact] timestamp: T::Moment,
            location: Option<ReadPoint>,
            readings: Option<Vec<Reading<T::Moment>>>,
        ) -> DispatchResultWithPostInfo {
            // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate format of tracking ID
            Self::validate_identifier(&id)?;

            let mut track = <Tracking<T>>::get(&id).ok_or(Error::<T>::TrackingIsUnknown)?;

            ensure!(status != track.status, Error::<T>::TrackingStatusNotChanged);

            // Pastikan origin memiliki akses di organisasi (product owner)
            // atau origin memiliki akses sebagai ProductTracker
            ensure!(
                <pallet_organization::Module<T>>::ensure_access_active_id(&who, &track.owner)
                    .is_ok()
                    || <pallet_did::Module<T>>::valid_delegate(
                        &track.owner,
                        b"ProductTracker",
                        &who
                    )
                    .is_ok(),
                Error::<T>::PermissionDenied
            );

            // Create shipping event
            let event = Self::new_tracking_event()
                .of_type(TrackingEventType::TrackingUpdateStatus)
                .for_tracking(id.clone())
                .at_location(location)
                .with_readings(readings.unwrap_or_default())
                .at_time(timestamp)
                .with_status(status.clone())
                .build();

            // Storage writes
            // --------------
            // Store shipping event (1 DB read, 3 DB writes)
            let event_idx = Self::store_event(event)?;
            // Update offchain notifications (1 DB write)
            <OcwNotifications<T>>::append(<frame_system::Module<T>>::block_number(), event_idx);

            // Update tracking (1 DB write)
            track.status = status.clone();
            track.updated = Some(pallet_timestamp::Module::<T>::now());

            <Tracking<T>>::insert(&id, track);

            // Raise events
            Self::deposit_event(Event::TrackingStatusUpdated(who, id, event_idx, status));

            Ok(().into())
        }
    }

    // ----------------------------------------------------------------
    //                      HOOKS
    // ----------------------------------------------------------------
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // Acquiring the lock
            let mut lock = StorageLock::<Time>::with_deadline(
                b"product_tracking_ocw::lock",
                rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
            );

            match lock.try_lock() {
                Ok(_guard) => {
                    Self::process_ocw_notifications(block_number);
                }
                Err(_err) => {
                    debug::info!("[product_tracking_ocw] lock is already acquired");
                }
            };
        }
    }
}

pub use pallet::*;

#[cfg(not(feature = "std"))]
use codec::alloc::vec;

impl<T: Config> Pallet<T> {
    fn new_tracking() -> TrackingBuilder<T::AccountId, T::Moment> {
        TrackingBuilder::<T::AccountId, T::Moment>::default()
    }

    fn new_tracking_event() -> TrackingEventBuilder<T::Moment> {
        TrackingEventBuilder::<T::Moment>::default()
    }

    fn store_event(event: TrackingEvent<T::Moment>) -> Result<TrackingEventIndex, Error<T>> {
        let event_idx = <EventCount<T>>::get()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::<T>::TrackingEventMaxExceeded)?;

        <EventCount<T>>::put(event_idx);
        <EventsOfTracking<T>>::append(&event.tracking_id, event_idx);
        <AllEvents<T>>::insert(event_idx, event);

        Ok(event_idx)
    }

    // (Public) Validation methods
    pub fn validate_identifier(id: &[u8]) -> Result<(), Error<T>> {
        // Basic identifier validation
        ensure!(!id.is_empty(), Error::<T>::InvalidOrMissingIdentifier);
        ensure!(
            id.len() <= IDENTIFIER_MAX_LENGTH,
            Error::<T>::InvalidOrMissingIdentifier
        );
        Ok(())
    }

    pub fn validate_new_tracking(id: &[u8]) -> Result<(), Error<T>> {
        // tracking id length
        ensure!(
            id.len() <= IDENTIFIER_MAX_LENGTH,
            Error::<T>::InvalidOrMissingIdentifier
        );
        // Tracking existence check
        ensure!(
            !<Tracking<T>>::contains_key(id),
            Error::<T>::TrackingAlreadyExists
        );
        Ok(())
    }

    pub fn validate_tracking_products(props: &[ProductId]) -> Result<(), Error<T>> {
        ensure!(
            props.len() <= SHIPMENT_MAX_PRODUCTS,
            Error::<T>::TrackingHasTooManyProducts,
        );
        Ok(())
    }

    // --- Offchain worker methods ---

    fn process_ocw_notifications(block_number: T::BlockNumber) {
        // Check last processed block
        let last_processed_block_ref =
            StorageValueRef::persistent(b"product_tracking_ocw::last_proccessed_block");

        let mut last_processed_block: u32 = match last_processed_block_ref.get::<T::BlockNumber>() {
            Some(Some(_last_proccessed_block)) if _last_proccessed_block >= block_number => {
                debug::info!(
                    "[product_tracking_ocw] Skipping: Block {:?} has already been processed.",
                    block_number
                );
                return;
            }
            Some(Some(_last_proccessed_block)) => _last_proccessed_block
                .try_into()
                .ok()
                .expect("numeric value; qed"),
            None => 0u32, //TODO: define a OCW_MAX_BACKTRACK_PERIOD param
            // None => {
            //     last_processed_block = 0u32;
            // }
            _ => {
                debug::error!("[product_tracking_ocw] Error reading product_tracking_ocw::last_proccessed_block.");
                return;
            }
        };

        let start_block = last_processed_block + 1;
        let end_block: u32 = block_number.try_into().ok().expect("numeric value; qed");
        for current_block in start_block..end_block {
            debug::debug!(
                "[product_tracking_ocw] Processing notifications for block {}",
                current_block
            );
            if let Some(ev_indices) =
                Self::ocw_notifications::<T::BlockNumber>(current_block.into())
            {
                let listener_results: Result<Vec<_>, _> = ev_indices
                    .iter()
                    .map(|idx| match Self::event_by_idx(idx) {
                        Some(ev) => Self::notify_listener(&ev),
                        None => Ok(()),
                    })
                    .collect();

                if let Err(err) = listener_results {
                    debug::warn!("[product_tracking_ocw] notify_listener error: {}", err);
                    break;
                }
            }

            // @TODO(Robin): clean up OcwNotifications storage.

            last_processed_block = current_block;
        }

        // Save last processed block
        if last_processed_block >= start_block {
            last_processed_block_ref.set(&last_processed_block);
            debug::info!(
                "[product_tracking_ocw] Notifications successfully processed up to block {}",
                last_processed_block
            );
        }
    }

    fn notify_listener(ev: &TrackingEvent<T::Moment>) -> Result<(), &'static str> {
        debug::info!("notifying listener: {:?}", ev);

        let request =
            sp_runtime::offchain::http::Request::post(&LISTENER_ENDPOINT, vec![ev.to_string()]);

        let timeout =
            sp_io::offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(3000));

        let pending = request
            .add_header(&"Content-Type", &"text/plain")
            .deadline(timeout) // Setting the timeout time
            .send() // Sending the request out by the host
            .map_err(|_| "http post request building error")?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| "http post request sent error")?
            .map_err(|_| "http post request sent error")?;

        if response.code != 200 {
            return Err("http response error");
        }

        Ok(())
    }
}
