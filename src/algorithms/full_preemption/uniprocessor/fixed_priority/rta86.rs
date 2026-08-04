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

impl<T> SchedAnalysis<Vec<Time>, T> for Analysis
    where T: task_traits::LLTask + task_traits::Priority
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !avg_processing_load_is_met(taskset) {
            Err(SchedError::Precondition(Some(
                anyhow::format_err!("average processing load is not met."))))
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<Vec<Time>, SchedError> {
        taskset.iter().enumerate()
            .map(|(i, task)| {
                let response_time = response_time(task, taskset);

                if response_time.is_none() {
                    Err(SchedError::NonSchedulable(Some(
                        anyhow::format_err!("task {i} misses its deadline.")
                    )))
                } else {
                    Ok(response_time.unwrap())
                }
            })
            .collect()
    }
}

// Condition 4 [1]
fn avg_processing_load_is_met<T>(taskset: &[T]) -> bool
    where T: task_traits::LLTask + task_traits::Priority
{
    let hyperperiod = LLTasksetUtils::hyperperiod(taskset);

    taskset.iter()
        .map(|task| required_resources_over_interval(task, hyperperiod))
        .sum::<Time>()
    <
        hyperperiod
}

// Function 3 + Function 2 [1]
fn required_resources_over_interval<T>(task: &T, interval: Time) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    f64::ceil(interval / task.get_period()) * task.get_wcet()
}

// Equation 6 + Function 5 [1]
fn response_time<T>(task: &T, taskset: &[T]) -> Option<Time>
    where T: task_traits::LLTask + task_traits::Priority
{
    debug_assert!(!taskset.is_empty());

    let response_time =
        fixpoint_search_with_limit(task.get_wcet(), task.get_deadline() + Time::one(), |response| {
            taskset.iter()
                .filter(|other| task.get_priority() > other.get_priority())
                .map(|hp_task| required_resources_over_interval(hp_task, *response))
                .sum::<Time>()
                +
                task.get_wcet()
        });

    if response_time <= task.get_deadline() {
        Some(response_time)
    } else {
        None
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
