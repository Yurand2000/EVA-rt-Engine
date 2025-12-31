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
//!   | pseudo-polynomial complexity
//! - [`generate_model_linear`] \
//!   | Generate the suitable interface given the taskset and the [`PRModel`]'s period. \
//!   | \
//!   | pseudo-polynomial complexity
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

    // Equation 9 [1]
    let schedulable =
        is_schedulable_demand(
            taskset,
            model,
            demand,
            time_intervals
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

/// Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003 \[1\] \
/// Derive the best [`PRModel`] using demand analysis.
///
/// Refer to the [module](`self`) level documentation.
pub fn generate_model_linear(taskset: &[RTTask], model_period: Time) -> DesignResult<PRModel> {
    if !RTUtils::implicit_deadlines(taskset) {
        return DesignResultFactory(ALGORITHM).implicit_deadlines();
    }

    // Equation 16 [1]
    let model =
        generate_model_from_demand_linear(
            taskset,
            model_period,
            demand,
            time_intervals
        );

    DesignResultFactory(ALGORITHM).from_option(model)
}

// Section 4.1 [1]
fn demand(taskset: &[RTTask], interval: Time) -> Time {
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