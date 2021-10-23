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

use super::*;
use crate as pallet_erc741;

use frame_support::{
    assert_err_ignore_postinfo as assert_err, assert_noop, assert_ok, parameter_types,
};
use pallet_balances::Error as BalancesError;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
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
        Assets: pallet_erc741::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
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
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const AssetDepositBase: u64 = 1;
    pub const AssetDepositPerZombie: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const StringUriLimit: u32 = 160;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl Config for Test {
    type Currency = Balances;
    type Event = Event;
    type Balance = u64;
    type CollectionId = u32;
    type AssetId = u32;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDepositBase = AssetDepositBase;
    type AssetDepositPerZombie = AssetDepositPerZombie;
    type StringLimit = StringLimit;
    type StringUriLimit = StringUriLimit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type WeightInfo = ();
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

#[test]
fn create_collection_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 10);
        assert_ok!(Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: true,
                max_token_supply: 100,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5
            }
        ));
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
    });
}

#[test]
fn invalid_name_and_symbol() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Assets::create_collection(
                Origin::signed(1),
                COLLECTION_ID,
                NewCollectionParam {
                    name: b"Test1Test1Test1Test1Test1Test1Test1Test1Test1Test11".to_vec(),
                    symbol: b"NFT".to_vec(),
                    owner: 1,
                    max_asset_count: 1000,
                    has_token: true,
                    max_token_supply: 100,
                    min_balance: 1,
                    public_mintable: true,
                    allowed_mint_accounts: Vec::new(),
                    max_asset_per_account: 0,
                    max_zombies: 5
                }
            ),
            Error::<Test>::BadMetadata
        );
        assert_noop!(
            Assets::create_collection(
                Origin::signed(1),
                COLLECTION_ID,
                NewCollectionParam {
                    name: b"123456789012345678901234567890123456789012345678901".to_vec(),
                    symbol: b"NFT".to_vec(),
                    owner: 1,
                    max_asset_count: 1000,
                    has_token: true,
                    max_token_supply: 100,
                    min_balance: 1,
                    public_mintable: true,
                    allowed_mint_accounts: Vec::new(),
                    max_asset_per_account: 0,
                    max_zombies: 5
                }
            ),
            Error::<Test>::BadMetadata
        );
    });
}

const COLLECTION_ID: u32 = 1;
const ASSET_ID: u32 = 1;

fn with_collection<F: FnOnce() -> ()>(cb: F) {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: false,
                max_token_supply: 0,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5,
            },
        )
        .expect("Cannot create asset");
        cb()
    });
}

fn with_collection_plus_token<F: FnOnce() -> ()>(cb: F) {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: true,
                max_token_supply: 200,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5,
            },
        )
        .expect("Cannot create asset");
        cb()
    });
}

fn with_minted_asset<F: FnOnce() -> ()>(cb: F) {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: false,
                max_token_supply: 0,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5,
            },
        )
        .expect("Cannot create asset");
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        cb()
    });
}

fn with_minted_asset_plus_token<F: FnOnce() -> ()>(cb: F) {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: true,
                max_token_supply: 200,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5,
            },
        )
        .expect("Cannot create asset");
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            Some(100)
        ));
        cb()
    });
}

#[test]
fn basic_destroy_collection() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_ok!(Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: true,
                max_token_supply: 100,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: vec![
                    AllowedMintAccount {
                        account: 3,
                        amount: 1
                    },
                    AllowedMintAccount {
                        account: 4,
                        amount: 1
                    },
                ],
                max_asset_per_account: 0,
                max_zombies: 5
            }
        ));
        assert_eq!(MintAllowed::<Test>::get(COLLECTION_ID, 3), Some(1));
        assert_eq!(Balances::free_balance(&1), 89);
        assert_eq!(Balances::reserved_balance(&1), 11);
        assert_eq!(Collection::<Test>::contains_key(COLLECTION_ID), true);
        assert_ok!(Assets::destroy_collection(Origin::signed(1), COLLECTION_ID));
        assert_eq!(Collection::<Test>::contains_key(COLLECTION_ID), false);
        // deposit is turned back into owner
        assert_eq!(Balances::reserved_balance(&1), 0);
        assert_eq!(Balances::free_balance(&1), 100);
        assert_eq!(MintAllowed::<Test>::get(COLLECTION_ID, 3), None);
    });
}

#[test]
fn basic_asset_minting_should_work() {
    with_collection_plus_token(|| {
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            Some(1)
        ));
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            10
        ));

        // 1 (initial mint asset) + 10 (minted) = 11
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 11);
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2,
            10
        ));
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 10);

        // check token holdings
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID + 1,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            Some(1)
        ));

        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 2);

        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 2);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &2), 0);
    });
}

#[test]
fn public_asset_minting_should_work() {
    with_collection(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
        assert_ok!(Assets::mint_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
    });
}

