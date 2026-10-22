//! ## Multiprocessor Fixed Priority - Bertogna, Cirinei, Lipari 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | O(*n^2*) complexity
//!
//! ---
//! #### References:
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of
//!    Global Scheduling Algorithms on Multiprocessor Platforms,” IEEE
//!    Transactions on Parallel and Distributed Systems, vol. 20, no. 4, pp.
//!    553–566, Apr. 2009, doi: 10.1109/TPDS.2008.129.

use crate::prelude::*;

const ALGORITHM: &str = "Multiprocessor Fixed Priority (Bertogna, Cirinei, Lipari 2009)";

/// Multiprocessor Fixed Priority - Bertogna, Cirinei, Lipari 2009
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl SchedAnalysis<(), PriorityTask<LLTask>> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        // Theorem 8 [1]
        // Section 4 Equation 10
        let schedulable =
            taskset.iter()
            .all(|task_k|
                global_fixed_priority_demand(taskset, task_k)
                    <
                self.num_processors as f64 * (task_k.deadline - task_k.wcet + Time::one())
            );

            SchedError::result_from_schedulable(schedulable)
    }
}

pub fn global_fixed_priority_demand(taskset: &[PriorityTask<LLTask>], task_k: &PriorityTask<LLTask>) -> Time
{
    use crate::algorithms::full_preemption::global_multiprocessor
             ::earliest_deadline_first::bcl09::workload_upperbound;

    taskset.iter()
        .filter(|task_i| task_i.priority > task_k.priority)
        .map(|task_i| {
            Time::min(
                workload_upperbound(task_k.deadline, task_i),
                task_k.deadline - task_k.wcet + Time::one(),
            )
        })
        .sum()
}
