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
    RuntimeDebug, SaturatedConversion,
};
use sp_std::{fmt::Debug, prelude::*};
pub use weights::WeightInfo;

pub use pallet::*;

include! {"types.rs"}

/// global max holding limit per token per aset per account
const MAX_TOKEN_PER_ACCOUNT: u32 = 100;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
        type AssetId: Member + Parameter + Default + Copy + HasCompact;

        /// The arithmetic type of token identifier.
        type TokenId: Member + Parameter + Default + Copy + HasCompact;

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
        T::TokenId = "TokenId"
    )]
    pub enum Event<T: Config> {
        /// Some asset class was created. \[asset_id, token_id, creator, owner\]
        Created(T::AssetId, T::TokenId, T::AccountId, T::AccountId),
        /// Some assets were issued. \[asset_id, owner, total_supply\]
        Issued(T::AssetId, T::TokenId, T::AccountId, T::Balance),
        /// Some assets were transferred. \[asset_id, from, to, amount\]
        Transferred(
            T::AssetId,
            T::TokenId,
            T::AccountId,
            T::AccountId,
            T::Balance,
        ),
        /// Some assets were destroyed. \[asset_id, owner, balance\]
        Burned(T::AssetId, T::TokenId, T::AccountId, T::Balance),
        /// The management team changed \[asset_id, admin, freezer, eligible_count\]
        TeamChanged(T::AssetId, T::AccountId, T::AccountId, u32),
        /// The asset's owner changed \[asset_id, owner\]
        AssetOwnerChanged(T::AssetId, T::AccountId),
        /// The owner changed \[asset_id, token_id, owner\]
        OwnerChanged(T::AssetId, T::TokenId, T::AccountId),
        /// Some assets was transferred by an admin. \[asset_id, from, to, amount\]
        ForceTransferred(
            T::AssetId,
            T::TokenId,
            T::AccountId,
            T::AccountId,
            T::Balance,
        ),
        /// Some account `who` was frozen. \[asset_id, who\]
        Frozen(T::AssetId, T::TokenId, T::AccountId),
        /// Some account `who` was thawed. \[asset_id, who\]
        Thawed(T::AssetId, T::TokenId, T::AccountId),
        /// Some asset `asset_id` was frozen. \[asset_id\]
        AssetFrozen(T::AssetId, T::TokenId),
        /// Some asset `asset_id` was thawed. \[asset_id\]
        AssetThawed(T::AssetId, T::TokenId),
        /// An asset class was destroyed.
        Destroyed(T::AssetId, T::TokenId),
        /// Some asset class was force-created. \[asset_id, token_id, owner\]
        ForceCreated(T::AssetId, T::TokenId, T::AccountId),
        /// The maximum amount of zombies allowed has changed. \[asset_id, max_zombies\]
        MaxZombiesChanged(T::AssetId, T::TokenId, u32),
        /// New metadata has been set for an asset. \[asset_id, token_id, name, symbol, decimals\]
        MetadataSet(T::AssetId, T::TokenId, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>),
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
        NoPermission,
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
    /// Details of an asset.
    pub(super) type Asset<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        ERC721Details<T::Balance, T::AccountId, BalanceOf<T>>,
    >;

    #[pallet::storage]
    /// Details of a collectible item.
    pub(super) type Collectible<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        Blake2_128Concat,
        T::TokenId,
        ERC20Details<T::Balance, T::AccountId, T::AssetId, BalanceOf<T>>,
    >;

    #[pallet::storage]
    /// The number of units of assets held by any given account.
    pub(super) type Account<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::AssetId, T::TokenId),
        Blake2_128Concat,
        T::AccountId,
        TokenBalance<T::Balance>,
        ValueQuery,
    >;

    #[pallet::storage]
    /// The token holding per account.
    pub(super) type AccountToToken<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        Blake2_128Concat,
        T::AccountId,
        Vec<T::TokenId>,
        ValueQuery,
    >;

    #[pallet::storage]
    /// Metadata of an asset.
    pub(super) type Metadata<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        Blake2_128Concat,
        T::TokenId,
        AssetMetadata<BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Build new instance of asset class from a public origin.
        ///
        /// The origin must be Signed and the sender must have sufficient funds free.
        ///
        /// Parameters:
        /// - `name` - The name of asset.
        /// - `symbol` - The symbol of asset.
        /// - `asset_id` - ID of asset to build.
        /// - `admin` - admin of this asset.
        /// - `eligible_mint_accounts` - List of account that eligible to mint,
        ///                              if empty then will set `eligible_mint_only` to false.
        #[pallet::weight(1_000_000)]
        pub(super) fn build(
            origin: OriginFor<T>,
            name: Vec<u8>,
            symbol: Vec<u8>,
            #[pallet::compact] asset_id: T::AssetId,
            admin: <T::Lookup as StaticLookup>::Source,
            eligible_mint_accounts: Vec<T::AccountId>,
            max_token_per_account: u32,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            let admin = T::Lookup::lookup(admin)?;

            ensure!(
                name.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                symbol.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                max_token_per_account <= MAX_TOKEN_PER_ACCOUNT,
                Error::<T>::BadMetadata
            );

            ensure!(!Asset::<T>::contains_key(asset_id), Error::<T>::InUse);

            let deposit = T::MetadataDepositPerByte::get()
                .saturating_mul(((name.len() + symbol.len()) as u32).into())
                .saturating_add(T::MetadataDepositBase::get())
                .saturating_add((eligible_mint_accounts.len() as u32).into());

            T::Currency::reserve(&owner, deposit)?;

            Asset::<T>::insert(
                asset_id,
                ERC721Details {
                    name: name.clone(),
                    symbol: symbol.clone(),
                    owner: owner.clone(),
                    eligible_mint_only: eligible_mint_accounts.len() > 0,
                    eligible_mint_accounts,
                    admin: admin.clone(),
                    freezer: admin.clone(),
                    supply: Zero::zero(),
                    deposit,
                    // max_zombies,
                    // min_balance,
                    // zombies: Zero::zero(),
                    accounts: Zero::zero(),
                    is_frozen: false,
                    max_token_per_account,
                },
            );

            Ok(().into())
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
        pub(super) fn mint_token(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            admin: <T::Lookup as StaticLookup>::Source,
            max_zombies: u32,
            min_balance: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            let admin = T::Lookup::lookup(admin)?;

            ensure!(
                !Collectible::<T>::contains_key(asset_id, token_id),
                Error::<T>::InUse
            );
            ensure!(!min_balance.is_zero(), Error::<T>::MinBalanceZero);

            let details = Asset::<T>::get(asset_id).ok_or(Error::<T>::NotFound)?;

            // check is user eligible to mint this token's asset
            if details.eligible_mint_only && owner != details.owner {
                ensure!(
                    details.eligible_mint_accounts.contains(&owner),
                    Error::<T>::NoPermission
                );
            }

            let owned_token_count = AccountToToken::<T>::get(asset_id, &owner).len() as u32;

            if details.max_token_per_account > Zero::zero() {
                ensure!(
                    owned_token_count < details.max_token_per_account,
                    Error::<T>::MaxLimitPerAccount
                );
            }

            ensure!(
                owned_token_count < MAX_TOKEN_PER_ACCOUNT,
                Error::<T>::MaxLimitPerAccount
            );

            let deposit = T::AssetDepositPerZombie::get()
                .saturating_mul(max_zombies.into())
                .saturating_add(T::AssetDepositBase::get());
            T::Currency::reserve(&owner, deposit)?;

            Collectible::<T>::insert(
                asset_id,
                token_id,
                ERC20Details {
                    asset_id: asset_id.clone(),
                    owner: owner.clone(),
                    // issuer: admin.clone(),
                    // admin: admin.clone(),
                    // freezer: admin.clone(),
                    supply: Zero::zero(),
                    deposit,
                    max_zombies,
                    min_balance,
                    zombies: Zero::zero(),
                    accounts: Zero::zero(),
                    is_frozen: false,
                },
            );

            AccountToToken::<T>::try_mutate_exists::<_, _, _, Error<T>, _>(
                asset_id,
                &owner,
                |maybe_att| {
                    let mut att = maybe_att.take().unwrap_or_else(|| Vec::new());
                    att.push(token_id);
                    *maybe_att = Some(att);
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::Created(asset_id, token_id, owner, admin));
            Ok(().into())
        }

        /// Mint asset for base token from a privileged origin.
        ///
        /// This new asset class has no assets initially.
        ///
        /// The origin must conform to `ForceOrigin`.
        ///
        /// Unlike `mint_asset`, no funds are reserved.
        ///
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
        /// Emits `ForceCreated` event when successful.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::force_create())]
        pub(super) fn force_mint_token(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            owner: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] max_zombies: u32,
            #[pallet::compact] min_balance: T::Balance,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;
            let owner = T::Lookup::lookup(owner)?;

            ensure!(
                !Collectible::<T>::contains_key(asset_id, token_id),
                Error::<T>::InUse
            );
            ensure!(!min_balance.is_zero(), Error::<T>::MinBalanceZero);

            // let details = Asset::<T>::get(asset_id);

            Collectible::<T>::insert(
                asset_id,
                token_id,
                ERC20Details {
                    asset_id: asset_id.clone(),
                    owner: owner.clone(),
                    // issuer: owner.clone(),
                    // admin: owner.clone(),
                    // freezer: owner.clone(),
                    supply: Zero::zero(),
                    deposit: Zero::zero(),
                    max_zombies,
                    min_balance,
                    zombies: Zero::zero(),
                    accounts: Zero::zero(),
                    is_frozen: false,
                },
            );
            Self::deposit_event(Event::ForceCreated(asset_id, token_id, owner));
            Ok(().into())
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
        /// Weight: `O(z)` where `z` is the number of zombie accounts.
        #[pallet::weight(T::WeightInfo::destroy(*zombies_witness))]
        pub(super) fn destroy_token(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            #[pallet::compact] zombies_witness: u32,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collectible::<T>::try_mutate_exists(asset_id, token_id, |maybe_details| {
                let details = maybe_details.take().ok_or(Error::<T>::Unknown)?;
                ensure!(details.owner == origin, Error::<T>::NoPermission);
                ensure!(details.accounts == details.zombies, Error::<T>::RefsLeft);
                ensure!(details.zombies <= zombies_witness, Error::<T>::BadWitness);

                let metadata = Metadata::<T>::take(&asset_id, &token_id);
                T::Currency::unreserve(
                    &details.owner,
                    details.deposit.saturating_add(metadata.deposit),
                );

                *maybe_details = None;
                Account::<T>::remove_prefix((&asset_id, &token_id));
                Self::deposit_event(Event::Destroyed(asset_id, token_id));
                Ok(().into())
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
        #[pallet::weight(T::WeightInfo::force_destroy(*zombies_witness))]
        pub(super) fn force_destroy_token(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            #[pallet::compact] zombies_witness: u32,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            Collectible::<T>::try_mutate_exists(asset_id, token_id, |maybe_details| {
                let details = maybe_details.take().ok_or(Error::<T>::Unknown)?;
                ensure!(details.accounts == details.zombies, Error::<T>::RefsLeft);
                ensure!(details.zombies <= zombies_witness, Error::<T>::BadWitness);

                let metadata = Metadata::<T>::take(asset_id, &token_id);
                T::Currency::unreserve(
                    &details.owner,
                    details.deposit.saturating_add(metadata.deposit),
                );

                *maybe_details = None;
                Account::<T>::remove_prefix(&(asset_id, token_id));
                Self::deposit_event(Event::Destroyed(asset_id, token_id));
                Ok(().into())
            })
        }

        /// Mint sub-token from asset.
        ///
        /// The origin must be Signed.
        ///
        /// - `asset_id`: The identifier of the asset to have some sub-token minted.
        /// - `token_id`: The identifier of the token to have some amount minted.
        /// - `beneficiary`: The account to be credited with the minted assets.
        /// - `amount`: The amount of the asset to be minted.
        ///
        /// Emits `Destroyed` event when successful.
        ///
        /// Weight: `O(1)`
        /// Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`.
        #[pallet::weight(T::WeightInfo::mint())]
        pub(super) fn mint_sub_token(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            beneficiary: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let beneficiary = T::Lookup::lookup(beneficiary)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

                // only owner of token that able to mint sub-token
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);

                // // check user's eligibility to mint
                // if details.eligible_mint_only {
                //     ensure!(
                //         details.eligible_mint_accounts.contains(&origin),
                //         Error::<T>::NoPermission
                //     );
                // }

                details.supply = details
                    .supply
                    .checked_add(&amount)
                    .ok_or(Error::<T>::Overflow)?;

                Account::<T>::try_mutate(
                    (asset_id, token_id),
                    &beneficiary,
                    |t| -> DispatchResultWithPostInfo {
                        let new_balance = t.balance.saturating_add(amount);
                        ensure!(new_balance >= details.min_balance, Error::<T>::BalanceLow);
                        if t.balance.is_zero() {
                            t.is_zombie = Self::new_account(&beneficiary, details)?;
                        }
                        t.balance = new_balance;
                        Ok(().into())
                    },
                )?;
                Self::deposit_event(Event::Issued(asset_id, token_id, beneficiary, amount));
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
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let who = T::Lookup::lookup(who)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &d.owner, Error::<T>::NoPermission);

                let burned = Account::<T>::try_mutate_exists(
                    (asset_id, token_id),
                    &who,
                    |maybe_account| -> Result<T::Balance, DispatchError> {
                        let mut account = maybe_account.take().ok_or(Error::<T>::BalanceZero)?;
                        let mut burned = amount.min(account.balance);
                        account.balance -= burned;
                        *maybe_account = if account.balance < d.min_balance {
                            burned += account.balance;
                            Self::dead_account(&who, d, account.is_zombie);
                            None
                        } else {
                            Some(account)
                        };
                        Ok(burned)
                    },
                )?;

                d.supply = d.supply.saturating_sub(burned);

                Self::deposit_event(Event::Burned(asset_id, token_id, who, burned));
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
        pub(super) fn transfer(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            target: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            let mut origin_account = Account::<T>::get((asset_id, token_id), &origin);
            ensure!(!origin_account.is_frozen, Error::<T>::Frozen);
            origin_account.balance = origin_account
                .balance
                .checked_sub(&amount)
                .ok_or(Error::<T>::BalanceLow)?;

            let dest = T::Lookup::lookup(target)?;
            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(!details.is_frozen, Error::<T>::Frozen);

                if dest == origin {
                    return Ok(().into());
                }

                let mut amount = amount;
                if origin_account.balance < details.min_balance {
                    amount += origin_account.balance;
                    origin_account.balance = Zero::zero();
                }

                let acc_key = &(asset_id, token_id);

                Account::<T>::try_mutate(acc_key, &dest, |a| -> DispatchResultWithPostInfo {
                    let new_balance = a.balance.saturating_add(amount);
                    ensure!(new_balance >= details.min_balance, Error::<T>::BalanceLow);
                    if a.balance.is_zero() {
                        a.is_zombie = Self::new_account(&dest, details)?;
                    }
                    a.balance = new_balance;
                    Ok(().into())
                })?;

                match origin_account.balance.is_zero() {
                    false => {
                        Self::dezombify(&origin, details, &mut origin_account.is_zombie);
                        Account::<T>::insert(acc_key, &origin, &origin_account)
                    }
                    true => {
                        Self::dead_account(&origin, details, origin_account.is_zombie);
                        Account::<T>::remove(acc_key, &origin);
                    }
                }

                Self::deposit_event(Event::Transferred(asset_id, token_id, origin, dest, amount));
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
        pub(super) fn force_transfer(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            source: <T::Lookup as StaticLookup>::Source,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let source = T::Lookup::lookup(source)?;
            let mut source_account = Account::<T>::get(&(asset_id, token_id), &source);
            let mut amount = amount.min(source_account.balance);
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            let dest = T::Lookup::lookup(dest)?;
            if dest == source {
                return Ok(().into());
            }

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);

                source_account.balance -= amount;
                if source_account.balance < details.min_balance {
                    amount += source_account.balance;
                    source_account.balance = Zero::zero();
                }

                let acc_key = &(asset_id, token_id);

                Account::<T>::try_mutate(acc_key, &dest, |a| -> DispatchResultWithPostInfo {
                    let new_balance = a.balance.saturating_add(amount);
                    ensure!(new_balance >= details.min_balance, Error::<T>::BalanceLow);
                    if a.balance.is_zero() {
                        a.is_zombie = Self::new_account(&dest, details)?;
                    }
                    a.balance = new_balance;
                    Ok(().into())
                })?;

                match source_account.balance.is_zero() {
                    false => {
                        Self::dezombify(&source, details, &mut source_account.is_zombie);
                        Account::<T>::insert(acc_key, &source, &source_account)
                    }
                    true => {
                        Self::dead_account(&source, details, source_account.is_zombie);
                        Account::<T>::remove(acc_key, &source);
                    }
                }

                Self::deposit_event(Event::ForceTransferred(
                    asset_id, token_id, source, dest, amount,
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
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let d = Collectible::<T>::get(asset_id, token_id).ok_or(Error::<T>::Unknown)?;
            // ensure!(&origin == &d.freezer, Error::<T>::NoPermission);
            ensure!(&origin == &d.owner, Error::<T>::NoPermission);
            let who = T::Lookup::lookup(who)?;
            ensure!(
                Account::<T>::contains_key(&(asset_id, token_id), &who),
                Error::<T>::BalanceZero
            );

            Account::<T>::mutate(&(asset_id, token_id), &who, |a| a.is_frozen = true);

            Self::deposit_event(Event::<T>::Frozen(asset_id, token_id, who));
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
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let details = Collectible::<T>::get(asset_id, token_id).ok_or(Error::<T>::Unknown)?;
            // ensure!(&origin == &details.admin, Error::<T>::NoPermission);
            ensure!(&origin == &details.owner, Error::<T>::NoPermission);
            let who = T::Lookup::lookup(who)?;
            ensure!(
                Account::<T>::contains_key(&(asset_id, token_id), &who),
                Error::<T>::BalanceZero
            );

            Account::<T>::mutate(&(asset_id, token_id), &who, |a| a.is_frozen = false);

            Self::deposit_event(Event::<T>::Thawed(asset_id, token_id, who));
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
        pub(super) fn freeze_asset(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                // ensure!(&origin == &d.freezer, Error::<T>::NoPermission);
                ensure!(&origin == &d.owner, Error::<T>::NoPermission);

                d.is_frozen = true;

                Self::deposit_event(Event::<T>::AssetFrozen(asset_id, token_id));
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
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let d = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                // ensure!(&origin == &d.admin, Error::<T>::NoPermission);
                ensure!(&origin == &d.owner, Error::<T>::NoPermission);

                d.is_frozen = false;

                Self::deposit_event(Event::<T>::AssetThawed(asset_id, token_id));
                Ok(().into())
            })
        }

        /// Change the Owner of an asset.
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
            #[pallet::compact] asset_id: T::AssetId,
            owner: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let owner = T::Lookup::lookup(owner)?;

            Asset::<T>::try_mutate(asset_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);

                if details.owner == owner {
                    return Ok(().into());
                }

                // Move the deposit to the new owner.
                T::Currency::repatriate_reserved(
                    &details.owner,
                    &owner,
                    details.deposit,
                    Reserved,
                )?;

                details.owner = owner.clone();

                Self::deposit_event(Event::AssetOwnerChanged(asset_id, owner));
                Ok(().into())
            })
        }

        /// Change the Owner of an asset's sub-token.
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
        pub(super) fn transfer_token_ownership(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            owner: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let owner = T::Lookup::lookup(owner)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);
                if details.owner == owner {
                    return Ok(().into());
                }

                // Move the deposit to the new owner.
                T::Currency::repatriate_reserved(
                    &details.owner,
                    &owner,
                    details.deposit,
                    Reserved,
                )?;

                details.owner = owner.clone();

                Self::deposit_event(Event::OwnerChanged(asset_id, token_id, owner));
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
            #[pallet::compact] asset_id: T::AssetId,
            // issuer: <T::Lookup as StaticLookup>::Source,
            eligible_mint_accounts: Vec<T::AccountId>,
            admin: <T::Lookup as StaticLookup>::Source,
            freezer: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            // let issuer = T::Lookup::lookup(issuer)?;
            let admin = T::Lookup::lookup(admin)?;
            let freezer = T::Lookup::lookup(freezer)?;

            Asset::<T>::try_mutate(asset_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);

                // @TODO(robin): adjust deposit here
                // ....

                // details.issuer = issuer.clone();
                details.eligible_mint_accounts = eligible_mint_accounts.clone();
                details.admin = admin.clone();
                details.freezer = freezer.clone();

                Self::deposit_event(Event::TeamChanged(
                    asset_id,
                    admin,
                    freezer,
                    eligible_mint_accounts.len().saturated_into::<u32>(),
                ));
                Ok(().into())
            })
        }

        /// Set the maximum number of zombie accounts for an asset.
        ///
        /// Origin must be Signed and the sender should be the Owner of the asset `id`.
        ///
        /// Funds of sender are reserved according to the formula:
        /// `AssetDepositBase + AssetDepositPerZombie * max_zombies` taking into account
        /// any already reserved funds.
        ///
        /// - `id`: The identifier of the asset to update zombie count.
        /// - `max_zombies`: The new number of zombies allowed for this asset.
        ///
        /// Emits `MaxZombiesChanged`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::set_max_zombies())]
        pub(super) fn set_max_zombies(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            #[pallet::compact] max_zombies: u32,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Collectible::<T>::try_mutate(asset_id, token_id, |maybe_details| {
                let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
                ensure!(&origin == &details.owner, Error::<T>::NoPermission);
                ensure!(max_zombies >= details.zombies, Error::<T>::TooManyZombies);

                let new_deposit = T::AssetDepositPerZombie::get()
                    .saturating_mul(max_zombies.into())
                    .saturating_add(T::AssetDepositBase::get());

                if new_deposit > details.deposit {
                    T::Currency::reserve(&origin, new_deposit - details.deposit)?;
                } else {
                    T::Currency::unreserve(&origin, details.deposit - new_deposit);
                }

                details.max_zombies = max_zombies;

                Self::deposit_event(Event::MaxZombiesChanged(asset_id, token_id, max_zombies));
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
        /// - `asset_id`: The identifier of the asset to update.
        /// - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
        /// - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
        /// - `decimals`: The number of decimals this asset uses to represent one unit.
        ///
        /// Emits `MaxZombiesChanged`.
        ///
        /// Weight: `O(1)`
        #[pallet::weight(T::WeightInfo::set_metadata(name.len() as u32, symbol.len() as u32))]
        pub(super) fn set_metadata(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: T::AssetId,
            #[pallet::compact] token_id: T::TokenId,
            name: Vec<u8>,
            symbol: Vec<u8>,
            token_uri: Vec<u8>,
            base_uri: Vec<u8>,
            // decimals: u8,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            ensure!(
                name.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );
            ensure!(
                symbol.len() <= T::StringLimit::get() as usize,
                Error::<T>::BadMetadata
            );

            let d = Collectible::<T>::get(asset_id, token_id).ok_or(Error::<T>::Unknown)?;
            ensure!(&origin == &d.owner, Error::<T>::NoPermission);

            Metadata::<T>::try_mutate_exists(asset_id, token_id, |metadata| {
                let bytes_used = name.len() + symbol.len();
                let old_deposit = match metadata {
                    Some(m) => m.deposit,
                    None => Default::default(),
                };

                // Metadata is being removed
                // if bytes_used.is_zero() && decimals.is_zero() {
                if bytes_used.is_zero() {
                    T::Currency::unreserve(&origin, old_deposit);
                    *metadata = None;
                } else {
                    let new_deposit = T::MetadataDepositPerByte::get()
                        .saturating_mul(((name.len() + symbol.len()) as u32).into())
                        .saturating_add(T::MetadataDepositBase::get());

                    if new_deposit > old_deposit {
                        T::Currency::reserve(&origin, new_deposit - old_deposit)?;
                    } else {
                        T::Currency::unreserve(&origin, old_deposit - new_deposit);
                    }

                    *metadata = Some(AssetMetadata {
                        deposit: new_deposit,
                        name: name.clone(),
                        symbol: symbol.clone(),
                        token_uri: token_uri.clone(),
                        base_uri: base_uri.clone(),
                        // decimals,
                    })
                }

                Self::deposit_event(Event::MetadataSet(
                    asset_id, token_id, name, symbol, token_uri, base_uri,
                ));
                Ok(().into())
            })
        }
    }
}

