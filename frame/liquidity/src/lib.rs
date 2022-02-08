//! # liquidity
//!
//! - [`Liquidity::Config`](./trait.Config.html)
//!
//! ## Overview
//!
//! Multichain liquidity bridge.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `transfer_in` - Transfer in tokens from external network.
//! * `transfer_out` - Transfer out tokens to external network.
//! * `set_operator` - Set operator key.
//! * `lock` - Lock pallet to prevent any further transfers.
//! * `unlock` - Unlock pallet to allow transfers.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    ensure,
    traits::{Currency, EnsureOrigin, Get, ReservableCurrency},
};
use frame_system::ensure_signed;
use sp_runtime::traits::StaticLookup;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};

type ProofId = u64;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::traits::{ExistenceRequirement, Imbalance, WithdrawReasons};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The origin which authorized to manage liquidity.
        type OperatorOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
    pub struct ProofTx<T: Config> {
        /// ID of proof
        pub id: ProofId,

        /// Block number where this proof is stored
        pub block: T::BlockNumber,

        /// Network source/destination ID
        pub network: u32,

        /// Transfered amount
        pub amount: BalanceOf<T>,

        /// Owner of the token
        pub owner: T::AccountId,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The Proof already exsits
        AlreadyExists,

        /// Proof doesn't exist.
        NotExists,

        /// Network mismatch
        InvalidNetwork,

        /// Pallet locked
        Locked,

        /// Overflow
        Overflow,

        /// Unknown error occurred
        Unknown,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", ProofId = "ProofId")]
    pub enum Event<T: Config> {
        /// New transfer in \[id, amount, owner, network id\]
        TransferIn(ProofId, BalanceOf<T>, T::AccountId, u32),

        /// New transfer out \[id, amount, owner, network id\]
        TransferOut(ProofId, BalanceOf<T>, T::AccountId, u32),
    }

    /// Index of id -> data
    #[pallet::storage]
    pub type ProofTxIns<T: Config> = StorageMap<_, Blake2_128Concat, ProofId, ProofTx<T>>;

    /// Index of id -> data
    #[pallet::storage]
    pub type ProofTxOuts<T: Config> = StorageMap<_, Blake2_128Concat, ProofId, ProofTx<T>>;

    #[pallet::storage]
    pub type TxInProofLink<T: Config> = StorageMap<_, Blake2_128Concat, u64, ProofId>;

    #[pallet::storage]
    pub type TxOutProofLink<T: Config> = StorageMap<_, Blake2_128Concat, u64, ProofId>;

    #[pallet::storage]
    #[pallet::getter(fn proof_txin_index)]
    pub type ProofTxInIndex<T> = StorageValue<_, u64>;

    #[pallet::storage]
    #[pallet::getter(fn proof_txout_index)]
    pub type ProofTxOutIndex<T> = StorageValue<_, u64>;

    #[pallet::storage]
    pub type OperatorKey<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn is_locked)]
    pub type Locked<T: Config> = StorageValue<_, bool>;

    /// Liquidity module declaration.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transfer from external to internal network.
        ///
        /// The dispatch origin for this call must be _Operator_.
        ///
        #[pallet::weight(T::WeightInfo::transfer_in())]
        pub(crate) fn transfer_in(
            origin: OriginFor<T>,
            id: ProofId,
            amount: BalanceOf<T>,
            owner: <T::Lookup as StaticLookup>::Source,
            network: u32,
        ) -> DispatchResultWithPostInfo {
            let _origin = T::OperatorOrigin::ensure_origin(origin)?;

            Self::ensure_not_locked()?;

            ensure!(
                !ProofTxIns::<T>::contains_key(id),
                Error::<T>::AlreadyExists
            );

            let owner = T::Lookup::lookup(owner)?;
            let index = Self::next_txin_index()?;

            ProofTxIns::<T>::insert(
                id as ProofId,
                ProofTx {
                    id,
                    block: <frame_system::Module<T>>::block_number(),
                    network,
                    amount,
                    owner: owner.clone(),
                },
            );

            let mut imbalance = <PositiveImbalanceOf<T>>::zero();

            imbalance.subsume(T::Currency::deposit_creating(&owner, amount));

            TxInProofLink::<T>::insert(index, id);

            Self::deposit_event(Event::TransferIn(id, amount, owner, network));

            Ok(().into())
        }

        /// Transfer from internal to external network.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        #[pallet::weight(T::WeightInfo::transfer_out())]
        pub(crate) fn transfer_out(
            origin: OriginFor<T>,
            id: ProofId,
            amount: BalanceOf<T>,
            network: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::ensure_not_locked()?;

            ensure!(
                !ProofTxOuts::<T>::contains_key(id),
                Error::<T>::AlreadyExists
            );

            let index = Self::next_txout_index()?;

            ProofTxOuts::<T>::insert(
                id as ProofId,
                ProofTx {
                    id,
                    block: <frame_system::Module<T>>::block_number(),
                    network,
                    amount,
                    owner: who.clone(),
                },
            );

            let mut imbalance = <NegativeImbalanceOf<T>>::zero();

            imbalance.subsume(T::Currency::withdraw(
                &who,
                amount,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?);

            TxOutProofLink::<T>::insert(index, id);

            Self::deposit_event(Event::TransferOut(id, amount, who, network));

            Ok(().into())
        }

        /// Set operator key
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub(crate) fn set_operator(
            origin: OriginFor<T>,
            key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _root = ensure_root(origin)?;

            OperatorKey::<T>::put(key);

            Ok(().into())
        }

        /// Lock this pallet and make sure that no more transfers can be made.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub(crate) fn lock(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _root = ensure_root(origin)?;

            Locked::<T>::put(true);

            Ok(().into())
        }

        /// Unlock this pallet and make sure that transfers can be made.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub(crate) fn unlock(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _root = ensure_root(origin)?;

            Locked::<T>::put(false);

            Ok(().into())
        }
    }

    // ----------------------------------------------------------------
    //                      HOOKS
    // ----------------------------------------------------------------
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // fn offchain_worker(n: T::BlockNumber){
        //     // @TODO(Robin): Your off-chain logic here
        // }
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

