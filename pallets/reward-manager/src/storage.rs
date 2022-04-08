use super::parameter::{AutionRewardConfig, FixedPayoutConfig, NonFixedPayoutConfig};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H160;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub enum EscrowType {
	/// Dao Escrow Account
	Dao,
	/// Eco Escrow Account
	Eco,
	/// Dapp Escrow Account
	Dapp,
	/// Inflation Escrow Account
	Eoa,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct AutionRewardModel {
	/// Pay for special block
	pub pay_once: u128,
	/// Pay per block
	pub pay_per_block: u128,
	/// Config
	pub config: AutionRewardConfig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct InflationRateModel {
	/// Config
	pub config: NonFixedPayoutConfig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct FixedPayoutModel {
	/// Config
	pub config: FixedPayoutConfig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct NonFixedPayoutModel {
	/// Pay per block
	pub pay_per_block: u128,
	/// Config
	pub config: NonFixedPayoutConfig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, scale_info::TypeInfo)]
pub struct Model {
	/// Aution Reward model
	pub aution_reward: AutionRewardModel,
	/// Collator Reward model
	pub collator_reward: FixedPayoutModel,
	/// DAO Trust model
	pub dao_trust: NonFixedPayoutModel,
	/// ECO Trust model
	pub eco_trust: NonFixedPayoutModel,
	/// Dapp Trust model
	pub dapp_trust: FixedPayoutModel,
	/// Inflation Rate model
	pub inflation_rate: InflationRateModel,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, scale_info::TypeInfo)]
pub enum Status {
	/// Draft version
	Draft,
	/// Snapshot version, only exist after apply candidate model
	Snapshot,
}

impl Default for Status {
	fn default() -> Self {
		Status::Draft
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, scale_info::TypeInfo)]
pub struct ModelWithStatus {
	/// Inflation Rate model
	pub model: Model,
	/// Status
	pub status: Status,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode, Default, TypeInfo)]
pub struct ContributorInfo {
	/// Eth address
	pub eth_address: H160,
	/// perthousand of auction reward
	pub perthousand: u32,
	/// Payout
	pub pay_per_block: u128,
}
