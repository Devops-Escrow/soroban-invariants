//! Test runner for scenarios.

use crate::harness::config::HarnessConfig;
use crate::harness::scenario::Scenario;
use proptest::collection::vec;
use proptest::test_runner::{
    Config as PropConfig, RngAlgorithm, TestCaseError, TestError, TestRng, TestRunner,
};
use soroban_sdk::Env;
use thiserror::Error;

/// Error returned when a scenario run fails.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HarnessError {
    /// Invariant failure during scenario execution.
    #[error("{0}")]
    FailureReport(String),
}

/// Runs a model-based scenario with the provided configuration.
///
/// Generates sequences of actions, initializes the environment and scenario,
/// applies each action step, verifies the outcome and evaluates invariants.
///
/// On any failure:
/// - Shrinks the action sequence to the minimal reproducing trace.
/// - Prints the deterministic reproduction seed.
/// - Returns a detailed, formatted failure report.
///
/// # Example
/// ```rust,no_run
/// use soroban_invariants::harness::{HarnessConfig, run_scenario};
/// # use soroban_invariants::harness::{Scenario, StepOutcome};
/// # use soroban_sdk::Env;
/// # use proptest::strategy::{Strategy, Just};
/// # struct Dummy;
/// # impl Scenario for Dummy {
/// #     type Action = ();
/// #     type Model = ();
/// #     fn init(_env: &Env) -> (Self, Self::Model) { (Dummy, ()) }
/// #     fn strategy() -> impl Strategy<Value = ()> { Just(()) }
/// #     fn apply(&mut self, _env: &Env, _model: &mut (), _action: &()) -> StepOutcome { StepOutcome::Success }
/// # }
///
/// let config = HarnessConfig::default().with_cases(50);
/// assert!(run_scenario::<Dummy>(config).is_ok());
/// ```
pub fn run_scenario<S: Scenario>(config: HarnessConfig) -> Result<(), HarnessError> {
    let seed: [u8; 32] = match config.seed {
        Some(s) => s,
        None => {
            let mut s = [0u8; 32];
            // Fill with deterministic pseudo-entropy or system time if unavailable
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(42);
            for (i, byte) in s.iter_mut().enumerate() {
                *byte = ((nanos >> (i % 16)) ^ (i as u128 * 0x517cc1b727220a95)) as u8;
            }
            s
        }
    };

    let prop_config = PropConfig {
        cases: config.test_cases,
        max_shrink_iters: config.max_shrink_iters,
        failure_persistence: None,
        ..Default::default()
    };

    let rng = TestRng::from_seed(RngAlgorithm::default(), &seed);
    let mut runner = TestRunner::new_with_rng(prop_config, rng);

    let action_seq_strategy = vec(S::strategy(), config.min_actions..=config.max_actions);

    let run_res = runner.run(&action_seq_strategy, |actions| {
        let env = Env::default();
        let (mut scenario, mut model) = S::init(&env);

        for (idx, action) in actions.iter().enumerate() {
            let outcome = scenario.apply(&env, &mut model, action);
            if let Err(inv_err) = scenario.check_invariants(&env, &model) {
                return Err(TestCaseError::fail(format!(
                    "Step #{} failed invariant:\nAction: {:?}\nOutcome: {:?}\nInvariant Error: {}",
                    idx, action, outcome, inv_err
                )));
            }
        }
        Ok(())
    });

    match run_res {
        Ok(()) => Ok(()),
        Err(TestError::Fail(reason, minimal_actions)) => {
            let reason_str = reason.to_string();
            let report = format_failure_report::<S>(&seed, &reason_str, &minimal_actions);
            Err(HarnessError::FailureReport(report))
        }
        Err(TestError::Abort(reason)) => Err(HarnessError::FailureReport(format!(
            "Scenario execution aborted: {}",
            reason
        ))),
    }
}

/// Format a human-readable failure report with minimal reproducing actions and replay seed.
pub fn format_failure_report<S: Scenario>(
    seed: &[u8; 32],
    reason: &str,
    actions: &[S::Action],
) -> String {
    let mut report = String::new();
    report.push_str(
        "\n================================================================================\n",
    );
    report.push_str("                    SOROBAN INVARIANTS FAILURE REPORT\n");
    report.push_str(
        "================================================================================\n",
    );
    report.push_str(&format!(
        "Replay Seed: {:?}\n(reproduce with: HarnessConfig::default().with_seed({:?}))\n\n",
        seed, seed
    ));
    report.push_str("Failure Detail:\n");
    report.push_str(reason);
    report.push_str("\n\n");
    report.push_str(&format!(
        "Minimal Reproducing Action Trace (shrunk to {} actions):\n",
        actions.len()
    ));

    for (i, action) in actions.iter().enumerate() {
        report.push_str(&format!("  {:2}: {:?}\n", i + 1, action));
    }

    report.push_str(
        "================================================================================\n",
    );
    report
}
