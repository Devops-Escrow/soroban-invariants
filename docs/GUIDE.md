# Writing Your First Invariant Scenario in Under 15 Minutes

This guide walks you through creating a stateful invariant test for a Soroban smart contract using `soroban-invariants`.

---

## 1. Concepts: Actions, Abstract Models, and Invariants

Traditional example-based tests call a contract in a predetermined order:
```text
deposit(100) -> withdraw(50) -> check balance is 50
```
Real networks experience unpredictable sequences of calls by arbitrary actors. Model-based property testing replaces fixed test scripts with a state machine:
- **`Action`**: An enum representing every valid transaction or environment advancement an actor can submit.
- **`Model`**: An in-memory, simple reference state machine that tracks the mathematical "ground truth" of what the contract state should be.
- **`Scenario`**: A trait that binds your contract and model together, generating actions with `proptest`, applying them step-by-step, and verifying invariants.

---

## 2. A Minimal Scenario: The PiggyBank Contract

Let's test a simple savings contract with two operations: `deposit(amount)` and `withdraw(amount)`.

### Step 1: Define the Actions
```rust
#[derive(Debug, Clone)]
pub enum BankAction {
    Deposit(i128),
    Withdraw(i128),
}
```

### Step 2: Define the Abstract Model
The reference model tracks how much money *should* be in the bank:
```rust
pub struct BankModel {
    pub expected_balance: i128,
}
```

### Step 3: Implement `Scenario`
```rust
use soroban_invariants::harness::{Scenario, StepOutcome};
use soroban_invariants::invariants::{non_negative_balances, InvariantError};
use soroban_invariants::strategies::bounded_amount;
use soroban_sdk::{Env, Address, testutils::Address as _};
use proptest::prop_oneof;
use proptest::strategy::Strategy;

pub struct BankScenario {
    pub contract_id: Address,
    pub user: Address,
}

impl Scenario for BankScenario {
    type Action = BankAction;
    type Model = BankModel;

    fn init(env: &Env) -> (Self, Self::Model) {
        let user = Address::generate(env);
        // Register your contract in the test environment
        let contract_id = env.register(PiggyBankContract, ());

        (
            BankScenario { contract_id, user },
            BankModel { expected_balance: 0 },
        )
    }

    fn strategy() -> impl Strategy<Value = Self::Action> {
        prop_oneof![
            bounded_amount(1, 10_000).prop_map(BankAction::Deposit),
            bounded_amount(1, 10_000).prop_map(BankAction::Withdraw),
        ]
    }

    fn apply(
        &mut self,
        env: &Env,
        model: &mut Self::Model,
        action: &Self::Action,
    ) -> StepOutcome {
        env.mock_all_auths();
        let client = PiggyBankContractClient::new(env, &self.contract_id);

        match action {
            BankAction::Deposit(amt) => {
                client.deposit(&self.user, amt);
                model.expected_balance += amt;
                StepOutcome::Success
            }
            BankAction::Withdraw(amt) => {
                if *amt <= model.expected_balance {
                    client.withdraw(&self.user, amt);
                    model.expected_balance -= amt;
                    StepOutcome::Success
                } else {
                    // Preconditions: withdrawal exceeds available funds, must fail!
                    let res = client.try_withdraw(&self.user, amt);
                    StepOutcome::from_result(res)
                }
            }
        }
    }

    fn check_invariants(&self, env: &Env, model: &Self::Model) -> Result<(), InvariantError> {
        let client = PiggyBankContractClient::new(env, &self.contract_id);
        let actual_balance = client.get_balance(&self.user);

        // Invariant 1: Contract balance must always equal model balance
        if actual_balance != model.expected_balance {
            return Err(InvariantError::Custom(format!(
                "Balance desync: contract {} != model {}",
                actual_balance, model.expected_balance
            )));
        }

        // Invariant 2: Balances must never be negative
        non_negative_balances(&[actual_balance])?;

        Ok(())
    }
}
```

---

## 3. Running the Test and Interpreting Output

Run the scenario within standard `cargo test`:

```rust
#[test]
fn test_piggy_bank_invariants() {
    let config = HarnessConfig::default()
        .with_cases(256)
        .with_max_actions(30);

    run_scenario::<BankScenario>(config).expect("Invariants must hold");
}
```

If a bug is found (for instance, an integer underflow or missing authorization check), the test runner shrinks the failing trace and prints a reproduction report:

```text
================================================================================
                    SOROBAN INVARIANTS FAILURE REPORT
================================================================================
Replay Seed: [0x5f, 0x12, ...]
(reproduce with: HarnessConfig::default().with_seed([...]))

Failure Detail:
Step #2 failed invariant: Balance desync: contract -10 != model 0

Minimal Reproducing Action Trace (shrunk to 2 actions):
   1: BankAction::Deposit(50)
   2: BankAction::Withdraw(60)
================================================================================
```

Copy the seed and pass it to `HarnessConfig::default().with_seed(seed)` to immediately reproduce the bug deterministically.