#[test]
fn asset_minting_deposit_calculation_works() {
    with_collection(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
        assert_ok!(Assets::mint_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID,
            b"asset #1".to_vec(),
            b"some description".to_vec(),
            Some(b"/asset-1".to_vec()),
            Some(b"https://ipfs.io".to_vec()),
            None,
            None
        ));
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
        let meta = MetadataOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("get metadata");
        let mut expected_deposit = <Test as Config>::MetadataDepositPerByte::get() * (8 + 16);
        expected_deposit = expected_deposit + <Test as Config>::MetadataDepositBase::get();
        expected_deposit =
            expected_deposit + <Test as Config>::MetadataDepositPerByte::get() * (8 + 15);
        expected_deposit += <Test as Config>::MetadataDepositPerByte::get() * (1 + 1 + 4); // indices cost
        assert_eq!(meta.deposit, expected_deposit);
        assert_eq!(meta.name, b"asset #1".to_vec());
        assert_eq!(meta.description, b"some description".to_vec());
    });
}

#[test]
fn force_minting_should_work() {
    with_collection(|| {
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
        assert_ok!(Assets::force_mint_asset(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
            1,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 1);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &2), 0);
    });
}

#[test]
fn cannot_destroy_collection_when_has_assets() {
    with_collection(|| {
        assert_ok!(Assets::force_mint_asset(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
            1,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_noop!(
            Assets::destroy_collection(Origin::signed(1), COLLECTION_ID),
            Error::<Test>::HasAssetLeft
        );
        assert_eq!(Collection::<Test>::contains_key(COLLECTION_ID), true);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
    });
}

#[test]
fn basic_transfer_asset_ownership_should_work() {
    with_minted_asset_plus_token(|| {
        Balances::make_free_balance_be(&2, 1);
        assert_ok!(Assets::transfer_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2
        ));
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 0);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &2), 1);
    });
}

#[test]
fn non_asset_owner_cannot_transfer_asset() {
    with_minted_asset_plus_token(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_noop!(
            Assets::transfer_asset(Origin::signed(2), COLLECTION_ID, ASSET_ID, 3),
            Error::<Test>::NotOwner
        );

        assert_ok!(Assets::mint_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID + 1,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            Some(100)
        ));

        // make 3 life (balance >= ED)
        Balances::make_free_balance_be(&3, 1);
        assert_ok!(Assets::transfer_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID + 1,
            3
        ));
        // dead account cannot receive transfer
        assert_noop!(
            Assets::transfer_asset(Origin::signed(2), COLLECTION_ID, ASSET_ID + 1, 4),
            BalancesError::<Test>::DeadAccount
        );
    });
}

