// This file is part of Nuchain.

// Copyright (C) 2017-2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

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

//! This pallet trying to combine the power of NFT with Intelectual Property (IP),
//! based on https://blog.oceanprotocol.com/nfts-ip-3-combining-erc721-erc20-b69ea659115e
//! by Trent McConaghy

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
// pub mod types;
pub mod weights;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    dispatch::DispatchError,
    ensure,
    traits::{BalanceStatus::Reserved, Currency, ReservableCurrency},
};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero},
    RuntimeDebug,
};
use sp_std::{fmt::Debug, prelude::*};
pub use weights::WeightInfo;

pub use pallet::*;

include! {"types.rs"}

/// global max holding limit per token per aset per account
const MAX_ASSET_PER_ACCOUNT: u32 = 1_000_000;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// type AllowedMintAccount<T> = (<T as frame_system::Config>::AccountId, u32);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    /// The module configuration trait.
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The units in which we record balances.
        type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

        /// The arithmetic type of asset identifier.
        type CollectionId: Member + Parameter + Default + Copy + HasCompact;

        /// The arithmetic type of token identifier.
        type AssetId: Member + Parameter + Default + Copy + HasCompact;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The origin which may forcibly create or destroy an asset.
        type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// The basic amount of funds that must be reserved when creating a new asset class.
        type AssetDepositBase: Get<BalanceOf<Self>>;

        /// The additional funds that must be reserved for every zombie account that an asset class
        /// supports.
        type AssetDepositPerZombie: Get<BalanceOf<Self>>;

        /// The maximum length of a name or symbol stored on-chain.
        type StringLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type StringUriLimit: Get<u32>;

        /// The basic amount of funds that must be reserved when adding metadata to your asset.
        type MetadataDepositBase: Get<BalanceOf<Self>>;

        /// The additional funds that must be reserved for the number of bytes you store in your
        /// metadata.
        type MetadataDepositPerByte: Get<BalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Balance = "Balance",
        T::AssetId = "AssetId"
    )]
    pub enum Event<T: Config> {
        /// Some collection was created. \[collection_id, owner\]
        CollectionCreated(T::CollectionId, T::AccountId),
        /// Some asset class was created. \[collection_id, asset_id, owner\]
        AssetMinted(T::CollectionId, T::AssetId, T::AccountId),
        /// Some asset class was created. \[collection_id, asset_id, creator, owner\]
        TokenMinted(T::CollectionId, T::AssetId, T::AccountId, T::AccountId),
        /// Some assets were issued. \[collection_id, owner, total_supply\]
        Issued(T::CollectionId, T::AssetId, T::AccountId, T::Balance),
        /// Some assets were transferred. \[collection_id, from, to, amount\]
        Transferred(
            T::CollectionId,
            T::AssetId,
            T::AccountId,
            T::AccountId,
            T::Balance,
        ),
        /// Some assets were destroyed. \[collection_id, owner, balance\]
        Burned(T::CollectionId, T::AssetId, T::AccountId, T::Balance),
        /// The management team changed \[collection_id, admin, freezer\]
        TeamChanged(T::CollectionId, T::AccountId, T::AccountId),
        /// The asset's owner changed \[collection_id, owner\]
        CollectionOwnerChanged(T::CollectionId, T::AccountId),
        /// The owner changed \[collection_id, asset_id, owner\]
        AssetOwnerChanged(T::CollectionId, T::AssetId, T::AccountId),
        /// Some assets was transferred by an admin. \[collection_id, from, to, amount\]
        ForceTransferred(
            T::CollectionId,
            T::AssetId,
            T::AccountId,
            T::AccountId,
            T::Balance,
        ),
        /// Some account `who` was frozen. \[collection_id, who\]
        Frozen(T::CollectionId, T::AssetId, T::AccountId),
        /// Some account `who` was thawed. \[collection_id, who\]
        Thawed(T::CollectionId, T::AssetId, T::AccountId),
        /// Some asset `collection_id` was frozen. \[collection_id\]
        CollectionFrozen(T::CollectionId),
        /// Some asset `collection_id` was thawed. \[collection_id\]
        CollectionThawed(T::CollectionId),
        /// An asset class was destroyed.
        Destroyed(T::CollectionId, T::AssetId),
        /// Some asset class was force-created. \[collection_id, asset_id, owner\]
        ForceCreated(T::CollectionId, T::AssetId, T::AccountId),
        /// The maximum amount of zombies allowed has changed. \[collection_id, max_zombies\]
        MaxZombiesChanged(T::CollectionId, T::AssetId, u32),
        /// New metadata has been set for an asset. \[collection_id, asset_id, ip_owner\]
        MetadataSet(T::CollectionId, T::AssetId, Vec<u8>, Vec<u8>, T::AccountId),
    }

    #[deprecated(note = "use `Event` instead")]
    pub type RawEvent<T> = Event<T>;

    #[pallet::error]
    pub enum Error<T> {
        /// Asset not found
        NotFound,
        /// Transfer amount should be non-zero.
        AmountZero,
        /// Account balance must be greater than or equal to the transfer amount.
        BalanceLow,
        /// Balance should be non-zero.
        BalanceZero,
        /// The signing account has no permission to do the operation.
        Unauthorized,
        /// The given asset ID is unknown.
        Unknown,
        /// The origin account is frozen.
        Frozen,
        /// The asset ID is already taken.
        InUse,
        /// Too many zombie accounts in use.
        TooManyZombies,
        /// Attempt to destroy an asset class when non-zombie, reference-bearing accounts exist.
        RefsLeft,
        /// Attempt to destroy collection when there is assets exists
        HasAssetLeft,
        /// Invalid witness data given.
        BadWitness,
        /// Minimum balance should be non-zero.
        MinBalanceZero,
        /// A mint operation lead to an overflow.
        Overflow,
        /// Some internal state is broken.
        BadState,
        /// Invalid metadata given.
        BadMetadata,
        /// max limit token ownership per account reached
        MaxLimitPerAccount,
    }

    #[pallet::storage]
    /// Collection for base token metadata and set of rules.
    pub(super) type Collection<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
    >;

    #[pallet::storage]
    /// Token holder account
    pub(super) type AccountAsset<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        T::AssetId,
        // TokenMetadata<T::Balance, T::AccountId, T::CollectionId, BalanceOf<T>>,
        T::AccountId,
    >;

    #[pallet::storage]
    /// The number of units of assets held by any given account.
    pub(super) type Account<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        (T::AssetId, T::AccountId),
        TokenBalance<T::Balance>,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Total asset held per account.
    pub(super) type OwnedAssetCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    pub(super) type MintAllowed<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        T::AccountId,
        u32,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Metadata of an asset.
    pub(super) type Metadata<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        T::AssetId,
        AssetMetadata<BalanceOf<T>, T::AccountId>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Asset id by index
    pub(super) type AssetIndex<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        u64,
        T::AssetId,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Build new instance of asset class from a public origin.
        ///
        /// The origin must be Signed and the sender must have sufficient funds free.
        ///
        /// Parameters:
        /// - `collection_id` - ID of asset to build.
        /// - `meta` - metadata contains collection parameters.
        #[pallet::weight(10_000_000)]
        pub(super) fn create_collection(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            meta: NewCollectionParam<T::Balance, T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            // let admin = T::Lookup::lookup(admin)?;

            ensure!(
                meta.name.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                meta.symbol.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                meta.max_asset_per_account <= MAX_ASSET_PER_ACCOUNT,
                Error::<T>::BadMetadata
            );

            ensure!(
                !Collection::<T>::contains_key(collection_id),
                Error::<T>::InUse
            );

            let deposit = T::MetadataDepositPerByte::get()
                .saturating_mul(((meta.name.len() + meta.symbol.len()) as u32).into())
                .saturating_add(T::MetadataDepositBase::get())
                .saturating_add((meta.allowed_mint_accounts.len() as u32).into());

            T::Currency::reserve(&owner, deposit)?;

            Collection::<T>::insert(
                collection_id,
                CollectionMetadata {
                    name: meta.name.clone(),
                    symbol: meta.symbol.clone(),
                    owner: meta.owner.clone(),
                    max_asset_count: meta.max_asset_count,
                    has_token: meta.has_token,
                    max_token_supply: meta.max_token_supply,
                    public_mintable: meta.public_mintable,
                    max_asset_per_account: meta.max_asset_per_account,
                    asset_count: Zero::zero(),
                    asset_index: Zero::zero(),
                    token_supply: Zero::zero(),
                    deposit,
                    min_balance: meta.min_balance,
                    accounts: Zero::zero(),
                    is_frozen: false,
                    max_zombies: meta.max_zombies,
                    zombies: 0,
                },
            );

            for allowed in meta.allowed_mint_accounts {
                MintAllowed::<T>::insert(collection_id, allowed.account, allowed.amount);
            }

            Self::deposit_event(Event::CollectionCreated(collection_id, owner));

            Ok(().into())
        }

        /// Destroy whole collection
        ///
        /// The origin must be Signed and the sender must be the collection owner.
        #[pallet::weight(10_000_000)]
        pub(super) fn destroy_collection(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // ensure!(Collection::<T>::contains(collection_id), Error::<T>::NotFound);

            Collection::<T>::try_mutate_exists(collection_id, |maybe_meta| {
                // let meta = maybe_meta.as_mut().ok_or(Error::<T>::NotFound)?;

                let deposit;
                if let Some(ref meta) = maybe_meta {
                    ensure!(meta.owner == who, Error::<T>::Unauthorized);
                    ensure!(meta.asset_count == 0, Error::<T>::HasAssetLeft);
                    deposit = meta.deposit;
                } else {
                    return Err(Error::<T>::NotFound.into());
                }

                *maybe_meta = None;

                T::Currency::unreserve(&who, deposit);

                // clean up mint allowed registry
                MintAllowed::<T>::remove_prefix(collection_id);

                Ok(().into())
            })
        }

        /// Update collection metadata.
        ///
        /// The origin must be Signed and the sender must be the collection owner.
        ///
        ///
        #[pallet::weight(100_000_000)]
        pub(super) fn update_collection(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
            public_mintable: Option<bool>,
            max_asset_per_account: Option<u32>,
            min_balance: Option<T::Balance>,
            has_token: Option<bool>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if let Some(max_asset_per_account) = max_asset_per_account {
                ensure!(max_asset_per_account > 0, Error::<T>::BadMetadata);
                ensure!(
                    max_asset_per_account <= MAX_ASSET_PER_ACCOUNT,
                    Error::<T>::MaxLimitPerAccount
                );
            }

            if public_mintable.is_none()
                && max_asset_per_account.is_none()
                && min_balance.is_none()
                && has_token.is_none()
            {
                return Ok(().into());
            }

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                ensure!(meta.owner == who, Error::<T>::Unauthorized);

                if let Some(public_mintable) = public_mintable {
                    meta.public_mintable = public_mintable;
                }

                if let Some(max_asset_per_account) = max_asset_per_account {
                    meta.max_asset_per_account = max_asset_per_account;
                }

                if let Some(min_balance) = min_balance {
                    meta.min_balance = min_balance;
                }

                if let Some(has_token) = has_token {
                    meta.has_token = has_token;
                }

                Ok(().into())
            })
        }

        /// Mint asset for the base token from public origin.
        ///
        /// This new asset class has no assets initially.
        ///
        /// The origin must be Signed and the sender must have sufficient funds free.
        ///
        /// Funds of sender are reserved according to the formula:
        /// `AssetDepositBase + AssetDepositPerZombie * max_zombies`.
        ///
        /// Parameters:
        /// - `id`: The identifier of the new asset. This must not be currently in use to identify
        /// an existing asset.
        /// - `owner`: The owner of this class of assets. The owner has full superuser permissions
        /// over this asset, but may later change and configure the permissions using `transfer_ownership`
        /// and `set_team`.
        /// - `max_zombies`: The total number of accounts which may hold assets in this class yet
        /// have no existential deposit.
        /// - `min_balance`: The minimum balance of this new asset that any single account must
        /// have. If an account's balance is reduced below this, then it collapses to zero.
        ///
        /// Emits `Created` event when successful.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::create())]
        pub(super) fn mint_asset(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            name: Vec<u8>,
            description: Vec<u8>,
            token_uri: Option<Vec<u8>>,
            base_uri: Option<Vec<u8>>,
            ip_owner: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                name.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                description.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );

            if let Some(ref token_uri) = token_uri {
                ensure!(
                    token_uri.len() <= T::StringUriLimit::get() as usize,
                    Error::<T>::BadMetadata
                );
            }
            if let Some(ref base_uri) = base_uri {
                ensure!(
                    base_uri.len() <= T::StringUriLimit::get() as usize,
                    Error::<T>::BadMetadata
                );
            }

            ensure!(
                !AccountAsset::<T>::contains_key(collection_id, asset_id),
                Error::<T>::InUse
            );

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let mut meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                // check is user allowed to mint this token's asset
                if !meta.public_mintable {
                    if who != meta.owner {
                        ensure!(
                            MintAllowed::<T>::get(collection_id, &who)
                                .map(|a| a > 0)
                                .unwrap_or(false),
                            Error::<T>::Unauthorized
                        );
                    }
                }

                Self::do_mint_asset(
                    who,
                    collection_id,
                    asset_id,
                    &mut meta,
                    name,
                    description,
                    token_uri,
                    base_uri,
                    ip_owner,
                )?;

                Ok(().into())
            })
        }

        /// Mint asset for base token from a privileged origin.
        ///
        /// This new asset class has no assets initially.
        ///
        /// The origin must conform to `ForceOrigin`.
        ///
        /// Unlike `mint_asset`, no funds are reserved, no max holding per account limit are checked,
        /// and no string length parameter validation checks.
        ///
        /// - `id`: The identifier of the new asset. This must not be currently in use to identify
        /// an existing asset.
        /// - `owner`: The owner of this class of assets. The owner has full superuser permissions
        /// over this asset, but may later change and configure the permissions using `transfer_ownership`
        ///
        /// Emits `ForceCreated` event when successful.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::force_create())]
        pub(super) fn force_mint_asset(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            owner: <T::Lookup as StaticLookup>::Source,
            name: Vec<u8>,
            description: Vec<u8>,
            token_uri: Option<Vec<u8>>,
            base_uri: Option<Vec<u8>>,
            ip_owner: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;
            let owner = T::Lookup::lookup(owner)?;

            ensure!(
                !AccountAsset::<T>::contains_key(collection_id, asset_id),
                Error::<T>::InUse
            );

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let mut meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                // check limit per account
                let owned_asset_count = OwnedAssetCount::<T>::get(collection_id, &owner);

                if meta.max_asset_per_account > 0 {
                    ensure!(
                        owned_asset_count < meta.max_asset_per_account,
                        Error::<T>::MaxLimitPerAccount
                    );
                }

                Self::do_mint_asset(
                    owner,
                    collection_id,
                    asset_id,
                    &mut meta,
                    name,
                    description,
                    token_uri,
                    base_uri,
                    ip_owner,
                )?;

                Ok(().into())
            })
        }

        /// Destroy an asset's token owned by sender.
        ///
        /// The origin must be Signed and the sender must be the owner of the token `id`.
        ///
        /// - `id`: The identifier of the asset to be destroyed. This must identify an existing
        /// asset.
        ///
        /// Emits `Destroyed` event when successful.
        ///
        #[pallet::weight(T::WeightInfo::destroy(0))]
        pub(super) fn destroy_asset(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                // let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                ensure!(
                    // AccountAsset::<T>::get(collection_id, asset_id) == origin
                    Self::is_asset_owner(&origin, collection_id, asset_id),
                    Error::<T>::Unauthorized
                );

                // meta.asset_count = meta.asset_count.saturating_sub(1);

                // AccountAsset::<T>::remove(collection_id, asset_id);

                // OwnedAssetCount::<T>::mutate(collection_id, &meta.owner, |count| {
                //     *count = count.saturating_sub(1);
                // });

                // Self::deposit_event(Event::Destroyed(collection_id, asset_id));

                // Ok(().into())

                let mut meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                Self::do_destroy_asset(collection_id, asset_id, &mut meta)
            })
        }

        /// Destroy an asset's token owned by sender.
        ///
        /// The origin must conform to `ForceOrigin`.
        ///
        /// - `id`: The identifier of the asset to be destroyed. This must identify an existing
        /// asset.
        ///
        /// Emits `Destroyed` event when successful.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::force_destroy(0))]
        pub(super) fn force_destroy_asset(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            ensure!(
                Metadata::<T>::contains_key(collection_id, asset_id),
                Error::<T>::Unknown
            );

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let mut meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                Self::do_destroy_asset(collection_id, asset_id, &mut meta)
            })
        }

        /// Mint token.
        ///
        /// The origin must be Signed.
        ///
        /// - `collection_id`: The identifier of the asset to have some token minted.
        /// - `asset_id`: The identifier of the token to have some amount minted.
        /// - `beneficiary`: The account to be credited with the minted assets.
        /// - `amount`: The amount of the asset to be minted.
        ///
        /// Emits `Destroyed` event when successful.
        ///
        /// Weight: `O(1)`
        /// Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`.
        #[pallet::weight(T::WeightInfo::mint())]
        pub(super) fn mint_token(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            beneficiary: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let beneficiary = T::Lookup::lookup(beneficiary)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                ensure!(&origin == &meta.owner, Error::<T>::Unauthorized);

                meta.token_supply = amount
                    .checked_add(&meta.token_supply.into())
                    .ok_or(Error::<T>::Overflow)?;

                Account::<T>::try_mutate(
                    collection_id,
                    (asset_id, &beneficiary),
                    |t| -> DispatchResultWithPostInfo {
                        let new_balance = t.balance.saturating_add(amount);
                        ensure!(new_balance >= meta.min_balance, Error::<T>::BalanceLow);
                        if t.balance.is_zero() {
                            t.is_zombie = Self::new_account(&beneficiary, meta)?;
                        }
                        t.balance = new_balance;
                        Ok(().into())
                    },
                )?;
                Self::deposit_event(Event::Issued(collection_id, asset_id, beneficiary, amount));
                Ok(().into())
            })
        }

        /// Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
        ///
        /// Origin must be Signed and the sender should be the Manager of the asset `id`.
        ///
        /// Bails with `BalanceZero` if the `who` is already dead.
        ///
        /// - `id`: The identifier of the asset to have some amount burned.
        /// - `who`: The account to be debited from.
        /// - `amount`: The maximum amount by which `who`'s balance should be reduced.
        ///
        /// Emits `Burned` with the actual amount burned. If this takes the balance to below the
        /// minimum for the asset, then the amount burned is increased to take it to zero.
        ///
        /// Weight: `O(1)`
        /// Modes: Post-existence of `who`; Pre & post Zombie-status of `who`.
        #[pallet::weight(T::WeightInfo::burn())]
        pub(super) fn burn_token(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let who = T::Lookup::lookup(who)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;

                ensure!(&origin == &meta.owner, Error::<T>::Unauthorized);

                let burned = Account::<T>::try_mutate_exists(
                    collection_id,
                    (asset_id, &who),
                    |maybe_account| -> Result<T::Balance, DispatchError> {
                        let mut account = maybe_account.take().ok_or(Error::<T>::BalanceZero)?;
                        let mut burned = amount.min(account.balance);
                        account.balance = account.balance.saturating_sub(burned);
                        *maybe_account = if account.balance < meta.min_balance {
                            burned = burned.saturating_add(account.balance);
                            Self::dead_account(&who, meta, account.is_zombie);
                            None
                        } else {
                            Some(account)
                        };
                        Ok(burned)
                    },
                )?;

                meta.token_supply = meta.token_supply.saturating_sub(burned.into());

                Self::deposit_event(Event::Burned(collection_id, asset_id, who, burned));

                Ok(().into())
            })
        }

        /// Move some assets from the sender account to another.
        ///
        /// Origin must be Signed.
        ///
        /// - `id`: The identifier of the asset to have some amount transferred.
        /// - `target`: The account to be credited.
        /// - `amount`: The amount by which the sender's balance of assets should be reduced and
        /// `target`'s balance increased. The amount actually transferred may be slightly greater in
        /// the case that the transfer would otherwise take the sender balance above zero but below
        /// the minimum balance. Must be greater than zero.
        ///
        /// Emits `Transferred` with the actual amount transferred. If this takes the source balance
        /// to below the minimum for the asset, then the amount transferred is increased to take it
        /// to zero.
        ///
        /// Weight: `O(1)`
        /// Modes: Pre-existence of `target`; Post-existence of sender; Prior & post zombie-status
        /// of sender; Account pre-existence of `target`.
        #[pallet::weight(T::WeightInfo::transfer())]
        pub(super) fn transfer_token(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            target: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            let mut origin_account = Account::<T>::get(collection_id, (asset_id, &origin));
            ensure!(!origin_account.is_frozen, Error::<T>::Frozen);
            origin_account.balance = origin_account
                .balance
                .checked_sub(&amount)
                .ok_or(Error::<T>::BalanceLow)?;

            let dest = T::Lookup::lookup(target)?;
            // Collectible::<T>::try_mutate(collection_id, asset_id, |maybe_meta| {
            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(!meta.is_frozen, Error::<T>::Frozen);

                if dest == origin {
                    return Ok(().into());
                }

                let mut amount = amount;
                if origin_account.balance < meta.min_balance {
                    amount = origin_account.balance.saturating_add(amount);
                    origin_account.balance = Zero::zero();
                }

                let acc_key = &collection_id; //&(collection_id, asset_id);

                Account::<T>::try_mutate(
                    acc_key,
                    (asset_id, &dest),
                    |a| -> DispatchResultWithPostInfo {
                        let new_balance = a.balance.saturating_add(amount);
                        ensure!(new_balance >= meta.min_balance, Error::<T>::BalanceLow);
                        if a.balance.is_zero() {
                            a.is_zombie = Self::new_account(&dest, meta)?;
                        }
                        a.balance = new_balance;
                        Ok(().into())
                    },
                )?;

                let key_b = (asset_id, origin.clone());

                match origin_account.balance.is_zero() {
                    false => {
                        Self::dezombify(&origin, meta, &mut origin_account.is_zombie);
                        Account::<T>::insert(acc_key, &key_b, &origin_account)
                    }
                    true => {
                        Self::dead_account(&origin, meta, origin_account.is_zombie);
                        Account::<T>::remove(acc_key, &key_b);
                    }
                }

                Self::deposit_event(Event::Transferred(
                    collection_id,
                    asset_id,
                    origin,
                    dest,
                    amount,
                ));
                Ok(().into())
            })
        }

        /// Move some assets from one account to another.
        ///
        /// Origin must be Signed and the sender should be the Admin of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to have some amount transferred.
        /// - `source`: The account to be debited.
        /// - `dest`: The account to be credited.
        /// - `amount`: The amount by which the `source`'s balance of assets should be reduced and
        /// `dest`'s balance increased. The amount actually transferred may be slightly greater in
        /// the case that the transfer would otherwise take the `source` balance above zero but
        /// below the minimum balance. Must be greater than zero.
        ///
        /// Emits `Transferred` with the actual amount transferred. If this takes the source balance
        /// to below the minimum for the asset, then the amount transferred is increased to take it
        /// to zero.
        ///
        /// Weight: `O(1)`
        /// Modes: Pre-existence of `dest`; Post-existence of `source`; Prior & post zombie-status
        /// of `source`; Account pre-existence of `dest`.
        #[pallet::weight(T::WeightInfo::force_transfer())]
        pub(super) fn force_transfer_token(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            source: <T::Lookup as StaticLookup>::Source,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let source = T::Lookup::lookup(source)?;
            let mut source_account = Account::<T>::get(&collection_id, (asset_id, &source));
            let mut amount = amount.min(source_account.balance);
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            let dest = T::Lookup::lookup(dest)?;
            if dest == source {
                return Ok(().into());
            }

            Collection::<T>::try_mutate(collection_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::Unauthorized);

                source_account.balance -= amount;
                if source_account.balance < details.min_balance {
                    amount = source_account.balance.saturating_add(amount);
                    source_account.balance = Zero::zero();
                }

                let acc_key = &collection_id;

                Account::<T>::try_mutate(
                    acc_key,
                    (asset_id, &dest),
                    |a| -> DispatchResultWithPostInfo {
                        let new_balance = a.balance.saturating_add(amount);
                        ensure!(new_balance >= details.min_balance, Error::<T>::BalanceLow);
                        if a.balance.is_zero() {
                            a.is_zombie = Self::new_account(&dest, details)?;
                        }
                        a.balance = new_balance;
                        Ok(().into())
                    },
                )?;

                match source_account.balance.is_zero() {
                    false => {
                        Self::dezombify(&source, details, &mut source_account.is_zombie);
                        Account::<T>::insert(acc_key, (asset_id, &source), &source_account)
                    }
                    true => {
                        Self::dead_account(&source, details, source_account.is_zombie);
                        Account::<T>::remove(acc_key, (asset_id, &source));
                    }
                }

                Self::deposit_event(Event::ForceTransferred(
                    collection_id,
                    asset_id,
                    source,
                    dest,
                    amount,
                ));
                Ok(().into())
            })
        }

        /// Disallow further unprivileged transfers from an account.
        ///
        /// Origin must be Signed and the sender should be the Freezer of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        /// - `who`: The account to be frozen.
        ///
        /// Emits `Frozen`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::freeze())]
        pub(super) fn freeze(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let holder =
                AccountAsset::<T>::get(collection_id, asset_id).ok_or(Error::<T>::Unknown)?;

            // ensure!(&origin == &d.freezer, Error::<T>::Unauthorized);
            ensure!(&origin == &holder, Error::<T>::Unauthorized);
            let who = T::Lookup::lookup(who)?;
            ensure!(
                Account::<T>::contains_key(&collection_id, (asset_id, &who)),
                Error::<T>::BalanceZero
            );

            Account::<T>::mutate(&collection_id, (asset_id, &who), |a| a.is_frozen = true);

            Self::deposit_event(Event::<T>::Frozen(collection_id, asset_id, who));
            Ok(().into())
        }

        /// Allow unprivileged transfers from an account again.
        ///
        /// Origin must be Signed and the sender should be the Admin of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        /// - `who`: The account to be unfrozen.
        ///
        /// Emits `Thawed`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::thaw())]
        pub(super) fn thaw(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let owner =
                AccountAsset::<T>::get(collection_id, asset_id).ok_or(Error::<T>::Unknown)?;
            // ensure!(&origin == &details.admin, Error::<T>::Unauthorized);
            ensure!(&origin == &owner, Error::<T>::Unauthorized);
            let who = T::Lookup::lookup(who)?;
            ensure!(
                Account::<T>::contains_key(&collection_id, (asset_id, &who)),
                Error::<T>::BalanceZero
            );

            Account::<T>::mutate(collection_id, (asset_id, &who), |a| a.is_frozen = false);

            Self::deposit_event(Event::<T>::Thawed(collection_id, asset_id, who));
            Ok(().into())
        }

        /// Disallow further unprivileged transfers for the asset class.
        ///
        /// Origin must be Signed and the sender should be the Freezer of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        ///
        /// Emits `Frozen`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::freeze_asset())]
        pub(super) fn freeze_collection(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            // #[pallet::compact] asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let d = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                // ensure!(&origin == &d.freezer, Error::<T>::Unauthorized);
                ensure!(&origin == &d.owner, Error::<T>::Unauthorized);

                d.is_frozen = true;

                Self::deposit_event(Event::<T>::CollectionFrozen(collection_id));
                Ok(().into())
            })
        }

        /// Allow unprivileged transfers for the asset again.
        ///
        /// Origin must be Signed and the sender should be the Admin of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        ///
        /// Emits `Thawed`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::thaw_asset())]
        pub(super) fn thaw_asset(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            // #[pallet::compact] asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collection::<T>::try_mutate(collection_id, |mybe_meta| {
                let d = mybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                // ensure!(&origin == &d.admin, Error::<T>::Unauthorized);
                ensure!(&origin == &d.owner, Error::<T>::Unauthorized);

                d.is_frozen = false;

                Self::deposit_event(Event::<T>::CollectionThawed(collection_id));
                Ok(().into())
            })
        }

        /// Change the Owner of collection.
        ///
        /// Origin must be Signed and the sender should be the Owner of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        /// - `owner`: The new Owner of this asset.
        ///
        /// Emits `OwnerChanged`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::transfer_ownership())]
        pub(super) fn transfer_collection_ownership(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            new_owner: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let new_owner = T::Lookup::lookup(new_owner)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &meta.owner, Error::<T>::Unauthorized);

                if meta.owner == new_owner {
                    return Ok(().into());
                }

                // Move the deposit to the new owner.
                T::Currency::repatriate_reserved(&meta.owner, &new_owner, meta.deposit, Reserved)?;

                meta.owner = new_owner.clone();

                Self::deposit_event(Event::CollectionOwnerChanged(collection_id, new_owner));
                Ok(().into())
            })
        }

        /// Change the Owner of asset.
        ///
        /// Origin must be Signed and the sender should be the Owner of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        /// - `owner`: The new Owner of this asset.
        ///
        /// Emits `OwnerChanged`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::transfer_ownership())]
        pub(super) fn transfer_asset_ownership(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            new_owner: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let new_owner = T::Lookup::lookup(new_owner)?;

            Collection::<T>::try_mutate(collection_id, |maybe_meta| {
                let meta = maybe_meta.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &meta.owner, Error::<T>::Unauthorized);

                if meta.owner == new_owner {
                    return Ok(().into());
                }

                let owned_asset_count = OwnedAssetCount::<T>::get(collection_id, &new_owner);

                if meta.max_asset_per_account > Zero::zero() {
                    ensure!(
                        owned_asset_count < meta.max_asset_per_account,
                        Error::<T>::MaxLimitPerAccount
                    );
                }

                ensure!(
                    owned_asset_count < MAX_ASSET_PER_ACCOUNT,
                    Error::<T>::MaxLimitPerAccount
                );

                // Move the deposit to the new owner.
                T::Currency::repatriate_reserved(&meta.owner, &new_owner, meta.deposit, Reserved)?;

                OwnedAssetCount::<T>::mutate(collection_id, &meta.owner, |count| {
                    *count = count.saturating_sub(1);
                });

                meta.owner = new_owner.clone();

                OwnedAssetCount::<T>::mutate(collection_id, &new_owner, |count| {
                    *count = count.saturating_add(1);
                });

                Self::deposit_event(Event::AssetOwnerChanged(collection_id, asset_id, new_owner));
                Ok(().into())
            })
        }

        /// Change the Issuer, Admin and Freezer of an asset.
        ///
        /// Origin must be Signed and the sender should be the Owner of the asset `id`.
        ///
        /// - `id`: The identifier of the asset to be frozen.
        /// - `issuer`: The new Issuer of this asset.
        /// - `admin`: The new Admin of this asset.
        /// - `freezer`: The new Freezer of this asset.
        ///
        /// Emits `TeamChanged`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::set_team())]
        pub(super) fn set_team(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            // issuer: <T::Lookup as StaticLookup>::Source,
            // allowed_mint_accounts: Vec<T::AccountId>,
            admin: <T::Lookup as StaticLookup>::Source,
            freezer: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            // let issuer = T::Lookup::lookup(issuer)?;
            let admin = T::Lookup::lookup(admin)?;
            let freezer = T::Lookup::lookup(freezer)?;

            Collection::<T>::try_mutate(collection_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::Unauthorized);

                // @TODO(robin): adjust deposit here
                // ....

                // details.issuer = issuer.clone();
                // details.allowed_mint_accounts = allowed_mint_accounts.clone();
                // details.admin = admin.clone();
                // details.freezer = freezer.clone();

                Self::deposit_event(Event::TeamChanged(collection_id, admin, freezer));
                Ok(().into())
            })
        }

        /// Set the metadata for an asset.
        ///
        /// NOTE: There is no `unset_metadata` call. Simply pass an empty name, symbol,
        /// and 0 decimals to this function to remove the metadata of an asset and
        /// return your deposit.
        ///
        /// Origin must be Signed and the sender should be the Owner of the asset `id`.
        ///
        /// Funds of sender are reserved according to the formula:
        /// `MetadataDepositBase + MetadataDepositPerByte * (name.len + symbol.len)` taking into
        /// account any already reserved funds.
        ///
        /// - `collection_id`: The identifier of the asset to update.
        /// - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
        /// - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
        /// - `decimals`: The number of decimals this asset uses to represent one unit.
        ///
        /// Emits `MetadataSet`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::set_metadata(token_uri.as_ref().map(|a| a.len()).unwrap_or(0) as u32, base_uri.as_ref().map(|a| a.len()).unwrap_or(0) as u32))]
        pub(super) fn set_metadata(
            origin: OriginFor<T>,
            #[pallet::compact] collection_id: T::CollectionId,
            #[pallet::compact] asset_id: T::AssetId,
            // name: Vec<u8>,
            // description: Vec<u8>,
            token_uri: Option<Vec<u8>>,
            base_uri: Option<Vec<u8>>,
            ip_owner: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            if let Some(ref token_uri) = token_uri {
                ensure!(
                    token_uri.len() <= T::StringUriLimit::get() as usize,
                    Error::<T>::BadMetadata
                );
            }
            if let Some(ref base_uri) = base_uri {
                ensure!(
                    base_uri.len() <= T::StringUriLimit::get() as usize,
                    Error::<T>::BadMetadata
                );
            }

            if token_uri.is_none() && base_uri.is_none() && ip_owner.is_none() {
                return Ok(().into());
            }

            // let owner =
            //     AccountAsset::<T>::get(collection_id, asset_id).ok_or(Error::<T>::Unknown)?;
            // ensure!(&origin == &owner, Error::<T>::Unauthorized);

            Metadata::<T>::try_mutate(collection_id, asset_id, |metadata| {
                let meta = metadata.as_mut().ok_or(Error::<T>::Unknown)?;
                let bytes_used = token_uri
                    .as_ref()
                    .map(|a| a.len())
                    .unwrap_or(0)
                    .saturating_add(base_uri.as_ref().map(|a| a.len()).unwrap_or(0));
                // let old_deposit = match metadata {
                //     Some(m) => m.deposit,
                //     None => Default::default(),
                // };
                let old_deposit = meta.deposit;

                // Metadata is being removed
                // if bytes_used.is_zero() && decimals.is_zero() {
                // if bytes_used.is_zero() {
                //     T::Currency::unreserve(&origin, old_deposit);
                //     *metadata = None;
                // } else {
                let new_deposit = T::MetadataDepositPerByte::get()
                    .saturating_mul((bytes_used as u32).into())
                    .saturating_add(T::MetadataDepositBase::get());

                if new_deposit > old_deposit {
                    T::Currency::reserve(&origin, new_deposit - old_deposit)?;
                } else {
                    T::Currency::unreserve(&origin, old_deposit - new_deposit);
                }

                // *metadata = Some(AssetMetadata {
                //     deposit: new_deposit,
                //     name: name.clone(),
                //     description: description.clone(),
                //     token_uri: token_uri.clone(),
                //     base_uri: base_uri.clone(),
                //     ip_owner: ip_owner.clone(),
                // })

                meta.deposit = new_deposit;
                if let Some(token_uri) = token_uri {
                    meta.token_uri = token_uri;
                }
                if let Some(base_uri) = base_uri {
                    meta.base_uri = base_uri;
                }
                if let Some(ip_owner) = ip_owner {
                    meta.ip_owner = ip_owner;
                }
                // }

                Self::deposit_event(Event::MetadataSet(
                    collection_id,
                    asset_id,
                    // name,
                    // description,
                    meta.token_uri.clone(),
                    meta.base_uri.clone(),
                    meta.ip_owner.clone(),
                ));
                Ok(().into())
            })
        }
    }
}

