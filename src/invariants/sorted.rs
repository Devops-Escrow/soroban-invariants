//! Invariant helper to verify items in a sequence remain sorted by a key.

use crate::invariants::error::InvariantError;

/// Asserts that elements in a slice are sorted in non-decreasing order according to `key_fn`.
///
/// # Arguments
/// * `items` - The slice of elements to check.
/// * `key_fn` - Projection function returning the sortable key.
///
/// # Errors
/// Returns [`InvariantError::UnsortedKeyViolation`] with the index of the first out-of-order element.
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::sorted_by_key;
///
/// let schedule = vec![(100u64, 50i128), (200u64, 100i128), (300u64, 25i128)];
/// assert!(sorted_by_key(&schedule, |entry| entry.0).is_ok());
/// ```
pub fn sorted_by_key<T, K, F>(items: &[T], key_fn: F) -> Result<(), InvariantError>
where
    K: Ord + core::fmt::Debug,
    F: Fn(&T) -> K,
{
    for i in 1..items.len() {
        let prev_key = key_fn(&items[i - 1]);
        let curr_key = key_fn(&items[i]);
        if curr_key < prev_key {
            return Err(InvariantError::UnsortedKeyViolation {
                index: i,
                message: format!(
                    "element at index {} (key: {:?}) is less than element at index {} (key: {:?})",
                    i,
                    curr_key,
                    i - 1,
                    prev_key
                ),
            });
        }
    }
    Ok(())
}

/// Asserts that elements in a slice are strictly sorted in increasing order (no duplicates).
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::strictly_sorted_by_key;
///
/// let timestamps = [100u64, 200, 300];
/// assert!(strictly_sorted_by_key(&timestamps, |&ts| ts).is_ok());
/// ```
pub fn strictly_sorted_by_key<T, K, F>(items: &[T], key_fn: F) -> Result<(), InvariantError>
where
    K: Ord + core::fmt::Debug,
    F: Fn(&T) -> K,
{
    for i in 1..items.len() {
        let prev_key = key_fn(&items[i - 1]);
        let curr_key = key_fn(&items[i]);
        if curr_key <= prev_key {
            return Err(InvariantError::UnsortedKeyViolation {
                index: i,
                message: format!(
                    "element at index {} (key: {:?}) is not strictly greater than element at index {} (key: {:?})",
                    i, curr_key, i - 1, prev_key
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_by_key_passing() {
        let items = [(10, "a"), (10, "b"), (20, "c"), (30, "d")];
        assert!(sorted_by_key(&items, |x| x.0).is_ok());

        let empty: [(u32, &str); 0] = [];
        assert!(sorted_by_key(&empty, |x| x.0).is_ok());

        let single = [(5, "x")];
        assert!(sorted_by_key(&single, |x| x.0).is_ok());
    }

    #[test]
    fn test_sorted_by_key_failing() {
        let items = [(10, "a"), (25, "b"), (20, "c")];
        let err = sorted_by_key(&items, |x| x.0).unwrap_err();
        match err {
            InvariantError::UnsortedKeyViolation { index, message } => {
                assert_eq!(index, 2);
                assert!(message.contains("is less than"));
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_strictly_sorted_by_key() {
        let unique = [1, 2, 3];
        assert!(strictly_sorted_by_key(&unique, |&x| x).is_ok());

        let duplicates = [1, 2, 2, 3];
        assert!(strictly_sorted_by_key(&duplicates, |&x| x).is_err());
    }
}
