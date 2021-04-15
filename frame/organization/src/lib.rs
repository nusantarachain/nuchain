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
//! * `create_org` - Create organization.
//! * `suspend_org` - Suspen organization.
//! * `add_member` - Add account as member to the organization.
//! * `remove_member` - Remove account member from organization.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    ensure,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get, OnUnbalanced,
        ReservableCurrency, WithdrawReasons,
    },
};
use frame_system::ensure_signed;
use sp_core::crypto::{UncheckedFrom, Wraps};
use sp_runtime::traits::{Hash, MaybeDisplay, StaticLookup};
use sp_std::{fmt::Debug, prelude::*};

use enumflags2::{bitflags, BitFlags};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode, EncodeLike};

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_did::{self as did, Did};
    use sp_std::vec;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config + did::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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
        type MaxMemberCount: Get<usize>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct Organization<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Organization ID
        pub id: AccountId,

        /// object name
        pub name: Vec<u8>,

        /// Description about the organization.
        pub description: Vec<u8>,

        /// admin of the object
        pub admin: AccountId,

        /// Official website url
        pub website: Vec<u8>,

        /// Official email address
        pub email: Vec<u8>,

        /// Whether the organization suspended or not
        pub suspended: bool,
    }

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

        /// Description too short
        DescriptionTooShort,

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

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
    pub enum Event<T: Config> {
        /// Some object added inside the system.
        OrganizationAdded(T::AccountId, T::AccountId),

        /// When object deleted
        OrganizationDeleted(T::AccountId),

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
    pub type Organizations<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Organization<T::AccountId>>;

    /// Link organization index -> organization hash.
    /// Useful for lookup organization from hash.
    #[pallet::storage]
    #[pallet::getter(fn organization_index)]
    pub type OrganizationIndexOf<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

    /// Pair user -> list of handled organizations
    #[pallet::storage]
    pub type OrganizationLink<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>>;

    /// Membership store, stored as an ordered Vec.
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AccountId>>;

    #[bitflags(default = Active)]
    #[repr(u64)]
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
    pub enum FlagDataBit {
        Active = 0b0000000000000000000000000000000000000000000000000000000000000001,
        Verified = 0b0000000000000000000000000000000000000000000000000000000000000010,
        Government = 0b0000000000000000000000000000000000000000000000000000000000000100,
        System = 0b0000000000000000000000000000000000000000000000000000000000001000,
        Edu = 0b0000000000000000000000000000000000000000000000000000000000010000,
        Company = 0b0000000000000000000000000000000000000000000000000000000000100000,
        Foundation = 0b0000000000000000000000000000000000000000000000000000000001000000,
    }

    #[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
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

    /// Flag of the organization
    #[pallet::storage]
    #[pallet::getter(fn flags)]
    pub type OrganizationFlagData<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, FlagDataBits>;

    pub struct EnsureOrgAdmin<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrgAdmin<T> {
        type Success = (T::AccountId, Vec<T::AccountId>);

        fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
            o.into().and_then(|o| match o {
                frame_system::RawOrigin::Signed(ref who) => {
                    let vs = OrganizationLink::<T>::get(who.clone())
                        .ok_or(T::Origin::from(o.clone()))?;
                    Ok((who.clone(), vs.clone()))
                }
                r => Err(T::Origin::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> T::Origin {
            O::from(RawOrigin::Signed(Default::default()))
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn object_index)]
    pub type OrgIdIndex<T> = StorageValue<_, u64>;

    /// Organization module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    {
        /// Add new object.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(100_000_000)]
        pub fn create_org(
            origin: OriginFor<T>,
            name: Vec<u8>,
            description: Vec<u8>,
            admin: <T::Lookup as StaticLookup>::Source,
            website: Vec<u8>,
            email: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            ensure!(
                name.len() >= T::MinOrgNameLength::get(),
                Error::<T>::NameTooShort
            );
            ensure!(
                name.len() <= T::MaxOrgNameLength::get(),
                Error::<T>::NameTooLong
            );

            let index = Self::next_index()?;

            ensure!(
                !OrganizationIndexOf::<T>::contains_key(index),
                Error::<T>::BadIndex
            );

            let admin = T::Lookup::lookup(admin)?;

            // Process the payment
            let cost = T::CreationFee::get();

            // Process payment
            T::Payment::on_unbalanced(T::Currency::withdraw(
                &origin,
                cost,
                WithdrawReasons::FEE,
                KeepAlive,
            )?);

            // generate organization hash
            let org_id: T::AccountId = UncheckedFrom::unchecked_from(T::Hashing::hash(
                &index
                    .to_le_bytes()
                    .into_iter()
                    .chain(name.iter())
                    .chain(description.iter())
                    .chain(website.iter())
                    .chain(email.iter())
                    .cloned()
                    .collect::<Vec<u8>>(),
            ));

            Organizations::<T>::insert(
                org_id.clone(),
                Organization {
                    id: org_id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    admin: admin.clone(),
                    website: website.clone(),
                    email: email.clone(),
                    suspended: false,
                },
            );

            <OrganizationIndexOf<T>>::insert(index, org_id.clone());

            if OrganizationLink::<T>::contains_key(&admin) {
                OrganizationLink::<T>::mutate(&admin, |ref mut vs| {
                    vs.as_mut().map(|vsi| vsi.push(org_id.clone()))
                });
            } else {
                OrganizationLink::<T>::insert(&admin, sp_std::vec![org_id.clone()]);
            }

            <OrganizationFlagData<T>>::insert::<_, FlagDataBits>(
                org_id.clone(),
                Default::default(),
            );

            // DID add attribute
            <pallet_did::Module<T>>::create_delegate(&org_id, &org_id, &admin, b"OrgAdmin", None)?;
            <pallet_did::Module<T>>::create_attribute(&admin, &admin, b"Org", &name, None)?;

            Self::deposit_event(Event::OrganizationAdded(org_id, admin));

            Ok(().into())
        }

        /// Suspend organization
        ///
        /// The dispatch origin for this call must match `T::ForceOrigin`.
        #[pallet::weight(100_000)]
        pub fn suspend_org(
            origin: OriginFor<T>,
            org_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            ensure!(
                Organizations::<T>::contains_key(&org_id),
                Error::<T>::NotExists
            );

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
        ///
        #[pallet::weight(100_000)]
        pub fn set_flags(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            flags: FlagDataBits,
        ) -> DispatchResultWithPostInfo {
            let origin_1 = ensure_signed(origin.clone())?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            if !did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin_1).is_ok()
                || flags.contains(FlagDataBit::System)
                || flags.contains(FlagDataBit::Verified)
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
        ///
        #[pallet::weight(100_000)]
        pub fn add_member(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            ensure!(did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin).is_ok(), 
                Error::<T>::PermissionDenied);

            ensure!(!org.suspended, Error::<T>::Suspended);

            let mut members = <Members<T>>::get(&org_id).unwrap_or_else(|| vec![]);

            ensure!(
                members.len() < T::MaxMemberCount::get(),
                Error::<T>::MaxMemberReached
            );
            ensure!(
                !members.iter().any(|a| *a == account_id),
                Error::<T>::BadIndex
            );

            members.push(account_id.clone());
            members.sort();

            <Members<T>>::insert(&org_id, members);

            Self::deposit_event(Event::MemberAdded(org_id, account_id));

            Ok(().into())
        }

        /// Remove member from organization.
        #[pallet::weight(100_000)]
        pub fn remove_member(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            // ensure!(org.admin == origin, Error::<T>::PermissionDenied);
            did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin)?;

            ensure!(!org.suspended, Error::<T>::Suspended);

            let mut members = <Members<T>>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            ensure!(
                members.iter().any(|a| *a == account_id),
                Error::<T>::NotExists
            );

            members = members.into_iter().filter(|a| *a != account_id).collect();
            Members::<T>::insert(org_id.clone(), members);

            Self::deposit_event(Event::MemberRemoved(org_id, account_id));

            Ok(().into())
        }

        /// Change organization admin,
        /// the origin must be current admin or conform to `ForceOrigin`.
        #[pallet::weight(100_000)]
        pub(crate) fn set_admin(
            origin: OriginFor<T>,
            org_id: T::AccountId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin_1 = ensure_signed(origin.clone())?;

            let org = Organizations::<T>::get(&org_id).ok_or(Error::<T>::NotExists)?;

            if !did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin_1).is_ok() {
                T::ForceOrigin::ensure_origin(origin)?;
            } else {
                ensure!(!org.suspended, Error::<T>::Suspended);
            }

            // ensure!(org.admin != account_id, Error::<T>::AlreadySet);

            // did::Module::<T>::valid_delegate(&org_id, b"OrgAdmin", &origin)?;

            did::Module::<T>::revoke_delegate_internal(&org_id, b"OrgAdmin", &account_id);
            did::Module::<T>::create_delegate(&org_id, &org_id, &account_id, b"OrgAdmin", None)?;

            <Organizations<T>>::mutate(&org_id, |org| {
                if let Some(org) = org {
                    org.admin = account_id.clone();
                }
            });

            Self::deposit_event(Event::AdminChanged(org_id, account_id));

            Ok(().into())
        }
    }

    // -------------------------------------------------------------------
    //                      GENESIS CONFIGURATION
    // -------------------------------------------------------------------

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        // pub dummy: u32,
        // pub bar: Vec<(T::AccountId, u32)>,
        // pub foo: u32,
        _phantom: PhantomData<T>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                // dummy: Default::default(),
                // bar: Default::default(),
                // foo: Default::default(),
                _phantom: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // <Dummy<T>>::put(&self.dummy);
            // for (a, b) in &self.bar {
            // 	<Bar<T>>::insert(a, b);
            // }
            // <Foo<T>>::put(&self.foo);
        }
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
    // /// Get the Organization detail
    // pub fn get(id: OrgId) -> Option<Organization> {
    //     Organizations::<T>::get(id)
    // }

    /// Get next Organization ID
    pub fn next_index() -> Result<u64, Error<T>> {
        <OrgIdIndex<T>>::mutate(|o| {
            *o = Some(o.map_or(1, |vo| vo.saturating_add(1)));
            *o
        })
        .ok_or(Error::<T>::CannotGenId)
    }

    /// Check whether account is member of the organization
    pub fn is_member(id: T::AccountId, account_id: T::AccountId) -> bool {
        <Members<T>>::get(id)
            .map(|a| a.iter().any(|id| *id == account_id))
            .unwrap_or(false)
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

// pub trait OrgProvider<T: Config> {
//     fn get(id: OrgId) -> Option<Organization>;
// }

// impl<T: Config> OrgProvider<T> for Pallet<T> {
//     fn get(id: OrgId) -> Option<Organization> {
//         Organizations::<T>::get(id)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_organization;

    use frame_support::{
        assert_err_ignore_postinfo, assert_noop, assert_ok, ord_parameter_types, parameter_types,
    };
    use frame_system::EnsureSignedBy;
    use sp_core::{sr25519, H256};
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        DispatchError,
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
            Did: pallet_did::{Module, Call, Storage, Event<T>},
            Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
            Organization: pallet_organization::{Module, Call, Storage, Event<T>},
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

    // impl Wraps for [u8; 32] {
    //     type Inner = [u8; 32];
    // }

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

    impl pallet_timestamp::Config for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ();
        type WeightInfo = ();
    }

    impl pallet_did::Config for Test {
        type Event = Event;
        type Public = sr25519::Public;
        type Signature = sr25519::Signature;
        type Time = Timestamp;
        type WeightInfo = pallet_did::weights::SubstrateWeight<Self>;
    }

    use sp_core::crypto::AccountId32;

    parameter_types! {
        pub const MinOrgNameLength: usize = 3;
        pub const MaxOrgNameLength: usize = 16;
        pub const MaxMemberCount: usize = 300;
        pub const CreationFee: u64 = 20;
    }

    lazy_static::lazy_static! {
        pub static ref ALICE: sr25519::Public = sr25519::Public::from_raw([1u8; 32]);
        pub static ref BOB: sr25519::Public = sr25519::Public::from_raw([2u8; 32]);
        pub static ref CHARLIE: sr25519::Public = sr25519::Public::from_raw([3u8; 32]);
        pub static ref DAVE: sr25519::Public = sr25519::Public::from_raw([4u8; 32]);
        pub static ref EVE: sr25519::Public = sr25519::Public::from_raw([5u8; 32]);

    }

    ord_parameter_types! {
        pub const One: sr25519::Public = *ALICE;
        pub const Two: sr25519::Public = *BOB;
    }
    impl Config for Test {
        type Event = Event;
        type CreationFee = CreationFee;
        type Currency = Balances;
        type Payment = ();
        type ForceOrigin = EnsureSignedBy<One, sr25519::Public>;
        type MinOrgNameLength = MinOrgNameLength;
        type MaxOrgNameLength = MaxOrgNameLength;
        type MaxMemberCount = MaxMemberCount;
        type WeightInfo = weights::SubstrateWeight<Test>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(*ALICE, 50), (*BOB, 10)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    #[test]
    fn can_create_organization() {
        new_test_ext().execute_with(|| {
            assert_ok!(Organization::create_org(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec()
            ));
        });
    }

    #[test]
    fn create_org_balance_deducted() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::total_balance(&*ALICE), 50);
            assert_ok!(Organization::create_org(
                Origin::signed(*ALICE),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Balances::total_balance(&*ALICE), 30);
        });
    }

    #[test]
    fn insufficient_balance_cannot_create_org() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::total_balance(&*BOB), 10);
            assert_err_ignore_postinfo!(
                Organization::create_org(
                    Origin::signed(*BOB),
                    b"ORG2".to_vec(),
                    b"ORG2 DESCRIPTION".to_vec(),
                    *BOB,
                    b"".to_vec(),
                    b"".to_vec()
                ),
                pallet_balances::Error::<Test, _>::InsufficientBalance
            );
            assert_eq!(Balances::total_balance(&*BOB), 10);
        });
    }

    #[test]
    fn org_id_incremented_correctly() {
        new_test_ext().execute_with(|| {
            assert_eq!(Pallet::<Test>::next_index().unwrap(), 1);
            assert_ok!(Organization::create_org(
                Origin::signed(*ALICE),
                b"ORG2".to_vec(),
                b"ORG2 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Pallet::<Test>::next_index().unwrap(), 3);
            assert_ok!(Organization::create_org(
                Origin::signed(*ALICE),
                b"ORG4".to_vec(),
                b"ORG4 DESCRIPTION".to_vec(),
                *BOB,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Pallet::<Test>::next_index().unwrap(), 5);
            assert_eq!(Pallet::<Test>::organization(*EVE), None);
            assert!(Pallet::<Test>::organization(*BOB)
                .map(|a| &a.name == b"ORG2")
                .unwrap_or(false));
            assert!(Pallet::<Test>::organization(*DAVE)
                .map(|a| &a.name == b"ORG4")
                .unwrap_or(false));
        });
    }

    type AccountId = <Test as frame_system::Config>::AccountId;

    fn with_org<F>(func: F)
    where
        F: FnOnce(AccountId, u64) -> (),
    {
        assert_ok!(Organization::create_org(
            Origin::signed(*ALICE),
            b"ORG1".to_vec(),
            b"ORG1 DESCRIPTION".to_vec(),
            *BOB,
            b"".to_vec(),
            b"".to_vec(),
        ));
        let index = <OrgIdIndex<Test>>::get().unwrap();
        func(Organization::organization_index(index).unwrap(), index);
    }

    #[test]
    fn new_created_org_active() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_eq!(Organization::is_active(org_id), true);
                assert_eq!(Organization::is_verified(org_id), false);
                assert_eq!(Organization::is_gov(org_id), false);
                assert_eq!(Organization::is_system(org_id), false);
            });
        });
    }

    #[test]
    fn set_flags_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_eq!(Organization::is_verified(org_id), false);
                assert_ok!(Organization::set_flags(
                    Origin::signed(*BOB),
                    org_id,
                    FlagDataBits(FlagDataBit::Foundation.into())
                ));
                assert_eq!(Organization::is_foundation(org_id), true);
                assert_eq!(Organization::is_gov(org_id), false);
                assert_ok!(Organization::set_flags(
                    Origin::signed(*BOB),
                    org_id,
                    FlagDataBits(FlagDataBit::Government.into())
                ));
                assert_eq!(Organization::is_gov(org_id), true);
            });
        });
    }

    #[test]
    fn set_flags_system_only_for_force_origin() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                // System
                assert_noop!(
                    Organization::set_flags(
                        Origin::signed(*BOB),
                        org_id,
                        FlagDataBits(FlagDataBit::System.into())
                    ),
                    DispatchError::BadOrigin
                );
                assert_eq!(Organization::is_system(org_id), false);
                assert_ok!(Organization::set_flags(
                    Origin::signed(*ALICE),
                    org_id,
                    FlagDataBits(FlagDataBit::System.into())
                ));
                assert_eq!(Organization::is_system(org_id), true);

                // Verified
                assert_noop!(
                    Organization::set_flags(
                        Origin::signed(*BOB),
                        org_id,
                        FlagDataBits(FlagDataBit::Verified.into())
                    ),
                    DispatchError::BadOrigin
                );
                assert_eq!(Organization::is_verified(org_id), false);
                assert_ok!(Organization::set_flags(
                    Origin::signed(*ALICE),
                    org_id,
                    FlagDataBits(FlagDataBit::Verified.into())
                ));
                assert_eq!(Organization::is_verified(org_id), true);
            });
        });
    }

    #[test]
    fn add_member_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_ok!(Organization::add_member(
                    Origin::signed(*BOB),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::is_member(org_id, *CHARLIE), true);
            });
        });
    }

    #[test]
    fn add_member_not_allowed_by_non_org_admin() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_err_ignore_postinfo!(
                    Organization::add_member(Origin::signed(*CHARLIE), org_id, *BOB),
                    Error::<Test>::PermissionDenied
                );
                assert_eq!(Organization::is_member(org_id, *CHARLIE), false);
            });
        });
    }

    #[test]
    fn remove_member_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_ok!(Organization::add_member(
                    Origin::signed(*BOB),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::is_member(org_id, *CHARLIE), true);
                assert_ok!(Organization::remove_member(
                    Origin::signed(*BOB),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::is_member(org_id, *CHARLIE), false);
            });
        });
    }

    #[test]
    fn remove_member_non_admin_not_allowed() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_ok!(Organization::add_member(
                    Origin::signed(*BOB),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::is_member(org_id, *CHARLIE), true);
                assert_err_ignore_postinfo!(
                    Organization::remove_member(Origin::signed(*EVE), org_id, *CHARLIE),
                    Error::<Test>::PermissionDenied
                );
                assert_eq!(Organization::is_member(org_id, *CHARLIE), true);
            });
        });
    }

    #[test]
    fn suspend_org_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_eq!(Organization::is_suspended(org_id), false);
                assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
                assert_eq!(Organization::is_suspended(org_id), true);
            });
        });
    }

    #[test]
    fn only_force_origin_can_suspend() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_noop!(
                    Organization::suspend_org(Origin::signed(*BOB), org_id),
                    DispatchError::BadOrigin
                );
                assert_eq!(Organization::is_suspended(org_id), false);
            });
        });
    }

    #[test]
    fn set_admin_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_eq!(Organization::get_admin(org_id), Some(*BOB));
                assert_ok!(Organization::set_admin(
                    Origin::signed(*BOB),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::get_admin(org_id), Some(*CHARLIE));
            });
        });
    }

    #[test]
    fn only_admin_or_force_origin_can_set_admin() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_eq!(Organization::get_admin(org_id), Some(*BOB));
                assert_ok!(Organization::set_admin(
                    Origin::signed(*ALICE),
                    org_id,
                    *CHARLIE
                ));
                assert_eq!(Organization::get_admin(org_id), Some(*CHARLIE));
                assert_ok!(Organization::set_admin(
                    Origin::signed(*CHARLIE),
                    org_id,
                    *DAVE
                ));
                assert_eq!(Organization::get_admin(org_id), Some(*DAVE));
                assert_noop!(
                    Organization::set_admin(Origin::signed(*CHARLIE), org_id, *BOB),
                    DispatchError::BadOrigin
                );
                assert_eq!(Organization::get_admin(org_id), Some(*DAVE));
            });
        });
    }

    #[test]
    fn cannot_dispatch_suspended_operation_when_suspended() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
                assert_err_ignore_postinfo!(
                    Organization::add_member(Origin::signed(*BOB), org_id, *CHARLIE),
                    Error::<Test>::Suspended
                );
                assert_err_ignore_postinfo!(
                    Organization::remove_member(Origin::signed(*BOB), org_id, *CHARLIE),
                    Error::<Test>::Suspended
                );
                assert_err_ignore_postinfo!(
                    Organization::set_flags(
                        Origin::signed(*BOB),
                        org_id,
                        FlagDataBits(FlagDataBit::Company.into())
                    ),
                    Error::<Test>::Suspended
                );
            });
        });
    }

    #[test]
    fn force_origin_can_set_flags_even_when_suspended() {
        new_test_ext().execute_with(|| {
            with_org(|org_id, index| {
                assert_ok!(Organization::suspend_org(Origin::signed(*ALICE), org_id));
                assert_ok!(Organization::set_flags(
                    Origin::signed(*ALICE),
                    org_id,
                    FlagDataBits(FlagDataBit::Government.into())
                ));
            });
        });
    }
}
