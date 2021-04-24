// This file is part of Nuchain.
//
// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! # Organization
//!
//! - [`Organization::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Organization pallet for Nuchain
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_org` - Create organization.
//! * `suspend_org` - Suspen organization.
//! * `add_member` - Add account as member to the organization.
//! * `remove_member` - Remove account member from organization.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get, OnUnbalanced,
        ReservableCurrency, WithdrawReasons,
    },
    types::Property,
};
use frame_system::ensure_signed;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::{Hash, StaticLookup};
use sp_std::{fmt::Debug, prelude::*};

use enumflags2::{bitflags, BitFlags};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode, EncodeLike};
use pallet_did::{self as did, Did};

pub const MAX_PROPS: usize = 5;
pub const PROP_NAME_MAX_LENGTH: usize = 10;
pub const PROP_VALUE_MAX_LENGTH: usize = 60;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config + did::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Creation fee.
        type CreationFee: Get<BalanceOf<Self>>;

        /// Payment for treasury
        type Payment: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// The origin which may forcibly set or remove a name. Root can always do this.
        type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// Min organization name length
        type MinOrgNameLength: Get<usize>;

        /// Max organization name length
        type MaxOrgNameLength: Get<usize>;

        /// Max number of member for the organization
        type MaxMemberCount: Get<usize>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct Organization<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Organization ID
        pub id: AccountId,

        /// object name
        pub name: Vec<u8>,

        /// Description about the organization.
        pub description: Vec<u8>,

        /// admin of the object
        pub admin: AccountId,

        /// Official website url
        pub website: Vec<u8>,

        /// Official email address
        pub email: Vec<u8>,

        /// Whether the organization suspended or not
        pub suspended: bool,

        /// Custom properties
        pub props: Option<Vec<Property>>,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The object already exsits
        AlreadyExists,

        /// Already set, no change have made
        AlreadySet,

        /// Name too long
        NameTooLong,

        /// Name too short
        NameTooShort,

        /// Description too short
        DescriptionTooShort,

        /// Object doesn't exist.
        NotExists,

        /// Origin has no authorization to do this operation
        PermissionDenied,

        /// ID already exists
        BadIndex,

        /// Cannot generate ID
        CannotGenId,

        /// Max member count reached
        MaxMemberReached,

        /// The organization is suspended
        Suspended,

        /// Too many properties in organization object.
        TooManyProps,

        /// Invalid properties name.
        InvalidPropName,

        /// Invalid properties value.
        InvalidPropValue,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
    pub enum Event<T: Config> {
        /// Some object added inside the system.
        ///
        /// 1: organization id (hash)
        /// 2: creator account id
        OrganizationAdded(T::AccountId, T::AccountId),

        /// When object deleted
        OrganizationDeleted(T::AccountId),

        /// Organization has been suspended.
        OrganizationSuspended(T::AccountId),

        /// Member added to an organization
        MemberAdded(T::AccountId, T::AccountId),

        /// Member removed from an organization
        MemberRemoved(T::AccountId, T::AccountId),

        /// Organization admin changed.
        AdminChanged(T::AccountId, T::AccountId),
    }

    /// Pair organization hash -> Organization data
    #[pallet::storage]
    #[pallet::getter(fn organization)]
    pub type Organizations<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Organization<T::AccountId>>;

    /// Link organization index -> organization hash.
    /// Useful for lookup organization from hash.
    #[pallet::storage]
    #[pallet::getter(fn organization_index)]
    pub type OrganizationIndexOf<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

    /// Pair user -> list of handled organizations
    #[pallet::storage]
    pub type OrganizationLink<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>>;

    /// Membership store, stored as an ordered Vec.
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AccountId>>;

    #[bitflags(default = Active)]
    #[repr(u64)]
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub enum FlagDataBit {
        Active = 0b0000000000000000000000000000000000000000000000000000000000000001,
        Verified = 0b0000000000000000000000000000000000000000000000000000000000000010,
        Government = 0b0000000000000000000000000000000000000000000000000000000000000100,
        System = 0b0000000000000000000000000000000000000000000000000000000000001000,
        Edu = 0b0000000000000000000000000000000000000000000000000000000000010000,
        Company = 0b0000000000000000000000000000000000000000000000000000000000100000,
        Foundation = 0b0000000000000000000000000000000000000000000000000000000001000000,
    }

    #[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
    pub struct FlagDataBits(pub BitFlags<FlagDataBit>);

    impl Eq for FlagDataBits {}
    impl Encode for FlagDataBits {
        fn using_encoded<R, F>(&self, f: F) -> R
        where
            F: FnOnce(&[u8]) -> R,
        {
            self.0.bits().using_encoded(f)
        }
    }
    impl Decode for FlagDataBits {
        fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
            let field = u64::decode(input)?;
            Ok(Self(
                BitFlags::<FlagDataBit>::from_bits(field as u64)
                    .map_err(|_| "invalid flag data value")?,
            ))
        }
    }
    impl EncodeLike for FlagDataBits {}
    impl core::ops::Deref for FlagDataBits {
        type Target = BitFlags<FlagDataBit>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl core::ops::DerefMut for FlagDataBits {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    /// Flag of the organization
    #[pallet::storage]
    #[pallet::getter(fn flags)]
    pub type OrganizationFlagData<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, FlagDataBits>;

    // pub struct EnsureOrgAdmin<T>(sp_std::marker::PhantomData<T>);

    // impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrgAdmin<T> {
    //     type Success = (T::AccountId, Vec<T::AccountId>);

    //     fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
    //         o.into().and_then(|o| match o {
    //             frame_system::RawOrigin::Signed(ref who) => {
    //                 let vs = OrganizationLink::<T>::get(who.clone())
    //                     .ok_or(T::Origin::from(o.clone()))?;
    //                 Ok((who.clone(), vs.clone()))
    //             }
    //             r => Err(T::Origin::from(r)),
    //         })
    //     }

    //     #[cfg(feature = "runtime-benchmarks")]
    //     fn successful_origin() -> T::Origin {
    //         O::from(RawOrigin::Signed(Default::default()))
    //     }
    // }

    #[pallet::storage]
    #[pallet::getter(fn object_index)]
    pub type OrgIdIndex<T> = StorageValue<_, u64>;

    /// Organization module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    {
        /// Add new Organization.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(100_000_000)]
        pub fn create(
            origin: OriginFor<T>,
            name: Vec<u8>,
            description: Vec<u8>,
            admin: <T::Lookup as StaticLookup>::Source,
            website: Vec<u8>,
            email: Vec<u8>,
            props: Option<Vec<Property>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;

            ensure!(
                name.len() >= T::MinOrgNameLength::get(),
                Error::<T>::NameTooShort
            );
            ensure!(
                name.len() <= T::MaxOrgNameLength::get(),
                Error::<T>::NameTooLong
            );

            Self::validate_props(&props)?;

            let index = Self::next_index()?;

            ensure!(
                !OrganizationIndexOf::<T>::contains_key(index),
                Error::<T>::BadIndex
            );

            let admin = T::Lookup::lookup(admin)?;

            // Process the payment
            let cost = T::CreationFee::get();

            // Process payment
            T::Payment::on_unbalanced(T::Currency::withdraw(
                &who,
                cost,
                WithdrawReasons::FEE,
                KeepAlive,
            )?);

            // generate organization id (hash)
            let org_id: T::AccountId = UncheckedFrom::unchecked_from(T::Hashing::hash(
                &index
                    .to_le_bytes()
                    .iter()
                    .chain(name.iter())
                    .chain(description.iter())
                    .chain(website.iter())
                    .chain(email.iter())
                    .cloned()
                    .collect::<Vec<u8>>(),
            ));

            Organizations::<T>::insert(
                org_id.clone(),
                Organization {
                    id: org_id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    admin: admin.clone(),
                    website: website.clone(),
                    email: email.clone(),
                    suspended: false,
                    props,
                },
            );

            <OrganizationIndexOf<T>>::insert(index, org_id.clone());

            if OrganizationLink::<T>::contains_key(&admin) {
                OrganizationLink::<T>::mutate(&admin, |ref mut vs| {
                    vs.as_mut().map(|vsi| vsi.push(org_id.clone()))
                });
            } else {
                OrganizationLink::<T>::insert(&admin, sp_std::vec![org_id.clone()]);
            }

            <OrganizationFlagData<T>>::insert::<_, FlagDataBits>(
                org_id.clone(),
                Default::default(),
            );

            // DID add attribute
            <pallet_did::Module<T>>::create_attribute(&org_id, &org_id, b"Org", &name, None)?;
            // Set owner of this organization in DID
            <pallet_did::Module<T>>::set_owner(&who, &org_id, &admin);

            Self::deposit_event(Event::OrganizationAdded(org_id, admin));

            Ok(().into())
        }

        /// Suspend organization
        ///
        /// The dispatch origin for this call must match `T::ForceOrigin`.
        #[pallet::weight(100_000)]
        pub fn suspend_org(
            origin: OriginFor<T>,
            org_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            ensure!(
                Organizations::<T>::contains_key(&org_id),
                Error::<T>::NotExists
            );

            Organizations::<T>::try_mutate(org_id, |org| {
                org.as_mut()
                    .map(|org| {
                        org.suspended = true;
                    })
                    .ok_or(Error::<T>::NotExists)
            })?;

            Ok(().into())
        }

        /// Set organization flags
        ///
        #[pallet::weight(100_000)]
        pub fn set_flags(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            flags: FlagDataBits,
        ) -> DispatchResultWithPostInfo {
            let origin_1 = ensure_signed(origin.clone())?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            if !(org.admin == origin_1
                || did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin_1).is_ok())
                || flags.contains(FlagDataBit::System)
                || flags.contains(FlagDataBit::Verified)
            {
                T::ForceOrigin::ensure_origin(origin)?;
            } else {
                ensure!(!org.suspended, Error::<T>::Suspended);
            }

            OrganizationFlagData::<T>::try_mutate(org_id, |v| -> Result<(), DispatchError> {
                *v = Some(flags);
                Ok(().into())
            })?;

            Ok(().into())
        }

        /// Add member to the organization.
        ///
        #[pallet::weight(100_000)]
        pub fn add_member(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let org = Self::ensure_access(&sender, &org_id)?;

            ensure!(!org.suspended, Error::<T>::Suspended);

            let mut members = <Members<T>>::get(&org_id).unwrap_or_else(|| vec![]);

            ensure!(
                members.len() < T::MaxMemberCount::get(),
                Error::<T>::MaxMemberReached
            );
            ensure!(
                !members.iter().any(|a| *a == account_id),
                Error::<T>::BadIndex
            );

            members.push(account_id.clone());
            members.sort();

            <Members<T>>::insert(&org_id, members);

            // <pallet_did::Module<T>>::create_delegate(&sender, &org.id, &account_id, b"OrgMember");

            Self::deposit_event(Event::MemberAdded(org_id, account_id));

            Ok(().into())
        }

        /// Remove member from organization.
        #[pallet::weight(100_000)]
        pub fn remove_member(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            // ensure!(org.admin == origin, Error::<T>::PermissionDenied);
            // did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin)?;
            Self::ensure_access(&origin, &org_id)?;

            ensure!(!org.suspended, Error::<T>::Suspended);

            let mut members = <Members<T>>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            ensure!(
                members.iter().any(|a| *a == account_id),
                Error::<T>::NotExists
            );

            members = members.into_iter().filter(|a| *a != account_id).collect();
            Members::<T>::insert(org_id.clone(), members);

            Self::deposit_event(Event::MemberRemoved(org_id, account_id));

            Ok(().into())
        }

        /// Change organization admin,
        /// the origin must be current admin or conform to `ForceOrigin`.
        #[pallet::weight(100_000)]
        pub(crate) fn set_admin(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin_1 = ensure_signed(origin.clone())?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            if org.admin != origin_1 {
                T::ForceOrigin::ensure_origin(origin)?;
            } else {
                ensure!(!org.suspended, Error::<T>::Suspended);
            }

            ensure!(org.admin != account_id, Error::<T>::AlreadySet);

            // did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin)?;

            // did::Module::<T>::revoke_delegate_internal(&org_id, b"OrgAdmin", &account_id);
            // did::Module::<T>::create_delegate(&org_id, &org_id, &account_id, b"OrgAdmin", None)?;

            <Organizations<T>>::mutate(&org_id, |org| {
                if let Some(org) = org {
                    org.admin = account_id.clone();
                }
            });

            Self::deposit_event(Event::AdminChanged(org_id, account_id));

            Ok(().into())
        }

        /// Delegate admin access to other.
        ///
        /// Use _did_ for share access with expiration.
        ///
        /// Only admin of organization can do this operation.
        ///
        #[pallet::weight(100_000)]
        pub(crate) fn delegate_access(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            to: T::AccountId,
            valid_for: Option<T::BlockNumber>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            // let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            // ensure!(!org.suspended, Error::<T>::Suspended);
            // ensure!(org.admin == origin, Error::<T>::PermissionDenied);

            // did::Module::<T>::create_delegate(&origin, &org_id, &to, b"OrgAdmin", valid_for)?;

            // Ok(().into())
            Self::h_delegate_access_as(&origin, &org_id, &to, b"OrgAdmin", valid_for)?;
            Ok(().into())
        }

        /// Delegate access to other account
        /// with custom type.
        #[pallet::weight(100_000)]
        pub fn delegate_access_as(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            to: T::AccountId,
            delegate_type: Vec<u8>,
            valid_for: Option<T::BlockNumber>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            Self::h_delegate_access_as(&origin, &org_id, &to, &delegate_type, valid_for)?;
            Ok(().into())
        }
    }

    // -------------------------------------------------------------------
    //                      GENESIS CONFIGURATION
    // -------------------------------------------------------------------

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        _phantom: PhantomData<T>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {}
    }
}

macro_rules! method_is_flag {
    ($funcname:ident, $flag:ident, $name:expr) => {
        #[doc = "Check whether organization is "]
        #[doc=$name]
        pub fn $funcname(id: T::AccountId) -> bool {
            <OrganizationFlagData<T>>::get(id)
                .map(|a| (*a).contains(FlagDataBit::$flag))
                .unwrap_or(false)
        }
    };
    ($funcname:ident, $flag:ident) => {
        method_is_flag!($funcname, $flag, stringify!($flag));
    };
}

/// The main implementation of this Organization pallet.
impl<T: Config> Pallet<T> {
    /// Validasi properties
    pub fn validate_props(props: &Option<Vec<Property>>) -> Result<(), Error<T>> {
        if let Some(props) = props {
            ensure!(props.len() <= MAX_PROPS, Error::<T>::TooManyProps);
            for prop in props {
                let len = prop.name().len();
                ensure!(
                    len > 0 && len <= PROP_NAME_MAX_LENGTH,
                    Error::<T>::InvalidPropName
                );
                let len = prop.value().len();
                ensure!(
                    len > 0 && len <= PROP_VALUE_MAX_LENGTH,
                    Error::<T>::InvalidPropValue
                );
            }
        }
        Ok(())
    }

    /// Memastikan origin dapat akses resource.
    ///
    /// Prosedur ini akan memeriksa apakah origin admin
    /// atau delegator.
    pub fn ensure_access(
        origin: &T::AccountId,
        org_id: &T::AccountId,
    ) -> Result<Organization<T::AccountId>, Error<T>> {
        let org = Self::organization(&org_id).ok_or(Error::<T>::NotExists)?;

        if &org.admin != origin {
            did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin)
                .map_err(|_| Error::<T>::PermissionDenied)?;
        }

        Ok(org)
    }

    /// Memastikan bahwa akun memiliki akses pada organisasi.
    /// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
    pub fn ensure_access_active_id(
        who: &T::AccountId,
        org_id: &T::AccountId,
    ) -> Result<(), Error<T>> {
        let org = Self::organization(&org_id).ok_or(Error::<T>::NotExists)?;
        Self::ensure_access_active(who, &org)
    }

    /// Memastikan bahwa akun memiliki akses pada organisasi.
    /// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
    pub fn ensure_access_active(
        who: &T::AccountId,
        org: &Organization<T::AccountId>,
    ) -> Result<(), Error<T>> {
        ensure!(&org.admin == who, Error::<T>::PermissionDenied);
        ensure!(!org.suspended, Error::<T>::PermissionDenied);
        Ok(())
    }

    /// Get next Organization ID
    pub fn next_index() -> Result<u64, Error<T>> {
        <OrgIdIndex<T>>::mutate(|o| {
            *o = Some(o.map_or(1, |vo| vo.saturating_add(1)));
            *o
        })
        .ok_or(Error::<T>::CannotGenId)
    }

    /// Check whether account is member of the organization
    pub fn is_member(id: T::AccountId, account_id: T::AccountId) -> bool {
        <Members<T>>::get(id)
            .map(|a| a.iter().any(|id| *id == account_id))
            .unwrap_or(false)
    }

    /// Check whether the ID is organization account.
    pub fn is_organization(id: &T::AccountId) -> bool {
        Self::organization(id).is_some()
    }

    /// Delegate access to someone with custom type.
    pub fn h_delegate_access_as(
        origin: &T::AccountId,
        org_id: &T::AccountId,
        to: &T::AccountId,
        delegate_type: &[u8],
        valid_for: Option<T::BlockNumber>,
    ) -> DispatchResult {
        let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

        ensure!(!org.suspended, Error::<T>::Suspended);
        ensure!(&org.admin == origin, Error::<T>::PermissionDenied);

        did::Module::<T>::create_delegate(&origin, &org_id, &to, delegate_type, valid_for)?;

        Ok(())
    }

    method_is_flag!(is_active, Active);
    method_is_flag!(is_verified, Verified);
    method_is_flag!(is_gov, Government);
    method_is_flag!(is_foundation, Foundation);
    method_is_flag!(is_system, System);

    /// Check whether organization suspended
    pub fn is_suspended(id: T::AccountId) -> bool {
        Self::organization(id).map(|a| a.suspended).unwrap_or(true)
    }

    /// Get admin of the organization
    pub fn get_admin(id: T::AccountId) -> Option<T::AccountId> {
        Self::organization(id).map(|a| a.admin)
    }
}

#[cfg(test)]
mod tests;
