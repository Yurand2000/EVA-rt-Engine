//! Analyzers and Designers for specific plaforms and scheduling algorithms.

/// Common trait shared across all schedulability analyzers.
/// 
/// ### Generic Parameters
/// - `T`: Task Model
/// - `P`: Platform Description
pub trait Analyzer<T, P> {
    /// verifies if the given `taskset` on the given `platform` is schedulable
    fn is_schedulable(taskset: &[T], platform: &P) -> anyhow::Result<bool>;
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