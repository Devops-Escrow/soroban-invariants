#![cfg(test)]

use proptest::strategy::Strategy;
use soroban_invariants::harness::{
    run_scenario, HarnessConfig, HarnessError, Scenario, StepOutcome,
};
use soroban_invariants::invariants::InvariantError;
use soroban_sdk::Env;

// 1. Correct Scenario: Counter model that increments and decrements, never goes below 0.
#[derive(Debug, Clone)]
enum CounterAction {
    Increment(u32),
    Decrement(u32),
}

struct CounterScenario {
    contract_val: u32,
}

struct CounterModel {
    model_val: u32,
}

impl Scenario for CounterScenario {
    type Action = CounterAction;
    type Model = CounterModel;

    fn init(_env: &Env) -> (Self, Self::Model) {
        (
            CounterScenario { contract_val: 0 },
            CounterModel { model_val: 0 },
        )
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        proptest::prop_oneof![
            (1u32..=100).prop_map(CounterAction::Increment),
            (1u32..=100).prop_map(CounterAction::Decrement),
        ]
    }

    fn apply(&mut self, _env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        match action {
            CounterAction::Increment(amt) => {
                self.contract_val = self.contract_val.saturating_add(*amt);
                model.model_val = model.model_val.saturating_add(*amt);
                StepOutcome::Success
            }
            CounterAction::Decrement(amt) => {
                if *amt <= self.contract_val {
                    self.contract_val -= *amt;
                    model.model_val -= *amt;
                    StepOutcome::Success
                } else {
                    StepOutcome::contract_error(1) // Insufficient balance
                }
            }
        }
    }

    fn check_invariants(&self, _env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        if self.contract_val != model.model_val {
            return Err(InvariantError::Custom(format!(
                "contract_val ({}) != model_val ({})",
                self.contract_val, model.model_val
            )));
        }
        Ok(())
    }
}

#[test]
fn test_correct_scenario_passes_256_cases() {
    let config = HarnessConfig::default()
        .with_cases(256)
        .with_max_actions(15);
    let result = run_scenario::<CounterScenario>(config);
    assert!(result.is_ok(), "Expected scenario to pass over 256 cases");
}

// 2. Known violation scenario: A buggy contract that desynchronizes if increment >= 75
#[derive(Debug, Clone)]
enum BuggyAction {
    Add(u32),
}

struct BuggyScenario {
    count: u32,
}

struct BuggyModel {
    expected: u32,
}

impl Scenario for BuggyScenario {
    type Action = BuggyAction;
    type Model = BuggyModel;

    fn init(_env: &Env) -> (Self, Self::Model) {
        (BuggyScenario { count: 0 }, BuggyModel { expected: 0 })
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        (1u32..=100).prop_map(BuggyAction::Add)
    }

    fn apply(&mut self, _env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        match action {
            BuggyAction::Add(amt) => {
                model.expected += *amt;
                if *amt >= 75 {
                    // BUG: skips 1 on large additions
                    self.count += *amt - 1;
                } else {
                    self.count += *amt;
                }
                StepOutcome::Success
            }
        }
    }

    fn check_invariants(&self, _env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        if self.count != model.expected {
            return Err(InvariantError::Custom(format!(
                "count ({}) != expected ({})",
                self.count, model.expected
            )));
        }
        Ok(())
    }
}

#[test]
fn test_known_violation_detected_and_shrunk() {
    let config = HarnessConfig::default()
        .with_cases(100)
        .with_min_actions(1)
        .with_max_actions(20);

    let result = run_scenario::<BuggyScenario>(config);
    assert!(
        result.is_err(),
        "Expected scenario to detect invariant failure"
    );

    if let Err(HarnessError::FailureReport(report)) = result {
        // Assert that shrinking found the minimal sequence: exactly 1 action!
        assert!(
            report.contains("shrunk to 1 actions"),
            "Expected sequence to shrink to 1 action, but got report:\n{}",
            report
        );
        assert!(report.contains("Add(75)"));
        assert!(report.contains("Replay Seed:"));
    }
}

#[test]
fn test_deterministic_replay_with_seed() {
    // Fixed seed
    let seed: [u8; 32] = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, 0x08,
    ];

    let config1 = HarnessConfig::default().with_cases(50).with_seed(seed);
    let config2 = HarnessConfig::default().with_cases(50).with_seed(seed);

    let res1 = run_scenario::<BuggyScenario>(config1);
    let res2 = run_scenario::<BuggyScenario>(config2);

    assert_eq!(
        res1, res2,
        "Replay with fixed seed must be strictly identical"
    );
}
