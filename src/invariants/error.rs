//! Error types for invariant checks.

use thiserror::Error;

/// Errors returned when an invariant assertion fails.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvariantError {
    /// Token conservation invariant failed: balances sum mismatch.
    #[error("token conservation violated: expected sum {expected_total} (initial {initial_total} + delta {delta}), found actual sum {actual_total}")]
    TokenConservationViolation {
        initial_total: i128,
        actual_total: i128,
        delta: i128,
        expected_total: i128,
    },

    /// An account balance dipped below zero.
    #[error(
        "negative balance detected at index {index} (address: {address:?}): balance is {balance}"
    )]
    NegativeBalanceViolation {
        address: Option<String>,
        balance: i128,
        index: usize,
    },

    /// Monotonic time was violated (time moved backwards).
    #[error("ledger time regression detected: previous time was {previous_time}, but current time moved backwards to {current_time}")]
    TimeRegressionViolation {
        previous_time: u64,
        current_time: u64,
    },

    /// Authorization invariant violated.
    #[error("authorization check failed: {0}")]
    AuthViolation(String),

    /// Ordering invariant violated.
    #[error("ordering violated at index {index}: {message}")]
    UnsortedKeyViolation { index: usize, message: String },

    /// Sum matching invariant violated.
    #[error("sum mismatch: reported total is {reported_total}, but sum of components is {actual_sum} (diff: {diff})")]
    SumMismatchViolation {
        reported_total: i128,
        actual_sum: i128,
        diff: i128,
    },

    /// Custom user-defined invariant violation.
    #[error("invariant violation: {0}")]
    Custom(String),
}
