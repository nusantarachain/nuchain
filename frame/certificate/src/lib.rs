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

type CertId = [u8; 32];
type IssuedId = [u8; 11];

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;

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
        /// ID of certificate
        pub cert_id: CertId,

        /// Human readable provider based ID representation.
        pub human_id: Vec<u8>,

        /// Recipient person name of the certificate
        pub recipient: Vec<u8>,

        /// Creation time
        pub time: Moment<T>,

        /// Expiration in days
        pub expired: Moment<T>,

        /// Flag whether this given certificate is revoked
        pub revoked: bool,

        /// Additional data to embed
        pub additional_data: Option<Vec<u8>>,
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
            detail: CertDetail<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(detail.name.len() >= 3, Error::<T>::TooShort);
            ensure!(detail.name.len() <= 100, Error::<T>::TooLong);

            ensure!(detail.description.len() >= 3, Error::<T>::TooShort);
            ensure!(detail.description.len() <= 1000, Error::<T>::TooLong);

            ensure!(detail.signer_name.len() <= 100, Error::<T>::TooLong);

            // ensure access
            let org = <pallet_organization::Module<T>>::organization(&detail.org_id)
                .ok_or(Error::<T>::OrganizationNotExists)?;
            Self::ensure_org_access2(&sender, &org)?;

            let index = Self::increment_index();
            let cert_id: CertId = Self::generate_hash(detail.encode());

            ensure!(
                !Certificates::<T>::contains_key(cert_id),
                Error::<T>::IdAlreadyExists
            );

            Self::deposit_event(Event::CertAdded(index, cert_id, detail.org_id.clone()));

            Certificates::<T>::insert(cert_id, detail);

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
            human_id: Vec<u8>, // human readable provider based id, eg: ORG/KOM/11321
            recipient: Vec<u8>, // person name
            additional_data: Vec<u8>,
            acc_handler: Option<T::AccountId>,
            expired: Option<Moment<T>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let _cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

            ensure!(additional_data.len() < 100, Error::<T>::TooLong);

            ensure!(human_id.len() < 100, Error::<T>::TooLong);
            ensure!(recipient.len() < 100, Error::<T>::TooLong);

            // ensure access
            let org = <pallet_organization::Module<T>>::organization(&org_id)
                .ok_or(Error::<T>::OrganizationNotExists)?;
            Self::ensure_org_access2(&sender, &org)?;

            // generate issue id
            // this id is unique per user per cert.
            let issue_id: IssuedId = Self::generate_issued_id(
                &org,
                org_id
                    .as_ref()
                    .iter()
                    .chain(cert_id.encode().iter())
                    .chain(human_id.iter())
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

            let proof = CertProof {
                cert_id,
                human_id,
                recipient,
                time: <T as pallet::Config>::Time::now(),
                expired: expired.unwrap_or_default(),
                revoked: false,
                additional_data: Some(additional_data),
            };

            if let Some(ref acc_handler) = acc_handler {
                // apabila sudah pernah diisi update isinya
                // dengan ditambahkan sertifikat pada koleksi penerima.
                IssuedCertOwner::<T>::mutate(&org_id, acc_handler, |vs| {
                    if let Some(vs) = vs.as_mut() {
                        vs.push(proof.clone());
                    } else {
                        *vs = Some(vec![proof.clone()]);
                    }
                });
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
            issued_id: IssuedId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Ok(().into())
        }
    }
}

use frame_support::traits::Time;
use core::convert::TryInto;

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
    pub fn ensure_org_access2(
        who: &T::AccountId,
        org: &Organization<T::AccountId>,
    ) -> Result<(), Error<T>> {
        pallet_organization::Module::<T>::ensure_org_access_active(who, &org)
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
    /// kemudian di-truncate agar pendek (10 chars) + karakter awal nama organisasi.
    ///
    /// dengan cara hanya mengambil 5 chars dari awal dan akhir
    /// dari hash dalam bentuk base58, contoh output: 4p9w6uE2Zs
    pub fn generate_issued_id(org: &Organization<T::AccountId>, data: Vec<u8>) -> IssuedId {
        let hash = T::Hashing::hash(&data).encode().to_base58();
        let first = hash.as_bytes().iter().skip(2).take(5);
        let last = hash.as_bytes().iter().skip(hash.len() - 5);
        org.name
            .iter()
            .take(1)
            .chain(first)
            .chain(last)
            .cloned()
            .collect::<Vec<u8>>()
            .try_into()
            .expect("fixed 11 length array; qed")
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
mod tests;
