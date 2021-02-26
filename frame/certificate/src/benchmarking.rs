//! Pallet Certificate pallet benchmarking
//!

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::Bounded;

use crate::Module as Certificate;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Module::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

// benchmarks! {
//     add_org {

//     }: _()
//     verify {
//     }

//     add_cert {
//     }: _()
//     verify {
//     }

//     issue_cert {
//     }: _()
//     verify {}
// }
