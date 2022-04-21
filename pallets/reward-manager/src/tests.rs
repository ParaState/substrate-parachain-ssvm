use crate::{mock::*, parameter::*, storage::*, tests::currency::GEM, Error};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::H160;
use sp_runtime::traits::BadOrigin;
use std::str::FromStr;

const BLOCK_NUMBER_PER_AUTION_PERIOD: u32 = 96 * 7 * 24 * 60 * 60 / 12; // 4838400
const NOT_ROOT_ACCOUNT: [u8; 32] = [1u8; 32];
const DELAY: u32 = 100;

#[test]
fn ensure_root() {
	new_test_ext().execute_with(|| {
		let config = AutionRewardConfig {
			percentage: 15,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
			pay_once: PayOnce { percentage: 20, at: 10 },
		};

		// update_candidate
		assert_noop!(
			RewardManager::update_candidate(
				Origin::signed(NOT_ROOT_ACCOUNT.into()),
				ModelConfig::AutionRewardConfig(config.clone())
			),
			BadOrigin
		);

		// apply_model
		assert_noop!(
			RewardManager::apply_model(Origin::signed(NOT_ROOT_ACCOUNT.into())),
			BadOrigin
		);

		// rollback_model
		assert_noop!(
			RewardManager::rollback_model(Origin::signed(NOT_ROOT_ACCOUNT.into())),
			BadOrigin
		);

		let address: H160 = H160::from_str("0000000000000000000000000000000000001234").unwrap();

		// set escrow account
		assert_noop!(
			RewardManager::set_escrow_account(
				Origin::signed(NOT_ROOT_ACCOUNT.into()),
				EscrowAccountSetter::Dao(address)
			),
			BadOrigin
		);

		// set contributor
		assert_noop!(
			RewardManager::set_contributors(Origin::signed(NOT_ROOT_ACCOUNT.into()), vec![]),
			BadOrigin
		);
	});
}

#[test]
fn update_candidate_aution_reward_model() {
	new_test_ext().execute_with(|| {
		let config = AutionRewardConfig {
			percentage: 15,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
			pay_once: PayOnce { percentage: 20, at: 10 },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::AutionRewardConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.aution_reward.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);

		// 210000000 GEM * 15% = 31500000
		// 31500000 * 20% = 6300000 GEM =  (6300000000000000000000000 Wei)
		assert_eq!(
			6300000000000000000000000u128,
			RewardManager::candidate_setting().model.aution_reward.pay_once
		);

		// 31500000 * 80% / BLOCK_NUMBER_PER_AUTION_PERIOD = 5208333333333333333
		assert_eq!(
			5208333333333333333u128,
			RewardManager::candidate_setting().model.aution_reward.pay_per_block
		);
	});
}

#[test]
fn update_candidate_collator_reward_model() {
	new_test_ext().execute_with(|| {
		let config = FixedPayoutConfig {
			pay_per_block: 1 * GEM,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::CollatorRewardConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.collator_reward.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);
	});
}

#[test]
fn update_candidate_dao_trust_model() {
	new_test_ext().execute_with(|| {
		let config = NonFixedPayoutConfig {
			percentage: 20,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD * 2 },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::DaoTrustConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.dao_trust.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);

		// 210000000 GEM * 20% / (BLOCK_NUMBER_PER_AUTION_PERIOD*2) = 4340277777777777777
		assert_eq!(
			4340277777777777777u128,
			RewardManager::candidate_setting().model.dao_trust.pay_per_block
		);
	});
}

#[test]
fn update_candidate_eco_trust_model() {
	new_test_ext().execute_with(|| {
		let config = NonFixedPayoutConfig {
			percentage: 15,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD * 2 },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::EcoTrustConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.eco_trust.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);

		// 210000000 GEM * 15% / (BLOCK_NUMBER_PER_AUTION_PERIOD*2) = 3255208333333333333
		assert_eq!(
			3255208333333333333u128,
			RewardManager::candidate_setting().model.eco_trust.pay_per_block
		);
	});
}

#[test]
fn update_candidate_dapp_trust_model() {
	new_test_ext().execute_with(|| {
		let config = FixedPayoutConfig {
			pay_per_block: 6 * GEM,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::DappTrustConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.dapp_trust.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);
	});
}

#[test]
fn update_candidate_inflation_rate_model() {
	new_test_ext().execute_with(|| {
		let config = NonFixedPayoutConfig {
			percentage: 2,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
		};
		assert_ok!(RewardManager::update_candidate(
			Origin::root(),
			ModelConfig::InflationRateConfig(config.clone()),
		));
		assert_eq!(config, RewardManager::candidate_setting().model.inflation_rate.config);
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);
	});
}

