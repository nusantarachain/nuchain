use super::*;
use crate as pallet_certificate;

use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, ord_parameter_types, parameter_types,
    traits::Time,
};
use frame_system::EnsureSignedBy;
use sp_core::{sr25519, H256};
use sp_runtime::{
    testing::Header,
    traits::{BadOrigin, BlakeTwo256, IdentityLookup},
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
        Timestamp: pallet_timestamp::{Module, Call, Storage},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Did: pallet_did::{Module, Call, Storage, Event<T>},
        Organization: pallet_organization::{Module, Call, Storage, Event<T>},
        Certificate: pallet_certificate::{Module, Call, Storage, Event<T>},
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
parameter_types! {
    pub const MinOrgNameLength: usize = 3;
    pub const MaxOrgNameLength: usize = 100;
    pub const MaxMemberCount: usize = 100;
    pub const CreationFee: u64 = 20;
}
// ord_parameter_types! {
//     pub const One: u64 = 1;
// }
parameter_types! {
    pub const MinimumPeriod: u64 = 5;
    pub const DaysUnit: u32 = 1;
}
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_did::Config for Test {
    type Event = Event;
    type Public = sr25519::Public;
    type Signature = sr25519::Signature;
    type Time = Timestamp;
    type WeightInfo = pallet_did::weights::SubstrateWeight<Self>;
}

ord_parameter_types! {
    pub const Root: sr25519::Public = sp_keyring::Sr25519Keyring::Alice.public();
}

impl pallet_organization::Config for Test {
    type Event = Event;
    type CreationFee = CreationFee;
    type Currency = Balances;
    type Payment = ();
    type ForceOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type MinOrgNameLength = MinOrgNameLength;
    type MaxOrgNameLength = MaxOrgNameLength;
    type MaxMemberCount = MaxMemberCount;
    type WeightInfo = pallet_organization::weights::SubstrateWeight<Self>;
}

impl Config for Test {
    type Event = Event;
    type ForceOrigin = EnsureSignedBy<Root, sr25519::Public>;
    type Time = Self;
    // type CreatorOrigin = pallet_organization::EnsureOrgAdmin<Self>;
    type WeightInfo = ();
}

impl Time for Test {
    type Moment = u64;
    fn now() -> Self::Moment {
        let start = std::time::SystemTime::now();
        let since_epoch = start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        since_epoch.as_millis() as u64
    }
}

type CertEvent = pallet_certificate::Event<Test>;

fn last_event() -> CertEvent {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::pallet_certificate(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .expect("Event expected")
}

// fn expect_event<E: Into<Event>>(e: E) {
//     assert_eq!(last_event(), e.into());
// }

use sp_keyring::Sr25519Keyring::{Alice, Bob, Charlie};

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(Alice.into(), 50), (Bob.into(), 10), (Charlie.into(), 20)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

macro_rules! create_org {
    ($name:literal, $to:expr) => {
        assert_ok!(Organization::create(
            Origin::signed(Alice.public()),
            $name.to_vec(),
            b"".to_vec(),
            $to,
            b"".to_vec(),
            b"".to_vec()
        ));
    };
}

fn get_last_created_cert_id() -> Option<CertId> {
    match last_event() {
        CertEvent::CertAdded(_, cert_id, _) => Some(cert_id),
        _ => None,
    }
}

fn get_last_issued_cert_id() -> Option<IssuedId> {
    match last_event() {
        CertEvent::CertIssued(cert_id, _) => Some(cert_id),
        _ => None,
    }
}

fn last_org_id() -> <Test as frame_system::Config>::AccountId {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|ev| {
            if let Event::pallet_organization(
                pallet_organization::Event::<Test>::OrganizationAdded(org_id, _),
            ) = ev
            {
                Some(org_id)
            } else {
                None
            }
        })
        .last()
        .expect("Org id expected")
}

lazy_static::lazy_static! {
    pub static ref ORG_CERT_REF: Vec<u8> = b"ORG/CERT/1".to_vec();
}

impl CertDetail<<Test as frame_system::Config>::AccountId> {
    fn new(org_id: <Test as frame_system::Config>::AccountId) -> Self {
        CertDetail {
            name: b"CERT1".to_vec(),
            description: b"CERT1 desc".to_vec(),
            org_id,
            signer_name: vec![],
        }
    }

    fn signer(mut self, signer_name: Vec<u8>) -> Self {
        self.signer_name = signer_name;
        self
    }
}

fn with_org_cert_issued<F>(func: F)
where
    F: FnOnce(<Test as frame_system::Config>::AccountId, CertId, IssuedId),
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        create_org!(b"ORG1", Bob.into());

        let org_id = last_org_id();

        assert_ok!(Certificate::create_cert(
            Origin::signed(Bob.into()),
            CertDetail::new(org_id).signer(b"Grohl".to_vec())
        ));

        let cert_id = get_last_created_cert_id().expect("cert_id of new created cert");
        println!("cert_id: {:#?}", cert_id.to_base58());
        assert_eq!(Certificate::get(&cert_id).map(|a| a.org_id), Some(org_id));
        assert_eq!(
            Certificate::get(&cert_id).map(|a| a.description),
            Some(b"CERT1 desc".to_vec())
        );
        assert_eq!(
            Certificate::get(&cert_id).map(|a| a.signer_name),
            Some(b"Grohl".to_vec())
        );

        System::set_block_number(2);

        assert_ok!(Certificate::issue_cert(
            Origin::signed(Bob.into()),
            org_id,
            cert_id,
            (*ORG_CERT_REF).clone(),
            b"Dave Grohl".to_vec(),
            b"ADDITIONAL DATA".to_vec(),
            None,
            None
        ));
        let issued_id = get_last_issued_cert_id().expect("get last issued id");
        println!("issued_id: {:?}", std::str::from_utf8(&issued_id));

        let issued_cert = Certificate::issued_cert(&issued_id).expect("issued cert");
        assert_eq!(issued_cert.cert_id, cert_id);
        assert_eq!(issued_cert.human_id, *ORG_CERT_REF);
        assert_eq!(issued_cert.recipient, b"Dave Grohl".to_vec());
        assert_eq!(
            issued_cert.additional_data,
            Some(b"ADDITIONAL DATA".to_vec())
        );

        func(org_id, cert_id, issued_id);
    })
}

