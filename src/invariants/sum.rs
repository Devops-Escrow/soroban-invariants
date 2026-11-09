//! Invariant helper asserting that an aggregated total equals the sum of its component parts.

use crate::invariants::error::InvariantError;

/// Asserts that a reported total matches the sum of an array of individual components.
///
/// Useful for validating that:
/// - A contract's total locked liquidity equals the sum of user deposits.
/// - Remaining budget equals total budget minus all allocated grants.
/// - Outstanding unvested shares equal total grant minus claimed shares.
///
/// # Arguments
/// * `reported_total` - The total reported by the contract or expected by the model.
/// * `parts` - Slice of individual component values.
///
/// # Errors
/// Returns [`InvariantError::SumMismatchViolation`] if `sum(parts) != reported_total`.
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::sum_matches;
///
/// let total_allocated = 100_000i128;
/// let grants = [30_000i128, 50_000i128, 20_000i128];
/// assert!(sum_matches(total_allocated, &grants).is_ok());
/// ```
pub fn sum_matches(reported_total: i128, parts: &[i128]) -> Result<(), InvariantError> {
    let actual_sum: i128 = parts.iter().sum();
    if actual_sum != reported_total {
        let diff = actual_sum.saturating_sub(reported_total);
        Err(InvariantError::SumMismatchViolation {
            reported_total,
            actual_sum,
            diff,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_matches_passing() {
        assert!(sum_matches(100, &[20, 30, 50]).is_ok());
        assert!(sum_matches(0, &[]).is_ok());
        assert!(sum_matches(42, &[42]).is_ok());
    }

    #[test]
    fn test_sum_matches_failing() {
        let err = sum_matches(100, &[20, 30, 40]).unwrap_err(); // sum is 90
        match err {
            InvariantError::SumMismatchViolation {
                reported_total,
                actual_sum,
                diff,
            } => {
                assert_eq!(reported_total, 100);
                assert_eq!(actual_sum, 90);
                assert_eq!(diff, -10);
            }
            _ => panic!("Unexpected error"),
        }
    }
}
