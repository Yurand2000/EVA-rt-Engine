//! ## Fixed Priority Deadline Monotonic - Audsley 1990
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed-Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//! - Deadline Monotonic priority assigment
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | linear *O(n^2)* complexity
//!
//! ---
//! #### References:
//! 1. N. C. Audsley, “Deadline Monotonic Scheduling,” Sept. 1990.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Fixed Priority DM (Audsley 1990)";

/// Fixed Priority Deadline Monotonic, Audsley 1990 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    if !RTUtils::is_taskset_sorted_by_deadline(taskset) {
        return SchedResultFactory(ALGORITHM).deadline_monotonic();
    }

    // Equation 8
    #[inline(always)]
    fn interference(taskset: &[RTTask]) -> Time {
        if taskset.len() == 0 {
            return Time::zero();
        }

        let last_task = taskset.last().unwrap();

        taskset.iter()
            .take(taskset.len() - 1)
            .map(|task| (last_task.deadline / task.period).ceil() * task.wcet)
            .sum()
    }

    // Equation 8
    let schedulable =
        taskset.iter().enumerate()
        .all(|(i, task)| {
            task.wcet + interference(&taskset[0..=i]) <= task.deadline
        });

    if schedulable {
        SchedResultFactory(ALGORITHM).schedulable(())
    } else {
        SchedResultFactory(ALGORITHM).non_schedulable()
    }
}