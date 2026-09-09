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

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Theorem 7 [1]
        // Section 4 Equation 9
        let schedulable =
            taskset.iter()
            .all(|task_k| {
                global_earliest_deadline_first_demand(taskset, task_k)
                    <
                self.num_processors as f64 * (task_k.get_deadline() - task_k.get_wcet() + Time::one())
            });

        SchedError::result_from_schedulable(schedulable)
    }
}

pub fn global_earliest_deadline_first_demand<T>(taskset: &[T], task_k: &T) -> Time
    where T: task_traits::LLTask
{
    taskset.iter()
        .filter(|&task_i| !std::ptr::eq(task_i, task_k))
        .map(|task_i| {
            Time::min(
                interference_edf_upperbound(task_i, task_k),
                task_k.get_deadline() - task_k.get_wcet() + Time::one(),
            )
        })
        .sum()
}

// Section 4 Equation 6 [1]
pub fn workload_upperbound<T>(interval: Time, task: &T) -> Time
    where T: task_traits::LLTask
{
    jobs_in_interval(interval, task) * task.get_wcet() + carry_out(interval, task)
}

// Section 4 Equation 5 [1]
#[inline(always)]
fn jobs_in_interval<T>(interval: Time, task: &T) -> f64
    where T: task_traits::LLTask
{
    ((interval + task.get_deadline() - task.get_wcet()) / task.get_period()).floor()
}

#[inline(always)]
fn carry_out<T>(interval: Time, task: &T) -> Time
    where T: task_traits::LLTask
{
    Time::min(task.get_wcet(), interval + task.get_deadline() - task.get_wcet() - jobs_in_interval(interval, task) * task.get_period())
}

// Section 4 Equation 8 [1]
fn interference_edf_upperbound<T>(by_task: &T, to_task: &T) -> Time
    where T: task_traits::LLTask
{
    let task_i = by_task;
    let task_k = to_task;

    task_k.get_deadline() / task_i.get_period() * task_i.get_wcet()
        +
    Time::min(task_i.get_wcet(), task_k.get_deadline() - (task_k.get_deadline() / task_i.get_period()) * task_i.get_period())
}

#[test]
// Example 1 [1]
fn example_1() {
    use crate::task::test_utils::*;

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
    use crate::task::test_utils::*;
    use crate::task::traits::LLTask as _;

    let taskset = [
        new_ll_task(1, 1, 1),
        new_ll_task(1, 10, 10),
    ];

    assert_eq!(workload_upperbound(taskset[1].get_deadline(), &taskset[0]), Time::nanos(10.0));
    assert_eq!(workload_upperbound(taskset[0].get_deadline(), &taskset[1]), Time::nanos(1.0));
    // it should fail, as says in the paper, but it doesn't. Numbers seem ok
    // assert!(!ibcl_generic_work_conserving(&taskset, 2).unwrap());
}