fn init_draft() {
	let config = AutionRewardConfig {
		percentage: 15,
		range: Range { start: 0 + DELAY, end: BLOCK_NUMBER_PER_AUTION_PERIOD + DELAY },
		pay_once: PayOnce { percentage: 20, at: 10 },
	};
	RewardManager::update_candidate(Origin::root(), ModelConfig::AutionRewardConfig(config)).ok();
	let config = FixedPayoutConfig {
		pay_per_block: 1 * GEM,
		range: Range { start: 0 + DELAY, end: BLOCK_NUMBER_PER_AUTION_PERIOD + DELAY },
	};
	RewardManager::update_candidate(Origin::root(), ModelConfig::CollatorRewardConfig(config)).ok();
	let config = NonFixedPayoutConfig {
		percentage: 20,
		range: Range { start: 0 + DELAY, end: BLOCK_NUMBER_PER_AUTION_PERIOD * 2 + DELAY },
	};
	RewardManager::update_candidate(Origin::root(), ModelConfig::DaoTrustConfig(config)).ok();
	let config = NonFixedPayoutConfig {
		percentage: 15,
		range: Range { start: 0 + DELAY, end: BLOCK_NUMBER_PER_AUTION_PERIOD * 2 + DELAY },
	};
	RewardManager::update_candidate(Origin::root(), ModelConfig::EcoTrustConfig(config.clone()))
		.ok();
	let config = FixedPayoutConfig {
		pay_per_block: 6 * GEM,
		range: Range { start: 0 + DELAY, end: BLOCK_NUMBER_PER_AUTION_PERIOD + DELAY },
	};
	RewardManager::update_candidate(Origin::root(), ModelConfig::DappTrustConfig(config.clone()))
		.ok();
	let config = NonFixedPayoutConfig {
		percentage: 2,
		range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD * 2 + DELAY },
	};
	RewardManager::update_candidate(
		Origin::root(),
		ModelConfig::InflationRateConfig(config.clone()),
	)
	.ok();
}

#[test]
fn apply_and_rollback_model() {
	new_test_ext().execute_with(|| {
		init_draft();
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);
		// we haven't take any snapshot before
		assert_err!(
			RewardManager::rollback_model(Origin::root()),
			Error::<Test>::SnapshotContamination
		);
		let orig_candicate_model = RewardManager::candidate_setting().model;
		let orig_current_model = RewardManager::current_setting();

		// apply candidate to current
		assert_ok!(RewardManager::apply_model(Origin::root()));
		assert_eq!(orig_candicate_model, RewardManager::current_setting());
		assert_eq!(orig_current_model, RewardManager::candidate_setting().model);
		assert_eq!(Status::Snapshot, RewardManager::candidate_setting().status);

		// rollback to original status
		assert_ok!(RewardManager::rollback_model(Origin::root()));
		assert_eq!(orig_candicate_model, RewardManager::candidate_setting().model);
		assert_eq!(orig_current_model, RewardManager::current_setting());
		assert_eq!(Status::Draft, RewardManager::candidate_setting().status);

		// test snapshot contamination
		RewardManager::apply_model(Origin::root()).ok();
		let config = AutionRewardConfig {
			percentage: 10,
			range: Range { start: 0, end: BLOCK_NUMBER_PER_AUTION_PERIOD },
			pay_once: PayOnce { percentage: 20, at: 10 },
		};
		RewardManager::update_candidate(Origin::root(), ModelConfig::AutionRewardConfig(config))
			.ok();
		assert_err!(
			RewardManager::rollback_model(Origin::root()),
			Error::<Test>::SnapshotContamination
		);
	});
}

#[test]
fn set_escrow_account() {
	new_test_ext().execute_with(|| {
		let alice: H160 = H160::from_str("0000000000000000000000000000000000000001").unwrap();
		let bob: H160 = H160::from_str("0000000000000000000000000000000000000002").unwrap();
		let charlie: H160 = H160::from_str("0000000000000000000000000000000000000003").unwrap();
		let dave: H160 = H160::from_str("0000000000000000000000000000000000000004").unwrap();

		assert_ok!(RewardManager::set_escrow_account(
			Origin::root(),
			EscrowAccountSetter::Dao(alice)
		));
		assert_ok!(RewardManager::set_escrow_account(
			Origin::root(),
			EscrowAccountSetter::Eco(bob)
		));
		assert_ok!(RewardManager::set_escrow_account(
			Origin::root(),
			EscrowAccountSetter::Dapp(charlie)
		));
		assert_ok!(RewardManager::set_escrow_account(
			Origin::root(),
			EscrowAccountSetter::Eoa(dave)
		));
		assert_eq!(alice, RewardManager::escrow_account(EscrowType::Dao));
		assert_eq!(bob, RewardManager::escrow_account(EscrowType::Eco));
		assert_eq!(charlie, RewardManager::escrow_account(EscrowType::Dapp));
		assert_eq!(dave, RewardManager::escrow_account(EscrowType::Eoa));
	});
}

