use super::*;
use crate as pallet_certificate;

use frame_support::{
    assert_err_ignore_postinfo, assert_ok, ord_parameter_types, parameter_types,
    traits::{Time, AllowAll}, types::Text,
};
use frame_system::EnsureSignedBy;
use sp_core::{sr25519, H256};
use sp_keyring::Sr25519Keyring;
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
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Did: pallet_did::{Pallet, Call, Storage, Event<T>},
        Organization: pallet_organization::{Pallet, Call, Storage, Event<T>},
        Certificate: pallet_certificate::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
    type BaseCallFilter = AllowAll;
    type OnSetCode = ();
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
    pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
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
    type Time = Timestamp;
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
    type Time = Timestamp;
    type ForceOrigin = EnsureSignedBy<Root, sr25519::Public>;
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

type AccountId = <Test as frame_system::Config>::AccountId;
type CertEvent = pallet_certificate::Event<Test>;

fn last_event() -> CertEvent {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Certificate(inner) = e {
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
            b"".to_vec(),
            None
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
        CertEvent::CertIssued(cert_id, _, _) => Some(cert_id),
        _ => None,
    }
}

fn last_org_id() -> <Test as frame_system::Config>::AccountId {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|ev| {
            if let Event::Organization(pallet_organization::Event::<Test>::OrganizationAdded(
                org_id,
                _,
            )) = ev
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
            signer_name: None,
        }
    }

    fn signer(mut self, signer_name: Text) -> Self {
        self.signer_name = Some(signer_name);
        self
    }

    fn set_name(mut self, name: Text) -> Self {
        self.name = name;
        self
    }

    #[allow(dead_code)]
    fn set_description(mut self, description: Text) -> Self {
        self.description = description;
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

        assert_ok!(Certificate::create(
            Origin::signed(Bob.into()),
            CertDetail::new(org_id).signer(b"Grohl".to_vec())
        ));

        let cert_id = get_last_created_cert_id().expect("cert_id of new created cert");
        println!("cert_id: {:#?}", cert_id.to_base58());
        assert_eq!(Certificate::get(&cert_id).map(|a| a.org_id), Some(org_id));
        let cert = Certificate::get(&cert_id).unwrap();
        assert_eq!(cert.description, b"CERT1 desc".to_vec());
        assert_eq!(cert.signer_name, Some(b"Grohl".to_vec()));

        System::set_block_number(2);

        assert_ok!(Certificate::issue(
            Origin::signed(Bob.into()),
            org_id,
            cert_id,
            (*ORG_CERT_REF).clone(),
            b"Dave Grohl".to_vec(),
            Some(vec![Property::new(b"satu", b"1")]),
            None,
            None
        ));
        let issued_id = get_last_issued_cert_id().expect("get last issued id");
        println!("issued_id: {:?}", std::str::from_utf8(&issued_id));

        let issued_cert = Certificate::issued_cert(&issued_id).expect("issued cert");
        assert_eq!(issued_cert.cert_id, cert_id);
        assert_eq!(issued_cert.human_id, *ORG_CERT_REF);
        assert_eq!(issued_cert.recipient, b"Dave Grohl".to_vec());
        assert_eq!(issued_cert.block, 2);
        assert_eq!(issued_cert.signer_name, cert.signer_name);
        assert_eq!(issued_cert.props, Some(vec![Property::new(b"satu", b"1")]));

        func(org_id, cert_id, issued_id);
    })
}

#[test]
fn create_cert_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        let last_org_id = last_org_id();
        assert_ok!(Certificate::create(
            Origin::signed(Bob.into()),
            CertDetail::new(last_org_id)
        ));
        match last_event() {
            CertEvent::CertAdded(index, cert_id, org_id) => {
                assert_eq!(index, 1);
                assert_eq!(cert_id.len(), 32);
                assert_eq!(org_id, last_org_id);

                let cert = Certificate::get(&cert_id).unwrap();

                assert_eq!(cert.name, b"CERT1".to_vec());
                assert_eq!(cert.description, b"CERT1 desc".to_vec());
                assert_eq!(cert.signer_name, None);
            }
            _ => assert!(false, "no event"),
        }
    });
}

