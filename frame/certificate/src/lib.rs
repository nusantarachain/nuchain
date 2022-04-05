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
//! - [`certificate::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Substrate pallet to manage online certificate
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Create certificate.
//! * `update` - Update certificate.
//! * `issue` - Issue certificate.
//! * `revoke` - Revoke certificate.

#![cfg_attr(not(feature = "std"), no_std)]

use base58::ToBase58;

use frame_support::{
	ensure,
	traits::{EnsureOrigin, StorageVersion, UnixTime},
	types::Text,
};
use frame_system::ensure_signed;
pub use pallet::*;
use sp_runtime::{traits::Hash, RuntimeDebug, SaturatedConversion};
use sp_std::{fmt::Debug, prelude::*, vec};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode, MaxEncodedLen};

use core::convert::TryInto;
use frame_support::BoundedVec;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

type CertId = [u8; 32];
type IssuedId = [u8; 11];

pub const MAX_PROPS: usize = 5;
pub const PROP_NAME_MAX_LENGTH: usize = 10;
pub const PROP_VALUE_MAX_LENGTH: usize = 60;

// type Property<T> = frame_support::types::Property<
// 	BoundedVec<u8, <T as pallet::Config>::MaxLength>,
// 	BoundedVec<u8, <T as pallet::Config>::MaxLength>,
// >;
use frame_support::types::Property;
use pallet_organization::Organization;

// type PropertyOrg<T> = frame_support::types::Property<
// 	BoundedVec<u8, <T as pallet_organization::Config>::MaxLength>,
// 	BoundedVec<u8, <T as pallet_organization::Config>::MaxLength>,
// >;

// type Organization<T> = pallet_organization::Organization<
// 	<T as frame_system::Config>::AccountId,
// 	<T as frame_system::Config>::BlockNumber,
// 	BoundedVec<u8, <T as pallet_organization::Config>::MaxLength>,
// 	BoundedVec<PropertyOrg<T>, <T as pallet_organization::Config>::MaxLength>,
// >;

