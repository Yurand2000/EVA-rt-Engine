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

/// Fixed Priority RM Hyperbolic, Bini, Buttazzo, Buttazzo 2001 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<()> {
    assert_rate_monotonic_assignment(taskset)?;
    assert_implicit_deadlines(taskset)?;

    // Theorem 1
    let bound: f64 =
        taskset.iter()
        .map(|task| task.utilization() + 1f64)
        .product();

    if bound <= 2f64 {
        Ok(())
    } else {
        SchedError::non_schedulable(anyhow::format_err!(
            "Fixed Priority RM Hyperbolic (Bini, Buttazzo, Buttazzo 2001), non schedulable"))
    }
}

fn assert_implicit_deadlines(taskset: &[RTTask]) -> SchedResult<()> {
    if RTUtils::implicit_deadlines(taskset) {
        Ok(())
    } else {
        SchedError::precondition(anyhow::format_err!(
            "Fixed Priority RM Hyperbolic (Bini, Buttazzo, Buttazzo 2001), taskset must have implicit deadlines"))
    }
}

fn assert_rate_monotonic_assignment(taskset: &[RTTask]) -> SchedResult<()> {
    if RTUtils::is_taskset_sorted_by_period(taskset) {
        Ok(())
    } else {
        SchedError::precondition(anyhow::format_err!(
            "Fixed Priority RM Hyperbolic (Bini, Buttazzo, Buttazzo 2001), taskset must be sorted by period"))
    }
}