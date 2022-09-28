//! # Decentralized ID
//!
//! - [`Did::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Nuchain decentralized identifiers (DIDs) pallet.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_delegate` -
//! * `valid_delegate` -
//! * `is_owner` -

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, ensure, traits::UnixTime, BoundedVec};
use frame_system::ensure_signed;
pub use pallet::*;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{IdentifyAccount, SaturatedConversion, Verify};
use sp_std::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// use crate::did::Did;
use crate::types::{Attribute, AttributeTransaction, AttributedId, DidVerifiableIdentifier};
use codec::{Decode, Encode};
pub use did::Did;
pub use weights::WeightInfo;

mod did;
mod errors;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

macro_rules! to_bounded {
	(*$name:ident, $error:expr) => {
		let $name: BoundedVec<_, _> = $name.clone().try_into().map_err(|()| $error)?;
	};
	($name:ident, $error:expr) => {
		let $name: BoundedVec<_, _> = $name.try_into().map_err(|()| $error)?;
	};
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	/// Reference to a payload of data of variable size.
	pub type Payload = [u8];

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Type for a DID subject identifier.
		type DidIdentifier: Parameter + DidVerifiableIdentifier + MaxEncodedLen;

		// /// The origin which may forcibly set or remove a name. Root can always do this.
		// type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information
		type WeightInfo: WeightInfo;

		type Public: IdentifyAccount<AccountId = Self::AccountId>;
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;
		type Time: UnixTime;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;

		/// The maximum length of a service ID.
		#[pallet::constant]
		type MaxServiceIdLength: Get<u32>;

		/// The maximum length of a service type.
		#[pallet::constant]
		type MaxServiceTypeLength: Get<u32>;

		/// The maximum length of a service endpoint.
		#[pallet::constant]
		type MaxServiceEndpointLength: Get<u32>;

		/// The maximum services per DID.
		#[pallet::constant]
		type MaxServicePerDid: Get<u32>;
	}

	/// Type for a DID subject identifier.
	pub type DidIdentifierOf<T> = <T as Config>::DidIdentifier;

	#[pallet::error]
	pub enum Error<T> {
		NotOwner,
		AlreadyExists,
		InvalidDelegate,
		BadSignature,
		AttributeNameTooLong,
		AttributeValueTooLong,
		AttributeCreationFailed,
		AttributeResetFailed,
		AttributeRemovalFailed,
		AttributeAlreadyExists,
		InvalidAttribute,
		DelegateTypeTooLong,
		Overflow,
		BadTransaction,
		TransactionNameTooLong,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance", T::Signature =
	// "Signature", T::BlockNumber = "BlockNumber")]
	pub enum Event<T: Config> {
		OwnerChanged(T::AccountId, T::AccountId, T::AccountId, T::BlockNumber),
		DelegateAdded(T::AccountId, Vec<u8>, T::AccountId, Option<T::BlockNumber>),
		DelegateRevoked(T::AccountId, Vec<u8>, T::AccountId),
		AttributeAdded(T::AccountId, Vec<u8>, Option<T::BlockNumber>),
		AttributeRevoked(T::AccountId, Vec<u8>, T::BlockNumber),
		AttributeDeleted(T::AccountId, Vec<u8>, T::BlockNumber),
		AttributeTransactionExecuted(
			AttributeTransaction<T::Signature, T::AccountId, BoundedVec<u8, T::MaxLength>>,
		),
	}

	/// Delegates are only valid for a specific period defined as blocks number.
	#[pallet::storage]
	#[pallet::getter(fn delegate_of)]
	pub type DelegateOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, BoundedVec<u8, T::MaxLength>, T::AccountId),
		T::BlockNumber,
	>;

	// Attributes are only valid for a specific period defined as blocks number.
	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub type AttributeOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, [u8; 32]),
		Attribute<T::BlockNumber, BoundedVec<u8, T::MaxLength>>,
	>;

	/// Attribute nonce used to generate a unique hash even if the attribute is deleted and
	/// recreated.
	#[pallet::storage]
	#[pallet::getter(fn nonce_of)]
	pub type AttributeNonce<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, BoundedVec<u8, T::MaxLength>), u64>;

	/// Identity owner.
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub type OwnerOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

	/// Tracking the latest identity update.
	#[pallet::storage]
	#[pallet::getter(fn updated_by)]
	pub type UpdatedBy<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, T::BlockNumber, u64)>;

	/// Did module declaration.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new delegate with an expiration period and for a specific purpose.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// # </weight>
		#[pallet::weight(T::WeightInfo::add_delegate())]
		pub fn add_delegate(
			origin: OriginFor<T>,
			identity: T::AccountId,
			delegate: T::AccountId,
			delegate_type: Vec<u8>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);

			Self::create_delegate(&who, &identity, &delegate, &delegate_type, valid_for)?;

			let now_timestamp = T::Time::now().as_millis().saturated_into::<u64>();
			let now_block_number = <frame_system::Pallet<T>>::block_number();
			<UpdatedBy<T>>::insert(&identity, (who, now_block_number, now_timestamp));

			Self::deposit_event(Event::DelegateAdded(identity, delegate_type, delegate, valid_for));
			Ok(().into())
		}

		/// Transfers ownership of an identity.
		#[pallet::weight(T::WeightInfo::change_owner())]
		pub fn change_owner(
			origin: OriginFor<T>,
			identity: T::AccountId,
			new_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::is_owner(&identity, &who)?;

			Self::set_owner(&who, &identity, &new_owner);

			Ok(().into())
		}

		/// Revokes an identity's delegate by setting its expiration to the current block number.
		#[pallet::weight(T::WeightInfo::revoke_delegate())]
		pub fn revoke_delegate(
			origin: OriginFor<T>,
			identity: T::AccountId,
			delegate_type: Vec<u8>,
			delegate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::is_owner(&identity, &who)?;
			Self::valid_listed_delegate(&identity, &delegate_type, &delegate)?;
			ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);

			Self::revoke_delegate_nocheck(&who, &identity, &delegate_type, &delegate)?;

			Self::deposit_event(Event::DelegateRevoked(identity, delegate_type, delegate));
			Ok(().into())
		}

		/// Creates a new attribute as part of an identity.
		/// Sets its expiration period.
		#[pallet::weight(T::WeightInfo::add_attribute())]
		pub fn add_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeNameTooLong);

			Self::create_attribute(&who, &identity, &name, &value, valid_for)?;
			Self::deposit_event(Event::AttributeAdded(identity, name, valid_for));
			Ok(().into())
		}

		/// Revokes an attribute/property from an identity.
		/// Sets its expiration period to the actual block number.
		#[pallet::weight(T::WeightInfo::revoke_attribute())]
		pub fn revoke_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);

			to_bounded!(name, Error::<T>::AttributeNameTooLong);

			Self::reset_attribute(who, &identity, &name)?;
			Self::deposit_event(Event::AttributeRevoked(
				identity,
				name.into(),
				<frame_system::Pallet<T>>::block_number(),
			));
			Ok(().into())
		}

		/// Removes an attribute from an identity. This attribute/property becomes unavailable.
		#[pallet::weight(T::WeightInfo::delete_attribute())]
		pub fn delete_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::is_owner(&identity, &who)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);

			to_bounded!(name, Error::<T>::AttributeNameTooLong);

			let now_block_number = <frame_system::Pallet<T>>::block_number();
			let result = Self::attribute_and_id(&identity, &name);

			match result {
				Some((_, id)) => <AttributeOf<T>>::remove((&identity, &id)),
				None => return Err(Error::<T>::AttributeRemovalFailed.into()),
			}

			let now = T::Time::now().as_millis().saturated_into::<u64>();

			<UpdatedBy<T>>::insert(&identity, (&who, &now_block_number, now));

			Self::deposit_event(Event::AttributeDeleted(identity, name.into(), now_block_number));
			Ok(().into())
		}

		/// Executes off-chain signed transaction.
		#[pallet::weight(20_000_000)]
		pub fn execute(
			origin: OriginFor<T>,
			transaction: AttributeTransaction<
				T::Signature,
				T::AccountId,
				BoundedVec<u8, T::MaxLength>,
			>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut encoded = transaction.name.encode();
			encoded.extend(transaction.value.encode());
			encoded.extend(transaction.validity.encode());
			encoded.extend(transaction.identity.encode());

			// Execute the storage update if the signer is valid.
			Self::signed_attribute(who, &encoded, &transaction)?;
			Self::deposit_event(Event::AttributeTransactionExecuted(transaction));
			Ok(().into())
		}
	}

	// ----------------------------------------------------------------
	//                      HOOKS
	// ----------------------------------------------------------------
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// fn offchain_worker(n: T::BlockNumber){
		//     // @TODO(you): Your off-chain logic here
		// }
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