use frame_support::{dispatch::DispatchResultWithPostInfo, traits::Get};
use sp_runtime::DispatchResult;

// The main implementation block for the module.
impl<T: Config> Pallet<T> {
    // Public immutables

    /// Get the asset `id` balance of `who`.
    pub fn balance(
        collection_id: T::CollectionId,
        asset_id: T::AssetId,
        who: T::AccountId,
    ) -> T::Balance {
        Account::<T>::get(&collection_id, (asset_id, &who)).balance
    }

    /// Check is account owned the collection
    #[cfg(test)]
    pub fn is_collection_owner(who: &T::AccountId, collection_id: T::CollectionId) -> bool {
        Collection::<T>::get(collection_id)
            .map(|a| &a.owner == who)
            .unwrap_or(false)
    }

    /// Check is account is owner of the asset
    pub fn is_asset_owner(
        who: &T::AccountId,
        collection_id: T::CollectionId,
        asset_id: T::AssetId,
    ) -> bool {
        AccountAsset::<T>::get(collection_id, asset_id)
            .map(|a| &a == who)
            .unwrap_or(false)
    }

    /// Get the total supply of an asset `id`.
    pub fn total_asset_count(collection_id: T::CollectionId) -> u64 {
        Collection::<T>::get(collection_id)
            .map(|x| x.asset_count)
            .unwrap_or_else(Zero::zero)
    }

