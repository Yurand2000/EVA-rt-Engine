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

impl SchedAnalysis<Vec<Time>, PriorityTask<LLTask>> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !avg_processing_load_is_met(taskset) {
            Err(SchedError::Precondition(Some(
                anyhow::format_err!("average processing load is not met."))))
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<Vec<Time>, SchedError> {
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
fn avg_processing_load_is_met(taskset: &[PriorityTask<LLTask>]) -> bool
{
    let hyperperiod = Taskset::<LLTask>::hyperperiod(taskset);

    taskset.iter()
        .map(|task| required_resources_over_interval(task, hyperperiod))
        .sum::<Time>()
    <
        hyperperiod
}

// Function 3 + Function 2 [1]
fn required_resources_over_interval(task: &PriorityTask<LLTask>, interval: Time) -> Time
{
    f64::ceil(interval / task.period) * task.wcet
}

// Equation 6 + Function 5 [1]
fn response_time(task: &PriorityTask<LLTask>, taskset: &[PriorityTask<LLTask>]) -> Option<Time>
{
    debug_assert!(!taskset.is_empty());

    let response_time =
        fixpoint_search_with_limit(task.wcet, task.deadline + Time::one(), |response| {
            taskset.iter()
                .filter(|other| task.priority > other.priority)
                .map(|hp_task| required_resources_over_interval(hp_task, *response))
                .sum::<Time>()
                +
                task.wcet
        });

    if response_time <= task.deadline {
        Some(response_time)
    } else {
        None
    }
}

#[test]
// Example 2 [1]
fn example_2() {
    use crate::utils::test_utils::*;

    let taskset = [
        new_prio_task(40, 100, 100, 0),
        new_prio_task(60, 140, 140, 1),
        new_prio_task(80, 500, 500, 2),
        new_prio_task(10, 1000, 1000, 3),
        new_prio_task(1, 1000, 1000, 4),
    ];

    assert_eq!(response_time(&taskset[0], &taskset), Some(Time::nanos(40.0)));
    assert_eq!(response_time(&taskset[1], &taskset), Some(Time::nanos(100.0)));
    assert_eq!(response_time(&taskset[2], &taskset), None);
    assert_eq!(response_time(&taskset[3], &taskset), None);
    assert_eq!(response_time(&taskset[4], &taskset), None);

    assert!(Analysis.is_schedulable(&taskset).is_err());
}
