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
//! - [`is_schedulable`] \
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming
//!    in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61,
//!    Jan. 1973, doi: 10.1145/321738.321743.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Earliest Deadline First (Liu & Layland 1973)";

/// Earliest Deadline First, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<()> {
    if !RTUtils::implicit_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).implicit_deadlines();
    }

    let total_utilization = RTUtils::total_utilization(taskset);

    SchedResultFactory(ALGORITHM)
        .is_schedulable(total_utilization <= 1f64)
}