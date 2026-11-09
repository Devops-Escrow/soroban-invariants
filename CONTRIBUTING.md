# Contributing to `soroban-invariants`

Thank you for your interest in contributing to `soroban-invariants`! This guide explains the development setup, branch and commit conventions, testing expectations, and pull request checklist.

---

## Prerequisites & Setup

1. **Rust Toolchain:**
   - Minimum Supported Rust Version (MSRV): **1.91.0**
   - Active development runs on the latest stable Rust toolchain.
   - Install Rust via [rustup](https://rustup.rs/):
     ```bash
     rustup update stable
     rustup toolchain install 1.91.0
     ```

2. **Verify Environment:**
   ```bash
   cargo check
   cargo test
   ```

---

## Branch & Commit Conventions

### Branch Names
Use imperative prefixes:
- `feature/<name>` for new features or capabilities
- `fix/<name>` for bug fixes
- `docs/<name>` for documentation improvements
- `refactor/<name>` for refactoring without behavior changes

### Commit Messages
We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
- `feat: add token conservation invariant helper`
- `fix: correctly handle zero amounts in schedule generator`
- `test: add shrinking regression test for buggy vesting contract`
- `docs: update quickstart guide with model check walkthrough`
- `chore: bump dependencies and deny config`

---

## How to Run Tests & Linters

Ensure all checks pass before submitting a pull request:

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy linter with denied warnings
cargo clippy --all-targets --all-features -- -D warnings

# 3. Unit, integration, and doctests
cargo test --all-targets
cargo test --doc

# 4. License and advisory audit
cargo deny check
cargo audit
```

---

## Pull Request Checklist

Before submitting a PR, verify:
- [ ] Code compiles cleanly on both stable Rust and MSRV (1.91.0).
- [ ] `cargo fmt` and `cargo clippy -- -D warnings` produce zero warnings.
- [ ] New public types, traits, and functions have rustdoc documentation with runnable doctests (`cargo test --doc`).
- [ ] Tests cover positive paths, error/failure paths, and edge cases.
- [ ] No temporary, scratch, or sensitive files are included.
- [ ] If adding dependencies, they are minimal and documented in `docs/DEPENDENCIES.md`.
