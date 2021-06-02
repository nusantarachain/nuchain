//! # Unique Assets Implementation: Commodities
//!
//! This pallet exposes capabilities for managing unique assets, also known as
//! non-fungible tokens (NFTs).
//!
//! - [`pallet_nft::Config`](./trait.Config.html)
//! - [`Calls`](./enum.Call.html)
//! - [`Errors`](./enum.Error.html)
//! - [`Events`](./enum.Event.html)
//!
//! ## Overview
//!
//! Assets that share a common metadata structure may be created and distributed
//! by an asset admin. Asset owners may burn assets or transfer their
//! ownership. Configuration parameters are used to limit the total number of a
//! type of asset that may exist as well as the number that any one account may
//! own. Assets are uniquely identified by the hash of the info that defines
//! them, as calculated by the runtime system's hashing algorithm.
//!
//! This pallet implements the [`UniqueAssets`](./nft/trait.UniqueAssets.html)
//! trait in a way that is optimized for assets that are expected to be traded
//! frequently.
//!
//! ### Dispatchable Functions
//!
//! * [`mint`](./enum.Call.html#variant.mint) - Use the provided commodity info
//!   to create a new commodity for the specified user. May only be called by
//!   the commodity admin.
//!
//! * [`burn`](./enum.Call.html#variant.burn) - Destroy a commodity. May only be
//!   called by commodity owner.
//!
//! * [`transfer`](./enum.Call.html#variant.transfer) - Transfer ownership of
//!   a commodity to another account. May only be called by current commodity
//!   owner.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    traits::{EnsureOrigin, Get},
    Hashable,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{Hash, Member};
use sp_std::{cmp::Eq, fmt::Debug, vec::Vec};