// type MaxLength<T> = <T as pallet::Config>::MaxLength;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
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
		type Time: UnixTime;

		/// Weight information
		type WeightInfo: WeightInfo;

		/// Maximum properties length
		#[pallet::constant]
		type MaxProps: Get<u32>;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;
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

		/// Too many properties in organization object.
		TooManyProps,

		/// Invalid properties name.
		InvalidPropName,

		/// Invalid properties value.
		InvalidPropValue,

		/// Unknown error occurred
		Unknown,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// #[pallet::metadata(
	//     T::AccountId = "AccountId",
	//     T::Balance = "Balance",
	//     T::AccountId = "T::AccountId"
	// )]
	pub enum Event<T: Config> {
		/// Some certificate added.
		///
		/// params:
		///     1 - index
		///     2 - certificate id
		///     3 - organization who created the certificate.
		CertAdded(u64, CertId, T::AccountId),

		/// Certificate updated.
		CertUpdated(CertId),

		/// Some cert was issued
		///
		/// params:
		///     1 - Hash of issued certificate.
		///     2 - Organization ID.
		///     3 - Recipient of certificate.
		CertIssued(IssuedId, T::AccountId, Option<T::AccountId>),
	}

	#[pallet::storage]
	pub type Certificates<T: Config> =
		StorageMap<_, Blake2_128Concat, CertId, CertDetail<T::AccountId>>;

	// type Moment<T> = <<T as pallet::Config>::Time as Time>::Moment;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct CertDetail<AccountId> {
		/// Certificate name
		pub name: Text,

		/// Description about the certificate.
		pub description: Text,

		/// Organization ID
		pub org_id: AccountId,

		/// Name of person who publish the certificate.
		pub signer_name: Option<Text>,
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct CertProof<BlockNumber> {
		/// ID of certificate
		pub cert_id: CertId,

		/// Human readable provider based ID representation.
		pub human_id: Text,

		/// Recipient person name of the certificate
		pub recipient: Text,

		/// Creation time
		pub time: u64,

		/// Expiration in days
		pub expired: Option<u64>,

		/// Flag whether this given certificate is revoked
		pub revoked: bool,

		/// Created at block
		pub block: BlockNumber,

		/// Signer person name
		pub signer_name: Option<Text>,

		/// Additional data to embed
		pub props: Option<Vec<Property<Text, Text>>>,
	}

	/// double map pair of: Issued id -> Proof
	#[pallet::storage]
	#[pallet::getter(fn issued_cert)]
	pub type IssuedCert<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		IssuedId, // ID of issued certificate
		CertProof<T::BlockNumber>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn issued_cert_owner)]
	pub type IssuedCertOwner<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // organization id
		Blake2_128Concat,
		T::AccountId,  // acc handler id
		Vec<IssuedId>, // proof: id of issued certs
	>;

	/// Collection of certificates inside organization
	#[pallet::storage]
	#[pallet::getter(fn certificate_of_org)]
	pub type CertificateOfOrg<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId, // organization id
		Vec<CertId>,
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
		/// Create new certificate.
		///
		/// The dispatch origin for this call must be _Signed_
		/// and has access to the organization.
		///
		/// # <weight>
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			detail: CertDetail<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(detail.name.len() >= 3, Error::<T>::TooShort);
			ensure!(detail.name.len() <= 100, Error::<T>::TooLong);

			ensure!(detail.description.len() >= 3, Error::<T>::TooShort);
			ensure!(detail.description.len() <= 1000, Error::<T>::TooLong);

			if let Some(ref signer_name) = detail.signer_name {
				ensure!(signer_name.len() <= 100, Error::<T>::TooLong);
			}

			// ensure access
			let org = <pallet_organization::Pallet<T>>::organization(&detail.org_id)
				.ok_or(Error::<T>::OrganizationNotExists)?;
			Self::ensure_org_access2(&sender, &org)?;

			let index = Self::increment_index();
			let cert_id: CertId = Self::generate_hash(detail.encode());

			ensure!(!Certificates::<T>::contains_key(cert_id), Error::<T>::IdAlreadyExists);

			Self::deposit_event(Event::CertAdded(index, cert_id, detail.org_id.clone()));

			<CertificateOfOrg<T>>::try_mutate::<&T::AccountId, _, Error<T>, _>(
				&detail.org_id,
				|vs| {
					if let Some(vs) = vs.as_mut() {
						vs.push(cert_id);
					} else {
						// let x: BoundedVec<CertId, MaxLength<T>> =
						// 	BoundedVec::try_from(vec![cert_id]).map_err(|e| Error::<T>::TooLong)?;
						*vs = Some(vec![cert_id]);
					}
					Ok(())
				},
			)?;
			Certificates::<T>::insert(cert_id, detail);

			Ok(().into())
		}

		/// Update certificate.
		///
		/// Currently only support update for the signer name.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create())]
		pub fn update(
			origin: OriginFor<T>,
			cert_id: CertId,
			signer_name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(signer_name.len() > 1, Error::<T>::TooShort);
			ensure!(signer_name.len() <= 100, Error::<T>::TooLong);

			let cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

			// ensure access
			let org = <pallet_organization::Pallet<T>>::organization(&cert.org_id)
				.ok_or(Error::<T>::OrganizationNotExists)?;
			Self::ensure_org_access2(&sender, &org)?;

			Certificates::<T>::try_mutate::<_, _, Error<T>, _>(&cert_id, |rec| {
				if let Some(rec) = rec.as_mut() {
					rec.signer_name =
						Some(signer_name.try_into().map_err(|_| Error::<T>::TooLong)?);
				}
				Ok(())
			})?;

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
		pub fn issue(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			cert_id: CertId,
			human_id: Text,  // human readable provider based id, eg: ORG/KOM/11321
			recipient: Text, // person name
			props: Option<Vec<Property<Text, Text>>>,
			acc_handler: Option<T::AccountId>,
			expired: Option<u64>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// let cert_id:BoundedVec<u8, MaxLength<T>> = cert_id.try_into().map_err(|_|
			// Error::<T>::TooLong)?;

			let cert = Certificates::<T>::get(cert_id).ok_or(Error::<T>::NotExists)?;

			if let Some(ref props) = props {
				ensure!((props.len() as u32) < T::MaxProps::get(), Error::<T>::TooManyProps);
			}

			ensure!(human_id.len() < 100, Error::<T>::TooLong);
			ensure!(recipient.len() < 100, Error::<T>::TooLong);

			Self::validate_props(&props)?;

			// ensure access
			let org = <pallet_organization::Pallet<T>>::organization(&org_id)
				.ok_or(Error::<T>::OrganizationNotExists)?;
			Self::ensure_org_access2(&sender, &org)?;

			// generate issue id
			// this id is unique per user per cert.
			let data = org_id
				.as_ref()
				.iter()
				.chain(cert_id.encode().iter())
				.chain(human_id.iter())
				.chain(recipient.iter())
				.cloned()
				.collect::<Vec<u8>>();

			let data = if let Some(ref props) = props {
				data.iter().chain(props.encode().iter()).cloned().collect::<Vec<u8>>()
			} else {
				data.iter().cloned().collect::<Vec<u8>>()
			};
			let issued_id: IssuedId = Self::generate_issued_id(&org, data);

			// pastikan belum pernah di-issue
			ensure!(!IssuedCert::<T>::contains_key(&issued_id), Error::<T>::AlreadyExists);

			let block = <frame_system::Pallet<T>>::block_number();
			let signer_name = cert.signer_name.clone();

			// let human_id: BoundedVec<u8, MaxLength<T>> =
			// 	human_id.try_into().map_err(|_| Error::<T>::TooLong)?;
			// let recipient: BoundedVec<u8, MaxLength<T>> =
			// 	recipient.try_into().map_err(|_| Error::<T>::TooLong)?;
			// let signer_name: Option<BoundedVec<u8, MaxLength<T>>> =
			// 	if let Some(signer_name) = signer_name {
			// 		Some(signer_name.try_into().map_err(|_| Error::<T>::TooLong)?)
			// 	} else {
			// 		return Err(Error::<T>::TooLong.into())
			// 	};
			// let props: Option<BoundedVec<Property<T>, MaxLength<T>>> = if let Some(props) = props
			// { 	props
			// 		.into_iter()
			// 		.flat_map(|p| {
			// 			let v: Option<Property<T>> = p.try_into().ok();
			// 			v
			// 		})
			// 		.collect::<Vec<_>>()
			// 		.try_into()
			// 		.ok()
			// } else {
			// 	return Err(Error::<T>::TooLong.into())
			// };

			let proof = CertProof {
				cert_id,
				human_id,
				recipient,
				time: <T as pallet::Config>::Time::now().as_millis().saturated_into::<u64>(),
				expired,
				revoked: false,
				block,
				signer_name,
				props,
			};

			if let Some(ref acc_handler) = acc_handler {
				// apabila sudah pernah diisi update isinya
				// dengan ditambahkan sertifikat pada koleksi penerima.
				IssuedCertOwner::<T>::try_mutate::<&T::AccountId, &T::AccountId, _, Error<T>, _>(
					&org_id,
					acc_handler,
					|vs| {
						if let Some(vs) = vs.as_mut() {
							vs.push(issued_id.clone());
						} else {
							// let x: BoundedVec<IssuedId, MaxLength<T>> =
							// vec![issued_id.clone()].try_into().map_err(|_| Error::<T>::TooLong)?;
							*vs = Some(vec![issued_id.clone()]);
						}
						Ok(())
					},
				)?;
			}

			IssuedCert::<T>::insert(&issued_id, proof);

			Self::deposit_event(Event::CertIssued(issued_id, org_id, acc_handler));

			Ok(().into())
		}

		/// Revoke sertifikat berdasarkan issue id-nya.
		#[pallet::weight(0)]
		pub fn revoke(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			issued_id: IssuedId,
			revoked: bool, // true untuk revoke, false untuk mengembalikan.
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let org = <pallet_organization::Pallet<T>>::organization(&org_id)
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
					},
					None => Err(Error::<T>::NotExists),
				}
			})?;

			Ok(().into())
		}

		/// Check whether certificate is valid.
		#[pallet::weight(0)]
		pub fn validate_certificate(
			origin: OriginFor<T>,
			_issued_id: IssuedId,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;
			Ok(().into())
		}
	}

	// -------------------------------------------------------------------
	//                      GENESIS CONFIGURATION
	// -------------------------------------------------------------------

	// The genesis config type.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: PhantomData<T>,
	}

	// The default value for the genesis config type.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { _phantom: Default::default() }
		}
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}
}

