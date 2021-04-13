//! # geo
//!
//! - [`Geo::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Geographic location database pallet for Substrate
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `register_location` -
//! * `update_location` -
//! * `delete_location` -
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

type LocationId = u32;

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

        /// Min location name length
        type MinLocationNameLength: Get<usize>;

        /// Max location name length
        type MaxLocationNameLength: Get<usize>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct Location<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Location name
        name: Vec<u8>,

        /// registrar of the Location
        registrar: AccountId,

        /// Population of registered people reside in the location.
        population: u32,

        /// This location is belong to another location.
        parent_location_id: LocationId,

        /// Location kind
        /// 1 = Country
        /// 2 = Province
        /// 3 = District
        /// 4 = Sub District
        /// 5 = Village
        /// 6 = Sub Village
        kind: u16,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct ProposedLocationUpdate<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
        /// Location name
        name: Vec<u8>,

        /// proposer of the Location
        proposer: AccountId,

        /// Population of registered people reside in the location.
        population: u32,

        /// This location is belong to another location.
        parent_location_id: LocationId,

        /// Location kind
        /// 1 = Country
        /// 2 = Province
        /// 3 = District
        /// 4 = Sub District
        /// 5 = Village
        /// 6 = Sub Village
        kind: u16,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The Location already exsits
        AlreadyExists,

        /// Name too long
        TooLong,

        /// Name too short
        TooShort,

        /// Location doesn't exist.
        NotExists,

        /// Origin has no authorization to do this operation
        PermissionDenied,

        /// ID already exists
        IdAlreadyExists,

        /// Max registrar reached to its limit.
        MaxRegistrarsReached,

        /// Max proposal reached limits.
        MaxProposedUpdates,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Balance = "Balance",
        LocationId = "LocationId"
    )]
    pub enum Event<T: Config> {
        /// Some Location added inside the system.
        LocationAdded(LocationId, T::AccountId),

        /// Some location data updated
        LocationUpdated(LocationId, T::AccountId),

        /// Location deleted
        LocationDeleted(LocationId),

        /// Someone propose location data update.
        ProposeLocationUpdate(LocationId, T::AccountId),
    }

    /// Index of id -> data
    #[pallet::storage]
    pub type Locations<T: Config> = StorageMap<_, Twox64Concat, LocationId, Location<T::AccountId>>;

    /// Pending/proposed location update data from user.
    #[pallet::storage]
    #[pallet::getter(fn proposed_updates)]
    pub type ProposedUpdates<T: Config> =
        StorageValue<_, Vec<ProposedLocationUpdate<T::AccountId>>, ValueQuery>;

    // #[pallet::storage]
    // pub type LocationLink<T: Config> = StorageMap<
    //     _,
    //     Blake2_128Concat,
    //     LocationId,
    //     u32, // change me
    // >;

    /// Registrar index
    #[pallet::storage]
    pub type Registrars<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    pub type LocationIdIndex<T> = StorageValue<_, u32>;

    macro_rules! validate_name {
        ($name:ident) => {
            ensure!(
                $name.len() >= T::MinLocationNameLength::get(),
                Error::<T>::TooShort
            );
            ensure!(
                $name.len() <= T::MaxLocationNameLength::get(),
                Error::<T>::TooLong
            );
        };
    }

    /// Geo module declaration.
    // pub struct Module<T: Config> for enum Call where origin: T::Origin {
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add new object.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// # <weight>
        /// # </weight>
        #[pallet::weight(T::WeightInfo::register_location())]
        fn register_location(
            origin: OriginFor<T>,
            name: Vec<u8>,
            #[pallet::compact] population: u32,
            #[pallet::compact] parent_location_id: LocationId,
            #[pallet::compact] kind: u16,
            // registrar: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            // T::ForceOrigin::ensure_origin(origin)?;

            let registrar = Self::ensure_registrar(origin)?;

            validate_name!(name);

            let id = Self::next_id();

            ensure!(
                !Locations::<T>::contains_key(id),
                Error::<T>::IdAlreadyExists
            );

            // let registrar = T::Lookup::lookup(registrar)?;

            Locations::<T>::insert(
                id as LocationId,
                Location {
                    name: name.clone(),
                    registrar: registrar.clone(),
                    population: population,
                    parent_location_id,
                    kind,
                },
            );

            Self::deposit_event(Event::LocationAdded(id, registrar));

            Ok(().into())
        }

        /// Add Registrar for Geo module.
        ///
        /// Registrar is responsible for mutating location records.
        ///
        /// The dispatch of this origin call must match `T::ForceOrigin`.
        ///
        #[pallet::weight(100_000)]
        fn add_registrar(origin: OriginFor<T>, id: T::AccountId) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            Registrars::<T>::try_mutate(|d| {
                if (d.len() >= 100) {
                    return Err(Error::<T>::MaxRegistrarsReached);
                }
                if !d.contains(&id) {
                    d.push(id);
                    Ok(())
                } else {
                    Err(Error::<T>::NotExists)
                }
            })?;

            Ok(().into())
        }

        /// Remove registrar from Geo module.
        ///
        /// The dispatch origin for this call must match `T::ForceOrigin`.
        #[pallet::weight(100_000)]
        fn remove_registrar(origin: OriginFor<T>, id: T::AccountId) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            Registrars::<T>::try_mutate(|d| {
                if !d.contains(&id) {
                    d.retain(|a| *a != id);
                    Ok(())
                } else {
                    Err(Error::<T>::NotExists)
                }
            })?;

            Ok(().into())
        }

        /// Update location data.
        ///
        /// The data must be exists.
        ///
        /// The dispatch origin for this call must be _signed_ registrar.
        ///
        #[pallet::weight(100_000)]
        fn update_location(
            origin: OriginFor<T>,
            name: Vec<u8>,
            id: LocationId,
            #[pallet::compact] population: u32,
            #[pallet::compact] parent_location_id: LocationId,
            #[pallet::compact] kind: u16,
        ) -> DispatchResultWithPostInfo {
            let registrar = Self::ensure_registrar(origin)?;

            validate_name!(name);

            Locations::<T>::mutate(id, |d| {
                if let Some(d) = d {
                    d.name = name.clone();
                    d.population = population;
                    d.parent_location_id = parent_location_id;
                    d.kind = kind;
                }
            });

            Self::deposit_event(Event::LocationUpdated(id, registrar));

            Ok(().into())
        }

        /// Propose update location to the registrar.
        ///
        #[pallet::weight(100_000)]
        fn propose_update_location(
            origin: OriginFor<T>,
            id: LocationId,
            name: Vec<u8>,
            #[pallet::compact] population: u32,
            #[pallet::compact] parent_location_id: LocationId,
            #[pallet::compact] kind: u16,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            validate_name!(name);

            let mut props = ProposedUpdates::<T>::get();

            /// Limit max proposed updates to 100 records
            ensure!(props.len() < 100, Error::<T>::MaxProposedUpdates);

            props.push(ProposedLocationUpdate {
                name: name.clone(),
                proposer: origin.clone(),
                population,
                parent_location_id,
                kind,
            });

            Self::deposit_event(Event::ProposeLocationUpdate(id, origin));

            Ok(().into())
        }

        /// Apply proposal update.
        ///
        /// This use index
        #[pallet::weight(100_000)]
        fn apply_proposal_update(origin: OriginFor<T>, index: u32) -> DispatchResultWithPostInfo {
            // @TODO: code here
            Ok(().into())
        }

        /// Delete some location data.
        ///
        /// The dispatch origin for this call must match `T::ForceOrigin`.
        ///
        #[pallet::weight(100_000)]
        fn delete_location(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // @TODO: code here
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

use crate::DispatchError::BadOrigin;
use frame_system::RawOrigin;

/// The main implementation of this Geo pallet.
impl<T: Config> Pallet<T> {
    /// Get the geo detail
    pub fn geo(id: LocationId) -> Option<Location<T::AccountId>> {
        Locations::<T>::get(id)
    }

    /// Get next geo ID
    pub fn next_id() -> u32 {
        let next_id = <LocationIdIndex<T>>::try_get()
            .unwrap_or(0)
            .saturating_add(1);
        <LocationIdIndex<T>>::put(next_id);
        next_id
    }

    /// Ensure origin is registrar
    fn ensure_registrar<OO>(origin: OO) -> Result<T::AccountId, DispatchError>
    where
        OO: Into<Result<RawOrigin<T::AccountId>, OO>>,
    {
        match origin.into() {
            Ok(RawOrigin::Signed(t)) => {
                if Registrars::<T>::get().contains(&t) {
                    Ok(t)
                } else {
                    Err(BadOrigin)
                }
            }
            _ => Err(BadOrigin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_Geo;

    use frame_support::{assert_noop, assert_ok, ord_parameter_types, parameter_types};
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
            Geo: pallet_Geo::{Module, Call, Storage, Event<T>},
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
        pub const MinLocationNameLength: usize = 3;
        pub const MaxLocationNameLength: usize = 16;
    }
    ord_parameter_types! {
        pub const One: u64 = 1;
    }
    impl Config for Test {
        type Event = Event;
        type ForceOrigin = EnsureSignedBy<One, u64>;
        type MinLocationNameLength = MinLocationNameLength;
        type MaxLocationNameLength = MaxLocationNameLength;
        type WeightInfo = weights::SubstrateWeight<Test>;
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
    fn force_origin_able_to_create_location() {
        new_test_ext().execute_with(|| {
            assert_ok!(Geo::register_location(
                Origin::signed(1),
                b"ORG1".to_vec(),
                2
            ));
        });
    }

    #[test]
    fn non_force_origin_cannot_create_location() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Geo::register_location(Origin::signed(2), b"ORG1".to_vec(), 2),
                DispatchError::BadOrigin
            );
        });
    }
}