#[test]
fn approved_to_transfer_asset_works() {
    with_minted_asset(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_ok!(Assets::approve_to_transfer(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Some(2)
        ));
        Balances::make_free_balance_be(&3, 1); // make target account alive
        assert_ok!(Assets::transfer_asset_from(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID,
            1,
            3
        ),);
        // nullify to remove approved account
        assert_ok!(Assets::approve_to_transfer(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            None
        ));
        assert_noop!(
            Assets::transfer_asset_from(Origin::signed(2), COLLECTION_ID, ASSET_ID, 1, 3),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn approved_to_transfer_token_works() {
    with_minted_asset_plus_token(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_ok!(Assets::approve_to_transfer_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Some(2)
        ));
        Balances::make_free_balance_be(&3, 1); // make target account alive
        assert_ok!(Assets::transfer_token_from(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID,
            1,
            3,
            50
        ),);
        // nullify to remove approved account
        assert_ok!(Assets::approve_to_transfer_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            None
        ));
        assert_noop!(
            Assets::transfer_token_from(Origin::signed(2), COLLECTION_ID, ASSET_ID, 1, 3, 50),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn transfer_collection_ownership() {
    with_minted_asset(|| {
        Balances::make_free_balance_be(&2, 1);
        assert_eq!(Assets::is_collection_owner(&1, COLLECTION_ID), true);
        assert_eq!(Balances::reserved_balance(&1), 16);
        assert_ok!(Assets::transfer_collection_ownership(
            Origin::signed(1),
            COLLECTION_ID,
            2
        ));
        assert_eq!(Assets::is_collection_owner(&1, COLLECTION_ID), false);
        assert_eq!(Balances::reserved_balance(&1), 7);
        assert_eq!(Assets::is_collection_owner(&2, COLLECTION_ID), true);
        assert_eq!(Balances::reserved_balance(&2), 9);
    });
}

#[test]
fn allowed_minting_mechanism_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 25); // owner
        Balances::make_free_balance_be(&2, 10); // not allowed
        Balances::make_free_balance_be(&3, 11); // allowed
        Balances::make_free_balance_be(&4, 11); // allowed
        Balances::make_free_balance_be(&5, 10); // not allowed

        assert_ok!(Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: false,
                max_token_supply: 100,
                min_balance: 1,
                public_mintable: false,
                allowed_mint_accounts: vec![
                    AllowedMintAccount {
                        account: 3,
                        amount: 1
                    },
                    AllowedMintAccount {
                        account: 4,
                        amount: 1
                    },
                ],
                max_asset_per_account: 0,
                max_zombies: 5
            }
        ));
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_eq!(MintAllowed::<Test>::get(COLLECTION_ID, &3), Some(1));
        // exclusive minting only
        assert_noop!(
            Assets::mint_asset(
                Origin::signed(2),
                COLLECTION_ID,
                ASSET_ID + 1,
                Vec::new(),
                Vec::new(),
                None,
                None,
                None,
                None
            ),
            Error::<Test>::Unauthorized,
        );
        assert_ok!(Assets::mint_asset(
            Origin::signed(3),
            COLLECTION_ID,
            ASSET_ID + 2,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_ok!(Assets::mint_asset(
            Origin::signed(4),
            COLLECTION_ID,
            ASSET_ID + 3,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_noop!(
            Assets::mint_asset(
                Origin::signed(5),
                COLLECTION_ID,
                ASSET_ID + 4,
                Vec::new(),
                Vec::new(),
                None,
                None,
                None,
                None
            ),
            Error::<Test>::Unauthorized,
        );

        // change to public mintable
        assert_ok!(Assets::update_collection(
            Origin::signed(1),
            COLLECTION_ID,
            Some(true),
            None,
            None,
            None
        ));
        // now everybody can mint
        Balances::make_free_balance_be(&10, 100);
        assert_ok!(Assets::mint_asset(
            Origin::signed(10),
            COLLECTION_ID,
            ASSET_ID + 4,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
    });
}

#[test]
fn update_collection_should_works() {
    with_minted_asset(|| {
        // change to public mintable
        assert_noop!(
            Assets::update_collection(Origin::signed(1), COLLECTION_ID, None, Some(0), None, None),
            Error::<Test>::BadMetadata
        );
        assert_noop!(
            Assets::update_collection(
                Origin::signed(1),
                COLLECTION_ID,
                None,
                Some(MAX_ASSET_PER_ACCOUNT + 1),
                None,
                None
            ),
            Error::<Test>::MaxLimitPerAccount
        );

        // change to public mintable
        assert_ok!(Assets::update_collection(
            Origin::signed(1),
            COLLECTION_ID,
            Some(true),
            Some(7),
            Some(9),
            Some(false)
        ));

        let meta = Collection::<Test>::get(COLLECTION_ID).expect("cannot get collection");
        assert_eq!(meta.public_mintable, true);
        assert_eq!(meta.max_asset_per_account, 7);
        assert_eq!(meta.min_balance, 9);
        assert_eq!(meta.has_token, false);
    });
}

#[test]
fn token_transfer_should_update_token_holders() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        // account 2 is not holder yet
        assert_eq!(ownership.token_holders.contains(&2), false);

        assert_ok!(Assets::transfer_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2,
            15
        ));
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 15);
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 105);

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        assert_eq!(ownership.token_holders.contains(&1), true);
        // account 2 is now holder
        assert_eq!(ownership.token_holders.contains(&2), true);

        // move all token from account 2 to account 3
        assert_ok!(Assets::transfer_token(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID,
            3,
            15
        ));
        assert_eq!(
            Account::<Test>::get(COLLECTION_ID, (ASSET_ID, &2)).balance,
            0
        );

        assert_eq!(
            Account::<Test>::get(COLLECTION_ID, (ASSET_ID, &3)).balance,
            15
        );

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        // account 2 is no longer holder
        assert_eq!(ownership.token_holders.contains(&2), false);
        assert_eq!(ownership.token_holders.contains(&3), true);

        // tranfser parts of token from account 3 to account 2
        assert_ok!(Assets::transfer_token(
            Origin::signed(3),
            COLLECTION_ID,
            ASSET_ID,
            2,
            5
        ));
        assert_eq!(
            Account::<Test>::get(COLLECTION_ID, (ASSET_ID, &2)).balance,
            5
        );

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        // account 2 and 3 now both holder
        assert_eq!(ownership.token_holders.contains(&2), true);
        assert_eq!(ownership.token_holders.contains(&3), true);
    });
}

#[test]
fn low_balance_cannot_transfer_token() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));

        assert_noop!(
            Assets::transfer_token(Origin::signed(2), COLLECTION_ID, ASSET_ID, 3, 15),
            Error::<Test>::TokenBalanceLow
        );
    });
}

#[test]
fn root_able_to_force_transfer_token() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));

        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("get ownership");
        assert_eq!(ownership.token_holders.contains(&2), false);
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 0);

        assert_ok!(Assets::force_transfer_token(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
            1,
            2,
            15
        ));
        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("get ownership");
        assert_eq!(ownership.token_holders.contains(&2), true);
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 15);
    });
}

#[test]
fn non_root_unable_to_force_transfer_token() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));
        assert_noop!(
            Assets::force_transfer_token(Origin::signed(2), COLLECTION_ID, ASSET_ID, 1, 2, 15),
            DispatchError::BadOrigin
        );
    });
}