pub struct EnsureOperator<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> EnsureOrigin<T::Origin> for EnsureOperator<T> {
    type Success = ();

    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        match o.clone().into()? {
            frame_system::RawOrigin::Signed(ref who) => {
                if Pallet::<T>::operator().as_ref() == Some(who) {
                    Ok(())
                } else {
                    Err(o)
                }
            }
            r => Err(T::Origin::from(r)),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::Origin {
        frame_system::RawOrigin::Root.into()
    }
}

/// The main implementation of this Liquidity pallet.
impl<T: Config> Pallet<T> {
    /// Get the tx in proof by proof id
    pub fn proof_tx_ins(id: ProofId) -> Option<ProofTx<T>> {
        ProofTxIns::<T>::get(id)
    }

    /// Get the tx out proof by proof id
    pub fn proof_tx_out(id: ProofId) -> Option<ProofTx<T>> {
        ProofTxOuts::<T>::get(id)
    }

    /// Get next txin index
    pub fn next_txin_index() -> Result<u64, Error<T>> {
        let index = <ProofTxInIndex<T>>::try_get()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;
        <ProofTxInIndex<T>>::put(index);
        Ok(index)
    }

    /// Get next txout index
    pub fn next_txout_index() -> Result<u64, Error<T>> {
        let index = <ProofTxOutIndex<T>>::try_get()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::<T>::Overflow)?;
        <ProofTxOutIndex<T>>::put(index);
        Ok(index)
    }

    /// Get current operator
    pub fn operator() -> Option<T::AccountId> {
        OperatorKey::<T>::get()
    }