pub mod nft;
pub use crate::nft::UniqueAssets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The dispatch origin that is able to mint new instances of this type of commodity.
        type CommodityAdmin: EnsureOrigin<Self::Origin>;
        /// The data type that is used to describe this type of commodity.
        type CommodityInfo: Hashable + Member + Debug + Default + FullCodec + Ord;
        /// The maximum number of this type of commodity that may exist (minted - burned).
        type CommodityLimit: Get<u128>;
        /// The maximum number of this type of commodity that any single account may own.
        type UserCommodityLimit: Get<u64>;
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
    }

    /// The runtime system's hashing algorithm is used to uniquely identify commodities.
    pub type CommodityId<T> = <T as frame_system::Config>::Hash;

    /// Associates a commodity with its ID.
    pub type Commodity<T, I> = (CommodityId<T>, <T as Config<I>>::CommodityInfo);

    #[pallet::storage]
    #[pallet::getter(fn total)]
    pub type Total<T, I = ()> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn burned)]
    pub type Burned<T: Config<I>, I: 'static = ()> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn total_for_account)]
    pub type TotalForAccount<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::AccountId, u64>;

    #[pallet::storage]
    #[pallet::getter(fn commodities_for_account)]
    pub type CommoditiesForAccount<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<Commodity<T, I>>>;

    #[pallet::storage]
    #[pallet::getter(fn account_for_commodity)]
    pub type AccountForCommodity<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Identity, CommodityId<T>, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// The commodity has been burned.
        Burned(CommodityId<T>),
        /// The commodity has been minted and distributed to the account.
        Minted(CommodityId<T>, T::AccountId),
        /// Ownership of the commodity has been transferred to the account.
        Transferred(CommodityId<T>, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        // Thrown when there is an attempt to mint a duplicate commodity.
        CommodityExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent commodity.
        NonexistentCommodity,
        // Thrown when someone who is not the owner of a commodity attempts to transfer or burn it.
        NotCommodityOwner,
        // Thrown when the commodity admin attempts to mint a commodity and the maximum number of this
        // type of commodity already exists.
        TooManyCommodities,
        // Thrown when an attempt is made to mint or transfer a commodity to an account that already
        // owns the maximum number of this type of commodity.
        TooManyCommoditiesForAccount,
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Create a new commodity from the provided commodity info and identify the specified
        /// account as its owner. The ID of the new commodity will be equal to the hash of the info
        /// that defines it, as calculated by the runtime system's hashing algorithm.
        ///
        /// The dispatch origin for this call must be the commodity admin.
        ///
        /// This function will throw an error if it is called with commodity info that describes
        /// an existing (duplicate) commodity, if the maximum number of this type of commodity already
        /// exists or if the specified owner already owns the maximum number of this type of
        /// commodity.
        ///
        /// - `owner_account`: Receiver of the commodity.
        /// - `commodity_info`: The information that defines the commodity.
        #[pallet::weight(100_000)]
        pub fn mint(
            origin: OriginFor<T>,
            owner_account: T::AccountId,
            commodity_info: T::CommodityInfo,
        ) -> DispatchResultWithPostInfo {
            T::CommodityAdmin::ensure_origin(origin)?;

            let commodity_id = <Self as UniqueAssets<_>>::mint(&owner_account, commodity_info)?;
            Self::deposit_event(Event::Minted(commodity_id, owner_account.clone()));
            Ok(().into())
        }

        /// Destroy the specified commodity.
        ///q
        /// The dispatch origin for this call must be the commodity owner.
        ///
        /// - `commodity_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the commodity to destroy.
        #[pallet::weight(100_000)]
        pub fn burn(
            origin: OriginFor<T>,
            commodity_id: CommodityId<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(who) == Self::account_for_commodity(&commodity_id),
                Error::<T, I>::NotCommodityOwner
            );

            <Self as UniqueAssets<_>>::burn(&commodity_id)?;
            Self::deposit_event(Event::Burned(commodity_id.clone()));
            Ok(().into())
        }

        /// Transfer a commodity to a new owner.
        ///
        /// The dispatch origin for this call must be the commodity owner.
        ///
        /// This function will throw an error if the new owner already owns the maximum
        /// number of this type of commodity.
        ///
        /// - `dest_account`: Receiver of the commodity.
        /// - `commodity_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the commodity to destroy.
        #[pallet::weight(100_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest_account: T::AccountId,
            commodity_id: CommodityId<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(who) == Self::account_for_commodity(&commodity_id),
                Error::<T, I>::NotCommodityOwner
            );

            <Self as UniqueAssets<_>>::transfer(&dest_account, &commodity_id)?;
            Self::deposit_event(Event::Transferred(
                commodity_id.clone(),
                dest_account.clone(),
            ));
            Ok(().into())
        }
    }

    // ----------------------------------------------------------------
    //                      HOOKS
    // ----------------------------------------------------------------
    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}
}

pub use pallet::*;