// use pallet_organization::Organization;

/// The main implementation of this Certificate pallet.
impl<T: Config> Pallet<T> {
	/// Get detail of certificate
	pub fn get(id: &CertId) -> Option<CertDetail<T::AccountId>> {
		Certificates::<T>::get(id)
	}

	#[allow(dead_code)]
	/// Memastikan bahwa akun memiliki akses pada organisasi.
	/// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
	fn ensure_org_access(
		who: &T::AccountId,
		org_id: &T::AccountId,
	) -> Result<Organization<T>, Error<T>> {
		let org = pallet_organization::Pallet::<T>::ensure_access(who, org_id)
			.map_err(|_| Error::<T>::PermissionDenied)?;
		Self::ensure_org_access2(who, &org)?;
		Ok(org)
	}

	/// Memastikan bahwa akun memiliki akses pada organisasi.
	/// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
	pub fn ensure_org_access2(who: &T::AccountId, org: &Organization<T>) -> Result<(), Error<T>> {
		pallet_organization::Pallet::<T>::ensure_access_active(who, &org)
			.map_err(|_| Error::<T>::PermissionDenied)
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
	pub fn generate_issued_id(org: &Organization<T>, data: Vec<u8>) -> IssuedId {
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
				let now = <T as pallet::Config>::Time::now().as_millis().saturated_into::<u64>();
				proof.expired.map(|a| a < now).unwrap_or(true) && !proof.revoked
			})
			.unwrap_or(false)
	}

	/// Validasi properties
	pub fn validate_props(props: &Option<Vec<Property<Text, Text>>>) -> Result<(), Error<T>> {
		if let Some(props) = props {
			ensure!(props.len() <= MAX_PROPS, Error::<T>::TooManyProps);
			for prop in props {
				let len = prop.name().len();
				ensure!(len > 0 && len <= PROP_NAME_MAX_LENGTH, Error::<T>::InvalidPropName);
				let len = prop.value().len();
				ensure!(len > 0 && len <= PROP_VALUE_MAX_LENGTH, Error::<T>::InvalidPropValue);
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests;
