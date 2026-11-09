//! Reference contracts and example scenario implementations.
//!
//! Includes:
//! - [`escrow`]: A fully invariant-verified escrow contract.
//! - [`buggy_vesting`]: A deliberately buggy vesting contract demonstrating failure detection and shrinking.
//! - [`vesting`]: The fixed, verified counterpart to `buggy_vesting`.

pub mod buggy_vesting;
pub mod escrow;
pub mod vesting;