// The main implementation block for the module.
impl<T: Config> Pallet<T> {
    // Public immutables

    /// Get the asset `id` balance of `who`.
    pub fn balance(asset_id: T::AssetId, token_id: T::TokenId, who: T::AccountId) -> T::Balance {
        Account::<T>::get(&(asset_id, token_id), who).balance
    }

    /// Check is account is owner
    #[cfg(test)]
    pub fn is_owner(who: &T::AccountId, asset_id: T::AssetId) -> bool {
        Asset::<T>::get(asset_id)
            .map(|a| &a.owner == who)
            .unwrap_or(false)
    }

    /// Get the total supply of an asset `id`.
    pub fn total_asset_supply(asset_id: T::AssetId) -> T::Balance {
        Asset::<T>::get(asset_id)
            .map(|x| x.supply)
            .unwrap_or_else(Zero::zero)
    }

    /// Get the total supply of an asset `id`.
    pub fn total_token_supply(asset_id: T::AssetId, token_id: T::TokenId) -> T::Balance {
        Collectible::<T>::get(asset_id, token_id)
            .map(|x| x.supply)
            .unwrap_or_else(Zero::zero)
    }

    /// Check the number of zombies allow yet for an asset.
    pub fn zombie_allowance(asset_id: T::AssetId, token_id: T::TokenId) -> u32 {
        Collectible::<T>::get(asset_id, token_id)
            .map(|x| x.max_zombies - x.zombies)
            .unwrap_or_else(Zero::zero)
    }

    fn new_account(
        who: &T::AccountId,
        d: &mut ERC20Details<T::Balance, T::AccountId, T::AssetId, BalanceOf<T>>,
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
        d: &mut ERC20Details<T::Balance, T::AccountId, T::AssetId, BalanceOf<T>>,
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
        d: &mut ERC20Details<T::Balance, T::AccountId, T::AssetId, BalanceOf<T>>,
        is_zombie: bool,
    ) {
        if is_zombie {
            d.zombies = d.zombies.saturating_sub(1);
        } else {
            frame_system::Module::<T>::dec_consumers(who);
        }
        d.accounts = d.accounts.saturating_sub(1);
    }
}
