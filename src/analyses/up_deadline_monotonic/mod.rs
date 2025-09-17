use crate::prelude::*;

// Leung, J.Y.T. and Whitehead, J., 1982. On the complexity of fixed-priority
// scheduling of periodic, real-time tasks. Performance evaluation, 2(4),
// pp.237-250.
pub fn is_schedulable_pessimistic(taskset: &[RTTask]) -> Result<bool, Error> {
    assert_preconditions(taskset)?;

    let utilization: f64 =
        taskset.iter()
        .map(RTTask::get_density)
        .sum();

    let utilization_bound =
        (taskset.len() as f64) * (2f64.powf(taskset.len() as f64) - 1f64);

    Ok(utilization <= utilization_bound)
}

pub fn is_schedulable(taskset: &[RTTask]) -> Result<bool, Error> {
    assert_preconditions(taskset)?;

    Ok(taskset.iter().enumerate()
        .all(|(i, task)| {
            let interference = interference(&taskset[0..=i]) as u64;
            task.wcet.value_ns + interference <= task.deadline.value_ns
        }))
}

fn interference(tasksubset: &[RTTask]) -> f64 {
    if tasksubset.len() == 0 {
        return 0f64;
    }

    let last_task = tasksubset.last().unwrap();

    tasksubset.iter()
        .take(tasksubset.len() - 1)
        .map(|task| {
            f64::ceil(task.deadline.value_ns as f64 / last_task.period.value_ns as f64)
                * last_task.wcet.value_ns as f64
        })
        .sum()
}

fn assert_preconditions(taskset: &[RTTask]) -> Result<(), Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;

    Ok(())
}