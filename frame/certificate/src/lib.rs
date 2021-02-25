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
//! * `add_org` -
//! * `add_cert` -
//! * `issue_cert` -
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchError,
    ensure,
    traits::{Currency, EnsureOrigin, Get, OnUnbalanced, ReservableCurrency, UnixTime},
};
use frame_system::ensure_signed;
use sp_runtime::RuntimeDebug;
use sp_runtime::{
    traits::{StaticLookup, Zero},
    SaturatedConversion,
};
use sp_std::{fmt::Debug, prelude::*, vec};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode, HasCompact};


#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which may forcibly set or remove a name. Root can always do this.
        type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// The minimum length a name may be.
        type MinOrgNameLength: Get<usize>;

        /// The maximum length a name may be.
        type MaxOrgNameLength: Get<usize>;

        /// The ID of organization
        type OrgId: Member + Parameter + Default + Copy + HasCompact;

        /// Certificate ID
        type CertId: Member + Parameter + Default + Copy + HasCompact;

        /// Time used for marking issued certificate.
        type UnixTime: UnixTime;

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

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance", T::OrgId = "OrgId")]
    pub enum Event<T: Config> {
        /// Some organization added inside the system.
        OrgAdded(T::OrgId, T::AccountId),

        /// Some certificate added.
        CertAdded(T::CertId, T::OrgId),

        /// Some cert was issued
        CertIssued(T::CertId, T::AccountId),
    }

    #[pallet::storage]
    pub type Organizations<T: Config> =
        StorageMap<_, Blake2_128Concat, T::OrgId, OrgDetail<T::AccountId>>;

    #[pallet::storage]
    pub type Certificates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CertId, CertDetail<T::OrgId>>;

    #[pallet::storage]
    pub type IssuedCertificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::OrgId,
        Blake2_128Concat,
        T::AccountId,
        Vec<(T::CertId, Vec<u8>, u64)>,
    >;

    /// Certificate module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add new organization.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(T::WeightInfo::add_org())]
        fn add_org(
            origin: OriginFor<T>,
            #[pallet::compact] id: T::OrgId,
            name: Vec<u8>,
            admin: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let _origin = T::ForceOrigin::ensure_origin(origin)?;

            ensure!(
                name.len() >= T::MinOrgNameLength::get(),
                Error::<T>::TooShort
            );
            ensure!(
                name.len() <= T::MaxOrgNameLength::get(),
                Error::<T>::TooLong
            );

            ensure!(
                !Organizations::<T>::contains_key(id),
                Error::<T>::AlreadyExists
            );

            let admin = T::Lookup::lookup(admin)?;

            Organizations::<T>::insert(
                id,
                OrgDetail {
                    name: name.clone(),
                    admin: admin.clone(),
                    is_suspended: false,
                },
            );

            Self::deposit_event(Event::OrgAdded(id, admin));

            Ok(().into())
        }

        /// Create new certificate
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(T::WeightInfo::add_cert())]
        fn add_cert(
            origin: OriginFor<T>,
            #[pallet::compact] org_id: T::OrgId,
            cert_id: T::CertId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(name.len() >= 3, Error::<T>::TooShort);
            ensure!(name.len() <= 16, Error::<T>::TooLong);

            ensure!(
                Organizations::<T>::contains_key(org_id),
                Error::<T>::NotExists
            );

            // ensure admin
            let org = Organizations::<T>::get(org_id).ok_or(Error::<T>::Unknown)?;

            ensure!(&org.admin == &sender, Error::<T>::PermissionDenied);

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
        fn issue_cert(
            origin: OriginFor<T>,
            org_id: T::OrgId,
            cert_id: T::CertId,
            desc: Vec<u8>,
            target: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

            ensure!(desc.len() < 100, Error::<T>::TooLong);

            // ensure admin
            let org = Organizations::<T>::get(org_id).ok_or(Error::<T>::Unknown)?;
            ensure!(org.admin == sender, Error::<T>::PermissionDenied);

            let target = T::Lookup::lookup(target)?;

            // check is already exists
            ensure!(
                !IssuedCertificates::<T>::contains_key(org_id, &target),
                Error::<T>::AlreadyExists
            );

            IssuedCertificates::<T>::insert(org_id, &target, vec![(cert_id, desc, Self::now())]);

            Self::deposit_event(Event::CertIssued(cert_id, target));

            Ok(().into())
        }
    }
}

/// The main implementation of this Certificate pallet.
impl<T: Config> Pallet<T> {
    /// Get the organization detail
    pub fn organization(id: T::OrgId) -> Option<OrgDetail<T::AccountId>> {
        Organizations::<T>::get(id)
    }

