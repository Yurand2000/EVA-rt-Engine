//! ## Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003
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

const ALGORITHM: &str = "Periodic Resource Model, EDF Local Scheduling (Shin & Lee 2003)";

/// Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], model: &PRModel) -> SchedResult<()> {
    if !RTUtils::implicit_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).implicit_deadlines();
    }

    fn demand(taskset: &[RTTask], interval: Time) -> Time {
        // Section 4.1 [1]
        taskset.iter()
            .map(|task| (interval / task.period).floor() * task.wcet)
            .sum()
    }

    // Theorem 1 [1]
    fn time_intervals(taskset: &[RTTask]) -> Box<dyn Iterator<Item = Time>> {
        let max_time = RTUtils::hyperperiod(taskset) * 2.0;

        Box::new(
            (0 ..= max_time.as_nanos() as u64)
                .map(|time_ns| Time::nanos(time_ns as f64))
        )
    }

    let schedulable =
        is_schedulable_demand(
            taskset,
            model,
            demand,
            time_intervals
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}