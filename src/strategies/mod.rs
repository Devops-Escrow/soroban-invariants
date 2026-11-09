//! Property-based testing strategies tailored for Soroban smart contracts.
//!
//! Includes generators for bounded token amounts, monotonic ledger timestamps,
//! small pools of addresses and symbols, and vesting schedules.

pub mod address;
pub mod amount;
pub mod schedule;
pub mod symbol;
pub mod timestamp;

pub use address::{address_index, generate_address_pool, select_address};
pub use amount::{bounded_amount, positive_amount};
pub use schedule::{vesting_schedule, VestingScheduleOptions};
pub use symbol::{generate_symbol_pool, select_symbol};
pub use timestamp::{time_step, timestamp_sequence};
