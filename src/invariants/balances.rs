//! Non-negative balance invariant helper.

use crate::invariants::error::InvariantError;
use soroban_sdk::Address;

/// Asserts that all numeric balances in a slice are non-negative (`>= 0`).
///
/// # Errors
/// Returns [`InvariantError::NegativeBalanceViolation`] at the first negative entry found.
///
/// # Example
/// ```rust
/// use soroban_invariants::invariants::non_negative_balances;
///
/// let balances = [0i128, 50i128, 1_000_000i128];
/// assert!(non_negative_balances(&balances).is_ok());
/// ```
pub fn non_negative_balances(balances: &[i128]) -> Result<(), InvariantError> {
    for (index, &bal) in balances.iter().enumerate() {
        if bal < 0 {
            return Err(InvariantError::NegativeBalanceViolation {
                address: None,
                balance: bal,
                index,
            });
        }
    }
    Ok(())
}

/// Asserts that all balances associated with specific `Address` accounts are non-negative (`>= 0`).
///
/// # Errors
/// Returns [`InvariantError::NegativeBalanceViolation`] with the account address if a balance is `< 0`.
///
/// # Example
/// ```rust
/// use soroban_sdk::{Address, Env, testutils::Address as _};
/// use soroban_invariants::invariants::named_non_negative_balances;
///
/// let env = Env::default();
/// let user = Address::generate(&env);
/// let account_balances = [(&user, 500i128)];
/// assert!(named_non_negative_balances(&account_balances).is_ok());
/// ```
pub fn named_non_negative_balances(balances: &[(&Address, i128)]) -> Result<(), InvariantError> {
    for (index, (addr, bal)) in balances.iter().enumerate() {
        if *bal < 0 {
            return Err(InvariantError::NegativeBalanceViolation {
                address: Some(format!("{:?}", addr)),
                balance: *bal,
                index,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_non_negative_balances_passing() {
        assert!(non_negative_balances(&[0, 10, 500]).is_ok());
        assert!(non_negative_balances(&[]).is_ok());
    }

    #[test]
    fn test_non_negative_balances_failing() {
        let err = non_negative_balances(&[10, -5, 20]).unwrap_err();
        match err {
            InvariantError::NegativeBalanceViolation {
                index,
                balance,
                address,
            } => {
                assert_eq!(index, 1);
                assert_eq!(balance, -5);
                assert!(address.is_none());
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_named_non_negative_balances_failing() {
        let env = Env::default();
        let addr1 = Address::generate(&env);
        let addr2 = Address::generate(&env);
        let entries = [(&addr1, 100), (&addr2, -1)];
        let err = named_non_negative_balances(&entries).unwrap_err();
        match err {
            InvariantError::NegativeBalanceViolation {
                index,
                balance,
                address,
            } => {
                assert_eq!(index, 1);
                assert_eq!(balance, -1);
                assert!(address.is_some());
            }
            _ => panic!("Unexpected error"),
        }
    }
}
