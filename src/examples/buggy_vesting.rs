//! Deliberately buggy vesting contract.
//!
//! # INTENTIONAL BUG DOCUMENTATION
//! This contract contains an **intentional vulnerability** designed to demonstrate
//! how `soroban-invariants` detects subtle ordering and invariant violations.
//!
//! **Vulnerability Mechanism:**
//! In [`BuggyVestingContract::append_milestone`], new unlock milestones are pushed to the end
//! of the schedule vector WITHOUT verifying that the timestamp is greater than previous timestamps,
//! and WITHOUT maintaining sorted order.
//!
//! When [`BuggyVestingContract::claim`] is called, it iterates through the schedule milestones
//! assuming they are sorted in non-decreasing order of unlock timestamps. If a milestone with an
//! earlier unlock timestamp but larger amount was appended later in the array, the claim loop
//! evaluates it last, causing premature over-release of unvested funds.
//!
//! The secure, fixed implementation is available in `crate::examples::vesting`.

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
pub struct BuggyVestingContract;

#[contractimpl]
impl BuggyVestingContract {
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

    /// Append a milestone to the vesting schedule.
    ///
    /// # INTENTIONAL BUG
    /// This function does NOT enforce that `unlock_timestamp` is greater than the previous milestone's
    /// timestamp. If an out-of-order milestone is appended, the schedule becomes unsorted.
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

        // INTENTIONAL BUG: Unconditionally appends without checking timestamp ordering or sorting!
        schedule.push_back(Milestone {
            unlock_timestamp,
            cumulative_amount,
        });

        env.storage().instance().set(&DataKey::Schedule, &schedule);
        Ok(())
    }

    /// Claim vested tokens.
    ///
    /// # INTENTIONAL BUG
    /// Assumes the schedule is sorted by `unlock_timestamp`. Evaluates milestones in order of storage;
    /// if an unsorted entry with a small timestamp appears later, it overwrites `vested_so_far`
    /// and prematurely over-releases tokens.
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

        // Loop assumes milestones are sorted in ascending chronological order:
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

    /// Retrieve the schedule as a Soroban Vec.
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
