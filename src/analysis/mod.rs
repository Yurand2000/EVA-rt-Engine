//! Analyzers and Designers for specific plaforms and scheduling algorithms.

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        Analyzer,
        Designer,
        SchedTestResult,
        SchedTestResults,
    };
}

/// Schedulability Results for a single test of a given analyzer
pub struct SchedTestResult {
    pub test_name: &'static str,
    pub result: SchedResult<()>,
}

/// Schedulability Results for a set of tests of a given analyzer
pub struct SchedTestResults {
    pub schedulable: bool,
    pub results: Vec<SchedTestResult>,
}

impl std::ops::Deref for SchedTestResults {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.schedulable
    }
}

/// Common trait shared across all schedulability analyzers.
///
/// ### Generic Parameters
/// - `T`: Task Model
/// - `P`: Platform Description
pub trait Analyzer<T, P> {
    /// verifies if the given `taskset` on the given `platform` is schedulable
    fn is_schedulable(&self, taskset: &[T], platform: &P, short_circuit: bool) -> SchedTestResults;

    /// verifies if the given `taskset` on the given `platform` is schedulable
    /// using the given test. Test names can be queried by calling the
    /// [`available_tests`] method.
    fn run_schedulability_test(&self, taskset: &[T], platform: &P, test_name: &str) -> SchedResult<()>;

    /// returns the set of available schedulability tests provided by the
    /// analyzer that can be run through [`is_schedulable_test`]
    fn available_tests(&self) -> impl Iterator<Item = &'static str>;
}

/// Common trait shared across all schedulability designers.
///
/// ### Generic Parameters
/// - `T`: Task Model
/// - `M`: Designer Parameters
/// - `P`: Platform Description
pub trait Designer<T, M, P> {
    /// generates the best `platform` that can execute the given `taskset`. The
    /// best platform is chosen according to the given `parameters`.
    fn design(taskset: &[T], parameters: &M) -> anyhow::Result<P>;
}

// Fully-Preemptive Model
pub mod full_preemption {
    // UniProcessor Scheduling
    pub mod uniprocessor {
        pub mod earliest_deadline_first;
        pub mod fixed_priority;
    }
}