//! Scenario trait defining model-based invariant tests for Soroban contracts.

use crate::harness::outcome::StepOutcome;
use crate::invariants::InvariantError;
use proptest::strategy::Strategy;
use soroban_sdk::Env;

/// A model-based testing scenario for a Soroban smart contract.
///
/// Implementors define:
/// - `Action`: The universe of possible user or ledger actions.
/// - `Model`: An abstract reference model tracking expected contract state.
/// - `init`: Setup method that deploys contracts and constructs initial model state.
/// - `strategy`: A `proptest` strategy generating randomized `Action` instances.
/// - `apply`: State transition method that applies an action to both contract and model.
/// - `check_invariants`: Invariant assertion method executed after every single step.
pub trait Scenario: Sized {
    /// The action type representing contract calls or environment advancements.
    type Action: core::fmt::Debug + Clone;

    /// The abstract reference model tracking expected state.
    type Model;

    /// Initialize the Soroban environment, contract instances, and the reference model.
    fn init(env: &Env) -> (Self, Self::Model);

    /// Strategy generating randomized actions for this scenario.
    fn strategy() -> impl Strategy<Value = Self::Action>;

    /// Apply an action to both the smart contract and the model, returning a [`StepOutcome`].
    fn apply(&mut self, env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome;

    /// Invariant assertions evaluated after every step.
    ///
    /// If any invariant is violated, return `Err(InvariantError)`.
    /// The default implementation returns `Ok(())`.
    fn check_invariants(&self, _env: &Env, _model: &Self::Model) -> Result<(), InvariantError> {
        Ok(())
    }
}
