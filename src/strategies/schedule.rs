//! Strategies for generating vesting schedules `(unlock_timestamp, amount)`.

use proptest::collection::vec;
use proptest::prelude::*;

/// Configuration options for vesting schedule generation.
#[derive(Debug, Clone)]
pub struct VestingScheduleOptions {
    /// Minimum number of entries in the schedule.
    pub min_entries: usize,
    /// Maximum number of entries in the schedule.
    pub max_entries: usize,
    /// Minimum unlock timestamp (inclusive).
    pub min_timestamp: u64,
    /// Maximum unlock timestamp (inclusive).
    pub max_timestamp: u64,
    /// Minimum token amount per entry (must be >= 1).
    pub min_amount: i128,
    /// Maximum token amount per entry.
    pub max_amount: i128,
    /// Whether multiple entries can share the exact same unlock timestamp.
    pub allow_duplicates: bool,
    /// Whether entries must be strictly sorted by unlock timestamp in ascending order.
    pub sorted: bool,
}

impl Default for VestingScheduleOptions {
    fn default() -> Self {
        Self {
            min_entries: 1,
            max_entries: 10,
            min_timestamp: 1_000,
            max_timestamp: 1_000_000,
            min_amount: 1,
            max_amount: 10_000,
            allow_duplicates: false,
            sorted: true,
        }
    }
}

