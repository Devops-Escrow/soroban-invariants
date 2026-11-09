# soroban-invariants

[![CI](https://github.com/Devops-escrow/soroban-invariants/actions/workflows/ci.yml/badge.svg)](https://github.com/Devops-escrow/soroban-invariants/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-beta-orange.svg)]()

A property-based and model-based invariant testing library for Soroban smart contracts on the Stellar network.

---

## Why Example-Based Tests Are Not Enough

Ordinary unit and example-based tests execute a single, hand-crafted sequence of calls:

```rust
// Traditional example-based test:
contract.deposit(&alice, 100);
contract.claim(&alice);
assert_eq!(contract.get_balance(&alice), 0);
```

While each individual function call may work correctly in isolation, smart contracts regularly break under unexpected combinations and interleavings of valid transactions:
- Multi-user race conditions that leave an internal schedule unsorted.
- Sequences of deposit, transfer, and refund calls that leak fractional rounding dust or fail token conservation.
- Privilege escalation when authorization checks pass under default states but fail after state transitions.

Property and invariant testing eliminates this blindspot. Instead of testing one hardcoded execution trace, `soroban-invariants` generates hundreds of randomized multi-action sequences, runs them against the Soroban test host, evaluates system invariants after **every single step**, and shrinks any failure down to the minimal counterexample.

---

## Installation

Add `soroban-invariants` to your `Cargo.toml` under `[dev-dependencies]`:

```toml
[dev-dependencies]
soroban-invariants = "0.1.0"
soroban-sdk = { version = "29.0.1", features = ["testutils"] }
proptest = { version = "1.12.0", default-features = false, features = ["std"] }
```

### Supported `soroban-sdk` Versions
- **Current Support:** `soroban-sdk = "29.0.1"` (and compatible 29.x releases).
- **Minimum Supported Rust Version (MSRV):** `1.91.0` (matching `soroban-sdk` 29.0.1 requirement).

---

## 30-Line "Hello Invariants" Quickstart

```rust
use proptest::strategy::Strategy;
use soroban_invariants::harness::{run_scenario, HarnessConfig, Scenario, StepOutcome};
use soroban_invariants::invariants::{non_negative_balances, InvariantError};
use soroban_invariants::strategies::bounded_amount;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Counter;
#[contractimpl]
impl Counter {
    pub fn add(env: Env, n: i128) -> i128 { n }
}

struct CounterScenario { total: i128 }
impl Scenario for CounterScenario {
    type Action = i128;
    type Model = i128;
    fn init(_env: &Env) -> (Self, Self::Model) { (CounterScenario { total: 0 }, 0) }
    fn strategy() -> impl Strategy<Value = Self::Action> { bounded_amount(1, 100) }
    fn apply(&mut self, _env: &Env, model: &mut Self::Model, action: &Self::Action) -> StepOutcome {
        self.total += action;
        *model += action;
        StepOutcome::Success
    }
    fn check_invariants(&self, _env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        assert_eq!(self.total, *model);
        non_negative_balances(&[self.total])
    }
}

#[test]
fn test_counter_invariant() {
    run_scenario::<CounterScenario>(HarnessConfig::default().with_cases(256)).unwrap();
}
```

---

## Built-in Invariant Helpers

`soroban-invariants` provides reusable, composable invariant checks ready for smart contract audits:

1. **`token_conservation(initial_total, current_balances, minted_or_burned_delta)`**
   Verifies that the sum of all user balances and contract pools equals the expected total supply, strictly accounting for minted or burned amounts.
2. **`non_negative_balances(balances)` / `named_non_negative_balances(pairs)`**
   Ensures that no account or reserve balance dips below zero.
3. **`monotonic_ledger_time(previous_time, current_time)`**
   Guards against scenarios that inadvertently move ledger time backwards.
4. **`assert_requires_auth(env, required_authorizer, invocation)` / `assert_auth_recorded`**
   Verifies that a privileged call strictly requires authorization from the expected address and records that authorization in `env.auths()`.
5. **`sorted_by_key(items, key_fn)` / `strictly_sorted_by_key(items, key_fn)`**
   Asserts that internal collections (unlock schedules, queue orders, priority lists) remain properly sorted.
6. **`sum_matches(reported_total, parts)`**
   Validates that aggregated contract totals match the sum of individual component allocations.

---

## How Shrinking Output Reads

When an invariant is violated, `soroban-invariants` leverages `proptest` to shrink the failing sequence of 30+ random operations down to the **minimal reproducing trace** and displays a deterministic replay report:

```text
================================================================================
                    SOROBAN INVARIANTS FAILURE REPORT
================================================================================
Replay Seed: [0x12, 0x34, 0x56, 0x78, ...]
(reproduce with: HarnessConfig::default().with_seed([0x12, 0x34, 0x56, 0x78, ...]))

Failure Detail:
Step #2 failed invariant:
Action: AppendMilestone { unlock_timestamp: 2000, amount: 5000 }
Outcome: Success
Invariant Error: ordering violated at index 1: element at index 1 (key: 2000) is less than element at index 0 (key: 5000)

Minimal Reproducing Action Trace (shrunk to 2 actions):
   1: AppendMilestone { unlock_timestamp: 5000, amount: 1000 }
   2: AppendMilestone { unlock_timestamp: 2000, amount: 5000 }
================================================================================
```

### Reproducing Failures Deterministically
Copy the seed directly from the failure report and set it on `HarnessConfig`:

```rust
let config = HarnessConfig::default().with_seed([0x12, 0x34, 0x56, 0x78, /* ... */]);
run_scenario::<VestingScenario>(config).unwrap();
```

---

## Writing a Custom Invariant

Custom invariants are written as standalone functions or directly within your `Scenario::check_invariants` implementation:

```rust
use soroban_invariants::invariants::InvariantError;
use soroban_sdk::Env;

fn check_solvency_ratio(env: &Env, collateral: i128, debt: i128) -> Result<(), InvariantError> {
    if debt > 0 && collateral * 100 / debt < 150 {
        return Err(InvariantError::Custom(format!(
            "Protocol undercollateralized: collateral={}, debt={}",
            collateral, debt
        )));
    }
    Ok(())
}
```

---

## Examples in This Repository

Inspect the [`examples/`](examples/) directory and test suite:
- **`examples/escrow.rs`**: A complete, secure escrow contract verifying token conservation, non-negative balances, and arbiter authorization across 256 test cases.
- **`examples/buggy_vesting.rs`**: A deliberately vulnerable vesting contract where out-of-order schedule additions cause early over-releases.
- **`tests/vesting_proof_tests.rs`**: Proves that `soroban-invariants` detects the vulnerability and shrinks it to a 2-step counterexample, alongside a passing test for the fixed [`examples/vesting.rs`](examples/vesting.rs) contract.

---

## Limitations

- **Simulated Test Environment:** `soroban-invariants` runs against the official `soroban-sdk::Env` host simulation. While it accurately simulates host storage, authentication, and errors, it is not a live network node.
- **Resource Limits & Gas:** Host CPU and memory budget enforcement or network transaction fee limits are not actively modeled unless explicitly configured via host testutils.
- **Concurrency:** Multi-contract calls run synchronously in the host thread.

---

## Contributing

We welcome contributions! Please review our [Contribution Guidelines](CONTRIBUTING.md), [Architecture Specification](docs/ARCHITECTURE.md), and [Code of Conduct](CODE_OF_CONDUCT.md).

To run the complete verification suite locally:

```bash
# Format check
cargo fmt --all -- --check

# Clippy with denied warnings
cargo clippy --all-targets --all-features -- -D warnings

# Unit tests, integration tests, and doc tests
cargo test --all-targets
cargo test --doc

# License and security check
cargo deny check
```

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
