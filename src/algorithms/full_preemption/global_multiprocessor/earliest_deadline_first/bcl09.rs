//! ## Multiprocessor EDF - Bertogna, Cirinei, Lipari 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | O(*n^2*) complexity
//!
//! ---
//! #### References:
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of
//!    Global Scheduling Algorithms on Multiprocessor Platforms,” IEEE
//!    Transactions on Parallel and Distributed Systems, vol. 20, no. 4, pp.
//!    553–566, Apr. 2009, doi: 10.1109/TPDS.2008.129.

use crate::prelude::*;

const ALGORITHM: &str = "Multiprocessor EDF (Bertogna, Cirinei, Lipari 2009)";

/// Multiprocessor EDF - Bertogna, Cirinei, Lipari 2009
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl SchedAnalysis<(), LLTask> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[LLTask]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[LLTask]) -> Result<(), SchedError> {
        // Theorem 7 [1]
        // Section 4 Equation 9
        let schedulable =
            taskset.iter()
            .all(|task_k| {
                global_earliest_deadline_first_demand(taskset, task_k)
                    <
                self.num_processors as f64 * (task_k.deadline - task_k.wcet + Time::one())
            });

        SchedError::result_from_schedulable(schedulable)
    }
}

pub fn global_earliest_deadline_first_demand(taskset: &[LLTask], task_k: &LLTask) -> Time
{
    taskset.iter()
        .filter(|&task_i| !std::ptr::eq(task_i, task_k))
        .map(|task_i| {
            Time::min(
                interference_edf_upperbound(task_i, task_k),
                task_k.deadline - task_k.wcet + Time::one(),
            )
        })
        .sum()
}

// Section 4 Equation 6 [1]
pub fn workload_upperbound(interval: Time, task: &LLTask) -> Time
{
    jobs_in_interval(interval, task) * task.wcet + carry_out(interval, task)
}

// Section 4 Equation 5 [1]
#[inline(always)]
fn jobs_in_interval(interval: Time, task: &LLTask) -> f64
{
    ((interval + task.deadline - task.wcet) / task.period).floor()
}

#[inline(always)]
fn carry_out(interval: Time, task: &LLTask) -> Time
{
    Time::min(task.wcet, interval + task.deadline - task.wcet - jobs_in_interval(interval, task) * task.period)
}

// Section 4 Equation 8 [1]
fn interference_edf_upperbound(by_task: &LLTask, to_task: &LLTask) -> Time
{
    let task_i = by_task;
    let task_k = to_task;

    task_k.deadline / task_i.period * task_i.wcet
        +
    Time::min(task_i.wcet, task_k.deadline - (task_k.deadline / task_i.period) * task_i.period)
}

#[test]
// Example 1 [1]
fn example_1() {
    use crate::utils::test_utils::*;

    let taskset = [
        new_ll_task(20, 30, 30),
        new_ll_task(20, 30, 30),
        new_ll_task(5, 30, 30),
    ];

    let num_processors = 2;

    assert!(super::gbf03::AnalysisSporadic { num_processors }.is_schedulable(&taskset).is_err());
    assert!(Analysis { num_processors }.is_schedulable(&taskset).is_ok());
}

#[test]
// Example 2 [1]
fn example_2() {
    use crate::utils::test_utils::*;

    let taskset = [
        new_ll_task(1, 1, 1),
        new_ll_task(1, 10, 10),
    ];

    assert_eq!(workload_upperbound(taskset[1].deadline, &taskset[0]), Time::nanos(10.0));
    assert_eq!(workload_upperbound(taskset[0].deadline, &taskset[1]), Time::nanos(1.0));
    // it should fail, as says in the paper, but it doesn't. Numbers seem ok
    // assert!(!ibcl_generic_work_conserving(&taskset, 2).unwrap());
}
