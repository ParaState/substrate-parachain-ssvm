use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H160;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum EscrowAccountSetter {
	/// Dao Escrow Account
	Dao(H160),
	/// Eco Escrow Account
	Eco(H160),
	/// Dapp Escrow Account
	Dapp(H160),
	/// Inflation Escrow Account
	Eoa(H160),
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct Range {
	/// Reward start block
	pub start: u32,
	/// Reward end block
	pub end: u32,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct PayOnce {
	/// Percentage of initial total supply
	pub percentage: u32,
	/// At which block
	pub at: u32,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct AutionRewardConfig {
	/// Percentage of initial total supply
	pub percentage: u32,
	/// Reward active range
	pub range: Range,
	/// Pay for special block
	pub pay_once: PayOnce,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct FixedPayoutConfig {
	/// Reward per block
	pub pay_per_block: u128,
	/// Reward active range
	pub range: Range,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct NonFixedPayoutConfig {
	/// Use same struct denote two cases:
	/// 1. percentage of initial total supply or
	/// 2. Inflation rate per year
	pub percentage: u32,
	/// Reward active range
	pub range: Range,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum ModelConfig {
	/// Aution Reward configuration
	AutionRewardConfig(AutionRewardConfig),
	/// Collator Reward configuration
	CollatorRewardConfig(FixedPayoutConfig),
	/// DAO Trust configuration
	DaoTrustConfig(NonFixedPayoutConfig),
	/// ECO Trust configuration
	EcoTrustConfig(NonFixedPayoutConfig),
	/// Dapp Trust configuration
	DappTrustConfig(FixedPayoutConfig),
	/// Inflation Rate configuration
	InflationRateConfig(NonFixedPayoutConfig),
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct Contributor {
	/// Eth address
	pub eth_address: H160,
	/// perthousand of auction reward
	pub perthousand: u32,
}
