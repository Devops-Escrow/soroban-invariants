//! # soroban-invariants
//!
//! A property-based and model-based invariant testing library for Soroban smart contracts.
//!
//! Developers describe the actions a user can take against their contract and the invariants
//! that must always hold (such as "total token supply never changes", "no balance is ever negative",
//! "a privileged function always requires authorization", "an unlock schedule stays sorted").
//! The library generates random action sequences, runs them against the contract in the Soroban
//! test environment, checks the invariants after every step, and shrinks any failure to a minimal
//! reproducing sequence.

pub mod examples;
pub mod harness;
pub mod invariants;
pub mod strategies;

pub use harness::*;
pub use invariants::*;
pub use strategies::*;
