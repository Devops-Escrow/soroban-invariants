//! Built-in invariant verification helpers for Soroban contracts.
//!
//! Includes helpers for token conservation, non-negative balances, monotonic ledger time,
//! contract authorization verification, sorted order checks, and sum integrity.

pub mod auth;
pub mod balances;
pub mod error;
pub mod sorted;
pub mod sum;
pub mod time;
pub mod token_conservation;

pub use auth::{assert_auth_not_recorded, assert_auth_recorded, assert_requires_auth};
pub use balances::{named_non_negative_balances, non_negative_balances};
pub use error::InvariantError;
pub use sorted::{sorted_by_key, strictly_sorted_by_key};
pub use sum::sum_matches;
pub use time::monotonic_ledger_time;
pub use token_conservation::token_conservation;