#[test]
fn set_contributors() {
	use sp_core::H160;
	use std::str::FromStr;
	new_test_ext().execute_with(|| {
		let config = AutionRewardConfig {
			percentage: 1,
			range: Range { start: 0, end: 10 },
			pay_once: PayOnce { percentage: 0, at: 0 },
		};
		RewardManager::update_candidate(Origin::root(), ModelConfig::AutionRewardConfig(config))
			.ok();
		RewardManager::apply_model(Origin::root()).ok();

		let alice: H160 = H160::from_str("0000000000000000000000000000000000000001").unwrap();
		let bob: H160 = H160::from_str("0000000000000000000000000000000000000002").unwrap();
		let charlie: H160 = H160::from_str("0000000000000000000000000000000000000003").unwrap();
		let dave: H160 = H160::from_str("0000000000000000000000000000000000000004").unwrap();

		let mut contributors: Vec<Contributor> = Vec::new();
		contributors.push(Contributor { eth_address: alice, perthousand: 1 });
		contributors.push(Contributor { eth_address: bob, perthousand: 10 });
		contributors.push(Contributor { eth_address: charlie, perthousand: 100 });
		contributors.push(Contributor { eth_address: dave, perthousand: 200 });
		assert_ok!(RewardManager::set_contributors(Origin::root(), contributors));

		let contributor_infos = RewardManager::contributors();

		assert_eq!(alice, contributor_infos[0].eth_address);
		assert_eq!(1, contributor_infos[0].perthousand);
		// 210000000 GEM * 1% = 2100000
		// 2100000 * 0.1% / 10 = 210 GEM =  (210000000000000000000 Wei)
		assert_eq!(210000000000000000000, contributor_infos[0].pay_per_block);

		assert_eq!(bob, contributor_infos[1].eth_address);
		assert_eq!(10, contributor_infos[1].perthousand);
		// 210000000 GEM * 1% = 2100000
		// 2100000 * 1% / 10 = 2100 GEM =  (2100000000000000000000 Wei)
		assert_eq!(2100000000000000000000, contributor_infos[1].pay_per_block);

		assert_eq!(charlie, contributor_infos[2].eth_address);
		assert_eq!(100, contributor_infos[2].perthousand);
		// 210000000 GEM * 1% = 2100000
		// 2100000 * 10% / 10 = 21000 GEM =  (21000000000000000000000 Wei)
		assert_eq!(21000000000000000000000, contributor_infos[2].pay_per_block);

		assert_eq!(dave, contributor_infos[3].eth_address);
		assert_eq!(200, contributor_infos[3].perthousand);
		// 210000000 GEM * 1% = 2100000
		// 2100000 * 20% / 10 = 42000 GEM =  (42000000000000000000000 Wei)
		assert_eq!(42000000000000000000000, contributor_infos[3].pay_per_block);
	});
}