    /// Get the total supply of an asset `id`.
    pub fn total_token_supply(collection_id: T::CollectionId) -> T::Balance {
        Collection::<T>::get(collection_id)
            .map(|x| x.token_supply)
            .unwrap_or_else(Zero::zero)
    }

    fn do_mint_asset(
        who: T::AccountId,
        collection_id: T::CollectionId,
        asset_id: T::AssetId,
        meta: &mut CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
        name: Vec<u8>,
        description: Vec<u8>,
        token_uri: Option<Vec<u8>>,
        base_uri: Option<Vec<u8>>,
        ip_owner: Option<T::AccountId>,
    ) -> DispatchResult {
        let owned_asset_count = OwnedAssetCount::<T>::get(collection_id, &who);

        if meta.max_asset_per_account > 0 {
            ensure!(
                owned_asset_count < meta.max_asset_per_account,
                Error::<T>::MaxLimitPerAccount
            );
        }

        ensure!(
            owned_asset_count < MAX_ASSET_PER_ACCOUNT,
            Error::<T>::MaxLimitPerAccount
        );

        meta.asset_count = meta
            .asset_count
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;

        AccountAsset::<T>::insert(collection_id, asset_id, who.clone());

        OwnedAssetCount::<T>::mutate(collection_id, &who, |count| {
            *count = count.saturating_add(1);
        });

        // calculate deposit
        let mut deposit = T::MetadataDepositPerByte::get()
            .saturating_mul(((name.len() + description.len()) as u32).into())
            // storage cost calculation:
            //   deposit base + asset_index
            .saturating_add(T::MetadataDepositBase::get().saturating_add((1 as u32).into()));

        if let Some(ref token_uri) = token_uri {
            deposit = deposit.saturating_add(
                T::MetadataDepositPerByte::get().saturating_mul((token_uri.len() as u32).into()),
            );
        }

        if let Some(ref base_uri) = base_uri {
            deposit = deposit.saturating_add(
                T::MetadataDepositPerByte::get().saturating_mul((base_uri.len() as u32).into()),
            );
        }

        T::Currency::reserve(&who, deposit)?;

        Metadata::<T>::insert(
            collection_id,
            asset_id,
            AssetMetadata {
                name,
                description,
                token_uri: token_uri.unwrap_or(Vec::new()),
                base_uri: base_uri.unwrap_or(Vec::new()),
                ip_owner: ip_owner.unwrap_or_else(|| who.clone()),
                deposit,
            },
        );

        meta.asset_index = Self::next_asset_index(collection_id).ok_or(Error::<T>::Unknown)?;

        AssetIndex::<T>::insert(collection_id, meta.asset_index, asset_id);

        Self::deposit_event(Event::AssetMinted(collection_id, asset_id, who));
        Ok(())
    }

