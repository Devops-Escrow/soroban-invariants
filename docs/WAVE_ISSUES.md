# Wave Sprint Issue Backlog

This backlog contains 14 scoped, sprint-ready issue specifications for the upcoming open-source contribution cycle:
- **5 Trivial issues** (< 1 hour)
- **6 Medium issues** (~ half a day)
- **3 High issues** (1–2 days)

---

### Add dust threshold tolerance option to token conservation invariant helper
**Complexity:** Trivial
**Description:** In smart contracts involving fee deductions, dynamic interest curves, or liquidity pool calculations, balance sums may fluctuate by a minimal dust amount due to integer rounding. Currently, `token_conservation` enforces exact integer equivalence, causing tests on AMM or lending protocols to fail on 1-stroop rounding residuals. Adding a configurable tolerance parameter allows testing contracts that permit bounded rounding deviations without sacrificing safety.
**Requirements and context:**
- Add `token_conservation_with_tolerance(initial_total: i128, current_balances: &[i128], minted_or_burned_delta: i128, max_tolerance: i128) -> Result<(), InvariantError>`.
- Must validate that `max_tolerance >= 0`; negative tolerance must return an error or panic with a clear message.
- The absolute difference between `expected_total` and `actual_total` must be `<= max_tolerance`.
- Preserve existing zero-tolerance `token_conservation` as a pass-through calling `token_conservation_with_tolerance(..., 0)`.
**Suggested execution:**
- Touch: `src/invariants/token_conservation.rs` and `src/invariants/mod.rs`
- Branch: `feature/token-conservation-tolerance`
**Acceptance criteria / test plan:**
- Unit tests added in `src/invariants/token_conservation.rs` verifying:
  - Exact match passes.
  - Difference within `max_tolerance` passes.
  - Difference exceeding `max_tolerance` fails with `TokenConservationViolation`.
  - Negative tolerance panics or returns error.
- All doctests compile and pass (`cargo test --doc`).

---

### Add address label mapping helper to StepOutcome for readable trace printing
**Complexity:** Trivial
**Description:** When a scenario fails, the shrunk action trace prints raw Soroban address hashes (e.g. `Address("CA3D...")`), making it tedious to identify which actor performed which action. Adding an address label mapping registry allows replacing cryptic hashes with human-readable aliases like `Alice`, `Bob`, and `EscrowVault` in failure traces. This substantially reduces debugging time for failed invariant runs.
**Requirements and context:**
- Provide a lightweight `AddressBook` or label mapping struct: `pub struct AddressBook(std::collections::HashMap<Address, String>);`.
- Provide helper method `AddressBook::resolve(&self, address: &Address) -> String`.
- Integrate label resolution into `format_failure_report` when an optional address map is supplied.
- Must handle addresses not present in the address book gracefully by falling back to the default `Debug` representation.
**Suggested execution:**
- Touch: `src/harness/runner.rs`, `src/strategies/address.rs`, and `src/harness/mod.rs`
- Branch: `feature/address-label-resolver`
**Acceptance criteria / test plan:**
- Unit test in `tests/harness_tests.rs` asserting that formatted reports display alias names (`Alice`) instead of raw address strings when an `AddressBook` is provided.
- Zero breaking changes to `run_scenario` default signature.

---

### Add strictly monotonic timestamp generator variant
**Complexity:** Trivial
**Description:** Some contracts (such as price oracles or governance votes) reject transactions where block timestamps do not strictly advance (`t_{i+1} > t_i`). The current `timestamp_sequence` allows delta jumps of 0, which generates non-decreasing but duplicate consecutive timestamps. Providing a dedicated `strictly_increasing_timestamp_sequence` generator guarantees positive time progression.
**Requirements and context:**
- Add `strictly_increasing_timestamp_sequence(start_timestamp: u64, min_steps: usize, max_steps: usize, min_jump_secs: u64, max_jump_secs: u64) -> impl Strategy<Value = Vec<u64>>`.
- Must assert `min_jump_secs >= 1` and `max_jump_secs >= min_jump_secs`.
- Every consecutive timestamp pair in the generated sequence must satisfy `ts[i] > ts[i-1]`.
**Suggested execution:**
- Touch: `src/strategies/timestamp.rs` and `src/strategies/mod.rs`
- Branch: `feature/strict-timestamp-generator`
**Acceptance criteria / test plan:**
- Unit tests in `src/strategies/timestamp.rs` verifying 100 generated sequences have strictly increasing timestamps with pairwise difference `>= min_jump_secs`.
- Panic tests verifying invalid jump bounds are rejected.

---

