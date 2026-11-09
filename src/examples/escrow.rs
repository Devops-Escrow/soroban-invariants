//! Escrow contract example with model-based invariant verification.
//!
//! Demonstrates a secure, correct escrow contract where:
//! - Depositor deposits funds into escrow.
//! - Arbiter can release funds to beneficiary or refund to depositor.
//! - All invariants (token conservation, non-negative balances, requires_auth, sum_matches) hold.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientEscrowBalance = 4,
    Unauthorized = 5,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    Depositor,
    Beneficiary,
    Arbiter,
    Balance,
    TotalDeposited,
    TotalReleased,
    TotalRefunded,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow with participants.
    pub fn init(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        arbiter: Address,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&DataKey::Depositor) {
            return Err(EscrowError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&DataKey::Depositor, &depositor);
        env.storage()
            .instance()
            .set(&DataKey::Beneficiary, &beneficiary);
        env.storage().instance().set(&DataKey::Arbiter, &arbiter);
        env.storage().instance().set(&DataKey::Balance, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposited, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalReleased, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalRefunded, &0i128);

        Ok(())
    }

    /// Deposit funds into the escrow.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), EscrowError> {
        from.require_auth();

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let depositor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Depositor)
            .ok_or(EscrowError::NotInitialized)?;

        if from != depositor {
            return Err(EscrowError::Unauthorized);
        }

        let current_bal: i128 = env.storage().instance().get(&DataKey::Balance).unwrap_or(0);
        let total_dep: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDeposited)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(current_bal + amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposited, &(total_dep + amount));

        Ok(())
    }

    /// Release escrowed funds to beneficiary (requires arbiter authorization).
    pub fn release(env: Env, caller: Address, amount: i128) -> Result<(), EscrowError> {
        caller.require_auth();

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let arbiter: Address = env
            .storage()
            .instance()
            .get(&DataKey::Arbiter)
            .ok_or(EscrowError::NotInitialized)?;

        if caller != arbiter {
            return Err(EscrowError::Unauthorized);
        }

        let current_bal: i128 = env.storage().instance().get(&DataKey::Balance).unwrap_or(0);
        if current_bal < amount {
            return Err(EscrowError::InsufficientEscrowBalance);
        }

        let total_rel: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalReleased)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(current_bal - amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalReleased, &(total_rel + amount));

        Ok(())
    }

    /// Refund escrowed funds back to depositor (requires arbiter authorization).
    pub fn refund(env: Env, caller: Address, amount: i128) -> Result<(), EscrowError> {
        caller.require_auth();

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let arbiter: Address = env
            .storage()
            .instance()
            .get(&DataKey::Arbiter)
            .ok_or(EscrowError::NotInitialized)?;

        if caller != arbiter {
            return Err(EscrowError::Unauthorized);
        }

        let current_bal: i128 = env.storage().instance().get(&DataKey::Balance).unwrap_or(0);
        if current_bal < amount {
            return Err(EscrowError::InsufficientEscrowBalance);
        }

        let total_ref: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRefunded)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(current_bal - amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalRefunded, &(total_ref + amount));

        Ok(())
    }

    /// Query current escrow balance.
    pub fn get_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Balance).unwrap_or(0)
    }

    /// Query total deposited.
    pub fn get_total_deposited(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalDeposited)
            .unwrap_or(0)
    }

    /// Query total released.
    pub fn get_total_released(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalReleased)
            .unwrap_or(0)
    }

    /// Query total refunded.
    pub fn get_total_refunded(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalRefunded)
            .unwrap_or(0)
    }
}