/// Strategy for generating vesting-style schedules as a list of `(unlock_timestamp, amount)`.
///
/// # Panics
/// Panics if:
/// * `min_entries > max_entries`
/// * `min_timestamp > max_timestamp`
/// * `min_amount < 1` or `max_amount < min_amount`
/// * `!allow_duplicates` and `max_timestamp - min_timestamp + 1 < max_entries as u64`
///
/// # Example
/// ```rust
/// use soroban_invariants::strategies::{vesting_schedule, VestingScheduleOptions};
///
/// let options = VestingScheduleOptions {
///     min_entries: 2,
///     max_entries: 5,
///     sorted: true,
///     allow_duplicates: false,
///     ..Default::default()
/// };
/// let strat = vesting_schedule(options);
/// ```
pub fn vesting_schedule(
    options: VestingScheduleOptions,
) -> impl Strategy<Value = Vec<(u64, i128)>> {
    assert!(
        options.min_entries <= options.max_entries,
        "min_entries must be <= max_entries"
    );
    assert!(
        options.min_timestamp <= options.max_timestamp,
        "min_timestamp must be <= max_timestamp"
    );
    assert!(
        options.min_amount >= 1,
        "min_amount must be positive (>= 1)"
    );
    assert!(
        options.max_amount >= options.min_amount,
        "max_amount must be >= min_amount"
    );

    let span = options.max_timestamp.saturating_sub(options.min_timestamp);
    if !options.allow_duplicates {
        assert!(
            span >= (options.max_entries.saturating_sub(1)) as u64,
            "timestamp range too small to generate {} unique timestamps",
            options.max_entries
        );
    }

    let min_entries = options.min_entries;
    let max_entries = options.max_entries;
    let min_amt = options.min_amount;
    let max_amt = options.max_amount;
    let min_ts = options.min_timestamp;
    let max_ts = options.max_timestamp;
    let allow_dups = options.allow_duplicates;
    let sorted = options.sorted;

    (
        min_entries..=max_entries,
        vec(min_amt..=max_amt, min_entries..=max_entries),
    )
        .prop_flat_map(move |(count, amounts)| {
            let actual_amounts: Vec<i128> = amounts.into_iter().take(count).collect();
            let n = actual_amounts.len();

            let ts_strategy = if n == 0 {
                Just(Vec::new()).boxed()
            } else if !allow_dups && sorted {
                // Generate strictly increasing timestamps
                let max_jump = if n > 1 {
                    (span / n as u64).max(1)
                } else {
                    span.max(1)
                };
                vec(1..=max_jump, n)
                    .prop_map(move |jumps| {
                        let mut ts_vec = Vec::with_capacity(n);
                        let mut curr = min_ts;
                        for jump in jumps {
                            curr = curr.saturating_add(jump).min(max_ts);
                            ts_vec.push(curr);
                        }
                        ts_vec
                    })
                    .boxed()
            } else if allow_dups && sorted {
                // Generate non-decreasing timestamps (may have duplicates)
                let max_jump = if n > 1 {
                    (span / n as u64).max(1)
                } else {
                    span.max(1)
                };
                vec(0..=max_jump, n)
                    .prop_map(move |jumps| {
                        let mut ts_vec = Vec::with_capacity(n);
                        let mut curr = min_ts;
                        for jump in jumps {
                            curr = curr.saturating_add(jump).min(max_ts);
                            ts_vec.push(curr);
                        }
                        ts_vec
                    })
                    .boxed()
            } else if !allow_dups && !sorted {
                // Generate strictly increasing timestamps, then reverse or shuffle them if len >= 2
                let max_jump = if n > 1 {
                    (span / n as u64).max(1)
                } else {
                    span.max(1)
                };
                vec(1..=max_jump, n)
                    .prop_map(move |jumps| {
                        let mut ts_vec = Vec::with_capacity(n);
                        let mut curr = min_ts;
                        for jump in jumps {
                            curr = curr.saturating_add(jump).min(max_ts);
                            ts_vec.push(curr);
                        }
                        if ts_vec.len() >= 2 {
                            ts_vec.reverse();
                        }
                        ts_vec
                    })
                    .boxed()
            } else {
                // allow_dups && !sorted
                vec(min_ts..=max_ts, n)
                    .prop_map(move |mut ts_vec| {
                        if ts_vec.len() >= 2 && ts_vec.windows(2).all(|w| w[0] <= w[1]) {
                            // ensure not accidentally sorted
                            ts_vec.reverse();
                        }
                        ts_vec
                    })
                    .boxed()
            };

            ts_strategy.prop_map(move |timestamps| {
                timestamps
                    .into_iter()
                    .zip(actual_amounts.clone())
                    .collect::<Vec<(u64, i128)>>()
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    #[test]
    fn test_sorted_no_duplicates() {
        let mut runner = TestRunner::default();
        let options = VestingScheduleOptions {
            min_entries: 3,
            max_entries: 6,
            min_timestamp: 100,
            max_timestamp: 1_000,
            sorted: true,
            allow_duplicates: false,
            ..Default::default()
        };
        let strat = vesting_schedule(options);

        for _ in 0..50 {
            let sched = strat.new_tree(&mut runner).unwrap().current();
            assert!(sched.len() >= 3 && sched.len() <= 6);
            for w in sched.windows(2) {
                assert!(
                    w[0].0 < w[1].0,
                    "Expected strictly increasing: {} >= {}",
                    w[0].0,
                    w[1].0
                );
            }
        }
    }

    #[test]
    fn test_sorted_with_duplicates() {
        let mut runner = TestRunner::default();
        let options = VestingScheduleOptions {
            min_entries: 3,
            max_entries: 5,
            sorted: true,
            allow_duplicates: true,
            ..Default::default()
        };
        let strat = vesting_schedule(options);

        for _ in 0..50 {
            let sched = strat.new_tree(&mut runner).unwrap().current();
            for w in sched.windows(2) {
                assert!(
                    w[0].0 <= w[1].0,
                    "Expected non-decreasing: {} > {}",
                    w[0].0,
                    w[1].0
                );
            }
        }
    }

    #[test]
    fn test_unsorted_behavior() {
        let mut runner = TestRunner::default();
        let options = VestingScheduleOptions {
            min_entries: 3,
            max_entries: 5,
            sorted: false,
            allow_duplicates: false,
            ..Default::default()
        };
        let strat = vesting_schedule(options);

        for _ in 0..50 {
            let sched = strat.new_tree(&mut runner).unwrap().current();
            if sched.len() >= 2 {
                // Because we reversed strictly increasing elements, it must not be sorted ascending
                assert!(
                    sched.windows(2).any(|w| w[0].0 > w[1].0),
                    "Expected unsorted schedule"
                );
            }
        }
    }
}
