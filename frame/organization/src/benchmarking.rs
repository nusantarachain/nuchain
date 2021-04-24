//! Pallet Organization benchmarking
//!
//!

// Run with:
// nuchain benchmark
// --chain=dev
// --steps=10
// --repeat=5
// --pallet=pallet_organization
// --extrinsic="*"
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=../../../frame/organization/src/weights.rs
// --template=../../../.maintain/frame-weight-template.hbs

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::Bounded;

use crate::{Module as Organization, OrgIdIndex, OrganizationIndexOf, Organizations};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    let events = frame_system::Module::<T>::events();
    let system_event: <T as frame_system::Config>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

const ORG_NAME: &[u8] = b"org1";
const ORG_DESC: &[u8] = b"org1 desc";
const WEBSITE: &[u8] = b"https://some.org";
const EMAIL: &[u8] = b"info@some.org";

// If you feel intimidated with the code bellow
// Please read https://substrate.dev/docs/en/knowledgebase/runtime/benchmarking
benchmarks! {
    where_clause { where
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    }

    create {
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(caller.clone()),
            ORG_NAME.to_vec(),
            ORG_DESC.to_vec(),
            caller.clone(),
            WEBSITE.to_vec(),
            EMAIL.to_vec(),
            None)
    verify {
        // assert_last_event::<T>(Event::<T>::OrganizationAdded(caller, caller));
        assert_eq!(OrgIdIndex::<T>::get(), Some(1));
    }

    suspend_org {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let _ = Organization::<T>::create(RawOrigin::Signed(caller.clone()).into(),
            ORG_NAME.to_vec(),
            ORG_DESC.to_vec(),
            caller.clone(),
            WEBSITE.to_vec(),
            EMAIL.to_vec(),
            None);
        let org_id = OrganizationIndexOf::<T>::get(1).unwrap();
    }: _(RawOrigin::Root, org_id.clone())
    verify {
        assert_eq!(Organizations::<T>::get(org_id).map(|a| a.suspended), Some(true));
    }

    // kill_org {
    // }: _()
    // verify {}



}

impl_benchmark_test_suite!(
    Organization,
    crate::tests::new_test_ext(),
    crate::tests::Test,
);
