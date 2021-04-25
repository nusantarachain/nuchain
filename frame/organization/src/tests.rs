// This file is part of Nuchain.
//
// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use crate as pallet_organization;

use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, ord_parameter_types, parameter_types,
};
use frame_system::EnsureSignedBy;
use sp_core::{sr25519, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Did: pallet_did::{Module, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Organization: pallet_organization::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = ();
}

impl pallet_did::Config for Test {
    type Event = Event;
    type Public = sr25519::Public;
    type Signature = sr25519::Signature;
    type Time = Timestamp;
    type WeightInfo = pallet_did::weights::SubstrateWeight<Self>;
}

parameter_types! {
    pub const MinOrgNameLength: usize = 3;
    pub const MaxOrgNameLength: usize = 16;
    pub const MaxMemberCount: usize = 3;
    pub const CreationFee: u64 = 20;
}

lazy_static::lazy_static! {
    pub static ref ALICE: sr25519::Public = sr25519::Public::from_raw([1u8; 32]);
    pub static ref BOB: sr25519::Public = sr25519::Public::from_raw([2u8; 32]);
    pub static ref CHARLIE: sr25519::Public = sr25519::Public::from_raw([3u8; 32]);
    pub static ref DAVE: sr25519::Public = sr25519::Public::from_raw([4u8; 32]);
    pub static ref EVE: sr25519::Public = sr25519::Public::from_raw([5u8; 32]);
}

ord_parameter_types! {
    pub const One: sr25519::Public = *ALICE;
    pub const Two: sr25519::Public = *BOB;
}
impl Config for Test {
    type Event = Event;
    type CreationFee = CreationFee;
    type Currency = Balances;
    type Payment = ();
    type ForceOrigin = EnsureSignedBy<One, sr25519::Public>;
    type MinOrgNameLength = MinOrgNameLength;
    type MaxOrgNameLength = MaxOrgNameLength;
    type MaxMemberCount = MaxMemberCount;
    type WeightInfo = weights::SubstrateWeight<Test>;
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(*ALICE, 50), (*BOB, 10)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

type OrgEvent = pallet_organization::Event<Test>;

fn last_event() -> OrgEvent {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::pallet_organization(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .expect("Event expected")
}

fn last_org_id() -> Option<<Test as frame_system::Config>::AccountId> {
    match last_event() {
        OrgEvent::OrganizationAdded(org_id, _) => Some(org_id),
        _ => None,
    }
}

#[test]
fn can_create_organization() {
    new_test_ext().execute_with(|| {
        assert_ok!(Organization::create(
            Origin::signed(*ALICE),
            b"ORG1".to_vec(),
            b"ORG1 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
            None
        ));
    });
}

#[test]
fn create_org_balance_deducted() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::total_balance(&*ALICE), 50);
        assert_ok!(Organization::create(
            Origin::signed(*ALICE),
            b"ORG1".to_vec(),
            b"ORG1 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
            None
        ));
        assert_eq!(Balances::total_balance(&*ALICE), 30);
    });
}

#[test]
fn insufficient_balance_cannot_create() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::total_balance(&*BOB), 10);
        assert_err_ignore_postinfo!(
            Organization::create(
                Origin::signed(*BOB),
                b"ORG2".to_vec(),
                b"ORG2 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                None
            ),
            pallet_balances::Error::<Test, _>::InsufficientBalance
        );
        assert_eq!(Balances::total_balance(&*BOB), 10);
    });
}