    fn do_destroy_asset(
        collection_id: T::CollectionId,
        asset_id: T::AssetId,
        meta: &mut CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
    ) -> DispatchResultWithPostInfo {
        meta.asset_count = meta.asset_count.saturating_sub(1);

        let owner = AccountAsset::<T>::mutate_exists(collection_id, asset_id, |maybe_owner| {
            maybe_owner.take().ok_or(Error::<T>::Unknown)
        })?;

        OwnedAssetCount::<T>::mutate(collection_id, &meta.owner, |count| {
            *count = count.saturating_sub(1);
        });

        Metadata::<T>::try_mutate_exists(collection_id, asset_id, |maybe_asset_meta| {
            // let meta = maybe_asset_meta.as_mut().ok_or(Error::<T>::Unknown)?;
            if let Some(ref meta) = maybe_asset_meta {
                T::Currency::unreserve(&owner, meta.deposit);
            }

            *maybe_asset_meta = None;

            Self::deposit_event(Event::Destroyed(collection_id, asset_id));

            Ok(().into())
        })
    }

    /// Check the number of zombies allow yet for an asset.
    pub fn zombie_allowance(collection_id: T::CollectionId) -> u32 {
        Collection::<T>::get(collection_id)
            .map(|x| x.max_zombies - x.zombies)
            .unwrap_or_else(Zero::zero)
    }