### Expose StepOutcome predicate helpers for common contract error matches
**Complexity:** Trivial
**Description:** Scenarios frequently need to assert that an action failed with one of several acceptable error codes (for example, either `InsufficientAllowance` or `InsufficientBalance`). Currently, users must manually unpack `StepOutcome` or chain match arms. Adding collection matchers simplifies scenario writing and produces cleaner assertions.
**Requirements and context:**
- Implement `is_contract_error_in(&self, codes: &[u32]) -> bool` on `StepOutcome`.
- Implement `assert_contract_error_in(&self, codes: &[u32]) -> Result<(), InvariantError>` on `StepOutcome`.
- If the outcome is not one of the provided error codes, the returned error message must list all acceptable codes and the actual outcome.
**Suggested execution:**
- Touch: `src/harness/outcome.rs`
- Branch: `feature/step-outcome-code-predicates`
**Acceptance criteria / test plan:**
- Unit tests in `src/harness/outcome.rs` asserting:
  - `is_contract_error_in` returns true if code is present in slice, false otherwise.
  - `assert_contract_error_in` returns `Ok(())` for matching codes and detailed `InvariantError::Custom` on mismatch.

---

### Add markdown table summary formatter to HarnessError failure report
**Complexity:** Trivial
**Description:** In CI environments, failure reports are frequently posted as pull request comments. The existing text-based failure report is easy to read in terminal logs but does not format cleanly as GitHub Flavored Markdown. Adding a markdown-formatted report output allows automated CI actions to render clean summary tables of failing action traces.
**Requirements and context:**
- Add `format_failure_report_markdown<S: Scenario>(seed: &[u8; 32], reason: &str, actions: &[S::Action]) -> String`.
- Include seed in a copy-pasteable markdown code block.
- Render the action sequence as a structured markdown table with columns `| Step | Action | Outcome |`.
- Ensure special characters in actions are properly escaped to avoid breaking table layout.
**Suggested execution:**
- Touch: `src/harness/runner.rs`
- Branch: `feature/markdown-failure-formatter`
**Acceptance criteria / test plan:**
- Unit test in `tests/harness_tests.rs` asserting that `format_failure_report_markdown` outputs valid GitHub markdown with table headers and row counts matching the action count.

---

### Implement Map-based balance conservation helper for multi-token contracts
**Complexity:** Medium
**Description:** Multi-asset contracts (such as decentralized exchanges, cross-currency payment routers, and liquidity pools) track separate balances for multiple token identifiers (`Address` or `Symbol`). The existing `token_conservation` helper only operates on a single scalar token balance. Adding a multi-asset conservation helper verifies balance conservation across any number of token keys simultaneously.
**Requirements and context:**
- Add `multi_token_conservation(initial_totals: &std::collections::HashMap<Symbol, i128>, current_balances: &[std::collections::HashMap<Symbol, i128>], deltas: &std::collections::HashMap<Symbol, i128>) -> Result<(), InvariantError>`.
- Must check conservation for every key present in `initial_totals`, `current_balances`, or `deltas`.
- If any individual asset's sum does not match, return a detailed error specifying which token symbol failed, expected amount, and actual amount.
**Suggested execution:**
- Touch: `src/invariants/token_conservation.rs` and `src/invariants/mod.rs`
- Branch: `feature/multi-token-conservation`
**Acceptance criteria / test plan:**
- Unit tests in `src/invariants/token_conservation.rs` covering:
  - Passing multi-token pool with 3 distinct tokens.
  - Failing conservation on a single asset among multiple valid assets.
  - Correct handling of tokens present in user balances but absent in initial totals.

---

### Add weighted action strategy combinator for biased scenario exploration
**Complexity:** Medium
**Description:** By default, `prop_oneof!` selects actions with uniform distribution. However, smart contract edge cases often require biasing execution towards rare operations (e.g. emergency shutdowns, full pool liquidations, or boundary claims) while executing deposits and transfers more frequently. Adding a configurable `WeightedActionPool` enables developers to specify probability weights for actions.
**Requirements and context:**
- Provide a fluent builder: `pub struct WeightedActions<A> { ... }`.
- Methods: `new()`, `add(weight: u32, strategy: impl Strategy<Value = A>) -> Self`, `build() -> impl Strategy<Value = A>`.
- Must reject weights of 0 with a clear error or panic.
- Must ensure shrinking works predictably without getting stuck on high-weight paths.
**Suggested execution:**
- Touch: `src/strategies/mod.rs` (new module `src/strategies/weighted.rs`)
- Branch: `feature/weighted-action-strategy`
**Acceptance criteria / test plan:**
- Strategy tests asserting that generated distributions over 10,000 samples statistically match specified ratio weights within 5% tolerance.
- Integration test demonstrating use in a `Scenario` implementation.

---

