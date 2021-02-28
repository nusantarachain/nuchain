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
use sp_runtime::traits::StaticLookup;
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, prelude::*, vec};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};

type OrgId = u32;
type CertId = u64;

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
        name: Vec<u8>,

        /// Organization owner ID
        org_id: OrgId,
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
        CertAdded(CertId, OrgId),

        /// Some cert was issued
        CertIssued(CertId, T::AccountId),
    }

    #[pallet::storage]
    pub type Certificates<T: Config> = StorageMap<_, Blake2_128Concat, CertId, CertDetail<OrgId>>;

    #[pallet::storage]
    pub type IssuedCertificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        OrgId,
        Blake2_128Concat,
        T::AccountId,
        Vec<(
            CertId,
            Vec<u8>,
            <<T as pallet::Config>::Time as Time>::Moment,
        )>,
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
        ) -> DispatchResultWithPostInfo {
            // let sender = ensure_signed(origin)?;

            ensure!(name.len() >= 3, Error::<T>::TooShort);
            ensure!(name.len() <= 100, Error::<T>::TooLong);

            let (sender, org_ids) = T::CreatorOrigin::ensure_origin(origin)?;

            // pastikan origin adalah admin pada organisasi
            ensure!(
                org_ids.iter().any(|id| *id == org_id),
                Error::<T>::PermissionDenied
            );

            let org = T::Organization::get(org_id).ok_or(Error::<T>::NotExists)?;

            // ensure admin
            ensure!(&org.admin == &sender, Error::<T>::PermissionDenied);

            let cert_id = Self::next_cert_id();

            ensure!(
                !Certificates::<T>::contains_key(cert_id),
                Error::<T>::IdAlreadyExists
            );

            Certificates::<T>::insert(
                cert_id,
                CertDetail {
                    name: name.clone(),
                    org_id: org_id.clone(),
                },
            );

            Self::deposit_event(Event::CertAdded(cert_id, org_id));

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
            org_id: OrgId,
            cert_id: CertId,
            desc: Vec<u8>,
            target: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let _cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

            ensure!(desc.len() < 100, Error::<T>::TooLong);

            // ensure admin
            let org = T::Organization::get(org_id).ok_or(Error::<T>::Unknown)?;
            ensure!(org.admin == sender, Error::<T>::PermissionDenied);

            let target = T::Lookup::lookup(target)?;

            let collections = IssuedCertificates::<T>::get(org_id, &target);

            // pastikan penerima belum memiliki sertifikat yang dimaksud.
            ensure!(
                !collections
                    .as_ref()
                    .map(|ref o| o.iter().any(|ref v| v.0 == cert_id))
                    .unwrap_or(false),
                Error::<T>::AlreadyExists
            );

            let rv = if let Some(_colls) = collections {
                // apabila sudah pernah diisi update isinya
                // dengan ditambahkan sertifikat pada koleksi penerima.
                IssuedCertificates::<T>::try_mutate(org_id, &target, |vs| {
                    let vs = vs.as_mut().ok_or(Error::<T>::Unknown)?;
                    vs.push((cert_id, desc, T::Time::now()));
                    Ok(().into())
                })
            } else {
                // inisialisasi koleksi pertama.
                IssuedCertificates::<T>::insert(
                    org_id,
                    &target,
                    vec![(cert_id, desc, T::Time::now())],
                );
                Ok(().into())
            };

            Self::deposit_event(Event::CertIssued(cert_id, target));

            rv
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
    pub fn certificate(id: CertId) -> Option<CertDetail<OrgId>> {
        Certificates::<T>::get(id)
    }

    // /// Get current unix timestamp
    // pub fn now() -> T::Moment {
    //     // T::UnixTime::now().as_millis().saturated_into::<u64>()
    //     T::now()
    // }

    /// Get next organization ID
    pub fn next_org_id() -> u32 {
        let next_id = <OrgIdIndex<T>>::try_get().unwrap_or(0).saturating_add(1);
        <OrgIdIndex<T>>::put(next_id);
        next_id
    }
}

impl<T: Config> Pallet<T> {
    /// Get next Certificate ID
    pub fn next_cert_id() -> u64 {
        let next_id = <CertIdIndex<T>>::try_get().unwrap_or(0).saturating_add(1);
        <CertIdIndex<T>>::put(next_id);
        next_id
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
                $to,
                b"".to_vec(),
                b"".to_vec()
            ));
        };
    }

    #[test]
    fn issue_cert_should_work() {
        new_test_ext().execute_with(|| {
            create_org!(1, b"ORG1", 2);
            assert_ok!(Certificate::create_cert(
                Origin::signed(2),
                1,
                b"CERT1".to_vec()
            ));
            assert_ok!(Certificate::issue_cert(
                Origin::signed(2),
                1,
                1,
                b"DESC".to_vec(),
                2
            ));
            assert_eq!(Organization::get(2), None);
        });
    }

    #[test]
    fn only_org_admin_can_create_cert() {
        new_test_ext().execute_with(|| {
            create_org!(1, b"ORG2", 3);
            assert_noop!(
                Certificate::create_cert(Origin::signed(2), 1, b"CERT1".to_vec()),
                BadOrigin
            );
            assert_ok!(Certificate::create_cert(
                Origin::signed(3),
                1,
                b"CERT1".to_vec()
            ));
        });
    }

    #[test]
    fn only_org_admin_can_issue_cert() {
        new_test_ext().execute_with(|| {
            create_org!(1, b"ORG2", 3);
            assert_ok!(Certificate::create_cert(
                Origin::signed(3),
                1,
                b"CERT1".to_vec()
            ));
            assert_noop!(
                Certificate::issue_cert(Origin::signed(2), 1, 1, b"DESC".to_vec(), 2),
                Error::<Test>::PermissionDenied
            );
        });
    }
}