    /// Get current locked status, if locked will return error
    pub fn ensure_not_locked() -> Result<(), Error<T>> {
        match Self::is_locked() {
            Some(false) => Ok(()),
            _ => Err(Error::<T>::Locked),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_liquidity;

    use frame_support::{
        assert_noop, assert_ok, dispatch::DispatchError, ord_parameter_types, parameter_types,
    };

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
            Liquidity: pallet_liquidity::{Module, Call, Storage, Event<T>},
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
        pub const MinProofNameLength: usize = 3;
        pub const MaxProofNameLength: usize = 16;
    }
    ord_parameter_types! {
        pub const One: u64 = 1;
    }
    impl Config for Test {
        type Event = Event;
        type Currency = Balances;
        // type OperatorOrigin = EnsureSignedBy<One, u64>;
        type OperatorOrigin = EnsureOperator<Test>;
        type WeightInfo = weights::SubstrateWeight<Test>;
    }

    const NETWORK_1: u32 = 1;
    // const NETWORK_2: u32 = 2;
    // const NETWORK_3: u32 = 3;

    // mock user
    const ONE: u64 = 1;
    const TWO: u64 = 2;

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

    type LEvent = pallet_liquidity::Event<Test>;

    fn last_event() -> LEvent {
        System::events()
            .into_iter()
            .map(|r| r.event)
            .filter_map(|e| {
                if let Event::pallet_liquidity(inner) = e {
                    Some(inner)
                } else {
                    None
                }
            })
            .last()
            .expect("Event expected")
    }

    fn ensure_no_event() {
        assert!(System::events().into_iter().map(|r| r.event).all(|e| {
            if let Event::pallet_liquidity(_) = e {
                false
            } else {
                true
            }
        }));
    }

    fn ready<F>(func: F)
    where
        F: FnOnce(<Test as frame_system::Config>::AccountId) -> (),
    {
        new_test_ext().execute_with(|| {
            System::set_block_number(1);

            Locked::<Test>::put(false);

            // set operator
            OperatorKey::<Test>::put(ONE);

            func(ONE);
        })
    }

    #[test]
    fn operator_origin_able_to_create_proof_tx_in() {
        ready(|operator| {
            let issuance = Balances::total_issuance();

            assert_eq!(Balances::total_balance(&1), 10);
            assert_eq!(Balances::total_balance(&TWO), 10);
            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x123,
                2003,
                TWO,
                NETWORK_1
            ));
            assert_eq!(Balances::total_balance(&TWO), 10 + 2003);
            assert_eq!(Balances::total_balance(&operator), 10); // dispatcher balance unchanged

            // check proofs
            assert!(ProofTxIns::<Test>::get(0x123).is_some());
            assert!(ProofTxOuts::<Test>::get(0x123).is_none());

            // total issuance should be updated
            assert_eq!(Balances::total_issuance(), issuance + 2003);

            // ensure event emited
            let event = last_event();
            assert_eq!(event, LEvent::TransferIn(0x123, 2003, TWO, NETWORK_1));
        });
    }

    // test cannot transfer in if proof id already exists
    #[test]
    fn cannot_create_proof_tx_in_if_proof_id_already_exists() {
        ready(|operator| {
            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x123,
                2003,
                TWO,
                NETWORK_1
            ));

            // ensure cannot transfer in again
            assert_noop!(
                Liquidity::transfer_in(Origin::signed(operator), 0x123, 2003, TWO, NETWORK_1),
                Error::<Test>::AlreadyExists
            );
        });
    }

    #[test]
    fn non_force_origin_unable_to_create_proof_tx_in() {
        new_test_ext().execute_with(|| {
            Locked::<Test>::put(false);
            assert_noop!(
                Liquidity::transfer_in(Origin::signed(ONE), 0x123, 2003, 2, NETWORK_1),
                DispatchError::BadOrigin
            );
            assert_eq!(Balances::total_balance(&1), 10);
            ensure_no_event();
        });
    }

    // test owner can transfer out
    #[test]
    fn can_transfer_out() {
        ready(|_operator| {
            System::set_block_number(1);

            let issuance = Balances::total_issuance();

            assert_eq!(Balances::total_balance(&1), 10);
            assert_eq!(Balances::total_balance(&TWO), 10);
            assert_ok!(Liquidity::transfer_out(
                Origin::signed(TWO),
                0x123,
                3,
                NETWORK_1
            ));
            assert_eq!(Balances::total_balance(&TWO), 10 - 3);

            // check proofs
            assert!(ProofTxIns::<Test>::get(0x123).is_none());
            assert!(ProofTxOuts::<Test>::get(0x123).is_some());

            // total issuance should be updated
            assert_eq!(Balances::total_issuance(), issuance - 3);

            // ensure event emited
            let event = last_event();
            assert_eq!(event, LEvent::TransferOut(0x123, 3, TWO, NETWORK_1));
        });
    }

    // test cannot transfer out if proof id already exists
    #[test]
    fn cannot_transfer_out_if_proof_id_exists() {
        ready(|_operator| {
            System::set_block_number(1);

            assert_eq!(Balances::total_balance(&1), 10);
            assert_eq!(Balances::total_balance(&TWO), 10);
            assert_ok!(Liquidity::transfer_out(
                Origin::signed(TWO),
                0x123,
                3,
                NETWORK_1
            ));
            assert_eq!(System::events().len(), 1);
            assert_noop!(
                Liquidity::transfer_out(Origin::signed(TWO), 0x123, 3, NETWORK_1),
                Error::<Test>::AlreadyExists
            );
            assert_eq!(Balances::total_balance(&TWO), 10 - 3);
            assert_eq!(System::events().len(), 1);
        });
    }

    // test transfer in increase index
    #[test]
    fn transfer_in_increase_index() {
        ready(|operator| {
            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x123,
                2003,
                TWO,
                NETWORK_1
            ));

            // ensure index increased
            assert_eq!(Liquidity::proof_txin_index(), Some(1));
            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x124,
                2003,
                TWO,
                NETWORK_1
            ));

            // ensure index increased
            assert_eq!(Liquidity::proof_txin_index(), Some(2));
        });
    }

    // test transfer out increase index
    #[test]
    fn transfer_out_increase_index() {
        ready(|_operator| {
            assert_ok!(Liquidity::transfer_out(
                Origin::signed(TWO),
                0x123,
                1,
                NETWORK_1
            ));

            // ensure index increased
            assert_eq!(Liquidity::proof_txout_index(), Some(1));
            assert_eq!(Liquidity::proof_txin_index(), None); // not changed

            assert_ok!(Liquidity::transfer_out(
                Origin::signed(TWO),
                0x124,
                1,
                NETWORK_1
            ));

            // ensure index increased
            assert_eq!(Liquidity::proof_txout_index(), Some(2));
        });
    }

    // test only root can lock
    #[test]
    fn only_root_can_lock() {
        ready(|_operator| {
            assert_eq!(Liquidity::is_locked().unwrap(), false);

            assert_ok!(Liquidity::lock(Origin::root()));

            // ensure locked
            assert_eq!(Liquidity::is_locked().unwrap(), true);

            assert_noop!(
                Liquidity::lock(Origin::signed(TWO)),
                DispatchError::BadOrigin
            );
        });
    }

    // test only root can unlock
    #[test]
    fn only_root_can_unlock() {
        ready(|_operator| {
            assert_eq!(Liquidity::is_locked().unwrap(), false);

            assert_ok!(Liquidity::lock(Origin::root()));

            // ensure locked
            assert_eq!(Liquidity::is_locked().unwrap(), true);

            assert_ok!(Liquidity::unlock(Origin::root()));

            // ensure unlocked
            assert_eq!(Liquidity::is_locked().unwrap(), false);

            assert_noop!(
                Liquidity::unlock(Origin::signed(TWO)),
                DispatchError::BadOrigin
            );
        });
    }

    // test only root can set operator
    #[test]
    fn only_root_can_set_operator() {
        ready(|operator| {
            assert_eq!(Liquidity::operator(), Some(operator));

            assert_ok!(Liquidity::set_operator(Origin::root(), TWO));

            // ensure operator set
            assert_eq!(Liquidity::operator(), Some(TWO));

            assert_noop!(
                Liquidity::set_operator(Origin::signed(operator), TWO),
                DispatchError::BadOrigin
            );
        });
    }

    // test check proof index
    #[test]
    fn check_proof_index() {
        ready(|operator| {
            assert_eq!(Liquidity::proof_txin_index(), None);

            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x123,
                2003,
                TWO,
                NETWORK_1
            ));

            assert_eq!(Liquidity::proof_txin_index(), Some(1));
            assert_eq!(TxInProofLink::<Test>::get(1), Some(0x123));

            assert_ok!(Liquidity::transfer_in(
                Origin::signed(operator),
                0x124,
                22,
                TWO,
                NETWORK_1
            ));

            assert_eq!(Liquidity::proof_txin_index(), Some(2));
            assert_eq!(TxInProofLink::<Test>::get(2), Some(0x124));
        });
    }
}