#[test]
fn org_id_incremented_correctly() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_eq!(Pallet::<Test>::next_index().unwrap(), 1);
        assert_ok!(Organization::create(
            Origin::signed(*ALICE),
            b"ORG2".to_vec(),
            b"ORG2 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
            None
        ));
        let org_id1 = last_org_id().unwrap();

        assert_eq!(Pallet::<Test>::next_index().unwrap(), 3);
        assert_ok!(Organization::create(
            Origin::signed(*ALICE),
            b"ORG4".to_vec(),
            b"ORG4 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
            None
        ));
        let org_id2 = last_org_id().unwrap();
        assert_eq!(Pallet::<Test>::next_index().unwrap(), 5);
        assert_eq!(Pallet::<Test>::organization(*EVE), None);
        assert!(Pallet::<Test>::organization(org_id1)
            .map(|a| &a.name == b"ORG2")
            .unwrap_or(false));
        assert!(Pallet::<Test>::organization(org_id2)
            .map(|a| &a.name == b"ORG4")
            .unwrap_or(false));
    });
}

type AccountId = <Test as frame_system::Config>::AccountId;

fn with_org<F>(func: F)
where
    F: FnOnce(AccountId, u64) -> (),
{
    assert_ok!(Organization::create(
        Origin::signed(*ALICE),
        b"ORG1".to_vec(),
        b"ORG1 DESCRIPTION".to_vec(),
        *BOB,
        b"".to_vec(),
        b"".to_vec(),
        None
    ));
    let index = <OrgIdIndex<Test>>::get().unwrap();
    func(Organization::organization_index(index).unwrap(), index);
}

#[test]
fn new_created_org_active() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_eq!(Organization::is_active(org_id), true);
            assert_eq!(Organization::is_verified(org_id), false);
            assert_eq!(Organization::is_gov(org_id), false);
            assert_eq!(Organization::is_system(org_id), false);
        });
    });
}

#[test]
fn create_organization_with_properties() {
    new_test_ext().execute_with(|| {
        let props = vec![Property::new(b"satu", b"1")];
        assert_ok!(Organization::create(
            Origin::signed(*ALICE),
            b"ORG1".to_vec(),
            b"ORG1 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
            Some(props.clone())
        ));

        let org_id = Organization::organization_index(1).unwrap();
        let org = Organization::organization(org_id).unwrap();
        assert_eq!(org.props, Some(props));
    });
}

#[test]
fn create_organization_with_too_many_props() {
    new_test_ext().execute_with(|| {
        let props = vec![
            Property::new(b"satu", b"1"),
            Property::new(b"dua", b"2"),
            Property::new(b"tiga", b"3"),
            Property::new(b"empat", b"4"),
            Property::new(b"lima", b"5"),
            Property::new(b"enam", b"6"),
        ];
        assert_noop!(
            Organization::create(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                Some(props.clone())
            ),
            Error::<Test>::TooManyProps
        );
    });
}

#[test]
fn create_organization_with_invalid_pop_name() {
    new_test_ext().execute_with(|| {
        let props = vec![Property::new(b"", b"1")];
        assert_noop!(
            Organization::create(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                Some(props.clone())
            ),
            Error::<Test>::InvalidPropName
        );
        let props = vec![Property::new(b"123456789012", b"1")];
        assert_noop!(
            Organization::create(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                Some(props.clone())
            ),
            Error::<Test>::InvalidPropName
        );
    });
}

#[test]
fn create_organization_with_invalid_prop_value() {
    new_test_ext().execute_with(|| {
        let props = vec![Property::new(
            b"1234567890",
            b"1234567890123456789012345678901234567890123456789012345678901234567890",
        )];
        assert_noop!(
            Organization::create(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                Some(props.clone())
            ),
            Error::<Test>::InvalidPropValue
        );
        let props = vec![Property::new(b"1234567890", b"")];
        assert_noop!(
            Organization::create(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec(),
                Some(props.clone())
            ),
            Error::<Test>::InvalidPropValue
        );
    });
}

#[test]
fn set_flags_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_eq!(Organization::is_verified(org_id), false);
            assert_ok!(Organization::set_flags(
                Origin::signed(*BOB),
                org_id,
                FlagDataBits(FlagDataBit::Foundation.into())
            ));
            assert_eq!(Organization::is_foundation(org_id), true);
            assert_eq!(Organization::is_gov(org_id), false);
            assert_ok!(Organization::set_flags(
                Origin::signed(*BOB),
                org_id,
                FlagDataBits(FlagDataBit::Government.into())
            ));
            assert_eq!(Organization::is_gov(org_id), true);
        });
    });
}

