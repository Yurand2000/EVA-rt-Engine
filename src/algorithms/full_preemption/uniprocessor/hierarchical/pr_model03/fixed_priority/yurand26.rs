//! ## Periodic Resource Model, Fixed Priority Local Scheduling - * 2026
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
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. Unpublished
//! 2. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;
use crate::algorithms::full_preemption::uniprocessor::hierarchical::pr_model03::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Periodic Resource Model, Fixed Priority Local Scheduling (* 2026)";

/// Periodic Resource Model, Fixed Priority Local Scheduling - * 2026 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], model: &PRModel) -> SchedResult<Vec<Time>> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Equation 14 [2]
    let result: Result<Vec<_>, _> =
        taskset.iter().enumerate()
        .map(|(k, task_k)| {
            let response =
                fixpoint_search_with_limit(
                    task_k.wcet,
                    task_k.deadline + Time::nanos(1.0),
                    |response: &Time|
                        rta(taskset, k, task_k, *response, model)
                );

            if response > task_k.deadline {
                return Err(anyhow::format_err!("Response time of task {k} is greater than its deadline."));
            } else {
                return Ok(response);
            }
        })
        .collect();

    SchedResult {
        test_name: ALGORITHM.to_owned(),
        result: result.map_err(|err| SchedError::NonSchedulable(Some(err))),
    }
}

fn rta(taskset: &[RTTask], k: usize, task_k: &RTTask, response: Time, model: &PRModel) -> Time {
    // Standard RTA analysis [2]
    taskset.iter()
        .take(k)
        .map(|task_i| (response / task_i.period).ceil() * task_i.wcet)
        .sum::<Time>()
    +
        task_k.wcet
    // Periodic server's delay [1]
    +
        ((response + model.resource) / model.period).ceil() * (model.period - model.resource)
}

#[test]
fn equal_to_shin_lee03_rta()
{
    // TODO: add taskset generator
    let taskset = [
        RTTask::new_ns(2, 50, 100),
        RTTask::new_ns(49, 100, 100),
        RTTask::new_ns(34, 100, 100),
    ];

    for period in time_range_iterator_w_step(Time::nanos(10.0), Time::nanos(100.0), Time::nanos(1.0)) {
        for resource in time_range_iterator_w_step(period / 10.0, period, Time::nanos(1.0)) {
            let model = PRModel { resource, period };

            let shin_lee_test = super::shin_lee03::is_schedulable(&taskset, &model);
            let new_test = is_schedulable(&taskset, &model);

            assert_eq!(shin_lee_test.is_schedulable(), new_test.is_schedulable(), "{:?}", model);

            if !shin_lee_test.is_schedulable() {
                continue;
            }

            assert_eq!(shin_lee_test.result.unwrap(), new_test.result.unwrap(), "{:?}", model);
        }
    }
}