#[test]
fn create_cert_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        let last_org_id = last_org_id();
        assert_ok!(Certificate::create_cert(
            Origin::signed(Bob.into()),
            CertDetail::new(last_org_id)
        ));
        match last_event() {
            CertEvent::CertAdded(index, cert_id, org_id) => {
                assert_eq!(index, 1);
                assert_eq!(cert_id.len(), 32);
                assert_eq!(org_id, last_org_id);

                assert_eq!(
                    Certificate::get(&cert_id).map(|cert| cert.name),
                    Some(b"CERT1".to_vec())
                );
                assert_eq!(
                    Certificate::get(&cert_id).map(|cert| cert.description),
                    Some(b"CERT1 desc".to_vec())
                );
                assert_eq!(
                    Certificate::get(&cert_id).map(|cert| cert.signer_name),
                    Some(vec![])
                );
            }
            _ => assert!(false, "no event"),
        }
    });
}

#[test]
fn issue_cert_with_account_handler_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        let org_id = last_org_id();
        assert_ok!(Certificate::create_cert(
            Origin::signed(Bob.into()),
            CertDetail::new(org_id)
        ));
        let cert_id = get_last_created_cert_id().unwrap();

        assert_ok!(Certificate::issue_cert(
            Origin::signed(Bob.into()),
            org_id,
            cert_id,
            (*ORG_CERT_REF).clone(),
            b"Dave".to_vec(),
            b"ADDITIONAL DATA".to_vec(),
            Some(Charlie.into()),
            None
        ));
    });
}

#[test]
fn issue_cert_should_work() {
    with_org_cert_issued(|_, _, _| {});
}

#[test]
fn cannot_create_cert_without_org() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        assert_err_ignore_postinfo!(
            Certificate::create_cert(
                Origin::signed(Bob.into()),
                // use non existent org address
                CertDetail::new(sp_keyring::Sr25519Keyring::One.into())
            ),
            Error::<Test>::OrganizationNotExists
        );
    });
}

#[test]
fn only_org_admin_can_create_cert() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG2", Charlie.into());
        assert_err_ignore_postinfo!(
            Certificate::create_cert(Origin::signed(Bob.into()), CertDetail::new(last_org_id())),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn revoke_issued_cert_should_work() {
    with_org_cert_issued(|org_id, cert_id, issued_id| {
        assert_eq!(Certificate::valid_certificate(&issued_id), true);

        assert_ok!(Certificate::revoke_certificate(
            Origin::signed(Bob.into()),
            org_id,
            issued_id.clone(),
            true
        ));

        assert_eq!(Certificate::valid_certificate(&issued_id), false);

        // balikin lagi
        assert_ok!(Certificate::revoke_certificate(
            Origin::signed(Bob.into()),
            org_id,
            issued_id.clone(),
            false
        ));

        assert_eq!(Certificate::valid_certificate(&issued_id), true);
    });
}

#[test]
fn only_org_admin_can_revoke() {
    with_org_cert_issued(|org_id, cert_id, issued_id| {
        assert_eq!(Certificate::valid_certificate(&issued_id), true);

        assert_err_ignore_postinfo!(
            Certificate::revoke_certificate(
                Origin::signed(Charlie.into()),
                org_id,
                issued_id.clone(),
                true
            ),
            Error::<Test>::PermissionDenied
        );

        assert_eq!(Certificate::valid_certificate(&issued_id), true);
    });
}
