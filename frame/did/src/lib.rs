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
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, ensure, traits::Time};
use frame_system::ensure_signed;
pub use pallet::*;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_std::prelude::*;
use scale_info::TypeInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// use crate::did::Did;
use crate::types::{Attribute, AttributeTransaction, AttributedId};
use codec::{Decode, Encode};
pub use did::Did;
pub use weights::WeightInfo;

mod did;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // /// The origin which may forcibly set or remove a name. Root can always do this.
        // type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;

        type Public: IdentifyAccount<AccountId = Self::AccountId>;
        type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;
        type Time: Time;
    }

    #[pallet::error]
    pub enum Error<T> {
        NotOwner,
        AlreadyExists,
        InvalidDelegate,
        BadSignature,
        AttributeNameTooLong,
        AttributeCreationFailed,
        AttributeResetFailed,
        AttributeRemovalFailed,
        AttributeAlreadyExists,
        InvalidAttribute,
        Overflow,
        BadTransaction,
        TransactionNameTooLong,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance", T::Signature = "Signature", T::BlockNumber = "BlockNumber")]
    pub enum Event<T: Config> {
        OwnerChanged(T::AccountId, T::AccountId, T::AccountId, T::BlockNumber),
        DelegateAdded(T::AccountId, Vec<u8>, T::AccountId, Option<T::BlockNumber>),
        DelegateRevoked(T::AccountId, Vec<u8>, T::AccountId),
        AttributeAdded(T::AccountId, Vec<u8>, Option<T::BlockNumber>),
        AttributeRevoked(T::AccountId, Vec<u8>, T::BlockNumber),
        AttributeDeleted(T::AccountId, Vec<u8>, T::BlockNumber),
        AttributeTransactionExecuted(AttributeTransaction<T::Signature, T::AccountId>),
    }

    /// Delegates are only valid for a specific period defined as blocks number.
    #[pallet::storage]
    #[pallet::getter(fn delegate_of)]
    pub type DelegateOf<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, Vec<u8>, T::AccountId), T::BlockNumber>;

    // Attributes are only valid for a specific period defined as blocks number.
    #[pallet::storage]
    #[pallet::getter(fn attribute_of)]
    pub type AttributeOf<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (T::AccountId, [u8; 32]),
        Attribute<T::BlockNumber, <<T as Config>::Time as Time>::Moment>,
    >;

    /// Attribute nonce used to generate a unique hash even if the attribute is deleted and recreated.
    #[pallet::storage]
    #[pallet::getter(fn nonce_of)]
    pub type AttributeNonce<T: Config> = StorageMap<_, Twox64Concat, (T::AccountId, Vec<u8>), u64>;

    /// Identity owner.
    #[pallet::storage]
    #[pallet::getter(fn owner_of)]
    pub type OwnerOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

    /// Tracking the latest identity update.
    #[pallet::storage]
    #[pallet::getter(fn updated_by)]
    pub type UpdatedBy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (
            T::AccountId,
            T::BlockNumber,
            <<T as Config>::Time as Time>::Moment,
        ),
    >;

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

            let now_timestamp = T::Time::now();
            let now_block_number = <frame_system::Pallet<T>>::block_number();
            <UpdatedBy<T>>::insert(&identity, (who, now_block_number, now_timestamp));

            Self::deposit_event(Event::DelegateAdded(
                identity,
                delegate_type,
                delegate,
                valid_for,
            ));
            Ok(().into())
        }

        /// Transfers ownership of an identity.
        #[pallet::weight(0)]
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
        #[pallet::weight(0)]
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

            Self::revoke_delegate_internal(&who, &identity, &delegate_type, &delegate);

            Self::deposit_event(Event::DelegateRevoked(identity, delegate_type, delegate));
            Ok(().into())
        }

        /// Creates a new attribute as part of an identity.
        /// Sets its expiration period.
        #[pallet::weight(0)]
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
        #[pallet::weight(0)]
        pub fn revoke_attribute(
            origin: OriginFor<T>,
            identity: T::AccountId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);

            Self::reset_attribute(who, &identity, &name)?;
            Self::deposit_event(Event::AttributeRevoked(
                identity,
                name,
                <frame_system::Pallet<T>>::block_number(),
            ));
            Ok(().into())
        }

        /// Removes an attribute from an identity. This attribute/property becomes unavailable.
        #[pallet::weight(0)]
        pub fn delete_attribute(
            origin: OriginFor<T>,
            identity: T::AccountId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::is_owner(&identity, &who)?;
            ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);

            let now_block_number = <frame_system::Pallet<T>>::block_number();
            let result = Self::attribute_and_id(&identity, &name);

            match result {
                Some((_, id)) => <AttributeOf<T>>::remove((&identity, &id)),
                None => return Err(Error::<T>::AttributeRemovalFailed.into()),
            }

            <UpdatedBy<T>>::insert(&identity, (&who, &now_block_number, T::Time::now()));

            Self::deposit_event(Event::AttributeDeleted(identity, name, now_block_number));
            Ok(().into())
        }

        /// Executes off-chain signed transaction.
        #[pallet::weight(0)]
        pub fn execute(
            origin: OriginFor<T>,
            transaction: AttributeTransaction<T::Signature, T::AccountId>,
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
            Self {
                _phantom: Default::default(),
            }
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
    ///
    fn get_nonce(identity: &T::AccountId, name: &[u8]) -> u64 {
        match Self::nonce_of((&identity, name.to_vec())) {
            Some(nonce) => nonce,
            None => 0u64,
        }
    }

    /// Set identity owner.
    ///
    /// This function should not fail.
    pub fn set_owner(who: &T::AccountId, identity: &T::AccountId, new_owner: &T::AccountId) {
        let now_timestamp = T::Time::now();
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

    /// Revoke delegate without check
    pub fn revoke_delegate_internal(
        who: &T::AccountId,
        identity: &T::AccountId,
        delegate_type: &Vec<u8>,
        delegate: &T::AccountId,
    ) {
        let now_timestamp = T::Time::now();
        let now_block_number = <frame_system::Pallet<T>>::block_number();

        // Update only the validity period to revoke the delegate.
        <DelegateOf<T>>::mutate((&identity, delegate_type, &delegate), |b| {
            *b = Some(now_block_number)
        });
        <UpdatedBy<T>>::insert(&identity, (who, now_block_number, now_timestamp));
    }

    fn signed_attribute(
        who: T::AccountId,
        encoded: &[u8],
        transaction: &AttributeTransaction<T::Signature, T::AccountId>,
    ) -> DispatchResult {
        // Verify that the Data was signed by the owner or a not expired signer delegate.
        Self::valid_signer(
            &transaction.identity,
            &transaction.signature,
            &encoded,
            &transaction.signer,
        )?;
        Self::is_owner(&transaction.identity, &transaction.signer)?;
        ensure!(
            transaction.name.len() <= 64,
            Error::<T>::TransactionNameTooLong
        );

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
    Did<T::AccountId, T::BlockNumber, <<T as Config>::Time as Time>::Moment, T::Signature>
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
        delegate_type: &[u8],
        delegate: &T::AccountId,
    ) -> DispatchResult {
        ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);
        ensure!(
            Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok()
                || Self::is_owner(identity, delegate).is_ok(),
            Error::<T>::InvalidDelegate
        );
        Ok(())
    }

    /// Validates that a delegate contains_key for specific purpose and remains valid at this block high.
    fn valid_listed_delegate(
        identity: &T::AccountId,
        delegate_type: &[u8],
        delegate: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            <DelegateOf<T>>::contains_key((&identity, delegate_type, &delegate)),
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
        delegate_type: &[u8],
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

        <DelegateOf<T>>::insert((&identity, delegate_type, delegate), &validity);
        Ok(())
    }

    /// Checks if a signature is valid. Used to validate off-chain transactions.
    fn check_signature(
        signature: &T::Signature,
        msg: &[u8],
        signer: &T::AccountId,
    ) -> DispatchResult {
        if signature.verify(msg, signer) {
            Ok(())
        } else {
            Err(Error::<T>::BadSignature.into())
        }
    }

    /// Checks if a signature is valid. Used to validate off-chain transactions.
    fn valid_signer(
        identity: &T::AccountId,
        signature: &T::Signature,
        msg: &[u8],
        signer: &T::AccountId,
    ) -> DispatchResult {
        // Owner or a delegate signer.
        Self::valid_delegate(&identity, b"x25519VerificationKey2018", &signer)?;
        Self::check_signature(&signature, &msg, &signer)
    }

    /// Adds a new attribute to an identity and colects the storage fee.
    fn create_attribute(
        who: &T::AccountId,
        identity: &T::AccountId,
        name: &[u8],
        value: &[u8],
        valid_for: Option<T::BlockNumber>,
    ) -> DispatchResult {
        Self::is_owner(identity, &who)?;

        if Self::attribute_and_id(identity, name).is_some() {
            Err(Error::<T>::AttributeAlreadyExists.into())
        } else {
            let now_timestamp = T::Time::now();
            let now_block_number = <frame_system::Pallet<T>>::block_number();
            let validity: T::BlockNumber = match valid_for {
                Some(blocks) => now_block_number + blocks,
                None => u32::max_value().into(),
            };

            let mut nonce = Self::get_nonce(identity, name);

            let id = (&identity, name, nonce).using_encoded(blake2_256);
            let new_attribute = Attribute {
                name: (&name).to_vec(),
                value: (&value).to_vec(),
                validity,
                creation: now_timestamp,
                nonce,
            };

            // Prevent panic overflow
            nonce = nonce.checked_add(1).ok_or(Error::<T>::Overflow)?;
            <AttributeOf<T>>::insert((identity, &id), new_attribute);

            // update nonce
            <AttributeNonce<T>>::mutate((identity, name.to_vec()), |n| *n = Some(nonce));
            <UpdatedBy<T>>::insert(identity, (who, now_block_number, now_timestamp));
            Ok(())
        }
    }

    /// Updates the attribute validity to make it expire and invalid.
    fn reset_attribute(who: T::AccountId, identity: &T::AccountId, name: &[u8]) -> DispatchResult {
        Self::is_owner(&identity, &who)?;
        // If the attribute contains_key, the latest valid block is set to the current block.
        let result = Self::attribute_and_id(identity, name);
        match result {
            Some((mut attribute, id)) => {
                attribute.validity = <frame_system::Pallet<T>>::block_number();
                <AttributeOf<T>>::mutate((&identity, id), |a| *a = Some(attribute));
            }
            None => return Err(Error::<T>::AttributeResetFailed.into()),
        }

        // Keep track of the updates.
        <UpdatedBy<T>>::insert(
            identity,
            (
                who,
                <frame_system::Pallet<T>>::block_number(),
                T::Time::now(),
            ),
        );
        Ok(())
    }

    /// Validates if an attribute belongs to an identity and it has not expired.
    fn valid_attribute(identity: &T::AccountId, name: &[u8], value: &[u8]) -> DispatchResult {
        ensure!(name.len() <= 64, Error::<T>::InvalidAttribute);
        let result = Self::attribute_and_id(identity, name);

        let (attr, _) = match result {
            Some((attr, id)) => (attr, id),
            None => return Err(Error::<T>::InvalidAttribute.into()),
        };

        if (attr.validity > (<frame_system::Pallet<T>>::block_number()))
            && (attr.value == value.to_vec())
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
        name: &[u8],
    ) -> Option<AttributedId<T::BlockNumber, <<T as Config>::Time as Time>::Moment>> {
        let nonce = Self::nonce_of((&identity, name.to_vec())).unwrap_or(0u64);

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
