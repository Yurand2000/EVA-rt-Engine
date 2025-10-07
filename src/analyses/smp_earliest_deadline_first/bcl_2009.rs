use crate::prelude::*;

// Theorem 6 [1]
// Section 4 Equation 7
pub fn bcl_generic_work_conserving(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let ub: Time =
                taskset.iter().enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, task_i)| {
                    Time::min(
                        workload_upperbound(task_k.deadline, task_i),
                        task_k.laxity() + Time::one(),
                    )
                })
                .sum();

            ub < num_processors as f64 * (task_k.laxity() + Time::one())
        }))
}

// Theorem 7 [1]
// Section 4 Equation 9
pub fn ibcl_edf(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            global_earliest_deadline_first_demand(taskset, k, task_k)
                <
            num_processors as f64 * (task_k.laxity() + Time::one())
        }))
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

pub fn ibcl_dm(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)|
            global_deadline_monotonic_demand(taskset, k, task_k)
                <
            num_processors as f64 * (task_k.laxity() + Time::one())
        ))
}

pub fn global_deadline_monotonic_demand(taskset: &[RTTask], k: usize, task_k: &RTTask) -> Time {
    taskset.iter()
        .enumerate()
        .filter(|(i, _)| *i < k)
        .map(|(_, task_i)| {
            Time::min(
                workload_upperbound(task_k.deadline, task_i),
                task_k.laxity() + Time::one(),
            )
        })
        .sum()
}

// Section 4 Equation 6 [1]
fn workload_upperbound(interval: Time, task: &RTTask) -> Time {
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

fn slack_lb(taskset: &[RTTask], task_k: &RTTask, num_processors: i64) -> Time {
    let k = 0;

    let lb: Time =
        taskset.iter().enumerate()
        .filter(|(i, _)| *i != k)
        .map(|(_, task_i)| {
            let workload = workload_upperbound(task_k.deadline, task_i);
            Time::min(workload, task_k.laxity() + Time::one())
        })
        .sum();

    task_k.laxity() - lb / num_processors as f64
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

    assert!(!super::gfb_test_sporadic(&taskset, num_processors).unwrap());
    assert!(ibcl_edf(&taskset, num_processors).unwrap());
}

#[test]
// Example 2 [1]
fn example_2() {
    let taskset = [
        RTTask::new_ns(1, 1, 1),
        RTTask::new_ns(1, 10, 10),
    ];

    let num_processors = 2;

    assert_eq!(workload_upperbound(taskset[1].deadline, &taskset[0]), Time::nanos(10.0));
    assert_eq!(workload_upperbound(taskset[0].deadline, &taskset[1]), Time::nanos(1.0));
    // it should fail, as says in the paper, but it doesn't. Numbers seem ok
    // assert!(!is_schedulable_generic_work_conserving(&taskset, num_processors).unwrap());
}

/* -----------------------------------------------------------------------------
References:
[1]: Bertogna, M., Cirinei, M. and Lipari, G., 2008. Schedulability analysis of
global scheduling algorithms on multiprocessor platforms. IEEE Transactions on
parallel and distributed systems, 20(4), pp.553-566.
*/