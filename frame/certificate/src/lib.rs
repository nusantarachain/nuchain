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

use base58::ToBase58;

use frame_support::{ensure, traits::EnsureOrigin};
use frame_system::ensure_signed;
pub use pallet::*;
// use sp_core::H256;
use sp_runtime::traits::Hash;
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, prelude::*, vec};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};

// type T::AccountId = u32;
// type CertId<T> = T::Hash;
type CertId = [u8; 32];
type IssuedId = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    // use pallet_organization::OrgProvider;

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

        // /// Who is allowed to create certificate
        // type CreatorOrigin: EnsureOrigin<
        //     Self::Origin,
        //     Success = (Self::AccountId, Vec<Self::AccountId>),
        // >;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct CertDetail<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Certificate name
        pub name: Vec<u8>,

        /// Description about the certificate.
        pub description: Vec<u8>,

        /// Organization ID
        pub org_id: AccountId,

        /// Name of person who publish the certificate.
        pub signer_name: Vec<u8>,
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

        /// Organization not exists
        OrganizationNotExists,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Balance = "Balance",
        T::AccountId = "T::AccountId"
    )]
    pub enum Event<T: Config> {
        /// Some certificate added.
        CertAdded(u64, CertId, T::AccountId),

        /// Some cert was issued
        ///
        /// param:
        ///     1 - Hash of issued certificate.
        ///     2 - Recipient of certificate.
        CertIssued(IssuedId, Option<T::AccountId>),
    }

    #[pallet::storage]
    pub type Certificates<T: Config> =
        StorageMap<_, Blake2_128Concat, CertId, CertDetail<T::AccountId>>;

    type Moment<T> = <<T as pallet::Config>::Time as Time>::Moment;

    #[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug)]
    pub struct CertProof<T: Config> {
        pub cert_id: CertId,

        /// Human readable ID representation.
        pub human_id: Vec<u8>,

        /// Creation time
        pub time: Moment<T>,

        /// Expiration in days
        pub expired: Moment<T>,

        /// Flag whether this certificate is revoked
        pub revoked: bool,

        /// Additional data
        pub additional_data: Option<Vec<u8>>,
    }

    impl<T: Config> CertProof<T> {
        fn new(
            cert_id: CertId,
            human_id: Vec<u8>,
            time: Moment<T>,
            expired: Option<Moment<T>>,
            additional_data: Option<Vec<u8>>,
        ) -> Self {
            CertProof {
                cert_id,
                human_id,
                time,
                expired: expired.unwrap_or_default(),
                revoked: false,
                additional_data,
            }
        }
    }

    /// double map pair of: Issued id -> Proof
    #[pallet::storage]
    #[pallet::getter(fn issued_cert)]
    pub type IssuedCert<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        IssuedId, // ID of issued certificate
        CertProof<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn issued_cert_owner)]
    pub type IssuedCertOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // organization id
        Blake2_128Concat,
        T::AccountId,      // acc handler id
        Vec<CertProof<T>>, // proof
    >;

    // #[pallet::storage]
    // #[pallet::getter(fn account_id_index)]
    // pub type AccountIdIndex<T> = StorageValue<_, u32>;

    #[pallet::storage]
    pub type CertIdIndex<T> = StorageValue<_, u64>;

    /// Certificate module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: AsRef<[u8]>,
    {
        /// Create new certificate
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_cert())]
        pub(super) fn create_cert(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            name: Vec<u8>,
            description: Vec<u8>,
            signer_name: Option<Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(name.len() >= 3, Error::<T>::TooShort);
            ensure!(name.len() <= 100, Error::<T>::TooLong);

            let signer_name = if let Some(signer_name) = signer_name {
                ensure!(signer_name.len() >= 3, Error::<T>::TooShort);
                ensure!(signer_name.len() <= 100, Error::<T>::TooLong);
                signer_name
            } else {
                b"".to_vec()
            };

            // let (sender, org_ids) = T::CreatorOrigin::ensure_origin(origin)?;

            // // pastikan origin adalah admin pada organisasi
            // ensure!(
            //     org_ids.iter().any(|id| *id == org_id),
            //     Error::<T>::PermissionDenied
            // );

            // let org = <pallet_organization::Module<T>>::organization(&org_id)
            //     .ok_or(Error::<T>::OrganizationNotExists)?;

            // ensure admin
            // ensure!(&org.admin == &sender, Error::<T>::PermissionDenied);

            // ensure access
            let org = <pallet_organization::Module<T>>::organization(&org_id)
                .ok_or(Error::<T>::OrganizationNotExists)?;
            Self::ensure_org_access2(&sender, &org)?;

            let index = Self::increment_index();
            let cert_id: CertId = Self::generate_hash(
                index
                    .to_le_bytes()
                    .iter()
                    .chain(name.iter())
                    .chain(description.iter())
                    .cloned()
                    .collect::<Vec<u8>>(),
            );

            ensure!(
                !Certificates::<T>::contains_key(cert_id),
                Error::<T>::IdAlreadyExists
            );

            Certificates::<T>::insert(
                cert_id,
                CertDetail {
                    name: name.clone(),
                    org_id: org_id.clone(),
                    description: description.clone(),
                    signer_name,
                },
            );

            Self::deposit_event(Event::CertAdded(index, cert_id, org_id));

            Ok(().into())
        }

        /// Issue certificate.
        ///
        /// After organization create certificate; admin should be able to
        /// issue certificate to someone.
        ///
        /// The dispatch origin for this call must be _signed_
        /// and has access to organization as admin.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(70_000_000)]
        pub(super) fn issue_cert(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            cert_id: CertId,
            recipient: Vec<u8>, // person name
            additional_data: Option<Vec<u8>>,
            acc_handler: Option<T::AccountId>,
            expired: Option<Moment<T>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let _cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

            if let Some(ref additional_data) = additional_data {
                ensure!(additional_data.len() < 100, Error::<T>::TooLong);
            }

            // ensure access
            let org = <pallet_organization::Module<T>>::organization(&org_id)
                .ok_or(Error::<T>::OrganizationNotExists)?;
            Self::ensure_org_access2(&sender, &org)?;

            let additional_data = additional_data.unwrap_or_else(|| vec![]);

            // generate issue id
            // this id is unique per user per cert.
            let issue_id: IssuedId = Self::generate_issued_id(
                org_id
                    .as_ref()
                    // .to_le_bytes()
                    .iter()
                    .chain(cert_id.encode().iter())
                    .chain(recipient.iter())
                    .chain(additional_data.iter())
                    .cloned()
                    .collect::<Vec<u8>>(),
            );

            // pastikan belum pernah di-issue
            ensure!(
                !IssuedCert::<T>::contains_key(&issue_id),
                Error::<T>::AlreadyExists
            );

            let proof = CertProof::new(
                cert_id,
                recipient,
                <T as pallet::Config>::Time::now(),
                expired,
                Some(additional_data),
            );

            if let Some(ref acc_handler) = acc_handler {
                // apabila sudah pernah diisi update isinya
                // dengan ditambahkan sertifikat pada koleksi penerima.
                IssuedCertOwner::<T>::try_mutate(&org_id, acc_handler, |vs| {
                    if let Some(vs) = vs.as_mut() {
                        vs.push(proof.clone());
                        Ok(())
                    } else {
                        Err(Error::<T>::Unknown)
                    }
                })?;
            }

            IssuedCert::<T>::insert(&issue_id, proof);

            Self::deposit_event(Event::CertIssued(issue_id, acc_handler));

            Ok(().into())
        }

        /// Revoke sertifikat berdasarkan issue id-nya.
        #[pallet::weight(0)]
        pub(super) fn revoke_certificate(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            issued_id: IssuedId,
            revoked: bool, // true untuk revoke, false untuk mengembalikan.
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let org = <pallet_organization::Module<T>>::organization(&org_id)
                .ok_or(Error::<T>::Unknown)?;
            Self::ensure_org_access2(&who, &org)?;

            IssuedCert::<T>::try_mutate(&issued_id, |d| {
                match d {
                    Some(d) => {
                        d.revoked = revoked;

                        // // also update expiration time
                        // // to current time, this force issued cert to
                        // // expire at the current point of time.
                        // d.expired = <T as pallet::Config>::Time::now();

                        Ok(())
                    }
                    None => Err(Error::<T>::NotExists),
                }
            })?;

            Ok(().into())
        }

        /// Check whether certificate is valid.
        #[pallet::weight(0)]
        pub(super) fn validate_certificate(
            origin: OriginFor<T>,
            issue_id: IssuedId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Ok(().into())
        }
    }
}

