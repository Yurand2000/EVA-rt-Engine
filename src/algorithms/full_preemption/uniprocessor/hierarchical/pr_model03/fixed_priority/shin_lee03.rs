//! ## Periodic Resource Model, Fixed Priority Local Scheduling - Shin & Lee 2003
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | pseudo-polynomial complexity \
//!
//! ---
//! #### References:
//! 1. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;
use crate::algorithms::full_preemption::uniprocessor::hierarchical::pr_model03::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Periodic Resource Model, Fixed Priority Local Scheduling (Shin & Lee 2003)";

/// Periodic Resource Model, Fixed Priority Local Scheduling - Shin & Lee 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], model: &PRModel) -> SchedResult<Vec<Time>> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Equation 10 [1]
    fn rta(taskset: &[RTTask], k: usize, task_k: &RTTask, response: Time) -> Time {
        taskset.iter()
            .take(k - 1)
            .map(|task_i| (response / task_i.period).ceil() * task_i.wcet)
            .sum::<Time>()
        +
            task_k.wcet
    }

    let result =
        is_schedulable_response(
            taskset,
            model,
            rta,
        );

    SchedResult {
        test_name: ALGORITHM.to_owned(),
        result: result.map_err(|err| SchedError::NonSchedulable(Some(err))),
    }
}