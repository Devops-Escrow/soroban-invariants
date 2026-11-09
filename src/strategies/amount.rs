//! Strategies for generating bounded integer amounts for contract operations.

use proptest::prelude::*;

/// Generate positive `i128` amounts within a specified range `[min, max]`.
///
/// # Arguments
/// * `min` - Lower bound (inclusive, must be >= 1).
/// * `max` - Upper bound (inclusive, must be >= min).
///
/// # Panics
/// Panics if `min < 1` or `max < min`.
///
/// # Example
/// ```rust
/// use proptest::strategy::Strategy;
/// use soroban_invariants::strategies::bounded_amount;
///
/// let strat = bounded_amount(10, 100);
/// ```
pub fn bounded_amount(min: i128, max: i128) -> impl Strategy<Value = i128> {
    assert!(
        min >= 1,
        "minimum amount must be positive (>= 1), got {}",
        min
    );
    assert!(
        max >= min,
        "maximum amount ({}) must be >= minimum ({})",
        max,
        min
    );
    min..=max
}

/// Generate positive `i128` amounts between 1 and `max_amount`.
///
/// # Panics
/// Panics if `max_amount < 1`.
///
/// # Example
/// ```rust
/// use soroban_invariants::strategies::positive_amount;
///
/// let strat = positive_amount(1_000_000);
/// ```
pub fn positive_amount(max_amount: i128) -> impl Strategy<Value = i128> {
    bounded_amount(1, max_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    #[test]
    fn test_bounded_amount_within_range() {
        let mut runner = TestRunner::default();
        let strat = bounded_amount(50, 150);
        for _ in 0..100 {
            let val = strat.new_tree(&mut runner).unwrap().current();
            assert!((50..=150).contains(&val), "Value {} out of bounds", val);
        }
    }

    #[test]
    fn test_positive_amount() {
        let mut runner = TestRunner::default();
        let strat = positive_amount(1_000);
        for _ in 0..100 {
            let val = strat.new_tree(&mut runner).unwrap().current();
            assert!((1..=1_000).contains(&val), "Value {} out of bounds", val);
        }
    }

    #[test]
    #[should_panic(expected = "minimum amount must be positive")]
    fn test_bounded_amount_zero_min_panics() {
        let _ = bounded_amount(0, 100);
    }

    #[test]
    #[should_panic(expected = "maximum amount (5) must be >= minimum (10)")]
    fn test_bounded_amount_inverted_bounds_panics() {
        let _ = bounded_amount(10, 5);
    }
}