impl<T: Config<I>, I: 'static> UniqueAssets<T::AccountId> for Pallet<T, I> {
    type AssetId = CommodityId<T>;
    type AssetInfo = T::CommodityInfo;
    type AssetLimit = T::CommodityLimit;
    type UserAssetLimit = T::UserCommodityLimit;

    fn get_total() -> u128 {
        Self::total().unwrap_or(0)
    }

    fn get_burned() -> u128 {
        Self::burned().unwrap_or(0)
    }

    fn get_total_for_account(account: &T::AccountId) -> u64 {
        Self::total_for_account(account).unwrap_or(Default::default())
    }

    fn get_assets_for_account(account: &T::AccountId) -> Vec<Commodity<T, I>> {
        Self::commodities_for_account(account).unwrap_or(Default::default())
    }

    fn get_owner_of(commodity_id: &CommodityId<T>) -> Option<T::AccountId> {
        Self::account_for_commodity(commodity_id)
    }

    fn mint(
        owner_account: &T::AccountId,
        commodity_info: <T as Config<I>>::CommodityInfo,
    ) -> dispatch::result::Result<CommodityId<T>, dispatch::DispatchError> {
        let commodity_id = T::Hashing::hash_of(&commodity_info);

        ensure!(
            !AccountForCommodity::<T, I>::contains_key(&commodity_id),
            Error::<T, I>::CommodityExists
        );

        ensure!(
            Self::get_total_for_account(owner_account) < T::UserCommodityLimit::get(),
            Error::<T, I>::TooManyCommoditiesForAccount
        );

        ensure!(
            Self::get_total() < T::CommodityLimit::get(),
            Error::<T, I>::TooManyCommodities
        );

        let new_commodity = (commodity_id, commodity_info);

        Total::<T, I>::mutate(|total| *total = Some(total.unwrap_or(0).saturating_add(1)));
        TotalForAccount::<T, I>::mutate(owner_account, |total| {
            *total = Some(total.unwrap_or(0).saturating_add(1))
        });
        CommoditiesForAccount::<T, I>::mutate(owner_account, |commodities| {
            match commodities {
                Some(commodities) => match commodities.binary_search(&new_commodity) {
                    Ok(_pos) => {} // should never happen
                    Err(pos) => commodities.insert(pos, new_commodity),
                },
                None => {
                    *commodities = Some(vec![new_commodity]);
                }
            }
        });
        AccountForCommodity::<T, I>::insert(commodity_id, &owner_account);

        Ok(commodity_id)
    }

    fn burn(commodity_id: &CommodityId<T>) -> dispatch::DispatchResult {
        let owner = Self::get_owner_of(commodity_id);
        ensure!(
            // owner != T::AccountId::default(),
            owner.is_some(),
            Error::<T, I>::NonexistentCommodity
        );
        let owner = owner.unwrap(); // should never fail

        let burn_commodity = (*commodity_id, <T as Config<I>>::CommodityInfo::default());

        Total::<T, I>::mutate(|total| *total = Some(total.unwrap_or(0).saturating_sub(1)));
        Burned::<T, I>::mutate(|total| *total = Some(total.unwrap_or(0).saturating_add(1)));
        TotalForAccount::<T, I>::mutate(&owner, |total| {
            *total = Some(total.unwrap_or(0).saturating_sub(1))
        });
        CommoditiesForAccount::<T, I>::mutate(owner, |commodities| {
            if let Some(commodities) = commodities {
                let pos = commodities
                    .binary_search(&burn_commodity)
                    .expect("We already checked that we have the correct owner; qed");
                commodities.remove(pos);
            }
        });
        AccountForCommodity::<T, I>::remove(&commodity_id);

        Ok(())
    }

    fn transfer(
        dest_account: &T::AccountId,
        commodity_id: &CommodityId<T>,
    ) -> dispatch::DispatchResult {
        let owner = Self::get_owner_of(&commodity_id);
        ensure!(
            // owner != T::AccountId::default(),
            owner.is_some(),
            Error::<T, I>::NonexistentCommodity
        );

        let owner = owner.unwrap(); // should never fail

        ensure!(
            Self::get_total_for_account(dest_account) < T::UserCommodityLimit::get(),
            Error::<T, I>::TooManyCommoditiesForAccount
        );

        let xfer_commodity = (*commodity_id, <T as Config<I>>::CommodityInfo::default());

        TotalForAccount::<T, I>::mutate(&owner, |total| {
            *total = Some(total.unwrap_or(0).saturating_sub(1))
        });
        TotalForAccount::<T, I>::mutate(dest_account, |total| {
            *total = Some(total.unwrap_or(0).saturating_add(1))
        });
        let commodity = CommoditiesForAccount::<T, I>::mutate(owner, |commodities| {
            // let commodities = commodities.as_mut().expect("get commodities");
            if let Some(commodities) = commodities {
                let pos = commodities
                    .binary_search(&xfer_commodity)
                    .expect("We already checked that we have the correct owner; qed");
                commodities.remove(pos)
            } else {
                xfer_commodity
            }
        });
        CommoditiesForAccount::<T, I>::mutate(dest_account, |commodities| {
            if let Some(commodities) = commodities {
                match commodities.binary_search(&commodity) {
                    Ok(_pos) => {} // should never happen
                    Err(pos) => commodities.insert(pos, commodity),
                }
            } else {
                *commodities = Some(vec![commodity]);
            }
        });
        AccountForCommodity::<T, I>::insert(&commodity_id, &dest_account);

        Ok(())
    }
}
