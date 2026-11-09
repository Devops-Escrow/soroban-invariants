//! Configuration options for scenario execution and deterministic replay.

/// Configuration for running a scenario test suite.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Number of distinct scenario sequences to generate and evaluate.
    pub test_cases: u32,
    /// Minimum number of actions per generated scenario sequence.
    pub min_actions: usize,
    /// Maximum number of actions per generated scenario sequence.
    pub max_actions: usize,
    /// Optional PRNG seed `[u8; 32]` for deterministic failure replay.
    pub seed: Option<[u8; 32]>,
    /// Maximum iterations for the proptest shrinking engine.
    pub max_shrink_iters: u32,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            test_cases: 256,
            min_actions: 1,
            max_actions: 20,
            seed: None,
            max_shrink_iters: 10_000,
        }
    }
}

impl HarnessConfig {
    /// Set the number of test cases.
    pub fn with_cases(mut self, cases: u32) -> Self {
        self.test_cases = cases;
        self
    }

    /// Set the minimum action length per scenario sequence.
    pub fn with_min_actions(mut self, min: usize) -> Self {
        self.min_actions = min;
        self
    }

    /// Set the maximum action length per scenario sequence.
    pub fn with_max_actions(mut self, max: usize) -> Self {
        self.max_actions = max;
        self
    }

    /// Provide a fixed seed `[u8; 32]` to reproduce a specific test failure deterministically.
    pub fn with_seed(mut self, seed: [u8; 32]) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the maximum number of shrink iterations.
    pub fn with_max_shrink_iters(mut self, iters: u32) -> Self {
        self.max_shrink_iters = iters;
        self
    }
}
