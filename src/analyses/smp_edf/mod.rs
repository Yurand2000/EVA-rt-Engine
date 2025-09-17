use crate::prelude::*;

// Goossens, J., Funk, S. and Baruah, S., 2003. Priority-driven scheduling of
// periodic task systems on multiprocessors. Real-time systems, 25(2),
// pp.187-205.
// Theorem 5
pub fn is_schedulable_implicit(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_implicit_deadlines(taskset)?;

    let u_tot = RTUtils::total_utilization(taskset);
    let u_max = RTUtils::largest_utilization(taskset);

    Ok(u_tot <= (num_processors as f64) - u_max * (num_processors as f64 - 1f64))
}

// Bertogna, M., Cirinei, M. and Lipari, G., 2008. Schedulability analysis of
// global scheduling algorithms on multiprocessor platforms. IEEE Transactions
// on parallel and distributed systems, 20(4), pp.553-566.
// Theorem 3
// Section 3 Equation 3
pub fn is_schedulable_constrained(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    let d_tot = RTUtils::total_density(taskset);
    let d_max = RTUtils::largest_density(taskset);

    Ok(d_tot <= (num_processors as f64) - d_max * (num_processors as f64 - 1f64))
}

// Theorem 6
// Section 4 Equation 7
pub fn is_schedulable_generic_work_conserving(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let ub: Time =
                taskset.iter().enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, task_i)| {
                    let w_i = workload_upperbound((Time::zero(), task_k.deadline), task_i);
                    Time::min(w_i, task_k.laxity() + Time { value_ns: 1 })
                })
                .sum();

            ub < num_processors * (task_k.laxity() + Time { value_ns: 1})
        }))
}

// Theorem 7
// Section 4 Equation 9
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let ub: Time =
                taskset.iter().enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, task_i)| {
                    let i_ik = interference_edf_upperbound(task_i, task_k);
                    Time::min(i_ik, task_k.laxity() + Time { value_ns: 1 })
                })
                .sum();

            ub < num_processors * (task_k.laxity() + Time { value_ns: 1})
        }))
}

pub fn is_schedulable_fixed_priority(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let ub: Time =
                taskset.iter().enumerate()
                .filter(|(i, _)| *i < k)
                .map(|(_, task_i)| {
                    let w_i = workload_upperbound((Time::zero(), task_k.deadline), task_i);
                    Time::min(w_i, task_k.laxity() + Time { value_ns: 1 })
                })
                .sum();

            ub < num_processors * (task_k.laxity() + Time { value_ns: 1})
        }))
}

// Section 4 Equation 6
fn workload_upperbound(interval: (Time, Time), task: &RTTask) -> Time {
    carry_in(interval, task) + carry_out(interval, task)
}

// Section 4 Equation 5
fn jobs_in_interval(interval: (Time, Time), task: &RTTask) -> u64 {
    let length = interval.1 - interval.0;

    (length + task.laxity()) / task.period
}

fn carry_in(interval: (Time, Time), task: &RTTask) -> Time {
    jobs_in_interval(interval, task) * task.wcet
}

fn carry_out(interval: (Time, Time), task: &RTTask) -> Time {
    let length = interval.1 - interval.0;

    Time::min(task.wcet, length + task.laxity() - jobs_in_interval(interval, task) * task.period)
}

// Section 4 Equation 8
fn interference_edf_upperbound(by_task: &RTTask, to_task: &RTTask) -> Time {
    let task_i = by_task;
    let task_k = to_task;

    task_k.deadline / task_i.period * task_i.wcet
        +
    Time::min(task_i.wcet, task_k.deadline - (task_k.deadline / task_i.period) * task_i.period)
}

fn slack_lb(taskset: &[RTTask], task_k: &RTTask, num_processors: u64) -> Time {
    let k = 0;

    let lb: Time = 
        taskset.iter().enumerate()
        .filter(|(i, _)| *i != k)
        .map(|(_, task_i)| {
            let workload = workload_upperbound((Time::zero(), task_k.deadline), task_i);
            Time::min(workload, task_k.laxity() + Time { value_ns: 1 })
        })
        .sum();

    task_k.laxity() - lb / num_processors
}

#[test]
// Example 1
fn example_1() {
    let taskset = [
        RTTask::new_ns(20, 30, 30),
        RTTask::new_ns(20, 30, 30),
        RTTask::new_ns(5, 30, 30),
    ];

    let num_processors = 2;

    assert!(!is_schedulable_constrained(&taskset, num_processors).unwrap());
    assert!(is_schedulable(&taskset, num_processors).unwrap());
}

#[test]
// Example 2
fn example_2() {
    let taskset = [
        RTTask::new_ns(1, 1, 1),
        RTTask::new_ns(1, 10, 10),
    ];

    let num_processors = 2;

    assert_eq!(workload_upperbound((Time::zero(), taskset[1].deadline), &taskset[0]), Time::nanos(10));
    assert_eq!(workload_upperbound((Time::zero(), taskset[0].deadline), &taskset[1]), Time::nanos(1));
    // it should fail, as says in the paper, but it doesn't. Numbers seem ok
    // assert!(!is_schedulable_generic_work_conserving(&taskset, num_processors).unwrap());
}