/// The main implementation of this Did pallet.
impl<T: Config> Pallet<T> {
	/// Get nonce for _identity_ and _name_.
	fn get_nonce(identity: &T::AccountId, name: &BoundedVec<u8, T::MaxLength>) -> u64 {
		match Self::nonce_of((&identity, name)) {
			Some(nonce) => nonce,
			None => 0u64,
		}
	}

	fn signed_attribute(
		who: T::AccountId,
		encoded: &Vec<u8>,
		transaction: &AttributeTransaction<
			T::Signature,
			T::AccountId,
			BoundedVec<u8, T::MaxLength>,
		>,
	) -> DispatchResult {
		// Verify that the Data was signed by the owner or a not expired signer delegate.
		Self::valid_signer(
			&transaction.identity,
			&transaction.signature,
			&encoded,
			&transaction.signer,
		)?;
		Self::is_owner(&transaction.identity, &transaction.signer)?;
		ensure!(transaction.name.len() <= 64, Error::<T>::TransactionNameTooLong);

		let now_block_number = <frame_system::Pallet<T>>::block_number();
		let validity = now_block_number + transaction.validity.into();

		// If validity was set to 0 in the transaction,
		// it will set the attribute latest valid block to the actual block.
		if validity > now_block_number {
			Self::create_attribute(
				&who,
				&transaction.identity,
				&transaction.name,
				&transaction.value,
				Some(transaction.validity.into()),
			)?;
		} else {
			Self::reset_attribute(who, &transaction.identity, &transaction.name)?;
		}
		Ok(())
	}
}

