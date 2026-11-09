# Prior Art and Comparative Analysis

This document surveys existing tools, libraries, and frameworks for invariant testing, property-based testing, and formal verification within the Stellar Soroban and wider Rust smart-contract ecosystems. It clarifies where `soroban-invariants` fits and how it differs from existing solutions.

---

## Existing Tools Survey

### 1. `soroban-breakwater` (`breakwater`)
- **Repository / Registry:** [crates.io/crates/soroban-breakwater](https://crates.io/crates/soroban-breakwater) | [GitHub](https://github.com/stellar/rs-soroban-sdk)
- **What it does:** Breakwater is an Echidna/Foundry-inspired stateful fuzzing and invariant testing framework for Soroban contracts. It defines test configurations around **actors** (who can invoke operations), **handlers** (contract entrypoints wrapped with parameter constraints), and **invariants** evaluated against simulated ledger state. It also features modular "packs" (e.g. for TTL archival and policy bypass testing).
- **How `soroban-invariants` differs:**
  - `soroban-breakwater` is designed around heavyweight fuzzing architectures, often paired with dedicated CLI runners and transaction generation loops.
  - `soroban-invariants` is a lightweight, zero-external-binary Rust library built on the standard `proptest` engine and `soroban-sdk` testutils.
  - In `soroban-invariants`, developers write idiomatic Rust unit/integration tests running under standard `cargo test`.
  - It provides first-class model-based state machine verification (`Scenario` trait with state transitions `apply(&mut self, env, model, action) -> StepOutcome`), shrinking to minimal reproducing sequences with human-readable counterexamples, deterministic seed replay, and ready-to-use invariant assertions for balances, authorization, ordering, and totals.

### 2. `soroban-fuzzer`
- **Repository / Registry:** [crates.io/crates/soroban-fuzzer](https://crates.io/crates/soroban-fuzzer)
- **What it does:** Provides a stateful, property-based fuzzing harness for Soroban smart contracts in the Soroban test environment.
- **How `soroban-invariants` differs:**
  - `soroban-fuzzer` offers basic raw fuzz loops without high-level model checking, custom outcome assertions (`StepOutcome`), or a comprehensive suite of domain-specific contract invariants (auth checking, schedule ordering, conservation of tokens).
  - `soroban-invariants` provides reusable property strategies for bounded amounts, non-decreasing timestamps, address pools, and vesting schedules, alongside structured scenario runners.

### 3. Komet (Runtime Verification)
- **Repository / Website:** [runtimeverification.com](https://runtimeverification.com)
- **What it does:** A formal verification tool specifically engineered for Soroban bytecode using the K Framework. It enables mathematical proofs of invariants and symbolic execution.
- **How `soroban-invariants` differs:**
  - Komet requires deep mathematical specification in formal languages, specialized solvers, and significant setup overhead. It is intended for formal audit verification.
  - `soroban-invariants` is developer-centric property testing written entirely in standard Rust, allowing protocol teams to test invariants continuously during development and in rapid CI pipelines.

### 4. `proptest` (General Rust Ecosystem)
- **Repository / Registry:** [crates.io/crates/proptest](https://crates.io/crates/proptest)
- **What it does:** General-purpose property testing framework for Rust, featuring Hypothesis-like shrinking and random value generation.
- **How `soroban-invariants` differs:**
  - `proptest` has no domain knowledge of Soroban: it does not manage Soroban `Env`, contract addresses, `mock_auths`, ledger sequence or timestamp progression, or Soroban error conversions.
  - `soroban-invariants` builds on top of `proptest`, providing specialized generators for Soroban types, stateful multi-step scenario execution, step-level invariant evaluation, and contract error handling.

---

## Summary of Distinction

| Dimension | `proptest` | `soroban-breakwater` | Komet | `soroban-invariants` |
|:---|:---|:---|:---|:---|
| **Ecosystem Focus** | General Rust | Soroban Fuzzing / CLI | Soroban Formal Proofs | Soroban Property & Model Testing |
| **Execution Engine** | `cargo test` | Dedicated CLI / Runner | Symbolic / K Engine | Standard `cargo test` |
| **Model Checking** | Manual state machines | Handler / Actor graph | Mathematical Model | Integrated `Scenario` Trait |
| **Auth & Ledger Mocking** | None | Fuzz harness hooks | Bytecode semantics | Built-in testutils integration |
| **Shrinking & Replay** | Generic types | Custom regression export | Counterexample trace | Shrinks to minimal action list + Seed replay |
| **Learning Curve** | Low (general) | Medium | Very High | Low (Rust developers) |