use frame_support::traits::Time;

type Organization<T> = pallet_organization::Organization<T>;

/// The main implementation of this Certificate pallet.
impl<T: Config> Pallet<T> {
    /// Get detail of certificate
    ///
    pub fn get(id: &CertId) -> Option<CertDetail<T::AccountId>> {
        Certificates::<T>::get(id)
    }

    /// Memastikan bahwa akun memiliki akses pada organisasi.
    /// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
    fn ensure_org_access(
        who: &T::AccountId,
        org_id: &T::AccountId,
    ) -> Result<Organization<T::AccountId>, Error<T>> {
        let org = pallet_organization::Module::<T>::ensure_access(who, org_id)
            .map_err(|_| Error::<T>::PermissionDenied)?;
        Self::ensure_org_access2(who, &org)?;
        Ok(org)
    }

    /// Memastikan bahwa akun memiliki akses pada organisasi.
    /// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
    fn ensure_org_access2(
        who: &T::AccountId,
        org: &Organization<T::AccountId>,
    ) -> Result<(), Error<T>> {
        ensure!(&org.admin == who, Error::<T>::PermissionDenied);
        ensure!(!org.suspended, Error::<T>::PermissionDenied);
        // Ok(org)
        Ok(())
    }

    /// Incerment certificate index
    pub fn increment_index() -> u64 {
        let next_id = <CertIdIndex<T>>::try_get().unwrap_or(0).saturating_add(1);
        <CertIdIndex<T>>::put(next_id);
        next_id
    }

