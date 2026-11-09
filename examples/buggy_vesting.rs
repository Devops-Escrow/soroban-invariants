//! Deliberately buggy vesting contract example demonstrating failure detection and shrinking.
//!
//! Run with: `cargo run --example buggy_vesting`

use soroban_invariants::examples::buggy_vesting::{
    BuggyVestingContract, BuggyVestingContractClient,
};
use soroban_invariants::invariants::sorted_by_key;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

fn main() {
    println!("=== Running Deliberately Buggy Vesting Demonstration ===");
    let env = Env::default();
    env.mock_all_auths();

    let beneficiary = Address::generate(&env);
    let contract_id = env.register(BuggyVestingContract, ());
    let client = BuggyVestingContractClient::new(&env, &contract_id);

    client.init(&beneficiary);
    env.ledger().set_timestamp(1_000);

    // Step 1: Append late milestone
    println!("Action 1: Appending milestone at t=5000 with 1000 tokens...");
    client.append_milestone(&5_000, &1_000);

    // Step 2: Append early milestone (BUG: allowed and left unsorted!)
    println!(
        "Action 2: Appending milestone at t=2000 with 5000 tokens (deliberately unsorted!)..."
    );
    client.append_milestone(&2_000, &5_000);

    // Step 3: Check sorted_by_key invariant
    let sched = client.get_schedule();
    let mut timestamps = Vec::new();
    for i in 0..sched.len() {
        let m = sched.get(i).unwrap();
        timestamps.push(m.unlock_timestamp);
    }

    println!(
        "Checking sorted_by_key invariant on contract schedule: {:?}",
        timestamps
    );
    let inv_res = sorted_by_key(&timestamps, |&ts| ts);
    match inv_res {
        Err(err) => {
            println!("INVARIANT VIOLATION DETECTED: {}", err);
            println!(
                "The buggy vesting contract failed the sorted schedule invariant as expected."
            );
        }
        Ok(()) => panic!("Expected invariant violation, but got Ok!"),
    }
}