### Implement state transition invariant helper for finite state machine contracts
**Complexity:** Medium
**Description:** Many Soroban contracts implement explicit state machines (e.g., `Draft -> Active -> Paused -> Closed` or auction states `Bidding -> PendingSettlement -> Settled`). Invariant tests need to assert that invalid state transitions never occur (e.g., jumping from `Closed` back to `Active`). A reusable `valid_state_transitions` helper enables declarative verification of state transitions.
**Requirements and context:**
- Add `assert_valid_transition<S: PartialEq + core::fmt::Debug>(from: &S, to: &S, allowed_transitions: &[(S, S)]) -> Result<(), InvariantError>`.
- Add an error variant `InvariantError::InvalidStateTransition { from: String, to: String }`.
- Provide clear diagnostics indicating what transition occurred and what transitions were permitted from `from`.
**Suggested execution:**
- Touch: `src/invariants/error.rs`, `src/invariants/mod.rs`, new file `src/invariants/state_machine.rs`
- Branch: `feature/state-machine-invariant`
**Acceptance criteria / test plan:**
- Unit tests in `src/invariants/state_machine.rs` testing valid transitions, invalid transitions, and self-transitions.
- Integration with doctests on the function.

---

### Support ledger sequence number and TTL advancement in time strategies
**Complexity:** Medium
**Description:** Soroban contracts rely not only on ledger timestamps, but also on ledger sequence numbers (increments every ~5 seconds) and entry Time-To-Live (TTL) storage rent periods. Contracts using storage expiration (`extend_ttl`) can fail if sequence numbers do not advance alongside timestamps. Adding a synchronized ledger progression generator allows testing storage expiration dynamics.
**Requirements and context:**
- Add `LedgerProgression { timestamp: u64, sequence: u32 }`.
- Strategy `ledger_sequence_progression(start_ts: u64, start_seq: u32, max_jumps: usize, avg_seconds_per_ledger: u32) -> impl Strategy<Value = Vec<LedgerProgression>>`.
- Ensure `sequence` advances proportionally to `timestamp` delta (e.g. ~5 seconds per ledger).
- Helper function `apply_ledger_progression(env: &Env, step: &LedgerProgression)` setting both timestamp and sequence on `env.ledger()`.
**Suggested execution:**
- Touch: `src/strategies/timestamp.rs` and `src/strategies/mod.rs`
- Branch: `feature/ledger-sequence-progression`
**Acceptance criteria / test plan:**
- Unit tests in `src/strategies/timestamp.rs` confirming that sequence numbers monotonically increase and maintain the configured time-to-sequence ratio.
- Test demonstrating eviction/TTL verification in a Soroban test environment.

---

### Implement step precondition filter combinator in Scenario trait
**Complexity:** Medium
**Description:** In complex scenarios, certain actions are only semantically valid when specific contract conditions are met (e.g., a `Claim` action can only be attempted when the user has an active deposit). Currently, scenarios must manually ignore invalid actions in `apply` by returning `StepOutcome::Ignored`. Adding an optional `precondition(&self, model: &Self::Model, action: &Self::Action) -> bool` method to `Scenario` enables automatic action filtering.
**Requirements and context:**
- Add `fn precondition(&self, _model: &Self::Model, _action: &Self::Action) -> bool { true }` to `Scenario` trait with default `true`.
- In `run_scenario`, before invoking `apply`, evaluate `precondition`.
- If `precondition` returns `false`, record the step as skipped without executing contract invocation.
- Config option `HarnessConfig::fail_on_max_consecutive_invalid_preconditions(usize)` to guard against scenarios that accidentally filter out all actions.
**Suggested execution:**
- Touch: `src/harness/scenario.rs`, `src/harness/runner.rs`, and `src/harness/config.rs`
- Branch: `feature/scenario-preconditions`
**Acceptance criteria / test plan:**
- Integration test in `tests/harness_tests.rs` where an action is rejected by `precondition`, verifying that contract state is unaffected and the step is skipped.
- Test ensuring that infinite loops of invalid preconditions trigger the configured threshold limit.

---

### Add multi-scenario concurrent runner for comparative contract twin testing
**Complexity:** Medium
**Description:** When upgrading contract implementations or refactoring algorithms (e.g. optimizing an AMM invariant from exponential math to fixed-point approximation), developers need to verify that both the legacy implementation and the refactored "twin" produce identical state transitions for all inputs. A dual-contract comparative harness applies identical action sequences to two scenarios side-by-side and asserts equivalence.
**Requirements and context:**
- Provide `run_twin_scenarios<S1, S2>(config: HarnessConfig) -> Result<(), HarnessError>` where both scenarios consume the same `Action` type.
- Executes `action` on `S1` and `S2` in parallel test environments.
- Asserts that both scenarios return identical `StepOutcome` values.
- On discrepancy, shrink the sequence and report where the two implementations diverged.
**Suggested execution:**
- Touch: `src/harness/runner.rs` and `src/harness/mod.rs`
- Branch: `feature/twin-scenario-runner`
**Acceptance criteria / test plan:**
- Integration test in `tests/harness_tests.rs` comparing an unoptimized and optimized arithmetic contract, asserting equivalence.
- Test asserting that an intentional divergence between twins is shrunk and reported.

