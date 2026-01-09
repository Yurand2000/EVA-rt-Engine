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
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. M. Joseph and P. Pandya, “Finding Response Times in a Real-Time System,”
//!    Comput J, vol. 29, no. 5, pp. 390–395, 1986, doi: 10.1093/comjnl/29.5.390.

use crate::prelude::*;

const ALGORITHM: &str = "RTA (Joseph & Pandya 1986)";

/// Response Time Analysis, Joseph & Pandya 1986 \[1\]
///
/// Refer to the [module](`self`) level documentation.
///
/// Returns:
/// - Worst-Case Response Times of each task.
pub struct Analysis;

impl SchedAnalysis<Vec<Time>, &[RTTask]> for Analysis {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !avg_processing_load_is_met(taskset) {
            Err(SchedError::Precondition(Some(
                anyhow::format_err!("average processing load is not met."))))
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<Vec<Time>, SchedError> {
        taskset.iter().enumerate()
            .map(|(i, task)| {
                let response_time = response_time(&taskset[0..=i]);

                if response_time > task.deadline {
                    Err(SchedError::NonSchedulable(Some(
                        anyhow::format_err!("task {i} misses its deadline.")
                    )))
                } else {
                    Ok(response_time)
                }
            })
            .collect()
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

    assert!(Analysis.is_schedulable(&taskset).is_err());
}