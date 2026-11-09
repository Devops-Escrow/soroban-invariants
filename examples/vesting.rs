//! Standalone Fixed Vesting example runner.
//!
//! Run with: `cargo run --example vesting`

use soroban_invariants::examples::vesting::{VestingContract, VestingContractClient};
use soroban_invariants::invariants::sorted_by_key;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

fn main() {
    println!("=== Running Fixed Vesting Invariant Example ===");
    let env = Env::default();
    env.mock_all_auths();

    let beneficiary = Address::generate(&env);
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);

    client.init(&beneficiary);
    env.ledger().set_timestamp(1_000);

    // Append sorted milestones
    println!("Appending milestone 1: t=2000, cumulative=1000");
    client.append_milestone(&2_000, &1_000);

    println!("Appending milestone 2: t=5000, cumulative=2500");
    client.append_milestone(&5_000, &2_500);

    // Verify out-of-order append is rejected
    println!("Attempting out-of-order milestone append: t=3000 (must be rejected)...");
    let res = client.try_append_milestone(&3_000, &3_000);
    assert!(res.is_err(), "Expected out-of-order append to fail");
    println!("Rejected out-of-order append as expected!");

    // Advance time and claim
    env.ledger().set_timestamp(3_000);
    let claimed = client.claim(&beneficiary);
    println!("Claimed at t=3000: {}", claimed);
    assert_eq!(claimed, 1_000);

    // Verify schedule remains strictly sorted
    let sched = client.get_schedule();
    let mut entries = Vec::new();
    for i in 0..sched.len() {
        let m = sched.get(i).unwrap();
        entries.push(m.unlock_timestamp);
    }
    sorted_by_key(&entries, |&ts| ts).unwrap();
    println!("Schedule sorted_by_key invariant verified!");

    println!("Fixed vesting invariant checks passed successfully!");
}
