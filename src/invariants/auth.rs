//! Authorization invariant helpers for Soroban contracts.

use crate::invariants::error::InvariantError;
use soroban_sdk::{Address, Env};

/// Verifies that an expected address was recorded in the environment's authorizations (`env.auths()`).
///
/// In Soroban contract tests, invoking contract methods while `env.mock_all_auths()` is active
/// records all accounts that executed `caller.require_auth()` into `env.auths()`.
///
/// # Arguments
/// * `env` - The Soroban environment.
/// * `expected_authorizer` - The address required to have authorized the invocation.
///
/// # Errors
/// Returns [`InvariantError::AuthViolation`] if `expected_authorizer` was not found in `env.auths()`.
///
/// # Example
/// ```rust
/// use soroban_sdk::{Address, Env, testutils::Address as _};
/// use soroban_invariants::examples::escrow::{EscrowContract, EscrowContractClient};
/// use soroban_invariants::invariants::assert_auth_recorded;
///
/// let env = Env::default();
/// let depositor = Address::generate(&env);
/// let beneficiary = Address::generate(&env);
/// let arbiter = Address::generate(&env);
/// let contract_id = env.register(EscrowContract, ());
/// let client = EscrowContractClient::new(&env, &contract_id);
/// client.init(&depositor, &beneficiary, &arbiter);
///
/// env.mock_all_auths();
/// client.deposit(&depositor, &500);
/// assert!(assert_auth_recorded(&env, &depositor).is_ok());
/// ```
pub fn assert_auth_recorded(
    env: &Env,
    expected_authorizer: &Address,
) -> Result<(), InvariantError> {
    let auths = env.auths();
    let found = auths
        .iter()
        .any(|(addr, _invocation)| addr == expected_authorizer);

    if !found {
        Err(InvariantError::AuthViolation(format!(
            "expected authorization from {:?}, but recorded authorizations were: {:?}",
            expected_authorizer, auths
        )))
    } else {
        Ok(())
    }
}

/// Verifies that an unexpected address was NOT recorded in `env.auths()`.
///
/// # Errors
/// Returns [`InvariantError::AuthViolation`] if `unexpected_authorizer` was found in `env.auths()`.
///
/// # Example
/// ```rust
/// use soroban_sdk::{Address, Env, testutils::Address as _};
/// use soroban_invariants::invariants::assert_auth_not_recorded;
///
/// let env = Env::default();
/// let attacker = Address::generate(&env);
///
/// assert!(assert_auth_not_recorded(&env, &attacker).is_ok());
/// ```
pub fn assert_auth_not_recorded(
    env: &Env,
    unexpected_authorizer: &Address,
) -> Result<(), InvariantError> {
    let auths = env.auths();
    let found = auths.iter().any(|(addr, _)| addr == unexpected_authorizer);

    if found {
        Err(InvariantError::AuthViolation(format!(
            "unexpected authorization recorded from {:?}",
            unexpected_authorizer
        )))
    } else {
        Ok(())
    }
}

/// Comprehensive helper asserting that a contract operation requires authorization from `expected_authorizer`.
///
/// Executes the invocation under `env.mock_all_auths()` and asserts that `expected_authorizer`
/// is present in `env.auths()`.
///
/// # Example
/// ```rust
/// use soroban_sdk::{Address, Env, testutils::Address as _};
/// use soroban_invariants::examples::escrow::{EscrowContract, EscrowContractClient};
/// use soroban_invariants::invariants::assert_requires_auth;
///
/// let env = Env::default();
/// let depositor = Address::generate(&env);
/// let beneficiary = Address::generate(&env);
/// let arbiter = Address::generate(&env);
/// let contract_id = env.register(EscrowContract, ());
/// let client = EscrowContractClient::new(&env, &contract_id);
/// client.init(&depositor, &beneficiary, &arbiter);
///
/// let res = assert_requires_auth(&env, &depositor, || {
///     client.deposit(&depositor, &100);
/// });
/// assert!(res.is_ok());
/// ```
pub fn assert_requires_auth<F, R>(
    env: &Env,
    expected_authorizer: &Address,
    invocation: F,
) -> Result<(), InvariantError>
where
    F: FnOnce() -> R,
{
    env.mock_all_auths();
    let _ = invocation();
    assert_auth_recorded(env, expected_authorizer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Env};

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {
        pub fn auth_call(caller: Address) {
            caller.require_auth();
        }

        pub fn public_call() {}
    }

    #[test]
    fn test_auth_recorded_passing() {
        let env = Env::default();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let contract_id = env.register(DummyContract, ());
        let client = DummyContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        client.auth_call(&alice);

        assert!(assert_auth_recorded(&env, &alice).is_ok());
        assert!(assert_auth_not_recorded(&env, &bob).is_ok());
    }

    #[test]
    fn test_auth_recorded_failing() {
        let env = Env::default();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let contract_id = env.register(DummyContract, ());
        let client = DummyContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        client.auth_call(&alice);

        // Bob was not authorized
        let err = assert_auth_recorded(&env, &bob).unwrap_err();
        match err {
            InvariantError::AuthViolation(msg) => {
                assert!(msg.contains("expected authorization from"));
            }
            _ => panic!("Unexpected error variant"),
        }

        // Alice was authorized, so assert_auth_not_recorded(&env, &alice) should fail
        let err2 = assert_auth_not_recorded(&env, &alice).unwrap_err();
        match err2 {
            InvariantError::AuthViolation(msg) => {
                assert!(msg.contains("unexpected authorization"));
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_requires_auth_helper() {
        let env = Env::default();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let contract_id = env.register(DummyContract, ());
        let client = DummyContractClient::new(&env, &contract_id);

        assert!(assert_requires_auth(&env, &alice, || {
            client.auth_call(&alice);
        })
        .is_ok());

        // Calling public method should fail assert_requires_auth because no auth was requested
        assert!(assert_requires_auth(&env, &bob, || {
            client.public_call();
        })
        .is_err());
    }
}