    fn new_account(
        who: &T::AccountId,
        d: &mut CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
    ) -> Result<bool, DispatchError> {
        let accounts = d.accounts.checked_add(1).ok_or(Error::<T>::Overflow)?;
        let r = Ok(if frame_system::Module::<T>::account_exists(who) {
            frame_system::Module::<T>::inc_consumers(who).map_err(|_| Error::<T>::BadState)?;
            false
        } else {
            ensure!(d.zombies < d.max_zombies, Error::<T>::TooManyZombies);
            // d.zombies += 1;
            d.zombies = d.zombies.saturating_add(1);
            true
        });
        d.accounts = accounts;
        r
    }

    /// If `who`` exists in system and it's a zombie, dezombify it.
    fn dezombify(
        who: &T::AccountId,
        d: &mut CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
        is_zombie: &mut bool,
    ) {
        if *is_zombie && frame_system::Module::<T>::account_exists(who) {
            // If the account exists, then it should have at least one provider
            // so this cannot fail... but being defensive anyway.
            let _ = frame_system::Module::<T>::inc_consumers(who);
            *is_zombie = false;
            d.zombies = d.zombies.saturating_sub(1);
        }
    }

    fn dead_account(
        who: &T::AccountId,
        d: &mut CollectionMetadata<T::Balance, T::AccountId, BalanceOf<T>>,
        is_zombie: bool,
    ) {
        if is_zombie {
            d.zombies = d.zombies.saturating_sub(1);
        } else {
            frame_system::Module::<T>::dec_consumers(who);
        }
        d.accounts = d.accounts.saturating_sub(1);
    }

    pub fn next_asset_index(collection_id: T::CollectionId) -> Option<u64> {
        // AssetIndex::<T>::mutate(collection_id, |m_idx| {
        //     *m_idx = Some(m_idx.map_or(1, |idx| idx.saturating_add(1)));
        //     *m_idx
        // })
        Collection::<T>::get(collection_id).map(|o| o.asset_index + 1)
    }
}
