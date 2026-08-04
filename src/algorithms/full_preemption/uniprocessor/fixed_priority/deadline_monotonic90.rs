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
//! - [`Analysis::is_schedulable`] \
//!   | linear *O(n^2)* complexity
//!
//! ---
//! #### References:
//! 1. N. C. Audsley, “Deadline Monotonic Scheduling,” Sept. 1990.

use crate::prelude::*;

const ALGORITHM: &str = "Fixed Priority DM (Audsley 1990)";

/// Fixed Priority Deadline Monotonic, Audsley 1990 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis;

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask + task_traits::Priority
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !LLTasksetUtils::deadline_monotonic_priority_assignment(taskset) {
            Err(SchedError::deadline_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Equation 8 [1]
        let interference = |taskset: &[T]| -> Time {
            if taskset.len() == 0 {
                return Time::zero();
            }

            let last_task = taskset.last().unwrap();

            taskset.iter()
                .take(taskset.len() - 1)
                .map(|task| (last_task.get_deadline() / task.get_period()).ceil() * task.get_wcet())
                .sum()
        };

        // Equation 8 [1]
        let schedulable =
            taskset.iter().enumerate()
            .all(|(i, task)| {
                task.get_wcet() + interference(&taskset[0..=i]) <= task.get_deadline()
            });

        SchedError::result_from_schedulable(schedulable)
    }
}
