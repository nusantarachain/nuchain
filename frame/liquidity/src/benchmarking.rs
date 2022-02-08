//! Pallet Liquidity pallet benchmarking
//!

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::Bounded;

use crate::Module as Liquidity;

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
      let id = Liquidity::next_index() + 10001;
      let amount = 1;
      let owner = 2;
    }: _(RawOrigin::Signed(caller.clone()), id, amount, owner, NETWORK_1)

    // transfer_out {
    // }: _()
    // verify {
    // }

//     lock {
//     }: _()
//     verify {}
}
