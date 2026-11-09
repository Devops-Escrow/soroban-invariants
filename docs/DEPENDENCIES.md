# Dependency Justifications and Auditing

This document details every direct dependency in `soroban-invariants`, justifying why it was included, verifying that alternatives were evaluated, and describing how dependency security and compatibility are audited.

---

## Direct Runtime Dependencies

### 1. `soroban-sdk` (v29.0.1)
- **Role:** Official Soroban smart contract software development kit by Stellar Development Foundation.
- **Features Enabled:** `["testutils"]`
- **Justification:**
  - `soroban-invariants` is specifically built to test Soroban contracts within the local host simulation environment.
  - The `testutils` feature provides the `Env`, `Address::generate`, `mock_auths()`, `env.auths()`, `env.ledger()`, and contract deployment fixtures needed to execute contract invocations and inspect execution logs.
- **Why alternatives were rejected:**
  - `soroban-env-host` is the lower-level runtime; `soroban-sdk` is what smart contract developers actually use in their own contracts and tests. Integrating with `soroban-sdk` ensures seamless compatibility with developer contracts and types (`Address`, `Symbol`, `Vec`, `Map`, `Val`).

### 2. `proptest` (v1.12.0)
- **Role:** Property-based testing engine with shrinking support.
- **Features Enabled:** `["std"]` (default features disabled to avoid pulling in unnecessary fork/timeout wrappers unless requested).
- **Justification:**
  - Provides state-of-the-art randomized generation and automatic test-case shrinking (finding minimal reproducible failing sequences).
  - Shrinking is essential for invariant testing: a sequence of 50 contract operations that triggers a violation is difficult to diagnose, but shrinking reduces it to the minimal sequence of 2 or 3 calls that reproduce the bug.
- **Why alternatives were rejected:**
  - `quickcheck`: Lacks the sophisticated shrinking trees and state machine abstractions of `proptest`.
  - Custom PRNG loop: Rolling our own generator would lose battle-tested shrinking algorithms and ecosystem familiarity.

### 3. `thiserror` (v2.0)
- **Role:** Idiomatic error derive macro.
- **Justification:**
  - Used for defining structured, user-facing error types in `StepOutcome`, `HarnessError`, and invariant assertion violations without handwritten boilerplate.
- **Why alternatives were rejected:**
  - Zero runtime overhead and produces standard `core::fmt::Display` and `std::error::Error` implementations.

---

## Transitive Dependency Auditing

All dependencies are vetted using standard security auditing tooling:
1. `cargo deny check`: Validates licenses (MIT/Apache-2.0 compatible) and advisory databases.
2. `cargo audit`: Audits all dependencies against the RustSec Advisory Database for known vulnerabilities.
3. Locked builds: The `Cargo.lock` file is committed to source control to guarantee reproducible, bit-for-bit builds across development, CI, and consumers.

### Documented Upstream Transitive Advisories

The following advisories stem from transitive dependencies within `soroban-sdk` 29.0.1 (`soroban-env-host`):

1. **`RUSTSEC-2024-0436` (`paste` v1.0.15)**:
   - *Status:* Unmaintained advisory for the macro helper crate `paste`.
   - *Impact:* No security vulnerability; pulled transitively by `ark-ff` and `soroban-wasmi` within `soroban-env-host`. Documented and ignored until upstream SDF bumps the dependencies.
2. **`RUSTSEC-2026-0387` (`crypto-bigint` v0.5.5)**:
   - *Status:* Timing side-channel advisory on certain Montgomery multiplication operations in `crypto-bigint` 0.5.x.
   - *Impact:* Used internally in `k256`/`p256` signature verification within the host test simulation environment. Local testing environments are not susceptible to remote microarchitectural side-channels. Ignored until upstream SDF releases an SDK update pinning `crypto-bigint >= 0.7.6`.
