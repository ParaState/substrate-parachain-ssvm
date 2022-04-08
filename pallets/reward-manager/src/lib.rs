#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{inherent::Vec, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_collator_selection::EthAddressMapping;
use sp_core::{Hasher, H160, H256};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32, Perbill, SaturatedConversion};

use parameter::*;
use storage::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod parameter;
mod storage;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Polkadot account to H160 address.
		type EthAddressMapping: EthAddressMapping<Self::AccountId>;
		/// Mapping from address to account id.
		type AccountMapping: AccountMapping<Self::AccountId>;
		/// Currency type for dispatch reward.
		type Currency: Currency<Self::AccountId>;
		/// Initial total supply
		#[pallet::constant]
		type InitialSupply: Get<u128>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Current ModelSetting
	#[pallet::storage]
	#[pallet::getter(fn current_setting)]
	pub type CurrentModelSetting<T> = StorageValue<_, Model, ValueQuery>;

	/// Candidate ModelSetting with two status (draft or snapshot) and  won't take any effect until apply it
	#[pallet::storage]
	#[pallet::getter(fn candidate_setting)]
	pub type CandidateModelSetting<T> = StorageValue<_, ModelWithStatus, ValueQuery>;

	/// Escrow Accounts
	#[pallet::storage]
	#[pallet::getter(fn escrow_account)]
	pub type EscrowAccounts<T: Config> =
		StorageMap<_, Blake2_128Concat, EscrowType, H160, ValueQuery>;

	/// Contributors will store each pair information about eth address, reward distrubution per thousand, pay per block
	#[pallet::storage]
	#[pallet::getter(fn contributors)]
	pub type Contributors<T: Config> = StorageValue<_, Vec<ContributorInfo>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Post update event
		UpdateCandidateModelSetting(ModelConfig),
		/// Post apply event
		ApplyCandidateModelSetting(ModelConfig),
		/// Post rollback event
		RollbackToSnapshotModelSetting(ModelConfig),
		/// Set contributors
		SetContributors(Vec<ContributorInfo>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Set over limit 100%
		DistributionOverLimit,
		/// Snapshot has been overwrite
		SnapshotContamination,
	}

	/// Set or replace the config in candidate (won't take any effect until apply it)
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn update_candidate(origin: OriginFor<T>, config: ModelConfig) -> DispatchResult {
			fn get_quota(percentage: u32, amount: u128) -> u128 {
				return Perbill::from_percent(percentage) * amount
			}
			fn calc_pay_per_block(range: &Range, amount: u128) -> u128 {
				if range.end > range.start {
					return amount / (range.end - range.start) as u128
				} else {
					return 0u128
				}
			}

			ensure_root(origin)?;
			<CandidateModelSetting<T>>::mutate(|setting| match config.clone() {
				ModelConfig::AutionRewardConfig(c) => {
					let total = get_quota(c.percentage, T::InitialSupply::get());
					let pay_once = get_quota(c.pay_once.percentage, total);
					setting.model.aution_reward = AutionRewardModel {
						pay_once,
						pay_per_block: calc_pay_per_block(&c.range, total - pay_once),
						config: c,
					}
				},
				ModelConfig::CollatorRewardConfig(c) =>
					setting.model.collator_reward = FixedPayoutModel { config: c },
				ModelConfig::DaoTrustConfig(c) =>
					setting.model.dao_trust = NonFixedPayoutModel {
						pay_per_block: calc_pay_per_block(
							&c.range,
							get_quota(c.percentage, T::InitialSupply::get()),
						),
						config: c,
					},
				ModelConfig::EcoTrustConfig(c) =>
					setting.model.eco_trust = NonFixedPayoutModel {
						pay_per_block: calc_pay_per_block(
							&c.range,
							get_quota(c.percentage, T::InitialSupply::get()),
						),
						config: c,
					},
				ModelConfig::DappTrustConfig(c) =>
					setting.model.dapp_trust = FixedPayoutModel { config: c },
				ModelConfig::InflationRateConfig(c) =>
					setting.model.inflation_rate = InflationRateModel { config: c },
			});
			<CandidateModelSetting<T>>::mutate(|setting| setting.status = Status::Draft);
			Self::deposit_event(Event::UpdateCandidateModelSetting(config));
			Ok(())
		}

		/// Apply candidate config to runtime (auto take snapshot of runtime and swap with each other)
		#[pallet::weight(0)]
		pub fn apply_model(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			<CandidateModelSetting<T>>::mutate(|candidate| {
				<CurrentModelSetting<T>>::mutate(|current| {
					let snapshot = current.clone();
					*current = candidate.model.clone();
					candidate.model = snapshot;
					candidate.status = Status::Snapshot;
				});
			});
			Self::adjust_contributor_payout();
			Ok(())
		}

		/// Rollback setting to snapshot (snapshot has not been overwrite only)
		#[pallet::weight(0)]
		pub fn rollback_model(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			if <CandidateModelSetting<T>>::get().status == Status::Snapshot {
				<CandidateModelSetting<T>>::mutate(|candidate| {
					<CurrentModelSetting<T>>::mutate(|current| {
						let draft = current.clone();
						*current = candidate.model.clone();
						candidate.model = draft;
						candidate.status = Status::Draft;
					});
				});
				Self::adjust_contributor_payout();
				Ok(())
			} else {
				Err(Error::<T>::SnapshotContamination)?
			}
		}

		/// Set or replace escrow account (H160) for trust
		#[pallet::weight(0)]
		pub fn set_escrow_account(
			origin: OriginFor<T>,
			account: EscrowAccountSetter,
		) -> DispatchResult {
			ensure_root(origin)?;
			match account {
				EscrowAccountSetter::Dao(eth_address) =>
					<EscrowAccounts<T>>::insert(EscrowType::Dao, &eth_address),
				EscrowAccountSetter::Eco(eth_address) =>
					<EscrowAccounts<T>>::insert(EscrowType::Eco, &eth_address),
				EscrowAccountSetter::Dapp(eth_address) =>
					<EscrowAccounts<T>>::insert(EscrowType::Dapp, &eth_address),
				EscrowAccountSetter::Eoa(eth_address) =>
					<EscrowAccounts<T>>::insert(EscrowType::Eoa, &eth_address),
			}
			Ok(())
		}

		/// Set or replace escrow account (H160) for trust
		#[pallet::weight(0)]
		pub fn set_contributors(origin: OriginFor<T>, new: Vec<Contributor>) -> DispatchResult {
			ensure_root(origin)?;
			let mut new_contributors: Vec<ContributorInfo> = Vec::new();
			let mut sum: u32 = 0;
			for contributor in new.iter() {
				sum += contributor.perthousand;
				new_contributors.push(ContributorInfo {
					eth_address: contributor.eth_address,
					perthousand: contributor.perthousand,
					pay_per_block: Default::default(),
				});
			}
			if sum <= 1000 {
				<Contributors<T>>::put(new_contributors);
				Self::adjust_contributor_payout();
				Self::deposit_event(Event::SetContributors(<Contributors<T>>::get()));
				Ok(())
			} else {
				Err(Error::<T>::DistributionOverLimit)?
			}
		}
	}
}

impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
where
	T: Config + pallet_authorship::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		fn is_active(range: Range, block_number: u32) -> bool {
			if range.start <= block_number && block_number < range.end {
				return true
			} else {
				return false
			}
		}

		let current_block_number =
			<frame_system::Pallet<T>>::block_number().saturated_into::<u32>();
		let mut issuance_this_time: u128 = 0;

		// Aution reward pay once at special block
		if current_block_number == <CurrentModelSetting<T>>::get().aution_reward.config.pay_once.at
		{
			let total = <CurrentModelSetting<T>>::get().aution_reward.pay_once;
			for contributor in <Contributors<T>>::get().iter() {
				let quota = Perbill::from_perthousand(contributor.perthousand) * total;
				Self::adjus_balance(contributor.eth_address, quota);
				issuance_this_time += quota;
				log::debug!(target: "reward", "aution reward (once): {:?} to [{:?}]", quota, contributor.eth_address);
			}
		}
		if is_active(
			<CurrentModelSetting<T>>::get().aution_reward.config.range,
			current_block_number,
		) {
			for contributor in <Contributors<T>>::get().iter() {
				Self::adjus_balance(contributor.eth_address, contributor.pay_per_block);
				issuance_this_time += contributor.pay_per_block;
				log::debug!(target: "reward", "aution reward: {:?} to [{:?}]", contributor.pay_per_block, contributor.eth_address);
			}
		}

		// Collator reward
		if is_active(
			<CurrentModelSetting<T>>::get().collator_reward.config.range,
			current_block_number,
		) {
			if let Ok(eth_address) = T::EthAddressMapping::eth_address(author) {
				let amount = <CurrentModelSetting<T>>::get().collator_reward.config.pay_per_block;
				Self::adjus_balance(eth_address, amount);
				issuance_this_time += amount;
				log::debug!(target: "reward", "collator reward: {:?} to [{:?}]", amount, eth_address);
			}
		}

		// Dao trust
		if is_active(<CurrentModelSetting<T>>::get().dao_trust.config.range, current_block_number) {
			let amount = <CurrentModelSetting<T>>::get().dao_trust.pay_per_block;
			Self::adjus_balance(<EscrowAccounts<T>>::get(EscrowType::Dao), amount);
			issuance_this_time += amount;
			log::debug!(target: "reward", "dao trust: {:?} to [{:?}]", amount, <EscrowAccounts<T>>::get(EscrowType::Dao));
		}

		// Eco trust
		if is_active(<CurrentModelSetting<T>>::get().eco_trust.config.range, current_block_number) {
			let amount = <CurrentModelSetting<T>>::get().eco_trust.pay_per_block;
			Self::adjus_balance(<EscrowAccounts<T>>::get(EscrowType::Eco), amount);
			issuance_this_time += amount;
			log::debug!(target: "reward", "eco trust: {:?} to [{:?}]", amount, <EscrowAccounts<T>>::get(EscrowType::Eco));
		}

		// Dapp trust
		if is_active(<CurrentModelSetting<T>>::get().dapp_trust.config.range, current_block_number)
		{
			let amount = <CurrentModelSetting<T>>::get().dapp_trust.config.pay_per_block;
			Self::adjus_balance(<EscrowAccounts<T>>::get(EscrowType::Dapp), amount);
			issuance_this_time += amount;
			log::debug!(target: "reward", "dapp trust: {:?} to [{:?}]", amount, <EscrowAccounts<T>>::get(EscrowType::Dapp));
		}

		// Inflation rate
		if is_active(
			<CurrentModelSetting<T>>::get().inflation_rate.config.range,
			current_block_number,
		) {
			let amount = Perbill::from_percent(
				<CurrentModelSetting<T>>::get().inflation_rate.config.percentage,
			) * issuance_this_time;
			Self::adjus_balance(<EscrowAccounts<T>>::get(EscrowType::Eoa), amount);
			log::debug!(target: "reward", "inflation rate: {:?} to [{:?}]", amount, <EscrowAccounts<T>>::get(EscrowType::Eoa));
		}
	}
	fn note_uncle(_author: T::AccountId, _age: T::BlockNumber) {}
}

