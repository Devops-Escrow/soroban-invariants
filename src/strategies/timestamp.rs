//! Strategies for generating monotonic sequences of ledger timestamps.

use proptest::collection::vec;
use proptest::prelude::*;

/// Generate a monotonically non-decreasing sequence of `u64` timestamps.
///
/// Each subsequent timestamp is calculated by adding a random jump between `0` and `max_jump_secs`
/// to the previous timestamp.
///
/// # Arguments
/// * `start_timestamp` - The base starting timestamp for the sequence.
/// * `min_steps` - Minimum number of timestamp elements in the sequence.
/// * `max_steps` - Maximum number of timestamp elements in the sequence.
/// * `max_jump_secs` - Maximum delta in seconds added at each step.
///
/// # Panics
/// Panics if `min_steps > max_steps`.
///
/// # Example
/// ```rust
/// use soroban_invariants::strategies::timestamp_sequence;
///
/// let strat = timestamp_sequence(1_700_000_000, 1, 10, 3600);
/// ```
pub fn timestamp_sequence(
    start_timestamp: u64,
    min_steps: usize,
    max_steps: usize,
    max_jump_secs: u64,
) -> impl Strategy<Value = Vec<u64>> {
    assert!(
        min_steps <= max_steps,
        "min_steps ({}) must be <= max_steps ({})",
        min_steps,
        max_steps
    );

    vec(0..=max_jump_secs, min_steps..=max_steps).prop_map(move |jumps| {
        let mut result = Vec::with_capacity(jumps.len());
        let mut current = start_timestamp;
        for jump in jumps {
            current = current.saturating_add(jump);
            result.push(current);
        }
        result
    })
}

/// Strategy for advancing time forward from a current timestamp by `[0, max_jump_secs]`.
///
/// # Example
/// ```rust
/// use soroban_invariants::strategies::time_step;
///
/// let strat = time_step(1_000, 60);
/// ```
pub fn time_step(current_timestamp: u64, max_jump_secs: u64) -> impl Strategy<Value = u64> {
    (0..=max_jump_secs).prop_map(move |jump| current_timestamp.saturating_add(jump))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    #[test]
    fn test_monotonic_timestamps() {
        let mut runner = TestRunner::default();
        let strat = timestamp_sequence(1000, 5, 20, 60);
        for _ in 0..100 {
            let seq = strat.new_tree(&mut runner).unwrap().current();
            assert!(seq.len() >= 5 && seq.len() <= 20);
            let mut prev = 1000;
            for &ts in &seq {
                assert!(ts >= prev, "Timestamp decreased: {} < {}", ts, prev);
                assert!(
                    ts - prev <= 60,
                    "Jump exceeded max_jump_secs: {} > 60",
                    ts - prev
                );
                prev = ts;
            }
        }
    }

    #[test]
    fn test_time_step() {
        let mut runner = TestRunner::default();
        let strat = time_step(500, 100);
        for _ in 0..50 {
            let next_ts = strat.new_tree(&mut runner).unwrap().current();
            assert!((500..=600).contains(&next_ts));
        }
    }

    #[test]
    #[should_panic(expected = "min_steps (10) must be <= max_steps (5)")]
    fn test_timestamp_invalid_steps_panics() {
        let _ = timestamp_sequence(0, 10, 5, 10);
    }
}