// @TODO(Robin): code distribute royalties here
// #[test]
// fn distribute_royalties_work() {
//     with_minted_asset(|| {
//         assert_ok!(Assets::mint_token(
//             Origin::signed(1),
//             COLLECTION_ID,
//             ASSET_ID,
//             1,
//             3
//         ));
//         Assets::transfer
//     });
// }

// @TODO(Robin): cover collection freeze functionalities

#[test]
fn freeze_unfreeze_collection_works() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::freeze_collection(Origin::signed(1), COLLECTION_ID));
        Balances::make_free_balance_be(&2, 100);
        assert_noop!(
            Assets::transfer_asset(Origin::signed(1), COLLECTION_ID, ASSET_ID, 2),
            Error::<Test>::Frozen
        );
        assert_ok!(Assets::unfreeze_collection(
            Origin::signed(1),
            COLLECTION_ID
        ));
        assert_ok!(Assets::transfer_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2
        ));
    });
}

#[test]
fn force_freeze_unfreeze_collection_works() {
    with_minted_asset_plus_token(|| {
        assert_noop!(
            Assets::force_freeze_collection(Origin::signed(1), COLLECTION_ID),
            DispatchError::BadOrigin
        );
        assert_ok!(Assets::force_freeze_collection(
            Origin::root(),
            COLLECTION_ID
        ));
        Balances::make_free_balance_be(&2, 100);
        assert_noop!(
            Assets::transfer_asset(Origin::signed(1), COLLECTION_ID, ASSET_ID, 2),
            Error::<Test>::Frozen
        );
        assert_noop!(
            Assets::force_unfreeze_collection(Origin::signed(1), COLLECTION_ID),
            DispatchError::BadOrigin
        );
        assert_ok!(Assets::force_unfreeze_collection(
            Origin::root(),
            COLLECTION_ID
        ));
        assert_ok!(Assets::transfer_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2
        ));
    });
}

#[test]
fn enumerate_assets_via_asset_index() {
    with_minted_asset(|| {
        Balances::make_free_balance_be(&2, 100);
        assert_eq!(AssetOfIndex::<Test>::get(COLLECTION_ID, 1), Some(ASSET_ID));
        assert_eq!(AssetOfIndex::<Test>::get(COLLECTION_ID, 2), None);
        assert_ok!(Assets::mint_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID + 1,
            b"dua".to_vec(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_eq!(
            AssetOfIndex::<Test>::get(COLLECTION_ID, 2),
            Some(ASSET_ID + 1)
        );
        assert_eq!(
            AssetOfOwnerIndex::<Test>::get(COLLECTION_ID, (&1, 1)),
            Some(ASSET_ID)
        );
        assert_eq!(
            AssetOfOwnerIndex::<Test>::get(COLLECTION_ID, (&2, 1)),
            Some(ASSET_ID + 1)
        );
        assert_eq!(AssetOwnerIndex::<Test>::get(COLLECTION_ID, &1), Some(1));
        assert_eq!(AssetOwnerIndex::<Test>::get(COLLECTION_ID, &2), Some(1));
        assert_ok!(Assets::mint_asset(
            Origin::signed(2),
            COLLECTION_ID,
            ASSET_ID + 2,
            b"tiga".to_vec(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));
        assert_eq!(
            AssetOfIndex::<Test>::get(COLLECTION_ID, 3),
            Some(ASSET_ID + 2)
        );
        assert_eq!(
            AssetOfOwnerIndex::<Test>::get(COLLECTION_ID, (&1, 1)),
            Some(ASSET_ID)
        );
        assert_eq!(
            AssetOfOwnerIndex::<Test>::get(COLLECTION_ID, (&2, 2)),
            Some(ASSET_ID + 2)
        );
        assert_eq!(AssetOwnerIndex::<Test>::get(COLLECTION_ID, &1), Some(1));
        assert_eq!(AssetOwnerIndex::<Test>::get(COLLECTION_ID, &2), Some(2));
    });
}

#[test]
fn basic_collection_with_asset_token_support_should_works() {
    with_minted_asset_plus_token(|| {
        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("Couldn't get ownership");
        assert_eq!(ownership.owner, 1);
        assert_eq!(ownership.approved_to_transfer, None);
        assert_eq!(ownership.approved_to_transfer_token, None);
        assert_eq!(ownership.token_holders, vec![1]);
        // check tokens/shares for accountn 1
        assert_eq!(
            Account::<Test>::get(COLLECTION_ID, (ASSET_ID, &1)).balance,
            100
        );
    });
}

#[test]
fn asset_token_support_cannot_greather_than_token_supply() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Assets::create_collection(
            Origin::signed(1),
            COLLECTION_ID,
            NewCollectionParam {
                name: b"Test1".to_vec(),
                symbol: b"NFT".to_vec(),
                owner: 1,
                max_asset_count: 1000,
                has_token: true,
                max_token_supply: 100,
                min_balance: 1,
                public_mintable: true,
                allowed_mint_accounts: Vec::new(),
                max_asset_per_account: 0,
                max_zombies: 5,
            },
        )
        .expect("Cannot create asset");
        assert_err!(
            Assets::mint_asset(
                Origin::signed(1),
                COLLECTION_ID,
                ASSET_ID,
                Vec::new(),
                Vec::new(),
                None,
                None,
                None,
                Some(101)
            ),
            Error::<Test>::TokenBalanceMax
        );
    });
}

