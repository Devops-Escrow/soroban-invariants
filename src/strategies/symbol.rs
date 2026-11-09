//! Strategies for generating `Symbol` identifiers from a pool.

use proptest::prelude::*;
use soroban_sdk::{Env, Symbol};

/// Generate a pool of Soroban `Symbol` instances from a slice of string names.
///
/// # Panics
/// Panics if `names` is empty.
///
/// # Example
/// ```rust
/// use soroban_sdk::Env;
/// use soroban_invariants::strategies::generate_symbol_pool;
///
/// let env = Env::default();
/// let pool = generate_symbol_pool(&env, &["USDC", "XLM", "ETH"]);
/// assert_eq!(pool.len(), 3);
/// ```
pub fn generate_symbol_pool(env: &Env, names: &[&str]) -> Vec<Symbol> {
    assert!(!names.is_empty(), "symbol names slice must not be empty");
    names.iter().map(|s| Symbol::new(env, s)).collect()
}

/// Strategy for selecting a `Symbol` from a fixed pool.
///
/// # Panics
/// Panics if `pool` is empty.
///
/// # Example
/// ```rust
/// use soroban_sdk::Env;
/// use soroban_invariants::strategies::{generate_symbol_pool, select_symbol};
///
/// let env = Env::default();
/// let pool = generate_symbol_pool(&env, &["USDC", "XLM"]);
/// let strat = select_symbol(pool);
/// ```
pub fn select_symbol(pool: Vec<Symbol>) -> impl Strategy<Value = Symbol> + Clone {
    assert!(!pool.is_empty(), "symbol pool must not be empty");
    (0..pool.len()).prop_map(move |idx| pool[idx].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    #[test]
    fn test_select_symbol() {
        let env = Env::default();
        let pool = generate_symbol_pool(&env, &["TOK_A", "TOK_B", "TOK_C"]);
        assert_eq!(pool.len(), 3);
        let strat = select_symbol(pool.clone());
        let mut runner = TestRunner::default();

        for _ in 0..50 {
            let sym = strat.new_tree(&mut runner).unwrap().current();
            assert!(pool.contains(&sym));
        }
    }

    #[test]
    #[should_panic(expected = "symbol names slice must not be empty")]
    fn test_empty_names_panics() {
        let env = Env::default();
        let _ = generate_symbol_pool(&env, &[]);
    }
}
