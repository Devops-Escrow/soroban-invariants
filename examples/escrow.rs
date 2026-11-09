//! Standalone Escrow example runner.
//!
//! Run with: `cargo run --example escrow`

use soroban_invariants::examples::escrow::{EscrowContract, EscrowContractClient};
use soroban_invariants::invariants::{non_negative_balances, sum_matches, token_conservation};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn main() {
    println!("=== Running Escrow Invariant Verification Example ===");
    let env = Env::default();
    env.mock_all_auths();

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    client.init(&depositor, &beneficiary, &arbiter);

    let initial_depositor_funds = 500_000i128;
    let mut depositor_wallet = initial_depositor_funds;
    let mut beneficiary_wallet = 0i128;

    // Step 1: Deposit 200,000
    println!("Step 1: Depositing 200,000 into escrow...");
    client.deposit(&depositor, &200_000);
    depositor_wallet -= 200_000;

    let bal = client.get_balance();
    assert_eq!(bal, 200_000);
    token_conservation(
        initial_depositor_funds,
        &[depositor_wallet, beneficiary_wallet, bal],
        0,
    )
    .unwrap();
    non_negative_balances(&[depositor_wallet, beneficiary_wallet, bal]).unwrap();
    sum_matches(
        client.get_total_deposited(),
        &[
            bal,
            client.get_total_released(),
            client.get_total_refunded(),
        ],
    )
    .unwrap();
    println!("Invariants verified after Step 1!");

    // Step 2: Release 150,000 to beneficiary
    println!("Step 2: Arbiter releases 150,000 to beneficiary...");
    client.release(&arbiter, &150_000);
    beneficiary_wallet += 150_000;

    let bal = client.get_balance();
    assert_eq!(bal, 50_000);
    token_conservation(
        initial_depositor_funds,
        &[depositor_wallet, beneficiary_wallet, bal],
        0,
    )
    .unwrap();
    non_negative_balances(&[depositor_wallet, beneficiary_wallet, bal]).unwrap();
    sum_matches(
        client.get_total_deposited(),
        &[
            bal,
            client.get_total_released(),
            client.get_total_refunded(),
        ],
    )
    .unwrap();
    println!("Invariants verified after Step 2!");

    // Step 3: Refund remaining 50,000 to depositor
    println!("Step 3: Arbiter refunds remaining 50,000 to depositor...");
    client.refund(&arbiter, &50_000);
    depositor_wallet += 50_000;

    let bal = client.get_balance();
    assert_eq!(bal, 0);
    token_conservation(
        initial_depositor_funds,
        &[depositor_wallet, beneficiary_wallet, bal],
        0,
    )
    .unwrap();
    non_negative_balances(&[depositor_wallet, beneficiary_wallet, bal]).unwrap();
    sum_matches(
        client.get_total_deposited(),
        &[
            bal,
            client.get_total_released(),
            client.get_total_refunded(),
        ],
    )
    .unwrap();
    println!("Invariants verified after Step 3!");

    println!("All escrow invariant checks passed successfully!");
}
