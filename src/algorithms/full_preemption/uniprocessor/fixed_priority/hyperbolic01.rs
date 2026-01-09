//! ## Fixed Priority RM Hyperbolic - Bini, Buttazzo, Buttazzo 2001
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
//!
//! ---
//! #### References:
//! 1. E. Bini, G. Buttazzo, and G. Buttazzo, “A hyperbolic bound for the rate
//!    monotonic algorithm,” in Proceedings 13th Euromicro Conference on Real-Time
//!    Systems, June 2001, pp. 59–66. doi: 10.1109/EMRTS.2001.934000.

use crate::prelude::*;

const ALGORITHM: &str = "Fixed Priority RM Hyperbolic (Bini, Buttazzo, Buttazzo 2001)";

/// Fixed Priority RM Hyperbolic, Bini, Buttazzo, Buttazzo 2001 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis;

impl SchedAnalysis<(), &[RTTask]> for Analysis {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !RTUtils::is_taskset_sorted_by_period(taskset) {
            Err(SchedError::rate_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<(), SchedError> {
        // Theorem 1 [1]
        let bound: f64 =
            taskset.iter()
            .map(|task| task.utilization() + 1.0)
            .product();

        SchedError::result_from_schedulable(bound <= 2.0)
    }
}