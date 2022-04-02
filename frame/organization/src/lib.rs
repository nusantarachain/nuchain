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

//! # Organization
//!
//! - [`Organization::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Organization pallet for Nuchain
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Create organization.
//! * `update` - Update organization.
//! * `suspend_org` - Suspen organization.
//! * `add_members` - Add account as member to the organization.
//! * `remove_member` - Remove account member from organization.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		Currency, EnsureOrigin,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, OnUnbalanced, ReservableCurrency, UnixTime, WithdrawReasons,
	},
	types::{Property, Text},
	BoundedVec,
};
use frame_system::ensure_signed;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::{Hash, StaticLookup};
use sp_std::prelude::*;

use enumflags2::BitFlags;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode, EncodeLike};
use pallet_did::Did;

mod types;

pub use crate::types::Organization;

pub const MAX_PROPS: usize = 10;
pub const PROP_NAME_MAX_LENGTH: usize = 30;
pub const PROP_VALUE_MAX_LENGTH: usize = 60;

macro_rules! to_bounded {
	(*$name:ident, $error:expr) => {
		$name.clone().try_into().map_err(|()| $error)?
	};
	($name:ident, $error:expr) => {
		$name.try_into().map_err(|()| $error)?
	};
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter, prelude::vec};
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		SaturatedConversion,
	};
	// use sp_std::vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Timestamp
		type Time: UnixTime;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Creation fee.
		type CreationFee: Get<BalanceOf<Self>>;

		/// Payment for treasury
		type Payment: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The origin which may forcibly set or remove a name. Root can always do this.
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// Min organization name length
		type MinOrgNameLength: Get<usize>;

		/// Max organization name length
		type MaxOrgNameLength: Get<usize>;

		/// Max number of member for the organization
		#[pallet::constant]
		type MaxMemberCount: Get<u32>;

		/// Weight information
		type WeightInfo: WeightInfo;

		type Public: IdentifyAccount<AccountId = Self::AccountId>;
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;

		/// DId provider
		type Did: Did<
			Self::AccountId,
			Self::BlockNumber,
			Self::Time,
			Self::Signature,
			BoundedVec<u8, Self::MaxLength>,
		>;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;

		// #[pallet::constant]
		// type MaxLength: Get<u32>;
	}

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;
	// type Moment<T> = <<T as Config>::Time as Time>::Moment;

	#[pallet::error]
	pub enum Error<T> {
		/// The object already exsits
		AlreadyExists,

		/// Already set, no change have made
		AlreadySet,

		/// Name too long
		NameTooLong,

		/// Name too short
		NameTooShort,

		/// Description too long
		DescriptionTooLong,

		/// Description too short
		DescriptionTooShort,

		/// Website text is too long
		WebsiteTooLong,

		/// Email text is too long
		EmailTooLong,

		/// Object doesn't exist.
		NotExists,

		/// Origin has no authorization to do this operation
		PermissionDenied,

		/// ID already exists
		BadIndex,

		/// Cannot generate ID
		CannotGenId,

		/// Max member count reached
		MaxMemberReached,

		/// The organization is suspended
		Suspended,

		/// Too many properties in organization object.
		TooManyProps,

		/// Invalid properties name.
		InvalidPropName,

		/// Invalid properties value.
		InvalidPropValue,

		/// Account is not member of the organization.
		NotMember,

		InvalidParameter,

		/// Changes not made
		NotChanged,

		/// Unknown error occurred
		Unknown,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
	pub enum Event<T: Config> {
		/// New organization registered.
		///
		/// 1: organization id (hash)
		/// 2: creator account id
		OrganizationAdded(T::AccountId, T::AccountId),

		// /// When object deleted
		// OrganizationDeleted(T::AccountId),
		/// Organization data has been updated
		OrganizationUpdated(T::AccountId),

		/// Organization has been suspended.
		OrganizationSuspended(T::AccountId),

		/// Member added to an organization
		MemberAdded(T::AccountId, T::AccountId),

		/// Member removed from an organization
		MemberRemoved(T::AccountId, T::AccountId),

		/// Organization admin changed.
		AdminChanged(T::AccountId, T::AccountId),
	}

	/// Pair organization hash -> Organization data
	#[pallet::storage]
	#[pallet::getter(fn organization)]
	pub type Organizations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Organization<
			T::AccountId,
			T::BlockNumber,
			BoundedVec<u8, T::MaxLength>,
			BoundedVec<
				Property<BoundedVec<u8, T::MaxLength>, BoundedVec<u8, T::MaxLength>>,
				T::MaxLength,
			>,
		>,
	>;

	/// Link organization index -> organization hash.
	/// Useful for lookup organization from index.
	#[pallet::storage]
	#[pallet::getter(fn organization_index)]
	pub type OrganizationIndexOf<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

	// /// Pair user -> list of handled organizations
	// #[pallet::storage]
	// pub type OrganizationLink<T: Config> =
	// 	StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::AccountId, T::MaxLength>,
	// ValueQuery>;

	/// Membership store, stored as an ordered Vec.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::AccountId, T::MaxMemberCount>>;

	// #[bitflags(default = Active)]
	#[repr(u64)]
	#[derive(Clone, Copy, PartialEq, Eq, BitFlags, RuntimeDebug, TypeInfo)]
	pub enum FlagDataBit {
		Active = 0b0000000000000000000000000000000000000000000000000000000000000001,
		Verified = 0b0000000000000000000000000000000000000000000000000000000000000010,
		Government = 0b0000000000000000000000000000000000000000000000000000000000000100,
		System = 0b0000000000000000000000000000000000000000000000000000000000001000,
		Edu = 0b0000000000000000000000000000000000000000000000000000000000010000,
		Company = 0b0000000000000000000000000000000000000000000000000000000000100000,
		Foundation = 0b0000000000000000000000000000000000000000000000000000000001000000,
	}

	#[derive(Clone, Copy, PartialEq, RuntimeDebug)]
	pub struct FlagDataBits(pub BitFlags<FlagDataBit>);

	impl Eq for FlagDataBits {}
	impl Encode for FlagDataBits {
		fn using_encoded<R, F>(&self, f: F) -> R
		where
			F: FnOnce(&[u8]) -> R,
		{
			self.0.bits().using_encoded(f)
		}
	}
	impl Decode for FlagDataBits {
		fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
			let field = u64::decode(input)?;
			Ok(Self(
				BitFlags::<FlagDataBit>::from_bits(field as u64)
					.map_err(|_| "invalid flag data value")?,
			))
		}
	}

	impl MaxEncodedLen for FlagDataBits {
		fn max_encoded_len() -> usize {
			u64::max_encoded_len()
		}
	}

	impl EncodeLike for FlagDataBits {}
	impl core::ops::Deref for FlagDataBits {
		type Target = BitFlags<FlagDataBit>;
		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}
	impl core::ops::DerefMut for FlagDataBits {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.0
		}
	}

	impl TypeInfo for FlagDataBits {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("BitFlags", module_path!()))
				.type_params(vec![TypeParameter::new("T", Some(meta_type::<FlagDataBits>()))])
				.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("FlagDataBit")))
		}
	}

	impl Default for FlagDataBits {
		fn default() -> Self {
			Self(FlagDataBit::Active.into())
		}
	}

	/// Flag of the organization
	#[pallet::storage]
	#[pallet::getter(fn flags)]
	pub type OrganizationFlagData<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, FlagDataBits>;

	// pub struct EnsureOrgAdmin<T>(sp_std::marker::PhantomData<T>);

	// impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrgAdmin<T> {
	//     type Success = (T::AccountId, Vec<T::AccountId>);

	//     fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
	//         o.into().and_then(|o| match o {
	//             frame_system::RawOrigin::Signed(ref who) => {
	//                 let vs = OrganizationLink::<T>::get(who.clone())
	//                     .ok_or(T::Origin::from(o.clone()))?;
	//                 Ok((who.clone(), vs.clone()))
	//             }
	//             r => Err(T::Origin::from(r)),
	//         })
	//     }

	//     #[cfg(feature = "runtime-benchmarks")]
	//     fn successful_origin() -> T::Origin {
	//         O::from(RawOrigin::Signed(Default::default()))
	//     }
	// }

	#[pallet::storage]
	// #[pallet::getter(fn org_index)]
	pub type OrgIdIndex<T> = StorageValue<_, u64>;

	/// Organization module declaration.
	// pub struct Module<T: Config> for enum Call where origin: T::Origin {
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	{
		/// Add new Organization.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(N)` where:
		///     - `N` length of properties * 100_000.
		/// # </weight>
		#[pallet::weight(
		    <T as Config>::WeightInfo::create()
		        .saturating_add((props.as_ref().map(|a| a.len()).unwrap_or(0) * 100_000) as
		Weight) )]
		pub fn create(
			origin: OriginFor<T>,
			name: Text,
			description: Text,
			admin: T::AccountId,
			website: Text,
			email: Text,
			props: Option<Vec<Property<Text, Text>>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			ensure!(name.len() >= T::MinOrgNameLength::get(), Error::<T>::NameTooShort);
			ensure!(name.len() <= T::MaxOrgNameLength::get(), Error::<T>::NameTooLong);

			Self::validate_props(&props)?;

			let index = Self::next_index()?;

			ensure!(!OrganizationIndexOf::<T>::contains_key(index), Error::<T>::BadIndex);

			// let admin = T::Lookup::lookup(admin)?;

			// Process the payment
			let cost = T::CreationFee::get();

			// Process payment
			T::Payment::on_unbalanced(T::Currency::withdraw(
				&who,
				cost,
				WithdrawReasons::FEE,
				KeepAlive,
			)?);

			// generate organization id (hash)
			let org_id: T::AccountId = UncheckedFrom::unchecked_from(T::Hashing::hash(
				&index
					.to_le_bytes()
					.iter()
					.chain(name.iter())
					.chain(description.iter())
					.chain(website.iter())
					.chain(email.iter())
					.cloned()
					.collect::<Vec<u8>>(),
			));

			let block = <frame_system::Pallet<T>>::block_number();

			Organizations::<T>::insert(
				org_id.clone(),
				Organization {
					id: org_id.clone(),
					name: to_bounded!(*name, Error::<T>::NameTooLong),
					description: to_bounded!(description, Error::<T>::DescriptionTooLong),
					admin: admin.clone(),
					website: to_bounded!(website, Error::<T>::WebsiteTooLong),
					email: to_bounded!(email, Error::<T>::EmailTooLong),
					suspended: false,
					block,
					timestamp: T::Time::now().as_millis().saturated_into::<u64>(),
					props: props.and_then(|ps| {
						ps.into_iter()
							.flat_map(|p| {
								let x: Option<
									Property<
										BoundedVec<u8, T::MaxLength>,
										BoundedVec<u8, T::MaxLength>,
									>,
								> = p.try_into().ok();
								x
							})
							.collect::<Vec<_>>()
							.try_into()
							.ok()
					}),
				},
			);

			<OrganizationIndexOf<T>>::insert(index, org_id.clone());

			// if OrganizationLink::<T>::contains_key(&admin) {
			// 	OrganizationLink::<T>::mutate(&admin, |ref mut vs| {
			// 		// vs.as_mut().map(|vsi| vsi.try_push(org_id.clone()). )
			// 		vs.try_push(org_id.clone()).map_err(|_| Error::<T>::TooManyOrgLink)
			// 	});
			// } else {
			// 	let orgs: BoundedVec<T::AccountId, T::MaxLength> =
			// 		sp_std::vec![org_id.clone()].try_into().unwrap();
			// 	OrganizationLink::<T>::insert(&admin, orgs);
			// }

			<OrganizationFlagData<T>>::insert::<_, FlagDataBits>(
				org_id.clone(),
				Default::default(),
			);

			// admin added as member first
			let members: BoundedVec<T::AccountId, T::MaxMemberCount> =
				vec![admin.clone()].try_into().unwrap();
			<Members<T>>::insert(&org_id, members);

			// DID add attribute
			T::Did::create_attribute(&org_id, &org_id, &b"Org".to_vec(), &name, None)?;
			// Set owner of this organization in DID
			T::Did::set_owner(&who, &org_id, &admin);

			Self::deposit_event(Event::OrganizationAdded(org_id, admin));

			Ok(().into())
		}

		/// Update organization data.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(N)` where:
		///     - `N` length of properties * 100_000.
		/// # </weight>
		#[pallet::weight(
		    <T as Config>::WeightInfo::update()
		        .saturating_add((props.as_ref().map(|a| a.len()).unwrap_or(0) * 100_000) as
		Weight) )]
		pub fn update(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			name: Option<Text>,
			description: Option<Text>,
			website: Option<Text>,
			email: Option<Text>,
			props: Option<Vec<Property<Text, Text>>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			if let Some(ref name) = name {
				ensure!(name.len() >= T::MinOrgNameLength::get(), Error::<T>::NameTooShort);
				ensure!(name.len() <= T::MaxOrgNameLength::get(), Error::<T>::NameTooLong);
			}

			Self::validate_props(&props)?;

			let org = Self::ensure_access(&who, &org_id)?;
			ensure!(!org.suspended, Error::<T>::Suspended);

			// // W: 1 db read
			// gak perlu ini, try_mutate sudah melakukannya
			// ensure!(
			//     Organizations::<T>::contains_key(&org_id),
			//     Error::<T>::NotExists
			// );

			Organizations::<T>::try_mutate(&org_id, |ref mut org| {
				if let Some(org) = org {
					let mut updated = false;
					if let Some(name) = name {
						org.name = to_bounded!(name, Error::<T>::NameTooLong);
						updated = true;
					}
					if let Some(description) = description {
						org.description = to_bounded!(description, Error::<T>::DescriptionTooLong);
						updated = true;
					}
					if let Some(website) = website {
						org.website = to_bounded!(website, Error::<T>::WebsiteTooLong);
						updated = true;
					}
					if let Some(email) = email {
						org.email = to_bounded!(email, Error::<T>::EmailTooLong);
						updated = true;
					}
					if props.is_some() {
						org.props = props.and_then(|ps| {
							ps.into_iter()
								.flat_map(|p| {
									let x: Option<
										Property<
											BoundedVec<u8, T::MaxLength>,
											BoundedVec<u8, T::MaxLength>,
										>,
									> = p.try_into().ok();
									x
								})
								.collect::<Vec<_>>()
								.try_into()
								.ok()
						});
						updated = true;
					}
					if updated {
						Ok(())
					} else {
						Err(Error::<T>::NotChanged)
					}
				} else {
					Err(Error::<T>::NotExists)
				}
			})?;

			Self::deposit_event(Event::OrganizationUpdated(org_id));

			Ok(().into())
		}

		/// Suspend organization
		///
		/// The dispatch origin for this call must match `T::ForceOrigin`.
		#[pallet::weight(
            <T as Config>::WeightInfo::suspend_org()
        )]
		pub fn suspend_org(
			origin: OriginFor<T>,
			org_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;

			// W: 1 db read
			ensure!(Organizations::<T>::contains_key(&org_id), Error::<T>::NotExists);

			// W: 1 db write
			Organizations::<T>::try_mutate(org_id, |org| {
				org.as_mut()
					.map(|org| {
						org.suspended = true;
					})
					.ok_or(Error::<T>::NotExists)
			})?;

			Ok(().into())
		}

		/// Set organization flags
		#[pallet::weight(
            <T as Config>::WeightInfo::set_flags()
        )]
		pub fn set_flags(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			flags: FlagDataBits,
		) -> DispatchResultWithPostInfo {
			let origin_1 = ensure_signed(origin.clone())?;

			let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			if !(org.admin == origin_1 ||
				T::Did::valid_delegate(&org_id, &b"OrgAdmin".to_vec(), &origin_1).is_ok()) ||
				flags.contains(FlagDataBit::System) ||
				flags.contains(FlagDataBit::Verified)
			{
				T::ForceOrigin::ensure_origin(origin)?;
			} else {
				ensure!(!org.suspended, Error::<T>::Suspended);
			}

			OrganizationFlagData::<T>::try_mutate(org_id, |v| -> Result<(), DispatchError> {
				*v = Some(flags);
				Ok(().into())
			})?;

			Ok(().into())
		}

		/// Add member to the organization.
		#[pallet::weight(
            <T as Config>::WeightInfo::add_members( accounts.len() as u32 )
        )]
		pub fn add_members(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			accounts: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(accounts.len() > 0, Error::<T>::InvalidParameter);

			let org = Self::ensure_access(&sender, &org_id)?;

			ensure!(!org.suspended, Error::<T>::Suspended);

			let existing_members =
				<Members<T>>::get(&org_id).unwrap_or_else(|| BoundedVec::default());

			ensure!(
				(existing_members.len() as u32) < T::MaxMemberCount::get(),
				Error::<T>::MaxMemberReached
			);
			ensure!(
				!existing_members.iter().any(|a| accounts.iter().any(|b| *b == *a)),
				Error::<T>::AlreadyExists
			);

			let mut members: Vec<T::AccountId> = Vec::new();

			for account_id in existing_members.into_iter() {
				members.push(account_id);
			}

			for account_id in accounts.iter() {
				members.push(account_id.clone());
			}

			members.sort();

			let members: BoundedVec<T::AccountId, T::MaxMemberCount> =
				members.clone().try_into().map_err(|_| Error::<T>::MaxMemberReached)?;

			<Members<T>>::insert(&org_id, members);

			// <pallet_did::Pallet<T>>::create_delegate(&sender, &org.id, &account_id,
			// b"OrgMember");

			for account_id in accounts {
				Self::deposit_event(Event::MemberAdded(org_id.clone(), account_id));
			}

			Ok(().into())
		}

		/// Remove member from organization.
		#[pallet::weight(<T as Config>::WeightInfo::remove_member())]
		pub fn remove_member(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			account_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			Self::ensure_access(&origin, &org_id)?;

			ensure!(!org.suspended, Error::<T>::Suspended);

			let mut members = <Members<T>>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			ensure!(members.iter().any(|a| *a == account_id), Error::<T>::NotExists);

			let _members: Vec<T::AccountId> =
				members.into_iter().filter(|a| *a != account_id).collect();
			members = to_bounded!(_members, Error::<T>::MaxMemberReached);
			Members::<T>::insert(org_id.clone(), members);

			Self::deposit_event(Event::MemberRemoved(org_id, account_id));

			Ok(().into())
		}

		/// Change organization admin,
		/// the origin must be current admin or conform to `ForceOrigin`.
		#[pallet::weight(
            <T as Config>::WeightInfo::set_admin()
        )]
		pub fn set_admin(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			account_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// harus member terlebih dahulu untuk jadi admin
			ensure!(Self::is_member(&org_id, &account_id), Error::<T>::NotMember);

			let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			if org.admin != who {
				T::ForceOrigin::ensure_origin(origin)?;
			} else {
				ensure!(!org.suspended, Error::<T>::Suspended);
			}

			ensure!(org.admin != account_id, Error::<T>::AlreadySet);

			<Organizations<T>>::mutate(&org_id, |org| {
				if let Some(org) = org {
					org.admin = account_id.clone();
				}
			});

			Self::deposit_event(Event::AdminChanged(org_id, account_id));

			Ok(().into())
		}

		/// Delegate admin access to other user.
		/// User who delegated will have all admin access
		/// except:
		/// 1. Change super admin (set_admin).
		/// 2. Delegate access to other user.
		/// 3. Revoke access from other user.
		///
		/// Use _did_ for share access with expiration.
		///
		/// Only admin of organization can do this operation.
		#[pallet::weight(
            <T as Config>::WeightInfo::delegate_access()
        )]
		pub fn delegate_access(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			to: T::AccountId,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			Self::h_delegate_access_as(&origin, &org_id, &to, &b"OrgAdmin".to_vec(), valid_for)?;
			Ok(().into())
		}

		/// Revoke admin access from user.
		///
		/// Use _did_ for revoke delegation access.
		///
		/// Only super admin of this organization can do this.
		#[pallet::weight(<T as Config>::WeightInfo::revoke_access())]
		pub fn revoke_access(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			delegate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			ensure!(!org.suspended, Error::<T>::Suspended);
			ensure!(org.admin == who, Error::<T>::PermissionDenied);

			T::Did::revoke_delegate_nocheck(&who, &org_id, &b"OrgAdmin".to_vec(), &delegate)?;

			Ok(().into())
		}

		/// Delegate access to other account
		/// with custom type.
		#[pallet::weight(
            <T as Config>::WeightInfo::delegate_access_as()
        )]
		pub fn delegate_access_as(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			to: T::AccountId,
			delegate_type: Vec<u8>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			Self::h_delegate_access_as(&origin, &org_id, &to, &delegate_type, valid_for)?;
			Ok(().into())
		}

		/// Transfer balance from this organization to another org/account.
		///
		/// Only super admin allowed to do this opperation.
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] value: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		) -> DispatchResultWithPostInfo {
			let transactor = ensure_signed(origin)?;

			let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

			ensure!(!org.suspended, Error::<T>::Suspended);
			ensure!(org.admin == transactor, Error::<T>::PermissionDenied);

			let dest = T::Lookup::lookup(dest)?;
			T::Currency::transfer(&org_id, &dest, value, AllowDeath)?;
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

macro_rules! method_is_flag {
	($funcname:ident, $flag:ident, $name:expr) => {
		#[doc = "Check whether organization is "]
		#[doc=$name]
		pub fn $funcname(id: T::AccountId) -> bool {
			<OrganizationFlagData<T>>::get(id)
				.map(|a| (*a).contains(FlagDataBit::$flag))
				.unwrap_or(false)
		}
	};
	($funcname:ident, $flag:ident) => {
		method_is_flag!($funcname, $flag, stringify!($flag));
	};
}

/// The main implementation of this Organization pallet.
impl<T: Config> Pallet<T> {
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

	/// Memastikan origin dapat akses resource.
	///
	/// Prosedur ini akan memeriksa apakah origin admin
	/// atau delegator.
	pub fn ensure_access(
		origin: &T::AccountId,
		org_id: &T::AccountId,
	) -> Result<
		Organization<
			T::AccountId,
			T::BlockNumber,
			BoundedVec<u8, T::MaxLength>,
			BoundedVec<
				Property<BoundedVec<u8, T::MaxLength>, BoundedVec<u8, T::MaxLength>>,
				T::MaxLength,
			>,
		>,
		Error<T>,
	> {
		let org = Self::organization(&org_id).ok_or(Error::<T>::NotExists)?;

		if &org.admin != origin {
			T::Did::valid_delegate(&org_id, &b"OrgAdmin".to_vec(), &origin)
				.map_err(|_| Error::<T>::PermissionDenied)?;
		}

		Ok(org)
	}

	/// Memastikan bahwa akun memiliki akses pada organisasi.
	/// bukan hanya akses, ini juga memastikan organisasi dalam posisi tidak suspended.
	pub fn ensure_access_active_id(
		origin: &T::AccountId,
		org_id: &T::AccountId,
	) -> Result<(), Error<T>> {
		let org = Self::organization(&org_id).ok_or(Error::<T>::NotExists)?;
		Self::ensure_access_active(origin, &org)
	}

	/// Memastikan bahwa akun memiliki akses pada organisasi.
	/// bukan hanya akses, ini juga memastikan organisasi dalam posisi aktif (tidak suspended).
	pub fn ensure_access_active(
		origin: &T::AccountId,
		org: &Organization<
			T::AccountId,
			T::BlockNumber,
			BoundedVec<u8, T::MaxLength>,
			BoundedVec<
				Property<BoundedVec<u8, T::MaxLength>, BoundedVec<u8, T::MaxLength>>,
				T::MaxLength,
			>,
		>,
	) -> Result<(), Error<T>> {
		// ensure!(&org.admin == origin, Error::<T>::PermissionDenied);

		if &org.admin != origin {
			T::Did::valid_delegate(&org.id, &b"OrgAdmin".to_vec(), &origin)
				.map_err(|_| Error::<T>::PermissionDenied)?;
		}

		ensure!(!org.suspended, Error::<T>::PermissionDenied);
		Ok(())
	}

	/// Get next Organization ID
	pub fn next_index() -> Result<u64, Error<T>> {
		<OrgIdIndex<T>>::mutate(|o| {
			*o = Some(o.map_or(1, |vo| vo.saturating_add(1)));
			*o
		})
		.ok_or(Error::<T>::CannotGenId)
	}

	/// Check whether account is member of the organization
	pub fn is_member(id: &T::AccountId, account_id: &T::AccountId) -> bool {
		<Members<T>>::get(id)
			.map(|a| a.into_inner().iter().any(|id| id == account_id))
			.unwrap_or(false)
	}

	/// Check whether the ID is organization account.
	pub fn is_organization(id: &T::AccountId) -> bool {
		Self::organization(id).is_some()
	}

	/// Delegate access to someone with custom type.
	pub fn h_delegate_access_as(
		origin: &T::AccountId,
		org_id: &T::AccountId,
		to: &T::AccountId,
		delegate_type: &[u8],
		valid_for: Option<T::BlockNumber>,
	) -> DispatchResult {
		let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

		ensure!(!org.suspended, Error::<T>::Suspended);
		ensure!(&org.admin == origin, Error::<T>::PermissionDenied);

		T::Did::create_delegate(&origin, &org_id, &to, &delegate_type.to_vec(), valid_for)?;

		Ok(())
	}

	method_is_flag!(is_active, Active);
	method_is_flag!(is_verified, Verified);
	method_is_flag!(is_gov, Government);
	method_is_flag!(is_foundation, Foundation);
	method_is_flag!(is_system, System);

	/// Check whether organization suspended
	pub fn is_suspended(id: T::AccountId) -> bool {
		Self::organization(id).map(|a| a.suspended).unwrap_or(true)
	}

	/// Get admin of the organization
	pub fn get_admin(id: T::AccountId) -> Option<T::AccountId> {
		Self::organization(id).map(|a| a.admin)
	}
}

#[cfg(test)]
mod tests;