impl<T: Config> Pallet<T> {
	pub(crate) fn adjust_contributor_payout() {
		let over_all = <CurrentModelSetting<T>>::get().aution_reward.pay_per_block;
		<Contributors<T>>::mutate(|contributors| {
			for contributor in contributors.iter_mut() {
				contributor.pay_per_block =
					Perbill::from_perthousand(contributor.perthousand) * over_all;
			}
		});
	}
	pub(crate) fn adjus_balance(eth_address: H160, amount: u128) {
		T::Currency::issue(amount.unique_saturated_into());
		T::Currency::deposit_creating(
			&T::AccountMapping::into_account_id(eth_address),
			amount.unique_saturated_into(),
		);
	}
}

/// Trait that outputs the eth's address binding polkadot account. (port from frontier)
pub trait AccountMapping<A> {
	fn into_account_id(address: H160) -> A;
}

/// Hashed eth address to polkadot account mapping. (port from frontier)
pub struct HashedAddressMapping<H>(sp_std::marker::PhantomData<H>);
impl<H: Hasher<Out = H256>> AccountMapping<AccountId32> for HashedAddressMapping<H> {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data = [0u8; 24];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&address[..]);
		let hash = H::hash(&data);
		AccountId32::from(Into::<[u8; 32]>::into(hash))
	}
}
