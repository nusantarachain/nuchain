//! Pallet Liquidity pallet benchmarking
//!

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::{Bounded, Saturating};

use crate::Module as Liquidity;
// use crate::pallet::BalanceOf;

// type LEvent<T> = crate::pallet::Event<T>;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

const NETWORK_1: u32 = 1;

benchmarks! {
    transfer_in {
      let caller: T::AccountId = whitelisted_caller();

      pallet::OperatorKey::<T>::put(caller.clone());
      pallet::Locked::<T>::put(false);

      let owner:T::AccountId = account("owner", 0, 0);
      let owner_lookup = T::Lookup::unlookup(owner.clone());

      let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
      let _ = T::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

      let id:u64 = Liquidity::<T>::next_index().unwrap() + 10001u64;
      let amount = T::Currency::minimum_balance().saturating_add(10u32.into());

    }: _(RawOrigin::Signed(caller.clone()), id, amount, owner_lookup, NETWORK_1)
    verify {
      assert_last_event::<T>(Event::TransferIn(id, amount, owner.clone(), NETWORK_1).into());
    }

    transfer_out {
      pallet::Locked::<T>::put(false);

      let caller: T::AccountId = whitelisted_caller();
      // let owner:T::AccountId = account("owner", 0, 0);

      let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

      let id:u64 = Liquidity::<T>::next_index().unwrap() + 10001u64;
      let amount = T::Currency::minimum_balance().saturating_add(10u32.into());
    }: _(RawOrigin::Signed(caller.clone()), id, amount, NETWORK_1)
    verify {
      assert_last_event::<T>(Event::TransferOut(id, amount, caller.clone(), NETWORK_1).into());
    }

//     lock {
//     }: _()
//     verify {}
}