// #[test]
// fn lifecycle_should_work() {
//     new_test_ext().execute_with(|| {
//         Balances::make_free_balance_be(&1, 100);
//         assert_ok!(Assets::create(Origin::signed(1), 0, 1, 10, 1));
//         assert_eq!(Balances::reserved_balance(&1), 11);
//         assert!(Collectible::<Test>::contains_key(0));

//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![0],
//             vec![0],
//             12
//         ));
//         assert_eq!(Balances::reserved_balance(&1), 14);
//         assert!(MetadataOfAsset::<Test>::contains_key(0));

//         assert_ok!(Assets::mint(Origin::signed(1), 0, 10, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 20, 100));
//         assert_eq!(Account::<Test>::iter_prefix(0).count(), 2);

//         assert_ok!(Assets::destroy(Origin::signed(1), 0, 100));
//         assert_eq!(Balances::reserved_balance(&1), 0);

//         assert!(!Collectible::<Test>::contains_key(0));
//         assert!(!MetadataOfAsset::<Test>::contains_key(0));
//         assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);

//         assert_ok!(Assets::create(Origin::signed(1), 0, 1, 10, 1));
//         assert_eq!(Balances::reserved_balance(&1), 11);
//         assert!(Collectible::<Test>::contains_key(0));

//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![0],
//             vec![0],
//             12
//         ));
//         assert_eq!(Balances::reserved_balance(&1), 14);
//         assert!(MetadataOfAsset::<Test>::contains_key(0));

//         assert_ok!(Assets::mint(Origin::signed(1), 0, 10, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 20, 100));
//         assert_eq!(Account::<Test>::iter_prefix(0).count(), 2);

//         assert_ok!(Assets::force_destroy(Origin::root(), 0, 100));
//         assert_eq!(Balances::reserved_balance(&1), 0);

//         assert!(!Collectible::<Test>::contains_key(0));
//         assert!(!MetadataOfAsset::<Test>::contains_key(0));
//         assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);
//     });
// }

// #[test]
// fn destroy_with_non_zombies_should_not_work() {
//     new_test_ext().execute_with(|| {
//         Balances::make_free_balance_be(&1, 100);
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_noop!(
//             Assets::destroy(Origin::signed(1), 0, 100),
//             Error::<Test>::RefsLeft
//         );
//         assert_noop!(
//             Assets::force_destroy(Origin::root(), 0, 100),
//             Error::<Test>::RefsLeft
//         );
//         assert_ok!(Assets::burn(Origin::signed(1), 0, 1, 100));
//         assert_ok!(Assets::destroy(Origin::signed(1), 0, 100));
//     });
// }

#[test]
fn destroy_asset_should_work() {
    with_collection(|| {
        assert_ok!(Assets::mint_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            b"satu".to_vec(),
            Vec::new(),
            None,
            None,
            None,
            None
        ));

        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), true);
        assert_eq!(
            MetadataOfAsset::<Test>::contains_key(COLLECTION_ID, ASSET_ID),
            true
        );
        assert_eq!(Balances::reserved_balance(&1), 20);

        assert_ok!(Assets::destroy_asset(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
        ));

        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), false);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 0);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
        assert_eq!(
            MetadataOfAsset::<Test>::contains_key(COLLECTION_ID, ASSET_ID),
            false
        );
        assert_eq!(Balances::reserved_balance(&1), 9);
    });
}

#[test]
fn force_destroy_asset_should_work() {
    with_minted_asset(|| {
        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), true);

        assert_noop!(
            Assets::force_destroy_asset(Origin::signed(2), COLLECTION_ID, ASSET_ID,),
            DispatchError::BadOrigin
        );

        assert_ok!(Assets::force_destroy_asset(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
        ));

        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), false);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 0);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 0);
    });
}

#[test]
fn non_owner_cannot_destroy_asset() {
    with_minted_asset(|| {
        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), true);

        assert_noop!(
            Assets::destroy_asset(Origin::signed(2), COLLECTION_ID, ASSET_ID,),
            Error::<Test>::Unauthorized
        );

        assert_eq!(Assets::is_asset_owner(&1, COLLECTION_ID, ASSET_ID), true);
        assert_eq!(OwnedAssetCount::<Test>::get(COLLECTION_ID, &1), 1);
        assert_eq!(Assets::total_asset_count(COLLECTION_ID), 1);
    });
}

#[test]
fn mint_token_should_works() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));
        // 120 = 100 (initial) + 20 (mint)
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 120);
    });
}