#[test]
fn run_tokenomics() {
	use crate::{AccountId32, AccountMapping};

	fn into_balance_account(eth_address: H160) -> AccountId32 {
		<Test as crate::Config>::AccountMapping::into_account_id(eth_address)
	}

	new_test_ext().execute_with(|| {
		// initial
		init_draft();
		RewardManager::apply_model(Origin::root()).ok();

		let escrow_dao: H160 = H160::from_str("0000000000000000000000000000000000000001").unwrap();
		let escrow_eco: H160 = H160::from_str("0000000000000000000000000000000000000002").unwrap();
		let escrow_dapp: H160 = H160::from_str("0000000000000000000000000000000000000003").unwrap();
		let escrow_eoa: H160 = H160::from_str("0000000000000000000000000000000000000004").unwrap();
		RewardManager::set_escrow_account(Origin::root(), EscrowAccountSetter::Dao(escrow_dao))
			.ok();
		RewardManager::set_escrow_account(Origin::root(), EscrowAccountSetter::Eco(escrow_eco))
			.ok();
		RewardManager::set_escrow_account(Origin::root(), EscrowAccountSetter::Dapp(escrow_dapp))
			.ok();
		RewardManager::set_escrow_account(Origin::root(), EscrowAccountSetter::Eoa(escrow_eoa))
			.ok();

		let alice: H160 = H160::from_str("0000000000000000000000000000000000000005").unwrap();
		let bob: H160 = H160::from_str("0000000000000000000000000000000000000006").unwrap();
		let charlie: H160 = H160::from_str("0000000000000000000000000000000000000007").unwrap();
		let dave: H160 = H160::from_str("0000000000000000000000000000000000000008").unwrap();
		let mut contributors: Vec<Contributor> = Vec::new();
		contributors.push(Contributor { eth_address: alice, perthousand: 1 });
		contributors.push(Contributor { eth_address: bob, perthousand: 10 });
		contributors.push(Contributor { eth_address: charlie, perthousand: 100 });
		contributors.push(Contributor { eth_address: dave, perthousand: 200 });
		RewardManager::set_contributors(Origin::root(), contributors).ok();

		// auction reward pay once at block number 10 in test
		initialize_to_block(10);

		// 6300000000000000000000000 * 0.1% = 6300000000000000000000
		assert_eq!(Balances::free_balance(&into_balance_account(alice)), 6300000000000000000000);
		// 6300000000000000000000000 * 1% = 63000000000000000000000
		assert_eq!(Balances::free_balance(&into_balance_account(bob)), 63000000000000000000000);
		// 6300000000000000000000000 * 10% = 630000000000000000000000
		assert_eq!(
			Balances::free_balance(&into_balance_account(charlie)),
			630000000000000000000000
		);
		// 6300000000000000000000000 * 20% = 1260000000000000000000000
		assert_eq!(Balances::free_balance(&into_balance_account(dave)), 1260000000000000000000000);
		// (6300000000000000000000 + 63000000000000000000000 + 630000000000000000000000 + 1260000000000000000000000) * 2% = 3.9186e+22
		let eoa = into_balance_account(escrow_eoa);
		assert_eq!(Balances::free_balance(&eoa), 39186000000000000000000);

		// all per block rewards start at block number 100 (delay value) in test
		initialize_to_block(100);

		// 5208333333333333333 * 0.1% = 5208333333333333
		assert_eq!(Balances::free_balance(&into_balance_account(alice)), 6300005208333333333333);
		// 5208333333333333333 * 1% = 52083333333333333
		assert_eq!(Balances::free_balance(&into_balance_account(bob)), 63000052083333333333333);
		// 5208333333333333333 * 10% = 520833333333333333
		assert_eq!(
			Balances::free_balance(&into_balance_account(charlie)),
			630000520833333333333333
		);
		// 5208333333333333333 * 20% = 1041666666666666667
		assert_eq!(Balances::free_balance(&into_balance_account(dave)), 1260001041666666666666667);

		// 1 GEM
		let collator =
			into_balance_account(H160::from_str(REGISTERED_COLLATOR_ETH_ADDRESS).unwrap());
		assert_eq!(Balances::free_balance(&collator), 1 * GEM);

		// 210000000 GEM * 20% / (BLOCK_NUMBER_PER_AUTION_PERIOD*2) = 4340277777777777777
		let dao = into_balance_account(escrow_dao);
		assert_eq!(Balances::free_balance(&dao), 4340277777777777777u128);

		// 210000000 GEM * 15% / (BLOCK_NUMBER_PER_AUTION_PERIOD*2) = 3255208333333333333
		let eco = into_balance_account(escrow_eco);
		assert_eq!(Balances::free_balance(&eco), 3255208333333333333u128);

		// 6 GEM
		let dapp = into_balance_account(escrow_dapp);
		assert_eq!(Balances::free_balance(&dapp), 6 * GEM);

		// (6300005208333333333333 + 63000052083333333333333 + 630000520833333333333333 + 1260001041666666666666667 +
		// 1000000000000000000 + 4340277777777777777 + 3255208333333333333 + 6000000000000000000) * 2% = 3.9186324e+22
		let eoa = into_balance_account(escrow_eoa);
		assert_eq!(Balances::free_balance(&eoa), 39186324305555555555556);

		// all per block rewards end before block number (BLOCK_NUMBER_PER_AUTION_PERIOD * 2 + DELAY) in test
		let final_reward_block: u64 = (BLOCK_NUMBER_PER_AUTION_PERIOD * 2 + DELAY - 1).into();
		System::set_block_number(final_reward_block - 1);
		let before_final_total_issuance = Balances::total_issuance();
		initialize_to_block(final_reward_block);
		let final_total_issuance = Balances::total_issuance();
		assert_ne!(final_total_issuance, before_final_total_issuance);
		initialize_to_block(final_reward_block + 1);
		assert_eq!(Balances::total_issuance(), final_total_issuance);
	});
}