#[test]
fn set_flags_system_only_for_force_origin() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            // System
            assert_noop!(
                Organization::set_flags(
                    Origin::signed(*BOB),
                    org_id,
                    FlagDataBits(FlagDataBit::System.into())
                ),
                DispatchError::BadOrigin
            );
            assert_eq!(Organization::is_system(org_id), false);
            assert_ok!(Organization::set_flags(
                Origin::signed(*ALICE),
                org_id,
                FlagDataBits(FlagDataBit::System.into())
            ));
            assert_eq!(Organization::is_system(org_id), true);

            // Verified
            assert_noop!(
                Organization::set_flags(
                    Origin::signed(*BOB),
                    org_id,
                    FlagDataBits(FlagDataBit::Verified.into())
                ),
                DispatchError::BadOrigin
            );
            assert_eq!(Organization::is_verified(org_id), false);
            assert_ok!(Organization::set_flags(
                Origin::signed(*ALICE),
                org_id,
                FlagDataBits(FlagDataBit::Verified.into())
            ));
            assert_eq!(Organization::is_verified(org_id), true);
        });
    });
}

#[test]
fn add_member_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*CHARLIE]
            ));
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), true);
        });
    });
}

#[test]
fn add_member_not_allowed_by_non_org_admin() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_err_ignore_postinfo!(
                Organization::add_members(Origin::signed(*CHARLIE), org_id, vec![*BOB]),
                Error::<Test>::PermissionDenied
            );
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), false);
        });
    });
}

#[test]
fn remove_member_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*CHARLIE]
            ));
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), true);
            assert_ok!(Organization::remove_member(
                Origin::signed(*BOB),
                org_id,
                *CHARLIE
            ));
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), false);
        });
    });
}

#[test]
fn remove_member_non_admin_not_allowed() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*CHARLIE]
            ));
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), true);
            assert_err_ignore_postinfo!(
                Organization::remove_member(Origin::signed(*EVE), org_id, *CHARLIE),
                Error::<Test>::PermissionDenied
            );
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), true);
        });
    });
}

#[test]
fn add_member_max_limit() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                (5..10)
                    .map(|a| sr25519::Public::from_raw([a as u8; 32]))
                    .collect()
            ));
            assert_err_ignore_postinfo!(
                Organization::add_members(Origin::signed(*BOB), org_id, vec![*CHARLIE]),
                Error::<Test>::MaxMemberReached
            );
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), false);
        });
    });
}

#[test]
fn delegate_access_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            System::set_block_number(1);

            // berikan akses kepada DAVE
            assert_ok!(Organization::delegate_access(
                Origin::signed(*BOB),
                org_id,
                *DAVE,
                Some(5) // kasih expiration time 5 block
            ));

            // di block 3 akses masih valid
            // dan DAVE bisa add member pada organisasi BOB
            System::set_block_number(3);
            assert_ok!(Organization::add_members(
                Origin::signed(*DAVE),
                org_id,
                vec![*CHARLIE]
            ));
            assert_eq!(Organization::is_member(&org_id, &*CHARLIE), true);

            // Setelah block ke-5 akses DAVE telah expired
            System::set_block_number(6);
            assert_err_ignore_postinfo!(
                Organization::add_members(Origin::signed(*DAVE), org_id, vec![*EVE]),
                Error::<Test>::PermissionDenied
            );
            assert_eq!(Organization::is_member(&org_id, &*EVE), false);
        });
    });
}

#[test]
fn delegated_account_cannot_delegate_other_account() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            System::set_block_number(1);

            // berikan akses kepada DAVE
            assert_ok!(Organization::delegate_access(
                Origin::signed(*BOB),
                org_id,
                *DAVE,
                Some(5) // kasih expiration time 5 block
            ));

            // DAVE seharusnya tidak bisa akses fungsi delegasi
            assert_err_ignore_postinfo!(
                Organization::delegate_access(Origin::signed(*DAVE), org_id, *CHARLIE, None),
                Error::<Test>::PermissionDenied
            );
        });
    });
}