#[test]
fn basic_token_transfer_should_work() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::transfer_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            2,
            15
        ));
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 15);
        // 85 = 100 - 15
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 85);
    });
}

#[test]
fn force_token_transfer_should_work() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));
        assert_ok!(Assets::force_transfer_token(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
            1,
            2,
            15
        ));
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 2), 15);
        // 105 = 100 (initial) + 20 (minted) - 15 (transfer)
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 105);
    });
}

#[test]
fn max_holder_limitation_works() {
    with_minted_asset_plus_token(|| {
        // we needs to mint 2 more tokens
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            1
        ));
        let mut n_i: u64 = 0;
        for i in 1..MAX_ASSET_TOKEN_HOLDERS {
            n_i = i as u64;
            assert_ok!(Assets::force_transfer_token(
                Origin::root(),
                COLLECTION_ID,
                ASSET_ID,
                1,
                1 + n_i,
                1
            ));
        }
        assert_err!(
            Assets::force_transfer_token(Origin::root(), COLLECTION_ID, ASSET_ID, 1, 1 + n_i, 1),
            Error::<Test>::MaxTokenHolder
        );
    });
}

#[test]
fn random_holders_removed_when_token_balance_is_zero() {
    with_minted_asset_plus_token(|| {
        // we needs to mint 2 more tokens

        for i in 1..(MAX_ASSET_TOKEN_HOLDERS - 1) {
            assert_ok!(Assets::force_transfer_token(
                Origin::root(),
                COLLECTION_ID,
                ASSET_ID,
                1,
                1 + i as u64,
                1
            ));
        }

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        assert_eq!(ownership.token_holders.contains(&2), true);

        assert_eq!(
            Account::<Test>::get(COLLECTION_ID, (ASSET_ID, 2)).balance,
            1
        );

        assert_ok!(Assets::force_transfer_token(
            Origin::root(),
            COLLECTION_ID,
            ASSET_ID,
            2,
            3,
            1
        ));

        let ownership = OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID)
            .expect("Cannot get asset ownership");
        // account 2 is no longer holder
        assert_eq!(ownership.token_holders.contains(&2), false);
    });
}

#[test]
fn burn_token_should_works() {
    with_minted_asset_plus_token(|| {
        assert_ok!(Assets::mint_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            20
        ));
        assert_ok!(Assets::burn_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            5
        ));
        // 115: 100 (initial) + 20 (minted) - 5 (burn)
        assert_eq!(Assets::balance(COLLECTION_ID, ASSET_ID, 1), 115);
    });
}

#[test]
fn burn_token_should_update_token_holders() {
    with_minted_asset_plus_token(|| {
        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("Couldn't get ownership");

        assert!(!ownership.token_holders.is_empty());

        assert_ok!(Assets::burn_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            99
        ));

        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("Couldn't get ownership");

        assert!(!ownership.token_holders.is_empty());

        assert_ok!(Assets::burn_token(
            Origin::signed(1),
            COLLECTION_ID,
            ASSET_ID,
            1,
            100
        ));

        let ownership =
            OwnershipOfAsset::<Test>::get(COLLECTION_ID, ASSET_ID).expect("Couldn't get ownership");

        assert_eq!(ownership.token_holders.is_empty(), true);
    });
}

// #[test]
// fn destroy_with_bad_witness_should_not_work() {
//     new_test_ext().execute_with(|| {
//         Balances::make_free_balance_be(&1, 100);
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 10, 100));
//         assert_noop!(
//             Assets::destroy(Origin::signed(1), 0, 0),
//             Error::<Test>::BadWitness
//         );
//         assert_noop!(
//             Assets::force_destroy(Origin::root(), 0, 0),
//             Error::<Test>::BadWitness
//         );
//     });
// }

// #[test]
// fn max_zombies_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 2, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 0, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));

//         assert_eq!(Assets::zombie_allowance(0), 0);
//         assert_noop!(
//             Assets::mint(Origin::signed(1), 0, 2, 100),
//             Error::<Test>::TooManyZombies
//         );
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 50),
//             Error::<Test>::TooManyZombies
//         );
//         assert_noop!(
//             Assets::force_transfer(Origin::signed(1), 0, 1, 2, 50),
//             Error::<Test>::TooManyZombies
//         );

//         Balances::make_free_balance_be(&3, 100);
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 3, 100));

//         assert_ok!(Assets::transfer(Origin::signed(0), 0, 1, 100));
//         assert_eq!(Assets::zombie_allowance(0), 1);
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//     });
// }

// #[test]
// fn resetting_max_zombies_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 2, 1));
//         Balances::make_free_balance_be(&1, 100);
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 2, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 3, 100));

//         assert_eq!(Assets::zombie_allowance(0), 0);

//         assert_noop!(
//             Assets::set_max_zombies(Origin::signed(1), 0, 1),
//             Error::<Test>::TooManyZombies
//         );

