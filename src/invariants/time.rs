//! Monotonic ledger time invariant helper.

use crate::invariants::error::InvariantError;

/// Asserts that the ledger timestamp does not move backwards between execution steps.
///
/// # Arguments
/// * `previous_time` - The timestamp recorded at the prior step.
/// * `current_time` - The current ledger timestamp.
///
/// # Errors
/// Returns [`InvariantError::TimeRegressionViolation`] if `current_time < previous_time`.
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::monotonic_ledger_time;
///
/// assert!(monotonic_ledger_time(100, 100).is_ok());
/// assert!(monotonic_ledger_time(100, 150).is_ok());
/// assert!(monotonic_ledger_time(150, 100).is_err());
/// ```
pub fn monotonic_ledger_time(previous_time: u64, current_time: u64) -> Result<(), InvariantError> {
    if current_time < previous_time {
        Err(InvariantError::TimeRegressionViolation {
            previous_time,
            current_time,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_time_passing() {
        assert!(monotonic_ledger_time(1_000, 1_000).is_ok());
        assert!(monotonic_ledger_time(1_000, 1_005).is_ok());
        assert!(monotonic_ledger_time(0, 50).is_ok());
    }

    #[test]
    fn test_monotonic_time_regression() {
        let err = monotonic_ledger_time(1_000, 999).unwrap_err();
        match err {
            InvariantError::TimeRegressionViolation {
                previous_time,
                current_time,
            } => {
                assert_eq!(previous_time, 1_000);
                assert_eq!(current_time, 999);
            }
            _ => panic!("Unexpected error"),
        }
    }
}
