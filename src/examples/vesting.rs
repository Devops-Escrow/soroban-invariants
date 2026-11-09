//! Correct, secure vesting contract.
//!
//! Fixed counterpart to `crate::examples::buggy_vesting`.
//!
//! **Fixes Applied:**
//! - [`VestingContract::append_milestone`] strictly enforces that new unlock timestamps
//!   are strictly greater than the preceding milestone timestamp (`ts > last_ts`),
//!   and cumulative amounts are non-decreasing.
//! - Invalid or out-of-order appends are rejected with [`VestingError::ScheduleNotMonotonic`].
//! - The schedule is guaranteed to remain strictly sorted by timestamp, preventing
//!   premature or erroneous over-releases of unvested funds.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, vec, Address, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VestingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    NothingToClaim = 4,
    Unauthorized = 5,
    ScheduleNotMonotonic = 6,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub unlock_timestamp: u64,
    pub cumulative_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    Beneficiary,
    TotalClaimed,
    Schedule,
}

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Initialize vesting contract with a beneficiary.
    pub fn init(env: Env, beneficiary: Address) -> Result<(), VestingError> {
        if env.storage().instance().has(&DataKey::Beneficiary) {
            return Err(VestingError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&DataKey::Beneficiary, &beneficiary);
        env.storage().instance().set(&DataKey::TotalClaimed, &0i128);
        let empty_schedule: Vec<Milestone> = vec![&env];
        env.storage()
            .instance()
            .set(&DataKey::Schedule, &empty_schedule);

        Ok(())
    }

    /// Append a milestone to the vesting schedule with strict monotonicity validation.
    pub fn append_milestone(
        env: Env,
        unlock_timestamp: u64,
        cumulative_amount: i128,
    ) -> Result<(), VestingError> {
        if cumulative_amount <= 0 {
            return Err(VestingError::InvalidAmount);
        }

        let mut schedule: Vec<Milestone> = env
            .storage()
            .instance()
            .get(&DataKey::Schedule)
            .ok_or(VestingError::NotInitialized)?;

        // Enforce strict monotonicity: each new milestone must be strictly in the future
        if !schedule.is_empty() {
            let last_m = schedule.get(schedule.len() - 1).unwrap();
            if unlock_timestamp <= last_m.unlock_timestamp {
                return Err(VestingError::ScheduleNotMonotonic);
            }
            if cumulative_amount < last_m.cumulative_amount {
                return Err(VestingError::InvalidAmount);
            }
        }

        schedule.push_back(Milestone {
            unlock_timestamp,
            cumulative_amount,
        });

        env.storage().instance().set(&DataKey::Schedule, &schedule);
        Ok(())
    }

    /// Claim vested tokens according to current ledger timestamp.
    pub fn claim(env: Env, caller: Address) -> Result<i128, VestingError> {
        caller.require_auth();

        let beneficiary: Address = env
            .storage()
            .instance()
            .get(&DataKey::Beneficiary)
            .ok_or(VestingError::NotInitialized)?;

        if caller != beneficiary {
            return Err(VestingError::Unauthorized);
        }

        let current_time = env.ledger().timestamp();
        let schedule: Vec<Milestone> = env
            .storage()
            .instance()
            .get(&DataKey::Schedule)
            .unwrap_or(vec![&env]);

        let mut vested_so_far = 0i128;

        for i in 0..schedule.len() {
            let m = schedule.get(i).unwrap();
            if current_time >= m.unlock_timestamp {
                vested_so_far = m.cumulative_amount;
            }
        }

        let total_claimed: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalClaimed)
            .unwrap_or(0);
        let claimable = vested_so_far.saturating_sub(total_claimed);

        if claimable <= 0 {
            return Err(VestingError::NothingToClaim);
        }

        env.storage()
            .instance()
            .set(&DataKey::TotalClaimed, &(total_claimed + claimable));
        Ok(claimable)
    }

    /// Retrieve the schedule.
    pub fn get_schedule(env: Env) -> Vec<Milestone> {
        env.storage()
            .instance()
            .get(&DataKey::Schedule)
            .unwrap_or(vec![&env])
    }

    /// Retrieve total claimed so far.
    pub fn get_total_claimed(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalClaimed)
            .unwrap_or(0)
    }
}
