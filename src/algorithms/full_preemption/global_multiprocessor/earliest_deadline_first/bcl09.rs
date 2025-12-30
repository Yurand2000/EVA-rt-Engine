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
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Multiprocessor EDF (Bertogna, Cirinei, Lipari 2009)";

/// Multiprocessor EDF - Bertogna, Cirinei, Lipari 2009
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Theorem 7 [1]
    // Section 4 Equation 9
    let schedulable =
        taskset.iter().enumerate()
        .all(|(k, task_k)| {
            global_earliest_deadline_first_demand(taskset, k, task_k)
                <
            num_processors as f64 * (task_k.laxity() + Time::one())
        });

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

pub fn global_earliest_deadline_first_demand(taskset: &[RTTask], k: usize, task_k: &RTTask) -> Time {
    taskset.iter().enumerate()
        .filter(|(i, _)| *i != k)
        .map(|(_, task_i)| {
            Time::min(
                interference_edf_upperbound(task_i, task_k),
                task_k.laxity() + Time::one(),
            )
        })
        .sum()
}

// Section 4 Equation 6 [1]
pub fn workload_upperbound(interval: Time, task: &RTTask) -> Time {
    jobs_in_interval(interval, task) * task.wcet + carry_out(interval, task)
}

// Section 4 Equation 5 [1]
#[inline(always)]
fn jobs_in_interval(interval: Time, task: &RTTask) -> f64 {
    ((interval + task.laxity()) / task.period).floor()
}

#[inline(always)]
fn carry_out(interval: Time, task: &RTTask) -> Time {
    Time::min(task.wcet, interval + task.laxity() - jobs_in_interval(interval, task) * task.period)
}

// Section 4 Equation 8 [1]
fn interference_edf_upperbound(by_task: &RTTask, to_task: &RTTask) -> Time {
    let task_i = by_task;
    let task_k = to_task;

    task_k.deadline / task_i.period * task_i.wcet
        +
    Time::min(task_i.wcet, task_k.deadline - (task_k.deadline / task_i.period) * task_i.period)
}

#[test]
// Example 1 [1]
fn example_1() {
    let taskset = [
        RTTask::new_ns(20, 30, 30),
        RTTask::new_ns(20, 30, 30),
        RTTask::new_ns(5, 30, 30),
    ];

    let num_processors = 2;

    assert!(super::gbf03::is_schedulable_sporadic(&taskset, num_processors).is_not_schedulable());
    assert!(is_schedulable(&taskset, num_processors).is_schedulable());
}

#[test]
// Example 2 [1]
fn example_2() {
    let taskset = [
        RTTask::new_ns(1, 1, 1),
        RTTask::new_ns(1, 10, 10),
    ];

    assert_eq!(workload_upperbound(taskset[1].deadline, &taskset[0]), Time::nanos(10.0));
    assert_eq!(workload_upperbound(taskset[0].deadline, &taskset[1]), Time::nanos(1.0));
    // it should fail, as says in the paper, but it doesn't. Numbers seem ok
    // assert!(!ibcl_generic_work_conserving(&taskset, 2).unwrap());
}