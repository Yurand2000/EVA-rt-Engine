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
//! - [`is_schedulable`] \
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. E. Bini, G. Buttazzo, and G. Buttazzo, “A hyperbolic bound for the rate
//!    monotonic algorithm,” in Proceedings 13th Euromicro Conference on Real-Time
//!    Systems, June 2001, pp. 59–66. doi: 10.1109/EMRTS.2001.934000.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Fixed Priority RM Hyperbolic (Bini, Buttazzo, Buttazzo 2001)";

/// Fixed Priority RM Hyperbolic, Bini, Buttazzo, Buttazzo 2001 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<()> {
    if !RTUtils::is_taskset_sorted_by_period(taskset) {
        return SchedResultFactory(ALGORITHM).rate_monotonic();
    }

    if !RTUtils::implicit_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).implicit_deadlines();
    }

    // Theorem 1
    let bound: f64 =
        taskset.iter()
        .map(|task| task.utilization() + 1f64)
        .product();

    if bound <= 2f64 {
        SchedResultFactory(ALGORITHM).schedulable(())
    } else {
        SchedResultFactory(ALGORITHM).non_schedulable()
    }
}