---

### Implement automated Soroban events invariant verification engine
**Complexity:** High
**Description:** Smart contracts emit Soroban diagnostics and topics/data contract events on critical state transitions (deposits, admin transfers, parameter changes). Testing that contracts emit the exact required events—and never emit unprompted events—is critical for off-chain indexing security. A dedicated event invariant engine captures `env.events().all()`, filters by contract ID and topic, and asserts event emission invariants after every step.
**Requirements and context:**
- Implement `EventFilter` and `EventAssertion`:
  - `assert_event_emitted(env: &Env, contract_id: &Address, topic: Symbol, expected_data: &Val) -> Result<(), InvariantError>`.
  - `assert_no_events_emitted(env: &Env, contract_id: &Address) -> Result<(), InvariantError>`.
  - `assert_event_count(env: &Env, contract_id: &Address, expected_count: usize) -> Result<(), InvariantError>`.
- Handle incremental event offsets so only events emitted during the *current step* are evaluated.
- Support wildcards on topic vectors.
**Suggested execution:**
- Touch: `src/invariants/mod.rs`, new module `src/invariants/events.rs`, `src/harness/scenario.rs`
- Branch: `feature/contract-events-invariant`
**Acceptance criteria / test plan:**
- Unit tests in `src/invariants/events.rs` using a test contract that emits specific events.
- Test asserting missing expected event returns `InvariantError::Custom`.
- Test asserting unexpected event emissions are flagged.
- Integration test in an example contract verifying event emission on deposit.

---

### Implement state snapshot and transaction rollback runner in harness
**Complexity:** High
**Description:** In advanced invariant scenarios, testing speculative actions (e.g. "if this transaction fails, state must remain completely unaltered from before the call") requires capturing and rolling back environment state snapshots. While Soroban's host rolls back ledger storage on contract errors within a call, multi-step speculative rollbacks across actions currently require redeploying contracts. Leveraging Soroban ledger snapshots enables branching test trees.
**Requirements and context:**
- Integrate `soroban_ledger_snapshot` facilities from `soroban-sdk::testutils`.
- Provide `Scenario::snapshot(&self, env: &Env) -> SnapshotToken` and `Scenario::rollback(&mut self, env: &mut Env, token: SnapshotToken)`.
- Enable "branching action" strategies: try a dangerous speculative action, verify invariant, rollback to pre-action state, and proceed along main test branch.
- Ensure memory efficiency: snapshot tokens must clean up discarded test host state.
**Suggested execution:**
- Touch: `src/harness/mod.rs`, `src/harness/runner.rs`, `src/harness/scenario.rs`
- Branch: `feature/snapshot-rollback-runner`
**Acceptance criteria / test plan:**
- Integration test in `tests/harness_tests.rs` taking a snapshot at step 5, applying 3 actions that modify contract state, rolling back to step 5 snapshot, and asserting contract storage matches step 5 bit-for-bit.
- Full test suite passing without memory leak or panic.

---

### Add replay artifact serializer and regression test exporter
**Complexity:** High
**Description:** When property tests discover an invariant violation, reproducing it in continuous integration or sharing the counterexample with other developers currently requires passing the seed or manually copying the printed action sequence. Adding an automated code generator that exports shrunk failure traces as standalone Rust `#[test]` regression functions enables instantaneous regression test suites.
**Requirements and context:**
- Implement `export_regression_test<S: Scenario>(scenario_name: &str, actions: &[S::Action], output_path: &std::path::Path) -> std::io::Result<()>`.
- Generated file must contain valid, standalone Rust code:
  - Generates the same participants.
  - Replays each action step sequentially with explicit assertions.
  - Compiles and runs under `cargo test` without proptest dependencies.
- Add CLI flag or config option `HarnessConfig::with_regression_export_dir(path)`.
**Suggested execution:**
- Touch: `src/harness/runner.rs`, `src/harness/config.rs`, new module `src/harness/export.rs`
- Branch: `feature/regression-test-exporter`
**Acceptance criteria / test plan:**
- Unit test running a deliberately failing scenario, exporting the regression test to a temporary directory, and dynamically compiling/verifying the generated test string.
- Exported code matches `cargo fmt` formatting standards.
