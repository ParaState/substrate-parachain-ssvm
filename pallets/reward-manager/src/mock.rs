use crate as pallet_reward_manager;
use frame_support::{
	parameter_types,
	traits::{Everything, FindAuthor},
};
use frame_system as system;
use pallet_collator_selection::{AccountError, EthAddressMapping};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::marker::PhantomData;
use std::str::FromStr;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
		RewardManager: pallet_reward_manager::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type Balance = u128;
/// GEM, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	// Provide a common factor between runtimes based on a supply of 10_000_000 tokens.
	pub const INITIAL_SUPPLY: Balance = 210_000_000 * GEM;

	// pub const WEI: Balance = 1;
	// pub const KILOWEI: Balance = 1_000;
	// pub const MEGAWEI: Balance = 1_000_000;
	// pub const GIGAWEI: Balance = 1_000_000_000;
	// pub const MICROGEM: Balance = 1_000_000_000_000;
	// pub const MILLIGEM: Balance = 1_000_000_000_000_000;
	pub const GEM: Balance = 1_000_000_000_000_000_000;
	// pub const KILOGEM: Balance = 1_000_000_000_000_000_000_000;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
	pub const MaxReserves: u32 = 100;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u128;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub foo: sp_runtime::testing::UintAuthorityId,
	}
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId32> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[];

	fn on_genesis_session<Ks: sp_runtime::traits::OpaqueKeys>(_validators: &[(AccountId32, Ks)]) {}

	fn on_new_session<Ks: sp_runtime::traits::OpaqueKeys>(
		_: bool,
		_: &[(AccountId32, Ks)],
		_: &[(AccountId32, Ks)],
	) {
	}

	fn on_disabled(_: u32) {}
}

parameter_types! {
	pub const Offset: u64 = 0;
	pub const Period: u64 = 10;
}

impl pallet_session::Config for Test {
	type SessionManager = ();
	type Keys = SessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionHandler = TestSessionHandler;
	type Event = Event;
	type ValidatorId = AccountId32;
	type ValidatorIdOf = ();
	type WeightInfo = ();
}

pub struct AuthorDummy;
impl FindAuthor<AccountId32> for AuthorDummy {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId32>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some([0u8; 32].into())
	}
}

impl pallet_authorship::Config for Test {
	type FindAuthor = AuthorDummy;
	type UncleGenerations = ();
	type FilterUncle = ();
	type EventHandler = RewardManager;
}

parameter_types! {
	pub const InitialSupply: Balance = currency::INITIAL_SUPPLY;
}

impl pallet_reward_manager::Config for Test {
	type Event = Event;
	type EthAddressMapping = CollatorEthAddressMapping<Self>;
	type AccountMapping = pallet_reward_manager::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type InitialSupply = InitialSupply;
}

pub const REGISTERED_COLLATOR_ACCOUNT: [u8; 32] = [0u8; 32];
pub static REGISTERED_COLLATOR_ETH_ADDRESS: &str = "6be02d1d3665660d22ff9624b7be0551ee1ac91b";

/// Hook fake rewrard address
pub struct CollatorEthAddressMapping<T>(PhantomData<T>);
impl<T: pallet_reward_manager::Config> EthAddressMapping<T::AccountId>
	for CollatorEthAddressMapping<T>
where
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
	fn eth_address(account: <T>::AccountId) -> Result<H160, AccountError> {
		if account == REGISTERED_COLLATOR_ACCOUNT.into() {
			return Ok(H160::from_str(REGISTERED_COLLATOR_ETH_ADDRESS).unwrap())
		}
		return Err(AccountError::NotExist)
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn initialize_to_block(n: u64) {
	for i in System::block_number() + 1..=n {
		System::set_block_number(i);
		<AllPalletsWithSystem as frame_support::traits::OnInitialize<u64>>::on_initialize(i);
	}
}