#[test]
fn update_cert_works() {
    with_org_cert_issued(|_org_id, cert_id, _issued_id| {
        let cert = Certificate::get(&cert_id).unwrap();
        assert_eq!(cert.signer_name, Some(b"Grohl".to_vec()));

        let new_signer_name = b"Kurt Cobain".to_vec();
        assert_ok!(Certificate::update(
            Origin::signed(Bob.into()),
            cert_id,
            new_signer_name.clone()
        ));
        let cert = Certificate::get(&cert_id).unwrap();
        assert_eq!(cert.signer_name, Some(new_signer_name));
    });
}

fn create_cert(origin: Sr25519Keyring, org_id: AccountId, name: &str) -> CertId {
    assert_ok!(Certificate::create(
        Origin::signed(origin.into()),
        CertDetail::new(org_id).set_name(name.as_bytes().to_vec())
    ));
    match last_event() {
        CertEvent::CertAdded(_index, cert_id, _org_id) => cert_id,
        _ => panic!("cannot get cert id"),
    }
}

#[test]
fn list_certs_by_organization() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        let org_id = last_org_id();
        let cert1_id = create_cert(Bob, org_id, "cert1");
        let cert2_id = create_cert(Bob, org_id, "cert2");
        let vs = Certificate::certificate_of_org(&org_id).unwrap();
        assert_eq!(vs, vec![cert1_id, cert2_id]);
        create_org!(b"ORG2", Charlie.into());
        let org_id = last_org_id();
        let cert3_id = create_cert(Charlie, org_id, "cert1");
        let vs = Certificate::certificate_of_org(&org_id).unwrap();
        assert_eq!(vs, vec![cert3_id]);
    });
}

#[test]
fn issue_cert_with_account_handler_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        let org_id = last_org_id();
        assert_ok!(Certificate::create(
            Origin::signed(Bob.into()),
            CertDetail::new(org_id)
        ));
        let cert_id = get_last_created_cert_id().unwrap();

        assert_ok!(Certificate::issue(
            Origin::signed(Bob.into()),
            org_id,
            cert_id,
            (*ORG_CERT_REF).clone(),
            b"Dave".to_vec(),
            None,
            Some(Charlie.into()),
            None
        ));
        let issued_id = get_last_issued_cert_id().unwrap();
        let account: <Test as frame_system::Config>::AccountId = Charlie.into();
        assert_eq!(
            IssuedCertOwner::<Test>::get(&org_id, &account),
            Some(vec![issued_id])
        );
    });
}

#[test]
fn issue_cert_works() {
    with_org_cert_issued(|_, _, _| {});
}

#[test]
fn cannot_create_cert_without_org() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        create_org!(b"ORG1", Bob.into());
        assert_err_ignore_postinfo!(
            Certificate::create(
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
            Certificate::create(Origin::signed(Bob.into()), CertDetail::new(last_org_id())),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn revoke_issued_cert_should_work() {
    with_org_cert_issued(|org_id, _cert_id, issued_id| {
        assert_eq!(Certificate::valid_certificate(&issued_id), true);

        assert_ok!(Certificate::revoke(
            Origin::signed(Bob.into()),
            org_id,
            issued_id.clone(),
            true
        ));

        assert_eq!(Certificate::valid_certificate(&issued_id), false);

        // balikin lagi
        assert_ok!(Certificate::revoke(
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
    with_org_cert_issued(|org_id, _cert_id, issued_id| {
        assert_eq!(Certificate::valid_certificate(&issued_id), true);

        assert_err_ignore_postinfo!(
            Certificate::revoke(
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
