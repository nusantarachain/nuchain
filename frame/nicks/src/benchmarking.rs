//! Nicks pallet benchmarking
//!

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::Bounded;

use crate::Module as Nicks;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Module::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

benchmarks! {
    set_name {
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(caller.clone()), "Nicko Man".as_bytes().to_vec())
    verify {
        assert_last_event::<T>(Event::<T>::NameSet(caller.clone()).into());
    }

    clear_name {
        let caller: T::AccountId = whitelisted_caller();
        let caller_origin: <T as frame_system::Config>::Origin = RawOrigin::Signed(caller.clone()).into();

        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let _ = Nicks::<T>::set_name(caller_origin, "Nicko Man".as_bytes().to_vec());

    }: _(RawOrigin::Signed(caller.clone()))
    verify {
        assert_last_event::<T>(Event::<T>::NameCleared(caller.clone(), T::ReservationFee::get()).into());
    }
}
