//! ## Fixed Priority Rate Monotonic - Liu & Layland 1973
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed-Priority scheduling
//!
//! #### Preconditions:
//! - Implicit Deadlines
//! - Rate Monotonic priority assigment
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | linear *O(n)* complexity
//! - [`AnalysisSimple::is_schedulable`] \
//!   | limit approximation
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming
//!    in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61,
//!    Jan. 1973, doi: 10.1145/321738.321743.

use crate::prelude::*;

const ALGORITHM: &str = "Fixed Priority RM (Liu & Layland 1973)";

/// Fixed Priority Rate Monotonic, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis;

impl SchedAnalysis<(), PriorityTask<LLTask>> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else if !Taskset::<PriorityTask<LLTask>>::rate_monotonic_priority_assignment(taskset) {
            Err(SchedError::rate_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        // Theorem 5: let m = #Tasks, lub(Utilization) = m * (2^(1/m) - 1) [1]
        let total_utilization = Taskset::<LLTask>::total_utilization(taskset);
        let rate_monotonic_lub =
            (taskset.len() as f64) * (f64::powf(2.0, 1.0 / taskset.len() as f64) - 1.0);

        SchedError::result_from_schedulable(total_utilization <= rate_monotonic_lub)
    }
}

/// Fixed Priority Rate Monotonic, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
///
/// Use the limit approximation for the least upper bound on the total utilization.
pub struct AnalysisSimple;

impl SchedAnalysis<(), PriorityTask<LLTask>> for AnalysisSimple
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else if !Taskset::<PriorityTask<LLTask>>::rate_monotonic_priority_assignment(taskset) {
            Err(SchedError::rate_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        // Theorem 5 [1]
        let total_utilization = Taskset::<LLTask>::total_utilization(taskset);

        // Significant limit used for the derivation:
        //   forall a>0. lim x->0 ((a^x - 1) / x) = ln(a)
        let rate_monotonic_lub = f64::ln(2f64);

        SchedError::result_from_schedulable(total_utilization <= rate_monotonic_lub)
    }
}
