//! ## Fixed Priority Rate Monotonic - Liu & Layland 1973
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Implicit Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming
//!    in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61,
//!    Jan. 1973, doi: 10.1145/321738.321743.

use crate::prelude::*;

const ALGORITHM: &str = "Earliest Deadline First (Liu & Layland 1973)";

/// Earliest Deadline First, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis;

impl SchedAnalysis<(), &[RTTask]> for Analysis {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<(), SchedError> {
        let total_utilization = RTUtils::total_utilization(taskset);

        SchedError::result_from_schedulable(
            total_utilization <= 1f64
        )
    }
}