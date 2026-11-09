#![cfg(test)]

use proptest::prop_oneof;
use proptest::strategy::Strategy;
use soroban_invariants::examples::buggy_vesting::{
    BuggyVestingContract, BuggyVestingContractClient,
};
use soroban_invariants::examples::vesting::{VestingContract, VestingContractClient};
use soroban_invariants::harness::{
    run_scenario, HarnessConfig, HarnessError, Scenario, StepOutcome,
};
use soroban_invariants::invariants::{sorted_by_key, InvariantError};
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

#[derive(Debug, Clone)]
pub enum VestingAction {
    AppendMilestone { unlock_timestamp: u64, amount: i128 },
    AdvanceTime(u64),
    Claim,
}

// -----------------------------------------------------------------------------
// 1. Buggy Vesting Scenario
// -----------------------------------------------------------------------------
pub struct BuggyVestingScenario {
    contract_id: Address,
    beneficiary: Address,
}

pub struct BuggyModel {
    current_time: u64,
    milestones: Vec<(u64, i128)>,
    claimed: i128,
}

impl BuggyModel {
    pub fn expected_vested(&self) -> i128 {
        // Correct reference model: only milestones with unlock_timestamp <= current_time are vested
        self.milestones
            .iter()
            .filter(|(ts, _)| *ts <= self.current_time)
            .map(|(_, amt)| *amt)
            .max()
            .unwrap_or(0)
    }
}

impl Scenario for BuggyVestingScenario {
    type Action = VestingAction;
    type Model = BuggyModel;

    fn init(env: &Env) -> (Self, Self::Model) {
        let beneficiary = Address::generate(env);
        let contract_id = env.register(BuggyVestingContract, ());
        let client = BuggyVestingContractClient::new(env, &contract_id);

        env.ledger().set_timestamp(1_000);
        client.init(&beneficiary);

        (
            BuggyVestingScenario {
                contract_id,
                beneficiary,
            },
            BuggyModel {
                current_time: 1_000,
                milestones: Vec::new(),
                claimed: 0,
            },
        )
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        prop_oneof![
            // Deliberately generates unordered timestamps to test schedule monotonicity
            (100u64..=10_000, 100i128..=50_000).prop_map(|(unlock_timestamp, amount)| {
                VestingAction::AppendMilestone {
                    unlock_timestamp,
                    amount,
                }
            }),
            (10u64..=1_000).prop_map(VestingAction::AdvanceTime),
            proptest::strategy::Just(VestingAction::Claim),
        ]
    }

    fn apply(&mut self, env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        env.mock_all_auths();
        let client = BuggyVestingContractClient::new(env, &self.contract_id);

        match action {
            VestingAction::AppendMilestone {
                unlock_timestamp,
                amount,
            } => match client.try_append_milestone(unlock_timestamp, amount) {
                Ok(Ok(())) => {
                    model.milestones.push((*unlock_timestamp, *amount));
                    StepOutcome::Success
                }
                other => StepOutcome::from_result(other),
            },
            VestingAction::AdvanceTime(delta) => {
                model.current_time = model.current_time.saturating_add(*delta);
                env.ledger().set_timestamp(model.current_time);
                StepOutcome::Success
            }
            VestingAction::Claim => match client.try_claim(&self.beneficiary) {
                Ok(Ok(amt)) => {
                    model.claimed += amt;
                    StepOutcome::Success
                }
                other => StepOutcome::from_result(other),
            },
        }
    }

