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
//! - [`is_schedulable`] \
//!   | linear *O(n)* complexity
//! - [`is_schedulable_simple`] \
//!   | limit approximation
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. C. L. Liu and J. W. Layland, “Scheduling Algorithms for Multiprogramming
//!    in a Hard-Real-Time Environment,” J. ACM, vol. 20, no. 1, pp. 46–61,
//!    Jan. 1973, doi: 10.1145/321738.321743.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Fixed Priority RM (Liu & Layland 1973)";

/// Fixed Priority Rate Monotonic, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<()> {
    if !RTUtils::is_taskset_sorted_by_period(taskset) {
        return SchedErrors(ALGORITHM).rate_monotonic();
    }

    if !RTUtils::implicit_deadlines(taskset) {
        return SchedErrors(ALGORITHM).implicit_deadlines();
    }

    // Theorem 5: let m = #Tasks, lub(Utilization) = m * (2^(1/m) - 1)
    let total_utilization = RTUtils::total_utilization(taskset);
    let rate_monotonic_lub =
        (taskset.len() as f64) * (f64::powf(2.0, 1.0 / taskset.len() as f64) - 1.0);

    if total_utilization <= rate_monotonic_lub {
        Ok(())
    } else {
        SchedErrors(ALGORITHM).non_schedulable()
    }
}

/// Fixed Priority Rate Monotonic, Liu & Layland 1973 \[1\]
///
/// Refer to the [module](`self`) level documentation.
///
/// Use the limit approximation for the least upper bound on the total utilization.
pub fn is_schedulable_simple(taskset: &[RTTask]) -> SchedResult<()> {
    if !RTUtils::is_taskset_sorted_by_period(taskset) {
        return SchedErrors(ALGORITHM).rate_monotonic();
    }

    if !RTUtils::implicit_deadlines(taskset) {
        return SchedErrors(ALGORITHM).implicit_deadlines();
    }

    // Theorem 5
    let total_utilization = RTUtils::total_utilization(taskset);

    // Significant limit used for the derivation:
    //   forall a>0. lim x->0 ((a^x - 1) / x) = ln(a)
    let rate_monotonic_lub = f64::ln(2f64);

    if total_utilization <= rate_monotonic_lub {
        Ok(())
    } else {
        SchedErrors(ALGORITHM).non_schedulable()
    }
}