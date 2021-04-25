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

//! # Nuchain Product Registry
//!
//! This pallet intended to be use with supply chain functionality to register
//! and managing product state between various stakeholders. This data is typically
//! registered once by the product's manufacturer / supplier to be shared with other
//! network participants.
//!
//! It is inspired by existing projects & standards:
//! - [IBM Food Trust](https://github.com/IBM/IFT-Developer-Zone/wiki/APIs)
//! - [Hyperledger Grid](https://www.hyperledger.org/use/grid)
//! - [GS1 Standards](https://www.gs1.org/standards)
//!
//!
//! ## Usage
//!
//! To register a product, one must send a transaction with a [`Pallet::register`] extrinsic with the following arguments:
//! - `id` as the Product ID, typically this would be a GS1 GTIN (Global Trade Item Number), or ASIN (Amazon Standard Identification Number), or similar, a numeric or alpha-numeric code with a well-defined data structure.
//! - `org_id` as the Nuchain Account representing the organization owning this product, as in the manufacturer or supplier providing this product within the value chain.
//! - `year` the year where the product was produced.
//! - `props` which is a series of properties (name & value) describing the product. Typically, there would at least be a textual description, and SKU. It could also contain instance / lot master data e.g. expiration, weight, harvest date.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{ensure, sp_runtime::RuntimeDebug, sp_std::prelude::*, types::Property};
use frame_system::{self, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const PRODUCT_ID_MAX_LENGTH: usize = 36;
pub const PRODUCT_PROP_NAME_MAX_LENGTH: usize = 10;
pub const PRODUCT_PROP_VALUE_MAX_LENGTH: usize = 36;
pub const PRODUCT_MAX_PROPS: usize = 5;

// Custom types
pub type ProductId = Vec<u8>;
pub type Year = u32;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Product contains master data (aka class-level) about a trade item.
    // This data is typically registered once by the product's manufacturer / supplier,
    // to be shared with other network participants, and remains largely static.
    // It can also be used for instance-level (lot) master data.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct Product<AccountId, Moment> {
        // The product ID would typically be a GS1 GTIN (Global Trade Item Number),
        // or ASIN (Amazon Standard Identification Number), or similar,
        // a numeric or alpha-numeric code with a well-defined data structure.
        pub id: ProductId,
        // This is account that represents the owner of this product, as in
        // the manufacturer or supplier providing this product within the value chain.
        pub owner: AccountId,
        // This a series of properties describing the product.
        // Typically, there would at least be a textual description, and SKU.
        // It could also contain instance / lot master data e.g. expiration, weight, harvest date.
        pub props: Option<Vec<Property>>,
        // Timestamp (approximate) at which the prodct was registered on-chain.
        pub registered: Moment,
    }

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_timestamp::Config + pallet_organization::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
    }

    /// Get product by ID.
    #[pallet::storage]
    #[pallet::getter(fn product_by_id)]
    pub type Products<T: Config> =
        StorageMap<_, Blake2_128Concat, ProductId, Product<T::AccountId, T::Moment>>;

    /// Get list of products of the organization.
    #[pallet::storage]
    #[pallet::getter(fn products_of_org)]
    pub type ProductsOfOrganization<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Blake2_128Concat, Year, Vec<ProductId>>;

    /// Get owner (organization) of the product where belongs to.
    #[pallet::storage]
    #[pallet::getter(fn owner_of)]
    pub type OwnerOf<T: Config> = StorageMap<_, Twox64Concat, ProductId, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some product are registered in blockchain.
        ProductRegistered(T::AccountId, ProductId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Product not found in blockchain.
        ProductIdMissing,

        /// Product id is too long.
        ProductIdTooLong,

        /// Product with given id already exists.
        ProductIdExists,

        /// Too many properties.
        TooManyProps,

        /// Invalid property name.
        InvalidPropName,

        /// Invalid property value.
        InvalidPropValue,
    }

    /// Supply Chain product registry module.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a product into blockchain.
        ///
        /// The caller of this function must be _signed_.
        ///
        /// * `id` - ID of product.
        /// * `org_id` - Organization ID where product belongs to, please see [pallet_organization::Organization].
        /// * `year` - Year of the product produced.
        /// * `props` - Properties for the product.
        ///
        #[pallet::weight(
            (20_000_000 as Weight).saturating_add(
                T::DbWeight::get().reads(2 as Weight)
                .saturating_add(
                    T::DbWeight::get().writes(3 as Weight)
                ))
         )]
        pub fn register(
            origin: OriginFor<T>,
            id: ProductId,
            org_id: T::AccountId,
            year: Year,
            props: Option<Vec<Property>>,
        ) -> DispatchResultWithPostInfo {
            // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate product ID
            Self::validate_product_id(&id)?;

            // Validate product props
            Self::validate_product_props(&props)?;

            // Check product doesn't exist yet (1 DB read)
            Self::validate_new_product(&id)?;

            // Pastikan origin memiliki akses ke organisasi
            <pallet_organization::Module<T>>::ensure_access_active_id(&who, &org_id)?;

            // Create a product instance
            let product = Self::new_product()
                .identified_by(id.clone())
                .owned_by(org_id.clone())
                .registered_on(<pallet_timestamp::Module<T>>::now())
                .with_props(props)
                .build();

            // Add product & ownerOf (3 DB writes)
            <Products<T>>::insert(&id, product);
            <ProductsOfOrganization<T>>::append(&org_id, year, &id);
            <OwnerOf<T>>::insert(&id, &org_id);

            Self::deposit_event(Event::ProductRegistered(who, id, org_id));

            Ok(().into())
        }
    }

    // ----------------------------------------------------------------
    //                      HOOKS
    // ----------------------------------------------------------------
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // fn offchain_worker(n: T::BlockNumber){
        //     // @TODO(you): Your off-chain logic here
        // }
    }

    // // -------------------------------------------------------------------
    // //                      GENESIS CONFIGURATION
    // // -------------------------------------------------------------------

    // // The genesis config type.
    // #[pallet::genesis_config]
    // pub struct GenesisConfig<T: Config> {
    //     _phantom: PhantomData<T>,
    // }

    // // The default value for the genesis config type.
    // #[cfg(feature = "std")]
    // impl<T: Config> Default for GenesisConfig<T> {
    //     fn default() -> Self {
    //         Self {
    //             _phantom: Default::default(),
    //         }
    //     }
    // }

    // // The build of genesis for the pallet.
    // #[pallet::genesis_build]
    // impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    //     fn build(&self) {}
    // }
}

