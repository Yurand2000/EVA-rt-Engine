//! Response Time Analysis - Joseph & Pandya 1986
//!
//! Preconditions:
//! - Fully Preemptive model
//! - Fixed Priority scheduling
//! - Constrained Deadlines
//!
//! Worst-Case Complexity:
//! - Pseudo-Polynomial
//!
//! References:
//! [1]: M. Joseph and P. Pandya, “Finding Response Times in a Real-Time System,”
//!      Comput J, vol. 29, no. 5, pp. 390–395, 1986, doi: 10.1093/comjnl/29.5.390.

use eva_rt_common::prelude::*;
use eva_rt_common::utils::RTUtils;

pub struct IsSchedulable<T> {
    data: Option<T>
}

impl<T> IsSchedulable<T> {
    pub fn new(data: Option<T>) -> Self {
        Self { data }
    }

    pub fn schedulable(data: T) -> Self {
        Self { data: Some(data) }
    }

    pub fn not_schedulable() -> Self {
        Self { data: None }
    }

    pub fn is_schedulable(&self) -> bool {
        self.data.is_some()
    }
}

impl<T> Into<Option<T>> for IsSchedulable<T> {
    fn into(self) -> Option<T> {
        self.data
    }
}

/// Check if the taskset is schedulable and (if schedulable) return the
/// Worst-Case Response Times of each task. The function will return error if
/// any of the preconditions is not met, while it will return a None if the
/// taskset is not schedulable.
pub fn is_schedulable(taskset: &[RTTask]) -> anyhow::Result<IsSchedulable<Vec<Time>>> {
    if !RTUtils::constrained_deadlines(taskset) {
        return Err(anyhow::format_err!("RTA (Joseph & Pandya 1986) works only \
                                        with constrained deadlines."))
    }

    if !avg_processing_load_is_met(taskset) {
        return Ok(IsSchedulable::not_schedulable());
    }

    let response_times = taskset.iter().enumerate()
        .map(|(i, task)| (task, response_time(&taskset[0..=i])) )
        .fold(Some(Vec::with_capacity(taskset.len())), |acc, (task, response_time)| {
            let mut acc = acc?;

            if response_time > task.deadline {
                None
            } else {
                acc.push(response_time);
                Some(acc)
            }
        });

    Ok(IsSchedulable::new(response_times))
}

// Condition 4
fn avg_processing_load_is_met(taskset: &[RTTask]) -> bool {
    let hyperperiod = RTUtils::hyperperiod(taskset);

    required_resources_over_interval(taskset, hyperperiod) < hyperperiod
}

// Function 3 + Function 2
fn required_resources_over_interval(taskset: &[RTTask], interval: Time) -> Time {
    taskset.iter()
        .map(|task| f64::ceil(interval / task.period) * task.wcet)
        .sum()
}

// Equation 6 + Function 5
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
// Example 2
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

    assert!(!is_schedulable(&taskset).unwrap().is_schedulable());
}