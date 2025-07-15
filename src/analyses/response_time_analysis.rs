use crate::prelude::*;

#[derive(Clone)]
#[derive(Debug)]
pub enum Error {
    
}

// Joseph, M. and Pandya, P., 1986. Finding response times in a real-time
// system. The Computer Journal, 29(5), pp.390-395.
pub fn is_schedulable(taskset: &[RTTask], buffered_inputs: bool) -> Result<bool, Error> {
    if !avg_processing_load_is_met(taskset) {
        return Ok(false);
    }

    let schedulable =
        taskset.iter().enumerate()
        .all(|(i, task)| {
            let max_response_time =
                if buffered_inputs {
                    task.deadline
                } else {
                    Time::min(task.period, task.deadline)
                };

            response_time(&taskset[0..=i]) <= max_response_time
        });

    Ok(schedulable)
}

// Function 2
// Max number of activation in the given time interval
fn inputs(interval: (Time, Time), task: &RTTask) -> u64 {
    let (start, end) = interval;

    f64::ceil( (end.value_ns as f64) / (task.period.value_ns as f64) ) as u64 - 
    f64::ceil( (start.value_ns as f64) / (task.period.value_ns as f64) ) as u64
}

// Function 3
// Max computation time for the given task(sub)set in the given interval.
fn comp(interval: (Time, Time), tasksubset: &[RTTask]) -> Time {
    let total_computation_time =
        tasksubset.iter()
        .take(tasksubset.len() - 1)
        .map(|task| inputs(interval, task) * task.wcet.value_ns)
        .sum();

    Time::nanos(total_computation_time)
}

// Condition 4
fn avg_processing_load_is_met(tasksubset: &[RTTask]) -> bool {
    let period_lcm = tasksubset.iter()
        .map(|task| task.period.value_ns)
        .fold(1, |lcm, period| num::integer::lcm(lcm, period));
    
    let avg_load: u64 = 
        tasksubset.iter()
        .take(tasksubset.len() - 1)
        .map(|task| period_lcm / task.period.value_ns * task.wcet.value_ns)
        .sum();

    avg_load < period_lcm
}

// Function 5
fn response(interval: (Time, Time), tasksubset: &[RTTask]) -> Time {
    if comp(interval, tasksubset) == Time::zero() {
        interval.1
    } else {
        response((interval.1, interval.1 + comp(interval, tasksubset)), tasksubset)
    }
}

// Equation 6
fn response_time(tasksubset: &[RTTask]) -> Time {
    match tasksubset {
        [] => Time::zero(),
        _ => response((Time::zero(), tasksubset.last().unwrap().wcet), tasksubset)
    }
}

#[test]
// Example 1
fn example_1() {
    let taskset = [
        RTTask::new_ns(1, 10, 10),
        RTTask::new_ns(2, 12, 12),
        RTTask::new_ns(20, 30, 600),
        RTTask::new_ns(8, 40, 30),
    ];

    assert_eq!(response_time(&taskset[0..=0]), Time::nanos(1));
    assert_eq!(response_time(&taskset[0..=1]), Time::nanos(3));
    assert_eq!(response_time(&taskset[0..=2]), Time::nanos(29));
    assert_eq!(response_time(&taskset[0..=3]), Time::nanos(40));

    assert!(!is_schedulable(&taskset, false).unwrap());
    assert!(is_schedulable(&taskset, true).unwrap());
}

#[test]
// Example 2
fn example_2() {
    let taskset = [
        RTTask::new_ns(40, 100, 100),
        RTTask::new_ns(60, 140, 140),
        RTTask::new_ns(80, 500, 500),
        RTTask::new_ns(10, 1000, 1000),
        RTTask::new_ns(1, 1000, 1000),
    ];

    assert_eq!(response_time(&taskset[0..=0]), Time::nanos(40));
    assert_eq!(response_time(&taskset[0..=1]), Time::nanos(100));
    assert_eq!(response_time(&taskset[0..=2]), Time::nanos(560));
    assert_eq!(response_time(&taskset[0..=3]), Time::nanos(2490));
    assert_eq!(response_time(&taskset[0..=4]), Time::nanos(6991));

    assert!(!is_schedulable(&taskset, false).unwrap());
}