    /// Get detail of certificate
    ///
    pub fn certificate(id: T::CertId) -> Option<CertDetail<T::OrgId>> {
        Certificates::<T>::get(id)
    }

    /// Get current unix timestamp
    pub fn now() -> u64 {
        T::UnixTime::now().as_millis().saturated_into::<u64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_certificate;

    use frame_support::{assert_noop, assert_ok, ord_parameter_types, parameter_types};
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
        pub const ReservationFee: u64 = 2;
        pub const MinOrgNameLength: usize = 3;
        pub const MaxOrgNameLength: usize = 16;
    }
    ord_parameter_types! {
        pub const One: u64 = 1;
    }
    impl Config for Test {
        type Event = Event;
        type Currency = Balances;
        type ReservationFee = ReservationFee;
        type Slashed = ();
        type ForceOrigin = EnsureSignedBy<One, u64>;
        type MinOrgNameLength = MinOrgNameLength;
        type MaxOrgNameLength = MaxOrgNameLength;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 10)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    #[test]
    fn issue_cert_should_work() {
        new_test_ext().execute_with(|| {
            assert_ok!(Certificate::add_org(Origin::signed(2), b"Dave".to_vec()));
            assert_eq!(Balances::total_balance(&2), 10);
            assert_ok!(Certificate::issue_cert(Origin::signed(1), 2));
            assert_eq!(Balances::total_balance(&2), 8);
            assert_eq!(<OrgOf<Test>>::get(2), None);
        });
    }

    // #[test]
    // fn force_name_should_work() {
    // 	new_test_ext().execute_with(|| {
    // 		assert_noop!(
    // 			Certificate::add_org(Origin::signed(2), b"Dr. David Brubeck, III".to_vec()),
    // 			Error::<Test>::TooLong,
    // 		);

    // 		assert_ok!(Certificate::add_org(Origin::signed(2), b"Dave".to_vec()));
    // 		assert_eq!(Balances::reserved_balance(2), 2);
    // 		assert_ok!(Certificate::force_name(Origin::signed(1), 2, b"Dr. David Brubeck, III".to_vec()));
    // 		assert_eq!(Balances::reserved_balance(2), 2);
    // 		assert_eq!(<OrgOf<Test>>::get(2).unwrap(), (b"Dr. David Brubeck, III".to_vec(), 2));
    // 	});
    // }

    // #[test]
    // fn normal_operation_should_work() {
    // 	new_test_ext().execute_with(|| {
    // 		assert_ok!(Certificate::add_org(Origin::signed(1), b"Gav".to_vec()));
    // 		assert_eq!(Balances::reserved_balance(1), 2);
    // 		assert_eq!(Balances::free_balance(1), 8);
    // 		assert_eq!(<OrgOf<Test>>::get(1).unwrap().0, b"Gav".to_vec());

    // 		assert_ok!(Certificate::add_org(Origin::signed(1), b"Gavin".to_vec()));
    // 		assert_eq!(Balances::reserved_balance(1), 2);
    // 		assert_eq!(Balances::free_balance(1), 8);
    // 		assert_eq!(<OrgOf<Test>>::get(1).unwrap().0, b"Gavin".to_vec());

    // 		assert_ok!(Certificate::clear_name(Origin::signed(1)));
    // 		assert_eq!(Balances::reserved_balance(1), 0);
    // 		assert_eq!(Balances::free_balance(1), 10);
    // 	});
    // }

    // #[test]
    // fn error_catching_should_work() {
    // 	new_test_ext().execute_with(|| {
    // 		assert_noop!(Certificate::clear_name(Origin::signed(1)), Error::<Test>::Unnamed);

    // 		assert_noop!(
    // 			Certificate::add_org(Origin::signed(3), b"Dave".to_vec()),
    // 			pallet_balances::Error::<Test, _>::InsufficientBalance
    // 		);

    // 		assert_noop!(Certificate::add_org(Origin::signed(1), b"Ga".to_vec()), Error::<Test>::TooShort);
    // 		assert_noop!(
    // 			Certificate::add_org(Origin::signed(1), b"Gavin James Wood, Esquire".to_vec()),
    // 			Error::<Test>::TooLong
    // 		);
    // 		assert_ok!(Certificate::add_org(Origin::signed(1), b"Dave".to_vec()));
    // 		assert_noop!(Certificate::issue_cert(Origin::signed(2), 1), BadOrigin);
    // 		assert_noop!(Certificate::force_name(Origin::signed(2), 1, b"Whatever".to_vec()), BadOrigin);
    // 	});
    // }
}
