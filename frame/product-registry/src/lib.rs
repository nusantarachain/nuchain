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
//! This pallet is intended to be use with supply chain functionality like to register 
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
//! To register a product, one must send a transaction with a `productRegistry.registerProduct` extrinsic with the following arguments:
//! - `id` as the Product ID, typically this would be a GS1 GTIN (Global Trade Item Number), or ASIN (Amazon Standard Identification Number), or similar, a numeric or alpha-numeric code with a well-defined data structure.
//! - `owner` as the Substrate Account representing the organization owning this product, as in the manufacturer or supplier providing this product within the value chain.
//! - `props` which is a series of properties (name & value) describing the product. Typically, there would at least be a textual description, and SKU. It could also contain instance / lot master data e.g. expiration, weight, harvest date.
//! 
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    dispatch, ensure, sp_runtime::RuntimeDebug, sp_std::prelude::*, traits::EnsureOrigin,
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const PRODUCT_ID_MAX_LENGTH: usize = 36;
pub const PRODUCT_PROP_NAME_MAX_LENGTH: usize = 10;
pub const PRODUCT_PROP_VALUE_MAX_LENGTH: usize = 20;
pub const PRODUCT_MAX_PROPS: usize = 3;

// Custom types
pub type ProductId = Vec<u8>;
pub type PropName = Vec<u8>;
pub type PropValue = Vec<u8>;

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
        pub props: Option<Vec<ProductProperty>>,
        // Timestamp (approximate) at which the prodct was registered on-chain.
        pub registered: Moment,
    }

    // Contains a name-value pair for a product property e.g. description: Ingredient ABC
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct ProductProperty {
        // Name of the product property e.g. desc or description
        name: PropName,
        // Value of the product property e.g. Ingredient ABC
        value: PropValue,
    }

    impl ProductProperty {
        pub fn new(name: &[u8], value: &[u8]) -> Self {
            Self {
                name: name.to_vec(),
                value: value.to_vec(),
            }
        }

        pub fn name(&self) -> &[u8] {
            self.name.as_ref()
        }

        pub fn value(&self) -> &[u8] {
            self.value.as_ref()
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
    }

    // decl_storage! {
    //     trait Store for Module<T: Config> as ProductRegistry {
    //         pub Products get(fn product_by_id): map hasher(blake2_128_concat) ProductId => Option<Product<T::AccountId, T::Moment>>;
    //         pub ProductsOfOrganization get(fn products_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<ProductId>;
    //         pub OwnerOf get(fn owner_of): map hasher(blake2_128_concat) ProductId => Option<T::AccountId>;
    //     }
    // }

    #[pallet::storage]
    #[pallet::getter(fn product_by_id)]
    pub type Products<T: Config> =
        StorageMap<_, Blake2_128Concat, ProductId, Product<T::AccountId, T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn products_of_org)]
    pub type ProductsOfOrganization<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<ProductId>>;

    #[pallet::storage]
    #[pallet::getter(fn owner_of)]
    pub type OwnerOf<T: Config> = StorageMap<_, Twox64Concat, ProductId, T::AccountId>;

    // decl_event!(
    //     pub enum Event<T>
    //     where
    //         AccountId = <T as system::Config>::AccountId,
    //     {
    //         ProductRegistered(AccountId, ProductId, AccountId),
    //     }
    // );

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some product are registered in blockchain.
        ProductRegistered(T::AccountId, ProductId, T::AccountId),
    }

    // decl_error! {
    //     pub enum Error for Module<T: Config> {
    //         ProductIdMissing,
    //         ProductIdTooLong,
    //         ProductIdExists,
    //         ProductTooManyProps,
    //         ProductInvalidPropName,
    //         ProductInvalidPropValue
    //     }
    // }

    #[pallet::error]
    pub enum Error<T> {
        /// Product not found in blockchain.
        ProductIdMissing,

        /// Product id is too long.
        ProductIdTooLong,

        /// Product with given id already exists.
        ProductIdExists,

        /// Too many properties.
        ProductTooManyProps,

        /// Invalid property name.
        ProductInvalidPropName,

        /// Invalid property value.
        ProductInvalidPropValue,
    }

    /// Supply Chain product registry module.
    #[pallet::call]
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    impl<T: Config> Pallet<T> {
        /// Register a product into blockchain.
        #[pallet::weight(10_000)]
        // pub fn register_product(origin, id: ProductId, owner: T::AccountId, props: Option<Vec<ProductProperty>>) -> dispatch::DispatchResult {
        pub fn register_product(
            origin: OriginFor<T>,
            id: ProductId,
            owner: T::AccountId,
            props: Option<Vec<ProductProperty>>,
        ) -> DispatchResultWithPostInfo {
            T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate product ID
            Self::validate_product_id(&id)?;

            // Validate product props
            Self::validate_product_props(&props)?;

            // Check product doesn't exist yet (1 DB read)
            Self::validate_new_product(&id)?;

            // TODO: if organization has an attribute w/ GS1 Company prefix,
            //       additional validation could be applied to the product ID
            //       to ensure its validity (same company prefix as org).

            // Create a product instance
            let product = Self::new_product()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .registered_on(<pallet_timestamp::Module<T>>::now())
                .with_props(props)
                .build();

            // Add product & ownerOf (3 DB writes)
            <Products<T>>::insert(&id, product);
            <ProductsOfOrganization<T>>::append(&owner, &id);
            <OwnerOf<T>>::insert(&id, &owner);

            Self::deposit_event(Event::ProductRegistered(who, id, owner));

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
    fn new_product() -> ProductBuilder<T::AccountId, T::Moment> {
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

    pub fn validate_product_props(props: &Option<Vec<ProductProperty>>) -> Result<(), Error<T>> {
        if let Some(props) = props {
            ensure!(
                props.len() <= PRODUCT_MAX_PROPS,
                Error::<T>::ProductTooManyProps,
            );
            for prop in props {
                ensure!(
                    prop.name().len() <= PRODUCT_PROP_NAME_MAX_LENGTH,
                    Error::<T>::ProductInvalidPropName
                );
                ensure!(
                    prop.value().len() <= PRODUCT_PROP_VALUE_MAX_LENGTH,
                    Error::<T>::ProductInvalidPropValue
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
    props: Option<Vec<ProductProperty>>,
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

    pub fn with_props(mut self, props: Option<Vec<ProductProperty>>) -> Self {
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
