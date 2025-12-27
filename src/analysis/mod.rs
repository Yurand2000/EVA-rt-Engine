//! Analyzers for specific plaforms and scheduling algorithms.

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        Analyzer,
    };
}

/// Common trait shared across all schedulability analyzers.
///
/// ### Generic Parameters
/// - `T`: Task Model
/// - `P`: Platform Description
pub trait Analyzer<T, P> {
    /// verifies if the given `taskset` on the given `platform` is schedulable
    fn is_schedulable(&self, taskset: &[T], platform: &P, short_circuit: bool) -> Vec<SchedResult<()>>;

    /// verifies if the given `taskset` on the given `platform` is schedulable
    /// using the given test. Test names can be queried by calling the
    /// [`available_tests`] method.
    fn run_schedulability_test(&self, taskset: &[T], platform: &P, test_name: &str) -> SchedResult<()>;

    /// returns the set of available schedulability tests provided by the
    /// analyzer that can be run through [`is_schedulable_test`]
    fn available_tests(&self) -> &[&'static str];
}

// Fully-Preemptive Model
pub mod full_preemption {
    // UniProcessor Scheduling
    pub mod uniprocessor {
        pub mod earliest_deadline_first;
        pub mod fixed_priority;
    }
}