//         assert_ok!(Assets::set_max_zombies(Origin::signed(1), 0, 3));
//         assert_eq!(Assets::zombie_allowance(0), 1);
//     });
// }

// #[test]
// fn dezombifying_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 10));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::zombie_allowance(0), 9);

//         // introduce a bit of balance for account 2.
//         Balances::make_free_balance_be(&2, 100);

//         // transfer 25 units, nothing changes.
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 25));
//         assert_eq!(Assets::zombie_allowance(0), 9);

//         // introduce a bit of balance; this will create the account.
//         Balances::make_free_balance_be(&1, 100);

//         // now transferring 25 units will create it.
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 25));
//         assert_eq!(Assets::zombie_allowance(0), 10);
//     });
// }

// #[test]
// fn min_balance_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 10));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Collectible::<Test>::get(0).unwrap().accounts, 1);

//         // Cannot create a new account with a balance that is below minimum...
//         assert_noop!(
//             Assets::mint(Origin::signed(1), 0, 2, 9),
//             Error::<Test>::BalanceLow
//         );
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 9),
//             Error::<Test>::BalanceLow
//         );
//         assert_noop!(
//             Assets::force_transfer(Origin::signed(1), 0, 1, 2, 9),
//             Error::<Test>::BalanceLow
//         );

//         // When deducting from an account to below minimum, it should be reaped.

//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 91));
//         assert!(Assets::balance(0, 1).is_zero());
//         assert_eq!(Assets::balance(0, 2), 100);
//         assert_eq!(Collectible::<Test>::get(0).unwrap().accounts, 1);

//         assert_ok!(Assets::force_transfer(Origin::signed(1), 0, 2, 1, 91));
//         assert!(Assets::balance(0, 2).is_zero());
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_eq!(Collectible::<Test>::get(0).unwrap().accounts, 1);

//         assert_ok!(Assets::burn(Origin::signed(1), 0, 1, 91));
//         assert!(Assets::balance(0, 1).is_zero());
//         assert_eq!(Collectible::<Test>::get(0).unwrap().accounts, 0);
//     });
// }

// #[test]
// fn querying_total_supply_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//         assert_eq!(Assets::balance(0, 1), 50);
//         assert_eq!(Assets::balance(0, 2), 50);
//         assert_ok!(Assets::transfer(Origin::signed(2), 0, 3, 31));
//         assert_eq!(Assets::balance(0, 1), 50);
//         assert_eq!(Assets::balance(0, 2), 19);
//         assert_eq!(Assets::balance(0, 3), 31);
//         assert_ok!(Assets::burn(Origin::signed(1), 0, 3, u64::max_value()));
//         assert_eq!(Assets::total_supply(0), 69);
//     });
// }

// #[test]
// fn transferring_amount_below_available_balance_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//         assert_eq!(Assets::balance(0, 1), 50);
//         assert_eq!(Assets::balance(0, 2), 50);
//     });
// }

// #[test]
// fn transferring_frozen_user_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::freeze(Origin::signed(1), 0, 1));
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 50),
//             Error::<Test>::Frozen
//         );
//         assert_ok!(Assets::thaw(Origin::signed(1), 0, 1));
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//     });
// }

// #[test]
// fn transferring_frozen_asset_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::freeze_asset(Origin::signed(1), 0));
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 50),
//             Error::<Test>::Frozen
//         );
//         assert_ok!(Assets::thaw_asset(Origin::signed(1), 0));
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//     });
// }

// #[test]
// fn origin_guards_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_noop!(
//             Assets::transfer_collection_ownership(Origin::signed(2), 0, 2),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::set_team(Origin::signed(2), 0, 2, 2, 2),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::freeze(Origin::signed(2), 0, 1),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::thaw(Origin::signed(2), 0, 2),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::mint(Origin::signed(2), 0, 2, 100),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::burn(Origin::signed(2), 0, 1, 100),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::force_transfer(Origin::signed(2), 0, 1, 2, 100),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::set_max_zombies(Origin::signed(2), 0, 11),
//             Error::<Test>::Unauthorized,
//         );
//         assert_noop!(
//             Assets::destroy(Origin::signed(2), 0, 100),
//             Error::<Test>::Unauthorized,
//         );
//     });
// }

// #[test]
// fn transfer_owner_should_work() {
//     new_test_ext().execute_with(|| {
//         Balances::make_free_balance_be(&1, 100);
//         Balances::make_free_balance_be(&2, 1);
//         assert_ok!(Assets::create(Origin::signed(1), 0, 1, 10, 1));

//         assert_eq!(Balances::reserved_balance(&1), 11);

//         assert_ok!(Assets::transfer_collection_ownership(Origin::signed(1), 0, 2));
//         assert_eq!(Balances::reserved_balance(&2), 11);
//         assert_eq!(Balances::reserved_balance(&1), 0);

//         assert_noop!(
//             Assets::transfer_collection_ownership(Origin::signed(1), 0, 1),
//             Error::<Test>::Unauthorized,
//         );