    /// Generate hash for randomly generated certificate identification.
    pub fn generate_hash(data: Vec<u8>) -> CertId {
        let mut hash: [u8; 32] = Default::default();
        hash.copy_from_slice(&T::Hashing::hash(&data).encode()[..32]);
        hash
    }

    /// Generate Issued ID.
    ///
    /// Issue ID ini merupakan hash dari data yang
    /// kemudian di-truncate agar pendek (10 chars)
    /// dengan cara hanya mengambil 5 chars dari awal dan akhir
    /// dari hash dalam bentuk base58, contoh output: 4p9w6uE2Zs
    pub fn generate_issued_id(data: Vec<u8>) -> IssuedId {
        let hash = T::Hashing::hash(&data).encode().to_base58();
        let first = hash.as_bytes().iter().take(5);
        let last = hash.as_bytes().iter().skip(hash.len() - 5);
        first.into_iter().chain(last).cloned().collect::<Vec<u8>>()
    }

    /// Check whether issued certificate is valid.
    pub fn valid_certificate(id: &IssuedId) -> bool {
        Self::issued_cert(id)
            .map(|proof| {
                let now = <T as pallet::Config>::Time::now();
                proof.expired < now && !proof.revoked
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
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
                org_id,
                b"CERT1".to_vec(),
                b"CERT1 desc".to_vec(),
                Some(b"Grohl".to_vec())
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
                b"Dave".to_vec(),
                Some(b"ADDITIONAL DATA".to_vec()),
                None,
                None
            ));
            let issued_id = get_last_issued_cert_id().expect("get last issued id");
            println!("issued_id: {:?}", std::str::from_utf8(&issued_id));

            func(org_id, cert_id, issued_id);
        })
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
                    sp_keyring::Sr25519Keyring::One.into(), // non existent org address
                    b"CERT1".to_vec(),
                    b"CERT1 desc".to_vec(),
                    None
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
                Certificate::create_cert(
                    Origin::signed(Bob.into()),
                    last_org_id(),
                    b"CERT1".to_vec(),
                    b"CERT1 desc".to_vec(),
                    None
                ),
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
}
