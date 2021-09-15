// Creating mock runtime here

use crate::{self as pallet_nft, Config, Module};
use frame_support::{parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

// impl_outer_origin! {
//     pub enum Origin for Test where system = frame_system {}
// }

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Nft: pallet_nft::{Module, Call, Storage, Event<T>}
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

// impl system::Trait for Test {
//     type BaseCallFilter = ();
//     type Origin = Origin;
//     type Call = ();
//     type Index = u64;
//     type BlockNumber = u64;
//     type Hash = H256;
//     type Hashing = BlakeTwo256;
//     type AccountId = u64;
//     type Lookup = IdentityLookup<Self::AccountId>;
//     type Header = Header;
//     type Event = ();
//     type BlockHashCount = BlockHashCount;
//     type MaximumBlockWeight = MaximumBlockWeight;
//     type DbWeight = ();
//     type BlockExecutionWeight = ();
//     type ExtrinsicBaseWeight = ();
//     type MaximumExtrinsicWeight = MaximumBlockWeight;
//     type MaximumBlockLength = MaximumBlockLength;
//     type AvailableBlockRatio = AvailableBlockRatio;
//     type Version = ();
//     type PalletInfo = ();
//     type AccountData = ();
//     type OnNewAccount = ();
//     type OnKilledAccount = ();
//     type SystemWeightInfo = ();
// }

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
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const MaxCommodities: u128 = 5;
    pub const MaxCommoditiesPerUser: u64 = 2;
}

// // For testing the pallet, we construct most of a mock runtime. This means
// // first constructing a configuration type (`Test`) which `impl`s each of the
// // configuration traits of pallets we want to use.
// #[derive(Clone, Eq, PartialEq)]
// pub struct Test;

impl Config for Test {
    type Event = Event;
    type CommodityAdmin = frame_system::EnsureRoot<Self::AccountId>;
    type CommodityInfo = Vec<u8>;
    type CommodityLimit = MaxCommodities;
    type UserCommodityLimit = MaxCommoditiesPerUser;
}

// system under test
pub type SUT = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    // .into()
    let mut ext = sp_io::TestExternalities::from(storage);
    // Events are not emitted on block 0 -> advance to block 1.
    // Any dispatchable calls made during genesis block will have no events emitted.
    ext.execute_with(|| System::set_block_number(1));
    ext
}
