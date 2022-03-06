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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};
use sp_runtime::traits::Bounded;
use sp_std::vec;

use crate::{Module as Organization, OrgIdIndex, OrganizationIndexOf, Organizations};
use crate::pallet::BalanceOf;

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

fn setup_org<T: Config>(caller: &T::AccountId) -> T::AccountId
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    let _ = Organization::<T>::create(
        RawOrigin::Signed(caller.clone()).into(),
        ORG_NAME.to_vec(),
        ORG_DESC.to_vec(),
        caller.clone(),
        WEBSITE.to_vec(),
        EMAIL.to_vec(),
        None,
    );
    let org_id = OrganizationIndexOf::<T>::get(1).unwrap();
    org_id
}

fn setup_org_with_members<T: Config>(caller: &T::AccountId) -> (T::AccountId, T::AccountId)
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    let org_id = setup_org::<T>(caller);
    let member_id: T::AccountId = account("Bob", 0, 0);
    let _ = Organization::<T>::add_members(
        RawOrigin::Signed(caller.clone()).into(),
        org_id.clone(),
        vec![member_id.clone()],
    );
    (org_id, member_id)
}

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

    update {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let org_id:T::AccountId = setup_org::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id,
        Some(b"newname".to_vec()),
        Some(b"newdesc".to_vec()),
        Some(b"https://test.org".to_vec()),
        Some(b"info@test.org".to_vec()),
        None
        )

    suspend_org {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let org_id:T::AccountId = setup_org::<T>(&caller);
    }: _(RawOrigin::Root, org_id.clone())
    verify {
        assert_eq!(Organizations::<T>::get(org_id).map(|a| a.suspended), Some(true));
    }

    set_flags {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let org_id = setup_org::<T>(&caller);
        let flags = FlagDataBits(FlagDataBit::Active | FlagDataBit::Edu | FlagDataBit::Foundation);
    }: _(RawOrigin::Signed(caller.clone()), org_id, flags)

    add_members {
        let n in 1 .. T::MaxMemberCount::get() as u32;
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let org_id = setup_org::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, (0..n).map(|a| account("any", 0, a)).collect())

    remove_member {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let (org_id, member_id) = setup_org_with_members::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, member_id)

    set_admin {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let (org_id, member_id) = setup_org_with_members::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, member_id)

    delegate_access {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let (org_id, member_id) = setup_org_with_members::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, member_id, Some(100u32.into()))

    revoke_access {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let (org_id, member_id) = setup_org_with_members::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, member_id)

    delegate_access_as {
        let caller = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let (org_id, member_id) = setup_org_with_members::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), org_id, member_id, b"Tracker".to_vec(), Some(100u32.into()))


}

impl_benchmark_test_suite!(
    Organization,
    crate::tests::new_test_ext(),
    crate::tests::Test,
);