//         assert_ok!(Assets::transfer_collection_ownership(Origin::signed(2), 0, 1));
//         assert_eq!(Balances::reserved_balance(&1), 11);
//         assert_eq!(Balances::reserved_balance(&2), 0);
//     });
// }

// #[test]
// fn set_team_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::set_team(Origin::signed(1), 0, 2, 3, 4));

//         assert_ok!(Assets::mint(Origin::signed(2), 0, 2, 100));
//         assert_ok!(Assets::freeze(Origin::signed(4), 0, 2));
//         assert_ok!(Assets::thaw(Origin::signed(3), 0, 2));
//         assert_ok!(Assets::force_transfer(Origin::signed(3), 0, 2, 3, 100));
//         assert_ok!(Assets::burn(Origin::signed(3), 0, 3, 100));
//     });
// }

// #[test]
// fn transferring_to_frozen_account_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 2, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_eq!(Assets::balance(0, 2), 100);
//         assert_ok!(Assets::freeze(Origin::signed(1), 0, 2));
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//         assert_eq!(Assets::balance(0, 2), 150);
//     });
// }

// #[test]
// fn transferring_amount_more_than_available_balance_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::transfer(Origin::signed(1), 0, 2, 50));
//         assert_eq!(Assets::balance(0, 1), 50);
//         assert_eq!(Assets::balance(0, 2), 50);
//         assert_ok!(Assets::burn(Origin::signed(1), 0, 1, u64::max_value()));
//         assert_eq!(Assets::balance(0, 1), 0);
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 1, 50),
//             Error::<Test>::BalanceLow
//         );
//         assert_noop!(
//             Assets::transfer(Origin::signed(2), 0, 1, 51),
//             Error::<Test>::BalanceLow
//         );
//     });
// }

// #[test]
// fn transferring_less_than_one_unit_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 0),
//             Error::<Test>::AmountZero
//         );
//     });
// }

// #[test]
// fn transferring_more_units_than_total_supply_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_noop!(
//             Assets::transfer(Origin::signed(1), 0, 2, 101),
//             Error::<Test>::BalanceLow
//         );
//     });
// }

// #[test]
// fn burning_asset_balance_with_positive_balance_should_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 1), 100);
//         assert_ok!(Assets::burn(Origin::signed(1), 0, 1, u64::max_value()));
//         assert_eq!(Assets::balance(0, 1), 0);
//     });
// }

// #[test]
// fn burning_asset_balance_with_zero_balance_should_not_work() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
//         assert_eq!(Assets::balance(0, 2), 0);
//         assert_noop!(
//             Assets::burn(Origin::signed(1), 0, 2, u64::max_value()),
//             Error::<Test>::BalanceZero
//         );
//     });
// }

// #[test]
// fn set_metadata_should_work() {
//     new_test_ext().execute_with(|| {
//         // Cannot add metadata to unknown asset
//         assert_noop!(
//             Assets::set_metadata(Origin::signed(1), 0, vec![0u8; 10], vec![0u8; 10], 12),
//             Error::<Test>::Unknown,
//         );
//         assert_ok!(Assets::force_create(Origin::root(), 0, 1, 10, 1));
//         // Cannot add metadata to unowned asset
//         assert_noop!(
//             Assets::set_metadata(Origin::signed(2), 0, vec![0u8; 10], vec![0u8; 10], 12),
//             Error::<Test>::Unauthorized,,
//         );

//         // Cannot add oversized metadata
//         assert_noop!(
//             Assets::set_metadata(Origin::signed(1), 0, vec![0u8; 100], vec![0u8; 10], 12),
//             Error::<Test>::BadMetadata,
//         );
//         assert_noop!(
//             Assets::set_metadata(Origin::signed(1), 0, vec![0u8; 10], vec![0u8; 100], 12),
//             Error::<Test>::BadMetadata,
//         );

//         // Successfully add metadata and take deposit
//         Balances::make_free_balance_be(&1, 30);
//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![0u8; 10],
//             vec![0u8; 10],
//             12
//         ));
//         assert_eq!(Balances::free_balance(&1), 9);

//         // Update deposit
//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![0u8; 10],
//             vec![0u8; 5],
//             12
//         ));
//         assert_eq!(Balances::free_balance(&1), 14);
//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![0u8; 10],
//             vec![0u8; 15],
//             12
//         ));
//         assert_eq!(Balances::free_balance(&1), 4);

//         // Cannot over-reserve
//         assert_noop!(
//             Assets::set_metadata(Origin::signed(1), 0, vec![0u8; 20], vec![0u8; 20], 12),
//             BalancesError::<Test, _>::InsufficientBalance,
//         );

//         // Clear MetadataOfAsset
//         assert!(MetadataOfAsset::<Test>::contains_key(0));
//         assert_ok!(Assets::set_metadata(
//             Origin::signed(1),
//             0,
//             vec![],
//             vec![],
//             0
//         ));
//         assert!(!MetadataOfAsset::<Test>::contains_key(0));
//     });
// }