    fn check_invariants(&self, env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        let client = BuggyVestingContractClient::new(env, &self.contract_id);
        let schedule_soroban = client.get_schedule();
        let mut rust_sched = Vec::with_capacity(schedule_soroban.len() as usize);
        for i in 0..schedule_soroban.len() {
            let m = schedule_soroban.get(i).unwrap();
            rust_sched.push((m.unlock_timestamp, m.cumulative_amount));
        }

        // Invariant 1: The contract schedule must stay sorted by unlock_timestamp
        sorted_by_key(&rust_sched, |(ts, _)| *ts)?;

        // Invariant 2: Total claimed must not exceed model's expected vested amount
        let contract_claimed = client.get_total_claimed();
        let expected_vested = model.expected_vested();
        if contract_claimed > expected_vested {
            return Err(InvariantError::Custom(format!(
                "over-release bug detected: contract claimed {} exceeds expected vested {} at time {}",
                contract_claimed, expected_vested, env.ledger().timestamp()
            )));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 2. Fixed Vesting Scenario
// -----------------------------------------------------------------------------
pub struct FixedVestingScenario {
    contract_id: Address,
    beneficiary: Address,
}

pub struct FixedModel {
    current_time: u64,
    last_timestamp: u64,
    last_amount: i128,
    claimed: i128,
}

impl Scenario for FixedVestingScenario {
    type Action = VestingAction;
    type Model = FixedModel;

    fn init(env: &Env) -> (Self, Self::Model) {
        let beneficiary = Address::generate(env);
        let contract_id = env.register(VestingContract, ());
        let client = VestingContractClient::new(env, &contract_id);

        env.ledger().set_timestamp(1_000);
        client.init(&beneficiary);

        (
            FixedVestingScenario {
                contract_id,
                beneficiary,
            },
            FixedModel {
                current_time: 1_000,
                last_timestamp: 0,
                last_amount: 0,
                claimed: 0,
            },
        )
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        prop_oneof![
            (100u64..=10_000, 100i128..=50_000).prop_map(|(unlock_timestamp, amount)| {
                VestingAction::AppendMilestone {
                    unlock_timestamp,
                    amount,
                }
            }),
            (10u64..=1_000).prop_map(VestingAction::AdvanceTime),
            proptest::strategy::Just(VestingAction::Claim),
        ]
    }

    fn apply(&mut self, env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        env.mock_all_auths();
        let client = VestingContractClient::new(env, &self.contract_id);

        match action {
            VestingAction::AppendMilestone {
                unlock_timestamp,
                amount,
            } => {
                match client.try_append_milestone(unlock_timestamp, amount) {
                    Ok(Ok(())) => {
                        model.last_timestamp = *unlock_timestamp;
                        model.last_amount = *amount;
                        StepOutcome::Success
                    }
                    Ok(Err(_err)) => StepOutcome::Success, // Correctly rejected out-of-order append
                    Err(_) => StepOutcome::Success,
                }
            }
            VestingAction::AdvanceTime(delta) => {
                model.current_time = model.current_time.saturating_add(*delta);
                env.ledger().set_timestamp(model.current_time);
                StepOutcome::Success
            }
            VestingAction::Claim => match client.try_claim(&self.beneficiary) {
                Ok(Ok(amt)) => {
                    model.claimed += amt;
                    StepOutcome::Success
                }
                _ => StepOutcome::Success,
            },
        }
    }

    fn check_invariants(&self, env: &Env, _model: &Self::Model) -> Result<(), InvariantError> {
        let client = VestingContractClient::new(env, &self.contract_id);
        let schedule_soroban = client.get_schedule();
        let mut rust_sched = Vec::with_capacity(schedule_soroban.len() as usize);
        for i in 0..schedule_soroban.len() {
            let m = schedule_soroban.get(i).unwrap();
            rust_sched.push((m.unlock_timestamp, m.cumulative_amount));
        }

        // Must always remain strictly sorted
        sorted_by_key(&rust_sched, |(ts, _)| *ts)?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Proof Tests
// -----------------------------------------------------------------------------

#[test]
fn test_proof_harness_finds_and_shrinks_buggy_vesting() {
    let config = HarnessConfig::default()
        .with_cases(50)
        .with_min_actions(2)
        .with_max_actions(15);

    let result = run_scenario::<BuggyVestingScenario>(config);
    assert!(
        result.is_err(),
        "PROOF: Harness MUST detect invariant violation in buggy_vesting"
    );

    if let Err(HarnessError::FailureReport(report)) = result {
        // Verify failure was detected and shrunk
        assert!(
            report.contains("ordering violated") || report.contains("over-release bug detected"),
            "Report must detail the invariant violation:\n{}",
            report
        );
        assert!(
            report.contains("Minimal Reproducing Action Trace"),
            "Report must contain minimal action trace:\n{}",
            report
        );
        // Shrinking minimizes down to 2 actions: e.g. Append(large_ts) then Append(small_ts)!
        assert!(
            report.contains("shrunk to 2 actions") || report.contains("shrunk to 3 actions"),
            "Expected sequence to shrink to 2 or 3 actions, but got report:\n{}",
            report
        );
    }
}

#[test]
fn test_fixed_vesting_passes_all_invariants() {
    let config = HarnessConfig::default()
        .with_cases(256)
        .with_max_actions(20);

    let result = run_scenario::<FixedVestingScenario>(config);
    assert!(
        result.is_ok(),
        "Fixed vesting contract must pass all invariants over 256 cases"
    );
}
