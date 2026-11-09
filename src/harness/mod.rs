//! Model-based scenario testing harness for Soroban contracts.
//!
//! Provides the [`Scenario`] trait, execution runner [`run_scenario`],
//! test configuration [`HarnessConfig`], action outcome tracking [`StepOutcome`],
//! and error reporting with counterexample shrinking and seed replay.

pub mod config;
pub mod outcome;
pub mod runner;
pub mod scenario;

pub use config::HarnessConfig;
pub use outcome::StepOutcome;
pub use runner::{format_failure_report, run_scenario, HarnessError};
pub use scenario::Scenario;
