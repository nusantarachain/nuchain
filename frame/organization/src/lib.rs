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
//! * `create_org` -
//! * `susppend_org` -
//! * `kill_org` -
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
use sp_runtime::traits::StaticLookup;
use sp_std::{fmt::Debug, prelude::*};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};

type OrgId = u32;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

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
    pub trait Config: frame_system::Config {
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
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The object already exsits
        AlreadyExists,

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
        IdAlreadyExists,

        /// Cannot generate ID
        CannotGenId,

        /// Max member count reached
        MaxMemberReached,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance", OrgId = "OrgId")]
    pub enum Event<T: Config> {
        /// Some object added inside the system.
        OrganizationAdded(OrgId, T::AccountId),

        /// When object deleted
        OrganizationDeleted(OrgId),

        /// Organization has been suspended.
        OrganizationSuspended(OrgId),

        /// Member added to an organization
        MemberAdded(OrgId, T::AccountId),
    }

    #[pallet::storage]
    pub type Organizations<T: Config> =
        StorageMap<_, Blake2_128Concat, OrgId, Organization<T::AccountId>>;

    /// Pair user -> list of handled organizations
    #[pallet::storage]
    pub type OrganizationLink<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<OrgId>>;

    /// Membership store, stored as an ordered Vec.
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> = StorageMap<_, Twox64Concat, OrgId, Vec<T::AccountId>>;

    pub struct EnsureOrgAdmin<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrgAdmin<T> {
        type Success = (T::AccountId, Vec<OrgId>);

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
    pub type OrgIdIndex<T> = StorageValue<_, u32>;

    /// Organization module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T> {
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

            let id = Self::next_id()?;

            ensure!(
                !Organizations::<T>::contains_key(id),
                Error::<T>::IdAlreadyExists
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

            Organizations::<T>::insert(
                id as OrgId,
                Organization {
                    name: name.clone(),
                    description: description.clone(),
                    admin: admin.clone(),
                    website: website.clone(),
                    email: email.clone(),
                },
            );

            if OrganizationLink::<T>::contains_key(&admin) {
                OrganizationLink::<T>::mutate(&admin, |ref mut vs| {
                    vs.as_mut().map(|vsi| vsi.push(id))
                });
            } else {
                OrganizationLink::<T>::insert(&admin, sp_std::vec![id]);
            }

            Self::deposit_event(Event::OrganizationAdded(id, admin));

            Ok(().into())
        }

        /// Add member to the organization.
        ///
        #[pallet::weight(100_000)]
        pub fn add_member(
            origin: OriginFor<T>,
            org_id: OrgId,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let org = Organizations::<T>::get(org_id).ok_or(Error::<T>::NotExists)?;

            ensure!(org.admin == origin, Error::<T>::PermissionDenied);

            let mut members = <Members<T>>::get(org_id).unwrap_or_else(|| vec![]);

            ensure!(
                members.len() < T::MaxMemberCount::get(),
                Error::<T>::MaxMemberReached
            );
            ensure!(
                !members.iter().any(|a| *a == account_id),
                Error::<T>::IdAlreadyExists
            );

            members.push(account_id.clone());
            members.sort();

            <Members<T>>::insert(org_id, members);

            Self::deposit_event(Event::MemberAdded(org_id, account_id));

            Ok(().into())
        }
    }

    // -------------------------------------------------------------------
    //                      GENESIS CONFIGURATION
    // -------------------------------------------------------------------

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub dummy: u32,
        pub bar: Vec<(T::AccountId, u32)>,
        pub foo: u32,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                dummy: Default::default(),
                bar: Default::default(),
                foo: Default::default(),
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

/// The main implementation of this Organization pallet.
impl<T: Config> Pallet<T> {
    /// Get the Organization detail
    pub fn get(id: OrgId) -> Option<Organization<T::AccountId>> {
        Organizations::<T>::get(id)
    }

    /// Get next Organization ID
    pub fn next_id() -> Result<u32, Error<T>> {
        <OrgIdIndex<T>>::mutate(|o| {
            *o = Some(o.map_or(1, |vo| vo.saturating_add(1)));
            *o
        })
        .ok_or(Error::<T>::CannotGenId)
    }

    /// Check whether account is member of the organization
    pub fn is_member(id: OrgId, account_id: T::AccountId) -> bool {
        <Members<T>>::get(id)
            .map(|a| a.iter().any(|id| *id == account_id))
            .unwrap_or(false)
    }
}

pub trait OrgProvider<T: Config> {
    fn get(id: OrgId) -> Option<Organization<T::AccountId>>;
}

impl<T: Config> OrgProvider<T> for Pallet<T> {
    fn get(id: OrgId) -> Option<Organization<T::AccountId>> {
        Organizations::<T>::get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_organization;

    use frame_support::{
        assert_err_ignore_postinfo, assert_ok, ord_parameter_types, parameter_types,
    };
    use frame_system::EnsureSignedBy;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
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
        pub const MaxOrgNameLength: usize = 16;
        pub const MaxMemberCount: usize = 300;
        pub const CreationFee: u64 = 20;
    }
    ord_parameter_types! {
        pub const One: u64 = 1;
    }
    impl Config for Test {
        type Event = Event;
        type CreationFee = CreationFee;
        type Currency = Balances;
        type Payment = ();
        type ForceOrigin = EnsureSignedBy<One, u64>;
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
            balances: vec![(1, 50), (2, 10)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    #[test]
    fn can_create_organization() {
        new_test_ext().execute_with(|| {
            assert_ok!(Organization::create_org(
                Origin::signed(1),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                2,
                b"".to_vec(),
                b"".to_vec()
            ));
        });
    }

    #[test]
    fn create_org_balance_deducted() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::total_balance(&1), 50);
            assert_ok!(Organization::create_org(
                Origin::signed(1),
                b"ORG1".to_vec(),
                b"ORG1 DESCRIPTION".to_vec(),
                2,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Balances::total_balance(&1), 30);
        });
    }

    #[test]
    fn insufficient_balance_cannot_create_org() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::total_balance(&2), 10);
            assert_err_ignore_postinfo!(
                Organization::create_org(
                    Origin::signed(2),
                    b"ORG2".to_vec(),
                    b"ORG2 DESCRIPTION".to_vec(),
                    2,
                    b"".to_vec(),
                    b"".to_vec()
                ),
                pallet_balances::Error::<Test, _>::InsufficientBalance
            );
            assert_eq!(Balances::total_balance(&2), 10);
        });
    }

    #[test]
    fn org_id_incremented_correctly() {
        new_test_ext().execute_with(|| {
            assert_eq!(Pallet::<Test>::next_id().unwrap(), 1);
            assert_ok!(Organization::create_org(
                Origin::signed(1),
                b"ORG2".to_vec(),
                b"ORG2 DESCRIPTION".to_vec(),
                2,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Pallet::<Test>::next_id().unwrap(), 3);
            assert_ok!(Organization::create_org(
                Origin::signed(1),
                b"ORG4".to_vec(),
                b"ORG4 DESCRIPTION".to_vec(),
                2,
                b"".to_vec(),
                b"".to_vec()
            ));
            assert_eq!(Pallet::<Test>::next_id().unwrap(), 5);
            assert_eq!(Pallet::<Test>::get(5), None);
            assert!(Pallet::<Test>::get(2)
                .map(|a| &a.name == b"ORG2")
                .unwrap_or(false));
            assert!(Pallet::<Test>::get(4)
                .map(|a| &a.name == b"ORG4")
                .unwrap_or(false));
        });
    }

    fn with_org<F>(func: F)
    where
        F: FnOnce(OrgId) -> (),
    {
        assert_ok!(Organization::create_org(
            Origin::signed(1),
            b"ORG1".to_vec(),
            b"ORG1 DESCRIPTION".to_vec(),
            2,
            b"".to_vec(),
            b"".to_vec(),
        ));
        func(<OrgIdIndex<Test>>::get().unwrap());
    }

    #[test]
    fn add_member_works() {
        new_test_ext().execute_with(|| {
            with_org(|org_id| {
                assert_ok!(Organization::add_member(Origin::signed(2), org_id, 2));
                assert_eq!(Organization::is_member(org_id, 2), true);
            });
        });
    }

    #[test]
    fn add_member_not_allowed_by_non_org_admin() {
        new_test_ext().execute_with(|| {
            with_org(|org_id| {
                assert_err_ignore_postinfo!(
                    Organization::add_member(Origin::signed(3), org_id, 2),
                    Error::<Test>::PermissionDenied
                );
                assert_eq!(Organization::is_member(org_id, 3), false);
            });
        });
    }
}