#[test]
fn cannot_delegate_when_suspended() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
            assert_err_ignore_postinfo!(
                Organization::delegate_access(Origin::signed(*BOB), org_id, *CHARLIE, None),
                Error::<Test>::Suspended
            );
        });
    });
}

#[test]
fn suspend_org_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_eq!(Organization::is_suspended(org_id), false);
            assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
            assert_eq!(Organization::is_suspended(org_id), true);
        });
    });
}

#[test]
fn only_force_origin_can_suspend() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_noop!(
                Organization::suspend_org(Origin::signed(*BOB), org_id),
                DispatchError::BadOrigin
            );
            assert_eq!(Organization::is_suspended(org_id), false);
        });
    });
}

#[test]
fn set_admin_works() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_eq!(Organization::get_admin(org_id), Some(*BOB));
            // make member first
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*CHARLIE]
            ));
            assert_ok!(Organization::set_admin(
                Origin::signed(*BOB),
                org_id,
                *CHARLIE
            ));
            assert_eq!(Organization::get_admin(org_id), Some(*CHARLIE));
        });
    });
}

#[test]
fn only_admin_or_force_origin_can_set_admin() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_eq!(Organization::get_admin(org_id), Some(*BOB));
            // make member first
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*CHARLIE]
            ));
            assert_ok!(Organization::set_admin(
                Origin::signed(*ALICE),
                org_id,
                *CHARLIE
            ));
            assert_eq!(Organization::get_admin(org_id), Some(*CHARLIE));
            // make member first
            assert_ok!(Organization::add_members(
                Origin::signed(*BOB),
                org_id,
                vec![*DAVE]
            ));
            assert_ok!(Organization::set_admin(
                Origin::signed(*CHARLIE),
                org_id,
                *DAVE
            ));
            assert_eq!(Organization::get_admin(org_id), Some(*DAVE));
            assert_noop!(
                Organization::set_admin(Origin::signed(*CHARLIE), org_id, *BOB),
                DispatchError::BadOrigin
            );
            assert_eq!(Organization::get_admin(org_id), Some(*DAVE));
        });
    });
}

#[test]
fn account_must_members_to_become_admin() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_err_ignore_postinfo!(
                Organization::set_admin(Origin::signed(*ALICE), org_id, *CHARLIE),
                Error::<Test>::NotMember
            );
        });
    });
}

#[test]
fn cannot_dispatch_suspended_operation_when_suspended() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
            assert_err_ignore_postinfo!(
                Organization::add_members(Origin::signed(*BOB), org_id, vec![*CHARLIE]),
                Error::<Test>::Suspended
            );
            assert_err_ignore_postinfo!(
                Organization::remove_member(Origin::signed(*BOB), org_id, *CHARLIE),
                Error::<Test>::Suspended
            );
            assert_err_ignore_postinfo!(
                Organization::set_flags(
                    Origin::signed(*BOB), // as org admin
                    org_id,
                    FlagDataBits(FlagDataBit::Company.into())
                ),
                Error::<Test>::Suspended
            );
        });
    });
}

#[test]
fn force_origin_can_set_flags_even_when_suspended() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
            assert_ok!(Organization::set_flags(
                Origin::signed(*ALICE),
                org_id,
                FlagDataBits(FlagDataBit::Government.into())
            ));
        });
    });
}

#[test]
fn minimum_add_members_is_one_account() {
    new_test_ext().execute_with(|| {
        with_org(|org_id, _index| {
            assert_err_ignore_postinfo!(
                Organization::add_members(Origin::signed(*BOB), org_id, vec![]),
                Error::<Test>::InvalidParameter
            );
        });
    });
}
