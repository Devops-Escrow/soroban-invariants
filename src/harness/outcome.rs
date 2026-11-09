//! StepOutcome captures the result of a single contract invocation during a scenario.

use crate::invariants::InvariantError;

/// Outcome of a contract action step.
///
/// Records whether the contract call succeeded, failed with a specific Soroban contract error code,
/// or encountered a host/runtime error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// Contract call succeeded normally.
    Success,
    /// Contract call failed with a specific Soroban `ContractError` code.
    ContractError(u32),
    /// Host-level error (e.g. authorization failure, budget exceeded, invalid conversion).
    HostError(String),
    /// Contract execution panicked.
    Panicked(String),
    /// The step was ignored or skipped (e.g. invalid preconditions in model).
    Ignored,
}

impl StepOutcome {
    /// Create a success outcome.
    pub fn success() -> Self {
        Self::Success
    }

    /// Create a contract error outcome with the given error code.
    pub fn contract_error(code: u32) -> Self {
        Self::ContractError(code)
    }

    /// Create a host error outcome.
    pub fn host_error(msg: impl Into<String>) -> Self {
        Self::HostError(msg.into())
    }

    /// Returns `true` if the step succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Returns `true` if the step resulted in any error.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ContractError(_) | Self::HostError(_) | Self::Panicked(_)
        )
    }

    /// Returns `true` if the step resulted in a specific contract error code.
    pub fn is_contract_error(&self, expected_code: u32) -> bool {
        matches!(self, Self::ContractError(code) if *code == expected_code)
    }

    /// Assert that the step succeeded, returning an `InvariantError` if it did not.
    pub fn assert_success(&self) -> Result<(), InvariantError> {
        if self.is_success() {
            Ok(())
        } else {
            Err(InvariantError::Custom(format!(
                "expected contract step to succeed, but got {:?}",
                self
            )))
        }
    }

    /// Assert that the step failed with an expected contract error code.
    pub fn assert_contract_error(&self, expected_code: u32) -> Result<(), InvariantError> {
        match self {
            Self::ContractError(code) if *code == expected_code => Ok(()),
            other => Err(InvariantError::Custom(format!(
                "expected contract error code {}, but got {:?}",
                expected_code, other
            ))),
        }
    }

    /// Construct a `StepOutcome` from a standard `Result<T, E>`.
    pub fn from_result<T, E: core::fmt::Debug>(result: Result<T, E>) -> Self {
        match result {
            Ok(_) => Self::Success,
            Err(err) => {
                let debug_str = format!("{:?}", err);
                // Inspect if string contains a known error code format like Error(Contract, #1)
                Self::HostError(debug_str)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_outcome_variants() {
        let ok = StepOutcome::success();
        assert!(ok.is_success());
        assert!(!ok.is_error());
        assert!(ok.assert_success().is_ok());

        let err = StepOutcome::contract_error(42);
        assert!(!err.is_success());
        assert!(err.is_error());
        assert!(err.is_contract_error(42));
        assert!(!err.is_contract_error(99));
        assert!(err.assert_contract_error(42).is_ok());
        assert!(err.assert_success().is_err());
    }

    #[test]
    fn test_step_outcome_from_result() {
        let res_ok: Result<&str, &str> = Ok("data");
        assert_eq!(StepOutcome::from_result(res_ok), StepOutcome::Success);

        let res_err: Result<&str, &str> = Err("auth failed");
        assert!(StepOutcome::from_result(res_err).is_error());
    }
}
