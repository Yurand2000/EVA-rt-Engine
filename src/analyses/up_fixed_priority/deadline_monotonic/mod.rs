use crate::prelude::*;

/// Leung, J.Y.T. and Whitehead, J., 1982. On the complexity of fixed-priority
/// scheduling of periodic, real-time tasks. Performance evaluation, 2(4),
/// pp.237-250.
///
/// **Prerequisites:**
/// - Periodic tasks.
/// - Constrained deadlines.
/// - Ordered by deadline.
///
/// **Worst-Case Complexity:** *O(n)*
pub fn is_schedulable_pessimistic(taskset: &[RTTask]) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;

    let utilization: f64 =
        taskset.iter()
        .map(RTTask::density)
        .sum();

    let utilization_bound =
        (taskset.len() as f64) * (2f64.powf(taskset.len() as f64) - 1f64);

    Ok(utilization <= utilization_bound)
}

/// Leung, J.Y.T. and Whitehead, J., 1982. On the complexity of fixed-priority
/// scheduling of periodic, real-time tasks. Performance evaluation, 2(4),
/// pp.237-250.
///
/// **Prerequisites:**
/// - Periodic tasks.
/// - Constrained deadlines.
/// - Ordered by deadline.
///
/// **Worst-Case Complexity:** *O(n)*
pub fn is_schedulable(taskset: &[RTTask]) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;

    #[inline(always)]
    fn interference(tasksubset: &[RTTask]) -> Time {
        if tasksubset.len() == 0 {
            return Time::zero();
        }

        let last_task = tasksubset.last().unwrap();

        tasksubset.iter()
            .take(tasksubset.len() - 1)
            .map(|task| (task.deadline / last_task.period).ceil() * last_task.wcet )
            .sum()
    }

    Ok(taskset.iter().enumerate()
        .all(|(i, task)| {
            task.wcet + interference(&taskset[0..=i]) <= task.deadline
        }))
}