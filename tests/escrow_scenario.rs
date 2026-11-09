#![cfg(test)]

use proptest::prop_oneof;
use proptest::strategy::Strategy;
use soroban_invariants::examples::escrow::{EscrowContract, EscrowContractClient};
use soroban_invariants::harness::{run_scenario, HarnessConfig, Scenario, StepOutcome};
use soroban_invariants::invariants::{
    non_negative_balances, sum_matches, token_conservation, InvariantError,
};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[derive(Debug, Clone)]
pub enum EscrowAction {
    Deposit(i128),
    Release(i128),
    Refund(i128),
}

pub struct EscrowScenario {
    contract_id: Address,
    depositor: Address,
    _beneficiary: Address,
    arbiter: Address,
}

pub struct EscrowModel {
    depositor_wallet: i128,
    beneficiary_wallet: i128,
    escrow_balance: i128,
    initial_total: i128,
    total_deposited: i128,
    total_released: i128,
    total_refunded: i128,
}

impl Scenario for EscrowScenario {
    type Action = EscrowAction;
    type Model = EscrowModel;

    fn init(env: &Env) -> (Self, Self::Model) {
        let depositor = Address::generate(env);
        let beneficiary = Address::generate(env);
        let arbiter = Address::generate(env);

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(env, &contract_id);

        client.init(&depositor, &beneficiary, &arbiter);

        let initial_depositor = 1_000_000i128;

        (
            EscrowScenario {
                contract_id,
                depositor,
                _beneficiary: beneficiary,
                arbiter,
            },
            EscrowModel {
                depositor_wallet: initial_depositor,
                beneficiary_wallet: 0,
                escrow_balance: 0,
                initial_total: initial_depositor,
                total_deposited: 0,
                total_released: 0,
                total_refunded: 0,
            },
        )
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        prop_oneof![
            (1i128..=50_000).prop_map(EscrowAction::Deposit),
            (1i128..=50_000).prop_map(EscrowAction::Release),
            (1i128..=50_000).prop_map(EscrowAction::Refund),
        ]
    }

    fn apply(&mut self, env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        env.mock_all_auths();
        let client = EscrowContractClient::new(env, &self.contract_id);

        match action {
            EscrowAction::Deposit(amt) => {
                if *amt <= model.depositor_wallet {
                    match client.try_deposit(&self.depositor, amt) {
                        Ok(Ok(())) => {
                            model.depositor_wallet -= *amt;
                            model.escrow_balance += *amt;
                            model.total_deposited += *amt;
                            StepOutcome::Success
                        }
                        other => StepOutcome::from_result(other),
                    }
                } else {
                    StepOutcome::Ignored
                }
            }
            EscrowAction::Release(amt) => {
                if *amt <= model.escrow_balance {
                    match client.try_release(&self.arbiter, amt) {
                        Ok(Ok(())) => {
                            model.escrow_balance -= *amt;
                            model.beneficiary_wallet += *amt;
                            model.total_released += *amt;
                            StepOutcome::Success
                        }
                        other => StepOutcome::from_result(other),
                    }
                } else {
                    StepOutcome::Ignored
                }
            }
            EscrowAction::Refund(amt) => {
                if *amt <= model.escrow_balance {
                    match client.try_refund(&self.arbiter, amt) {
                        Ok(Ok(())) => {
                            model.escrow_balance -= *amt;
                            model.depositor_wallet += *amt;
                            model.total_refunded += *amt;
                            StepOutcome::Success
                        }
                        other => StepOutcome::from_result(other),
                    }
                } else {
                    StepOutcome::Ignored
                }
            }
        }
    }

    fn check_invariants(&self, env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        let client = EscrowContractClient::new(env, &self.contract_id);
        let contract_bal = client.get_balance();
        let total_dep = client.get_total_deposited();
        let total_rel = client.get_total_released();
        let total_ref = client.get_total_refunded();

        // 1. Contract balance matches model escrow balance
        if contract_bal != model.escrow_balance {
            return Err(InvariantError::Custom(format!(
                "contract balance ({}) != model balance ({})",
                contract_bal, model.escrow_balance
            )));
        }

        // 2. Token conservation across all participants and contract
        token_conservation(
            model.initial_total,
            &[
                model.depositor_wallet,
                model.beneficiary_wallet,
                contract_bal,
            ],
            0,
        )?;

        // 3. Non-negative balances
        non_negative_balances(&[
            model.depositor_wallet,
            model.beneficiary_wallet,
            contract_bal,
        ])?;

        // 4. Sum matches: total_deposited = contract_bal + total_rel + total_ref
        sum_matches(total_dep, &[contract_bal, total_rel, total_ref])?;

        Ok(())
    }
}

#[test]
fn test_escrow_invariants_pass_256_cases() {
    let config = HarnessConfig::default()
        .with_cases(256)
        .with_max_actions(25);

    let res = run_scenario::<EscrowScenario>(config);
    assert!(
        res.is_ok(),
        "Expected escrow scenario to pass all invariants"
    );
}