pub use pallet::*;

impl<T: Config> Pallet<T> {
    // Helper methods
    pub fn new_product() -> ProductBuilder<T::AccountId, T::Moment> {
        ProductBuilder::<T::AccountId, T::Moment>::default()
    }

    pub fn validate_product_id(id: &[u8]) -> Result<(), Error<T>> {
        // Basic product ID validation
        ensure!(!id.is_empty(), Error::<T>::ProductIdMissing);
        ensure!(
            id.len() <= PRODUCT_ID_MAX_LENGTH,
            Error::<T>::ProductIdTooLong
        );
        Ok(())
    }

    pub fn validate_new_product(id: &[u8]) -> Result<(), Error<T>> {
        // Product existence check
        ensure!(
            !<Products<T>>::contains_key(id),
            Error::<T>::ProductIdExists
        );
        Ok(())
    }

    pub fn validate_product_props(props: &Option<Vec<Property>>) -> Result<(), Error<T>> {
        if let Some(props) = props {
            ensure!(props.len() <= PRODUCT_MAX_PROPS, Error::<T>::TooManyProps,);
            for prop in props {
                let len = prop.name().len();
                ensure!(
                    len > 0 && len <= PRODUCT_PROP_NAME_MAX_LENGTH,
                    Error::<T>::InvalidPropName
                );
                let len = prop.value().len();
                ensure!(
                    len > 0 && len <= PRODUCT_PROP_VALUE_MAX_LENGTH,
                    Error::<T>::InvalidPropValue
                );
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ProductBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    id: ProductId,
    owner: AccountId,
    props: Option<Vec<Property>>,
    registered: Moment,
}

impl<AccountId, Moment> ProductBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    pub fn identified_by(mut self, id: ProductId) -> Self {
        self.id = id;
        self
    }

    pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_props(mut self, props: Option<Vec<Property>>) -> Self {
        self.props = props;
        self
    }

    pub fn registered_on(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> Product<AccountId, Moment> {
        Product::<AccountId, Moment> {
            id: self.id,
            owner: self.owner,
            props: self.props,
            registered: self.registered,
        }
    }
}
