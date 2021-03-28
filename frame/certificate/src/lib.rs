//! # Pallet Certificate
//!
//! - [`nicks::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! Substrate pallet to manage online certificate
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_cert` -
//! * `issue_cert` -
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{ensure, traits::EnsureOrigin};
use frame_system::ensure_signed;
pub use pallet::*;
use sp_core::H256;
use sp_runtime::traits::{Hash, StaticLookup};
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, prelude::*, vec};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};

type OrgId = u32;
// type CertId<T> = T::Hash;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use pallet_organization::OrgProvider;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_organization::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which may forcibly set or remove a name. Root can always do this.
        type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// Time used for marking issued certificate.
        type Time: Time;

        /// Who is allowed to create certificate
        type CreatorOrigin: EnsureOrigin<Self::Origin, Success = (Self::AccountId, Vec<OrgId>)>;

        /// Organization provider
        type Organization: pallet_organization::OrgProvider<Self>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct OrgDetail<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Organization name
        name: Vec<u8>,

        /// Admin of the organization.
        admin: AccountId,

        /// Whether this organization suspended
        is_suspended: bool,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct CertDetail<OrgId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Certificate name
        pub name: Vec<u8>,

        /// Description about the certificate.
        pub description: Vec<u8>,

        /// Organization owner ID
        pub org_id: OrgId,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The object already exsits
        AlreadyExists,

        /// Name too long
        TooLong,

        /// Name too short
        TooShort,

        /// Object doesn't exist.
        NotExists,

        /// Origin has no authorization to do this operation
        PermissionDenied,

        /// ID already exists
        IdAlreadyExists,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance", OrgId = "OrgId")]
    pub enum Event<T: Config> {
        /// Some certificate added.
        CertAdded(u64, T::Hash, OrgId),

        /// Some cert was issued
        CertIssued(T::Hash, T::AccountId),
    }

    #[pallet::storage]
    pub type Certificates<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, CertDetail<OrgId>>;

    #[derive(Decode, Encode, Eq, PartialEq, RuntimeDebug)]
    pub struct CertProof<T: Config> {
        pub cert_id: T::Hash,
        pub human_id: Vec<u8>,
        pub time: <<T as pallet::Config>::Time as Time>::Moment,
    }

    #[pallet::storage]
    pub type IssuedCertificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        OrgId,
        Blake2_128Concat,
        T::Hash, // ID of issued certificate
        CertProof<T>,
    >;

    #[pallet::storage]
    pub type IssuedCertificateOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        OrgId,
        Blake2_128Concat,
        T::AccountId,
        Vec<CertProof<T>>,
    >;

    #[pallet::storage]
    pub type OrgIdIndex<T> = StorageValue<_, u32>;

    #[pallet::storage]
    pub type CertIdIndex<T> = StorageValue<_, u64>;

    /// Certificate module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create new certificate
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_cert())]
        pub(super) fn create_cert(
            origin: OriginFor<T>,
            #[pallet::compact] org_id: OrgId,
            name: Vec<u8>,
            description: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // let sender = ensure_signed(origin)?;

            ensure!(name.len() >= 3, Error::<T>::TooShort);
            ensure!(name.len() <= 100, Error::<T>::TooLong);

            ensure!(description.len() >= 3, Error::<T>::TooShort);
            ensure!(description.len() <= 1000, Error::<T>::TooLong);

            let (sender, org_ids) = T::CreatorOrigin::ensure_origin(origin)?;

            // pastikan origin adalah admin pada organisasi
            ensure!(
                org_ids.iter().any(|id| *id == org_id),
                Error::<T>::PermissionDenied
            );

            let org = T::Organization::get(org_id).ok_or(Error::<T>::NotExists)?;

            // ensure admin
            ensure!(&org.admin == &sender, Error::<T>::PermissionDenied);

            let cert_id = Self::increment_index();
            let cert_hash = Self::generate_hash(cert_id);

            ensure!(
                !Certificates::<T>::contains_key(cert_hash),
                Error::<T>::IdAlreadyExists
            );

            Certificates::<T>::insert(
                cert_hash,
                CertDetail {
                    name: name.clone(),
                    org_id: org_id.clone(),
                    description: description.clone(),
                },
            );

            Self::deposit_event(Event::CertAdded(cert_id, cert_hash, org_id));

            Ok(().into())
        }

        /// Issue certificate
        ///
        /// The dispatch origin for this call must match `T::ForceOrigin`.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(70_000_000)]
        pub(super) fn issue_cert(
            origin: OriginFor<T>,
            #[pallet::compact] org_id: OrgId,
            cert_id: T::Hash,
            notes: Vec<u8>,
            recipient: Vec<u8>,
            acc_handler: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let _cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

            ensure!(notes.len() < 100, Error::<T>::TooLong);

            // ensure admin
            let org = T::Organization::get(org_id).ok_or(Error::<T>::Unknown)?;
            ensure!(org.admin == sender, Error::<T>::PermissionDenied);

            let acc_handler = T::Lookup::lookup(acc_handler)?;

            let issue_id = T::Hashing::hash(
                &org_id
                    .to_le_bytes()
                    .into_iter()
                    .chain(cert_id.encode().iter())
                    .chain(notes.iter())
                    .chain(recipient.iter())
                    .cloned()
                    .collect::<Vec<u8>>(),
            );

            let issued = IssuedCertificates::<T>::get(org_id, &issue_id);

            // pastikan penerima belum memiliki sertifikat yang dimaksud.
            ensure!(
                !issued
                    .as_ref()
                    .map(|ref o| o.cert_id == cert_id)
                    // .map(|ref o| o.cert_id == cert_id)
                    .unwrap_or(false),
                Error::<T>::AlreadyExists
            );

            // inisialisasi koleksi pertama.
            IssuedCertificates::<T>::insert(
                org_id,
                &issue_id,
                CertProof {
                    cert_id: issued.clone(),
                    human_id: 
                },
            );

            Self::deposit_event(Event::CertIssued(cert_id, acc_handler));

            Ok(().into())
        }
    }
}

/// The main implementation of this Certificate pallet.
impl<T: Config> Pallet<T>
where
    T: pallet_timestamp::Config,
{
    /// Get detail of certificate
    ///
    pub fn get(id: &T::Hash) -> Option<CertDetail<OrgId>> {
        Certificates::<T>::get(id)
    }

    // /// Get current unix timestamp
    // pub fn now() -> T::Moment {
    //     // T::UnixTime::now().as_millis().saturated_into::<u64>()
    //     T::now()
    // }

    // /// Get next organization ID
    // pub fn next_org_id() -> u32 {
    //     let next_id = <OrgIdIndex<T>>::try_get().unwrap_or(0).saturating_add(1);
    //     <OrgIdIndex<T>>::put(next_id);
    //     next_id
    // }
}

// use sp_core::Hasher;

impl<T: Config> Pallet<T> {
    /// Incerment certificate index
    pub fn increment_index() -> u64 {
        let next_id = <CertIdIndex<T>>::try_get().unwrap_or(0).saturating_add(1);
        <CertIdIndex<T>>::put(next_id);
        next_id
    }

    /// Generate hash for randomly generated certificate identification.
    pub fn generate_hash(index: u64) -> T::Hash {
        T::Hashing::hash(&index.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_certificate;

    use frame_support::{
        assert_noop, assert_ok, ord_parameter_types, parameter_types, traits::Time,
    };
    use frame_system::EnsureSignedBy;
    use sp_core::H256;
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
            Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
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
    ord_parameter_types! {
        pub const One: u64 = 1;
    }
    parameter_types! {
        pub const MinimumPeriod: u64 = 5;
    }
    impl pallet_timestamp::Config for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
        type WeightInfo = ();
    }

    impl pallet_organization::Config for Test {
        type Event = Event;
        type CreationFee = CreationFee;
        type Currency = Balances;
        type Payment = ();
        type ForceOrigin = EnsureSignedBy<One, u64>;
        type MinOrgNameLength = MinOrgNameLength;
        type MaxOrgNameLength = MaxOrgNameLength;
        type MaxMemberCount = MaxMemberCount;
        type WeightInfo = pallet_organization::weights::SubstrateWeight<Self>;
    }

    impl Config for Test {
        type Event = Event;
        type ForceOrigin = EnsureSignedBy<One, u64>;
        type Time = Self;
        type CreatorOrigin = pallet_organization::EnsureOrgAdmin<Self>;
        type Organization = pallet_organization::Pallet<Self>;
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

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 50), (2, 10), (3, 20)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    macro_rules! create_org {
        ($o:expr, $name:literal, $to:expr) => {
            assert_ok!(Organization::create_org(
                Origin::signed(1),
                $name.to_vec(),
                b"".to_vec(),
                $to,
                b"".to_vec(),
                b"".to_vec()
            ));
        };
    }

    fn get_last_created_cert_hash() -> Option<<Test as frame_system::Config>::Hash> {
        match last_event() {
            CertEvent::CertAdded(_, hash, _) => Some(hash),
            _ => None,
        }
    }

    #[test]
    fn issue_cert_should_work() {
        new_test_ext().execute_with(|| {
            System::set_block_number(1);
            create_org!(1, b"ORG1", 2);
            assert_ok!(Certificate::create_cert(
                Origin::signed(2),
                1,
                b"CERT1".to_vec(),
                b"CERT1 description".to_vec()
            ));
            // let event = last_event();
            // println!("EVENT: {:#?}", event);
            let hash = get_last_created_cert_hash().expect("Hash of new created cert");
            println!("hash: {:#?}", hash);
            assert_eq!(Certificate::get(&hash).map(|a| a.org_id), Some(1));
            // assert_eq!(event, TestEvent::CertAdded(0, 1, 2));
            // let cert_id = match event {
            //     Event::CertAdded(index, hash, _) => hash,
            //     _ => 0
            // };
            // assert_ok!(Certificate::issue_cert(
            //     Origin::signed(2),
            //     1,
            //     1,
            //     b"DESC".to_vec(),
            //     2
            // ));
            // assert_eq!(Organization::get(2), None);
        });
    }

    #[test]
    fn only_org_admin_can_create_cert() {
        new_test_ext().execute_with(|| {
            create_org!(1, b"ORG2", 3);
            assert_noop!(
                Certificate::create_cert(Origin::signed(2), 1, b"CERT1".to_vec(), b"CERT1 descriptor".to_vec()),
                BadOrigin
            );
            assert_ok!(Certificate::create_cert(
                Origin::signed(3),
                1,
                b"CERT1".to_vec(),
                b"CERT1 description".to_vec()
            ));
        });
    }

    // #[test]
    // fn only_org_admin_can_issue_cert() {
    //     new_test_ext().execute_with(|| {
    //         create_org!(1, b"ORG2", 3);
    //         assert_ok!(Certificate::create_cert(
    //             Origin::signed(3),
    //             1,
    //             b"CERT1".to_vec()
    //         ));
    //         assert_noop!(
    //             Certificate::issue_cert(Origin::signed(2), 1, 1, b"DESC".to_vec(), 2),
    //             Error::<Test>::PermissionDenied
    //         );
    //     });
    // }
}
