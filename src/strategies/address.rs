//! Strategies for selecting `Address` accounts from a small pool.

use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Generate a fixed pool of distinct `Address` values for scenario participants.
///
/// Small pools (e.g. 2 to 5 addresses) ensure that generated operations interact with
/// shared contract state rather than operating in isolation.
///
/// # Arguments
/// * `env` - The Soroban environment.
/// * `count` - Number of distinct addresses to generate (must be >= 1).
///
/// # Panics
/// Panics if `count == 0`.
///
/// # Example
/// ```rust
/// use soroban_sdk::Env;
/// use soroban_invariants::strategies::generate_address_pool;
///
/// let env = Env::default();
/// let pool = generate_address_pool(&env, 3);
/// assert_eq!(pool.len(), 3);
/// ```
pub fn generate_address_pool(env: &Env, count: usize) -> Vec<Address> {
    assert!(count >= 1, "address pool count must be >= 1");
    (0..count).map(|_| Address::generate(env)).collect()
}

/// Strategy for randomly selecting an `Address` from a predefined pool.
///
/// # Panics
/// Panics if `pool` is empty.
///
/// # Example
/// ```rust
/// use soroban_sdk::Env;
/// use soroban_invariants::strategies::{generate_address_pool, select_address};
///
/// let env = Env::default();
/// let pool = generate_address_pool(&env, 3);
/// let strat = select_address(pool);
/// ```
pub fn select_address(pool: Vec<Address>) -> impl Strategy<Value = Address> + Clone {
    assert!(!pool.is_empty(), "address pool must not be empty");
    (0..pool.len()).prop_map(move |idx| pool[idx].clone())
}

/// Strategy for selecting an index within an address pool.
///
/// Useful when actions store lightweight indices instead of full `Address` instances.
///
/// # Panics
/// Panics if `pool_size == 0`.
///
/// # Example
/// ```rust
/// use soroban_invariants::strategies::address_index;
///
/// let strat = address_index(4);
/// ```
pub fn address_index(pool_size: usize) -> impl Strategy<Value = usize> {
    assert!(pool_size >= 1, "pool_size must be >= 1");
    0..pool_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    #[test]
    fn test_address_pool_distinct() {
        let env = Env::default();
        let pool = generate_address_pool(&env, 4);
        assert_eq!(pool.len(), 4);
        assert_ne!(pool[0], pool[1]);
        assert_ne!(pool[1], pool[2]);
        assert_ne!(pool[2], pool[3]);
    }

    #[test]
    fn test_select_address() {
        let env = Env::default();
        let pool = generate_address_pool(&env, 3);
        let strat = select_address(pool.clone());
        let mut runner = TestRunner::default();

        for _ in 0..50 {
            let addr = strat.new_tree(&mut runner).unwrap().current();
            assert!(pool.contains(&addr));
        }
    }

    #[test]
    #[should_panic(expected = "address pool count must be >= 1")]
    fn test_zero_count_panics() {
        let env = Env::default();
        let _ = generate_address_pool(&env, 0);
    }
}
