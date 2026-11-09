//! Token conservation invariant helper.

use crate::invariants::error::InvariantError;

/// Asserts that the sum of all tracked balances (including contract reserves) matches
/// the expected total, accounting for minted or burned amounts.
///
/// For closed-loop transfers where no tokens are minted or burned, pass `minted_or_burned_delta = 0`.
///
/// # Arguments
/// * `initial_total` - The total tokens in existence prior to the step or at test start.
/// * `current_balances` - Slice of all current user balances and contract balance.
/// * `minted_or_burned_delta` - Net newly minted (+ve) or burned (-ve) tokens.
///
/// # Errors
/// Returns [`InvariantError::TokenConservationViolation`] if the actual sum does not equal
/// `initial_total + minted_or_burned_delta`.
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::token_conservation;
///
/// let initial_supply = 10_000i128;
/// let alice = 4_000i128;
/// let bob = 3_500i128;
/// let pool = 2_500i128;
///
/// // Conservation holds: 4000 + 3500 + 2500 == 10000 + 0
/// assert!(token_conservation(initial_supply, &[alice, bob, pool], 0).is_ok());
/// ```
pub fn token_conservation(
    initial_total: i128,
    current_balances: &[i128],
    minted_or_burned_delta: i128,
) -> Result<(), InvariantError> {
    let actual_total: i128 = current_balances.iter().sum();
    let expected_total = initial_total.saturating_add(minted_or_burned_delta);

    if actual_total != expected_total {
        Err(InvariantError::TokenConservationViolation {
            initial_total,
            actual_total,
            delta: minted_or_burned_delta,
            expected_total,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_conservation_passing() {
        let balances = [100, 200, 700];
        assert!(token_conservation(1000, &balances, 0).is_ok());

        // With minting 50 tokens
        let balances_after_mint = [100, 250, 700];
        assert!(token_conservation(1000, &balances_after_mint, 50).is_ok());

        // With burning 100 tokens
        let balances_after_burn = [100, 200, 600];
        assert!(token_conservation(1000, &balances_after_burn, -100).is_ok());
    }

    #[test]
    fn test_token_conservation_failing() {
        let balances = [100, 200, 699]; // 999 instead of 1000
        let err = token_conservation(1000, &balances, 0).unwrap_err();
        match err {
            InvariantError::TokenConservationViolation {
                initial_total,
                actual_total,
                delta,
                expected_total,
            } => {
                assert_eq!(initial_total, 1000);
                assert_eq!(actual_total, 999);
                assert_eq!(delta, 0);
                assert_eq!(expected_total, 1000);
            }
            _ => panic!("Unexpected error variant"),
        }
    }
}
