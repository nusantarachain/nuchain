use crate::{self as pallet_did, Config, Module};
use frame_support::{parameter_types, weights::Weight};
use frame_system as system;
use pallet_timestamp as timestamp;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
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
        // Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        // Organization: pallet_organization::{Module, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Did: pallet_did::{Module, Call, Storage, Event<T>},
    }
);

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
// #[derive(Clone, Eq, PartialEq)]
// pub struct Test;
parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

// impl system::Config for Test {
//     type BaseCallFilter = ();
//     type Origin = Origin;
//     type Call = ();
//     type Index = u64;
//     type BlockNumber = u64;
//     type Hash = H256;
//     type Hashing = BlakeTwo256;
//     type AccountId = sr25519::Public;
//     type Lookup = IdentityLookup<Self::AccountId>;
//     type Header = Header;
//     type Event = ();
//     type BlockHashCount = BlockHashCount;
//     // type MaximumBlockWeight = MaximumBlockWeight;
//     type DbWeight = ();
//     // type BlockExecutionWeight = ();
//     // type ExtrinsicBaseWeight = ();
//     // type MaximumExtrinsicWeight = MaximumBlockWeight;
//     // type MaximumBlockLength = MaximumBlockLength;
//     // type AvailableBlockRatio = AvailableBlockRatio;
//     type Version = ();
//     // type PalletInfo = frame_support::traits::PalletInfo;
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

impl timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = ();
}

impl Config for Test {
    type Event = Event;
    type Public = sr25519::Public;
    type Signature = sr25519::Signature;
    type Time = Timestamp;
    type WeightInfo = pallet_did::weights::SubstrateWeight<Self>;
}

pub type DID = Module<Test>;
// pub type System = system::Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

pub fn account_pair(s: &str) -> sr25519::Pair {
    sr25519::Pair::from_string(&format!("//{}", s), None).expect("static values are valid; qed")
}

pub fn account_key(s: &str) -> sr25519::Public {
    sr25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}