impl<T: Config>
	Did<T::AccountId, T::BlockNumber, T::Time, T::Signature, BoundedVec<u8, T::MaxLength>>
	for Pallet<T>
{
	/// Validates if the AccountId 'actual_owner' owns the identity.
	fn is_owner(identity: &T::AccountId, actual_owner: &T::AccountId) -> DispatchResult {
		let owner = Self::identity_owner(identity);
		match owner == *actual_owner {
			true => Ok(()),
			false => Err(Error::<T>::NotOwner.into()),
		}
	}

	/// Set identity owner.
	///
	/// This function should not fail.
	fn set_owner(who: &T::AccountId, identity: &T::AccountId, new_owner: &T::AccountId) {
		let now_timestamp = T::Time::now().as_millis().saturated_into::<u64>();
		let now_block_number = <frame_system::Pallet<T>>::block_number();

		if <OwnerOf<T>>::contains_key(&identity) {
			// Update to new owner.
			<OwnerOf<T>>::mutate(&identity, |o| *o = Some(new_owner.clone()));
		} else {
			// Add to new owner.
			<OwnerOf<T>>::insert(&identity, &new_owner);
		}
		// Save the update time and block.
		<UpdatedBy<T>>::insert(&identity, (&who, &now_block_number, &now_timestamp));

		Self::deposit_event(Event::<T>::OwnerChanged(
			identity.clone(),
			who.clone(),
			new_owner.clone(),
			now_block_number,
		));
	}

	/// Get the identity owner if set.
	/// If never changed, returns the identity as its owner.
	fn identity_owner(identity: &T::AccountId) -> T::AccountId {
		match Self::owner_of(identity) {
			Some(id) => id,
			None => identity.clone(),
		}
	}

	/// Validates if a delegate belongs to an identity and it has not expired.
	///
	/// return Ok if valid.
	fn valid_delegate(
		identity: &T::AccountId,
		delegate_type: &Vec<u8>,
		delegate: &T::AccountId,
	) -> DispatchResult {
		ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);
		ensure!(
			Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok() ||
				Self::is_owner(identity, delegate).is_ok(),
			Error::<T>::InvalidDelegate
		);
		Ok(())
	}

	/// Validates that a delegate contains_key for specific purpose and remains valid at this block
	/// high.
	fn valid_listed_delegate(
		identity: &T::AccountId,
		delegate_type: &Vec<u8>,
		delegate: &T::AccountId,
	) -> DispatchResult {
		to_bounded!(*delegate_type, Error::<T>::DelegateTypeTooLong);

		ensure!(
			<DelegateOf<T>>::contains_key((&identity, &delegate_type, &delegate)),
			Error::<T>::InvalidDelegate
		);

		let validity = Self::delegate_of((identity, delegate_type, delegate));
		match validity > Some(<frame_system::Pallet<T>>::block_number()) {
			true => Ok(()),
			false => Err(Error::<T>::InvalidDelegate.into()),
		}
	}

	// Creates a new delegete for an account.
	fn create_delegate(
		who: &T::AccountId,
		identity: &T::AccountId,
		delegate: &T::AccountId,
		delegate_type: &Vec<u8>,
		valid_for: Option<T::BlockNumber>,
	) -> DispatchResult {
		Self::is_owner(&identity, who)?;
		ensure!(who != delegate, Error::<T>::InvalidDelegate);
		ensure!(
			!Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok(),
			Error::<T>::AlreadyExists
		);

		let now_block_number = <frame_system::Pallet<T>>::block_number();
		let validity: T::BlockNumber = match valid_for {
			Some(blocks) => now_block_number + blocks,
			None => u32::max_value().into(),
		};

		to_bounded!(*delegate_type, Error::<T>::DelegateTypeTooLong);

		<DelegateOf<T>>::insert((&identity, delegate_type, delegate), &validity);
		Ok(())
	}

	/// Revoke delegate without check
	fn revoke_delegate_nocheck(
		who: &T::AccountId,
		identity: &T::AccountId,
		delegate_type: &Vec<u8>,
		delegate: &T::AccountId,
	) -> DispatchResult {
		let now_timestamp = T::Time::now().as_millis().saturated_into::<u64>();
		let now_block_number = <frame_system::Pallet<T>>::block_number();

		to_bounded!(*delegate_type, Error::<T>::DelegateTypeTooLong);

		// Update only the validity period to revoke the delegate.
		<DelegateOf<T>>::mutate((&identity, delegate_type, &delegate), |b| {
			*b = Some(now_block_number)
		});
		<UpdatedBy<T>>::insert(&identity, (who, now_block_number, now_timestamp));

		Ok(())
	}

	/// Checks if a signature is valid. Used to validate off-chain transactions.
	fn check_signature(
		signature: &T::Signature,
		msg: &Vec<u8>,
		signer: &T::AccountId,
	) -> DispatchResult {
		if signature.verify(&msg[..], signer) {
			Ok(())
		} else {
			Err(Error::<T>::BadSignature.into())
		}
	}

	/// Checks if a signature is valid. Used to validate off-chain transactions.
	fn valid_signer(
		identity: &T::AccountId,
		signature: &T::Signature,
		msg: &Vec<u8>,
		signer: &T::AccountId,
	) -> DispatchResult {
		// Owner or a delegate signer.
		Self::valid_delegate(&identity, &b"x25519VerificationKey2018".to_vec(), &signer)?;
		Self::check_signature(&signature, &msg, &signer)
	}

	/// Adds a new attribute to an identity and colects the storage fee.
	fn create_attribute(
		who: &T::AccountId,
		identity: &T::AccountId,
		name: &Vec<u8>,
		value: &Vec<u8>,
		valid_for: Option<T::BlockNumber>,
	) -> DispatchResult {
		Self::is_owner(identity, &who)?;

		let bounded_name: BoundedVec<_, _> =
			name.clone().try_into().map_err(|()| Error::<T>::AttributeNameTooLong)?;
		let bounded_value: BoundedVec<_, _> =
			value.clone().try_into().map_err(|()| Error::<T>::AttributeValueTooLong)?;

		if Self::attribute_and_id(identity, &bounded_name).is_some() {
			Err(Error::<T>::AttributeAlreadyExists.into())
		} else {
			let now_timestamp = T::Time::now().as_millis().saturated_into::<u64>();
			let now_block_number = <frame_system::Pallet<T>>::block_number();
			let validity: T::BlockNumber = match valid_for {
				Some(blocks) => now_block_number + blocks,
				None => u32::max_value().into(),
			};

			let mut nonce = Self::get_nonce(identity, &bounded_name);

			let id = (&identity, name, nonce).using_encoded(blake2_256);

			let new_attribute = Attribute {
				name: bounded_name.clone(),
				value: bounded_value,
				validity,
				creation: now_timestamp,
				nonce,
			};

			// Prevent panic overflow
			nonce = nonce.checked_add(1).ok_or(Error::<T>::Overflow)?;
			<AttributeOf<T>>::insert((identity, &id), new_attribute);

			// update nonce
			<AttributeNonce<T>>::mutate((identity, bounded_name), |n| *n = Some(nonce));
			<UpdatedBy<T>>::insert(identity, (who, now_block_number, now_timestamp));
			Ok(())
		}
	}

	/// Updates the attribute validity to make it expire and invalid.
	fn reset_attribute(
		who: T::AccountId,
		identity: &T::AccountId,
		name: &BoundedVec<u8, T::MaxLength>,
	) -> DispatchResult {
		Self::is_owner(&identity, &who)?;
		// If the attribute contains_key, the latest valid block is set to the current block.

		let result = Self::attribute_and_id(identity, name);
		match result {
			Some((mut attribute, id)) => {
				attribute.validity = <frame_system::Pallet<T>>::block_number();
				<AttributeOf<T>>::mutate((&identity, id), |a| *a = Some(attribute));
			},
			None => return Err(Error::<T>::AttributeResetFailed.into()),
		}

		// Keep track of the updates.
		<UpdatedBy<T>>::insert(
			identity,
			(
				who,
				<frame_system::Pallet<T>>::block_number(),
				T::Time::now().as_millis().saturated_into::<u64>(),
			),
		);
		Ok(())
	}

	/// Validates if an attribute belongs to an identity and it has not expired.
	fn valid_attribute(
		identity: &T::AccountId,
		name: &BoundedVec<u8, T::MaxLength>,
		value: &BoundedVec<u8, T::MaxLength>,
	) -> DispatchResult {
		ensure!(name.len() <= 64, Error::<T>::InvalidAttribute);
		let result = Self::attribute_and_id(identity, name);

		let (attr, _) = match result {
			Some((attr, id)) => (attr, id),
			None => return Err(Error::<T>::InvalidAttribute.into()),
		};

		if (attr.validity > (<frame_system::Pallet<T>>::block_number())) &&
			(attr.value == value.to_vec())
		{
			Ok(())
		} else {
			Err(Error::<T>::InvalidAttribute.into())
		}
	}

	/// Returns the attribute and its hash identifier.
	/// Uses a nonce to keep track of identifiers making them unique after attributes deletion.
	fn attribute_and_id(
		identity: &T::AccountId,
		name: &BoundedVec<u8, T::MaxLength>,
	) -> Option<AttributedId<T::BlockNumber, BoundedVec<u8, T::MaxLength>>> {
		let nonce = Self::nonce_of((&identity, name)).unwrap_or(0u64);

		// Used for first time attribute creation
		let lookup_nonce = match nonce {
			0u64 => 0u64,
			_ => nonce - 1u64,
		};

		// Looks up for the existing attribute.
		// Needs to use actual attribute nonce -1.
		let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);

		if <AttributeOf<T>>::contains_key((&identity, &id)) {
			Self::attribute_of((identity, id)).and_then(|a| Some((a, id)))
		} else {
			None
		}
	}
}
