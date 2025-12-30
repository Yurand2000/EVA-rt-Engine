//! ## Response Time Analysis - Joseph & Pandya 1986
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed-Priority scheduling
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
//! 1. M. Joseph and P. Pandya, “Finding Response Times in a Real-Time System,”
//!    Comput J, vol. 29, no. 5, pp. 390–395, 1986, doi: 10.1093/comjnl/29.5.390.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "RTA (Joseph & Pandya 1986)";

/// Response Time Analysis, Joseph & Pandya 1986 \[1\]
///
/// Refer to the [module](`self`) level documentation.
///
/// Returns:
/// - Worst-Case Response Times of each task.
pub fn is_schedulable(taskset: &[RTTask]) -> SchedResult<Vec<Time>> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    if !avg_processing_load_is_met(taskset) {
        return SchedResultFactory(ALGORITHM).non_schedulable_reason(
            anyhow::format_err!("average processing load is not met."));
    }

    match taskset.iter().enumerate()
        .map(|(i, task)| {
            let response_time = response_time(&taskset[0..=i]);

            if response_time > task.deadline {
                Err(anyhow::format_err!("task {i} misses its deadline."))
            } else {
                Ok(response_time)
            }
        })
        .collect()
    {
        Ok(response_times) =>
            SchedResultFactory(ALGORITHM).schedulable(response_times),
        Err(error) =>
            SchedResultFactory(ALGORITHM).non_schedulable_reason(error),
    }

}

// Condition 4 [1]
fn avg_processing_load_is_met(taskset: &[RTTask]) -> bool {
    let hyperperiod = RTUtils::hyperperiod(taskset);

    required_resources_over_interval(taskset, hyperperiod) < hyperperiod
}

// Function 3 + Function 2 [1]
fn required_resources_over_interval(taskset: &[RTTask], interval: Time) -> Time {
    taskset.iter()
        .map(|task| f64::ceil(interval / task.period) * task.wcet)
        .sum()
}

// Equation 6 + Function 5 [1]
fn response_time(taskset: &[RTTask]) -> Time {
    if taskset.is_empty() {
        return Time::zero();
    }

    let task = taskset.last().unwrap();
    let hp_tasks = &taskset[0..taskset.len() - 1];

    let mut response = task.wcet;
    loop {
        let new_response = required_resources_over_interval(hp_tasks, response) + task.wcet;
        if new_response == response {
            return response;
        }

        response = new_response;
    }
}

#[test]
// Example 2 [1]
fn example_2() {
    let taskset = [
        RTTask::new_ns(40, 100, 100),
        RTTask::new_ns(60, 140, 140),
        RTTask::new_ns(80, 500, 500),
        RTTask::new_ns(10, 1000, 1000),
        RTTask::new_ns(1, 1000, 1000),
    ];

    assert_eq!(response_time(&taskset[0..=0]), Time::nanos(40.0));
    assert_eq!(response_time(&taskset[0..=1]), Time::nanos(100.0));

    // Response Times (computed with this algorithm) greater than the period are
    // not the Worst Case Response Times of the given task, as we do not account
    // for self-interference.
    assert_eq!(response_time(&taskset[0..=2]), Time::nanos(560.0));
    assert_eq!(response_time(&taskset[0..=3]), Time::nanos(2490.0));
    assert_eq!(response_time(&taskset[0..=4]), Time::nanos(6991.0));

    assert!(is_schedulable(&taskset).is_not_schedulable());
}