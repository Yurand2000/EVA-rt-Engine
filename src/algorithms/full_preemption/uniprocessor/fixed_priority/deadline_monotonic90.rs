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

impl SchedAnalysis<(), PriorityTask<LLTask>> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !Taskset::<PriorityTask<LLTask>>::deadline_monotonic_priority_assignment(taskset) {
            Err(SchedError::deadline_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        // Equation 8 [1]
        let interference = |taskset: &[PriorityTask<LLTask>]| -> Time {
            if taskset.len() == 0 {
                return Time::zero();
            }

            let last_task = taskset.last().unwrap();

            taskset.iter()
                .take(taskset.len() - 1)
                .map(|task| (last_task.deadline / task.period).ceil() * task.wcet)
                .sum()
        };

        // Equation 8 [1]
        let schedulable =
            taskset.iter().enumerate()
            .all(|(i, task)| {
                task.wcet + interference(&taskset[0..=i]) <= task.deadline
            });

        SchedError::result_from_schedulable(schedulable)
    }
}
