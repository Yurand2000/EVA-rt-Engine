use crate::prelude::*;

// Liu, C. L., & Layland, J. W. (1973). Scheduling algorithms for
// multiprogramming in a hard-real-time environment. Journal of the ACM (JACM),
// 20(1), 46-61.
pub fn is_schedulable(taskset: &[RTTask]) -> Result<bool, Error> {
    assert_preconditions(taskset)?;

    // Theorem 5: let m = #Tasks, lub(Utilization) = m * (2^(1/m) - 1)
    let total_utilization = RTUtils::total_utilization(taskset);
    let rate_monotonic_lub = (taskset.len() as f64) * (2f64.powf(taskset.len() as f64) - 1f64);

    Ok(total_utilization <= rate_monotonic_lub)
}

// Use the limit approximation for the least upper bound, i.e. #Tasks -> +inf
// Significant limit: forall a>0. lim x->0 ((a^x - 1) / x) = ln(a)
pub fn is_schedulable_simple(taskset: &[RTTask]) -> Result<bool, Error> {
    assert_preconditions(taskset)?;

    // Theorem 5
    let total_utilization = RTUtils::total_utilization(taskset);
    let rate_monotonic_lub = f64::ln(2f64);

    Ok(total_utilization <= rate_monotonic_lub)
}

// Bini, E., Buttazzo, G. and Buttazzo, G., 2001, June. A hyperbolic bound for
// the rate monotonic algorithm. In Proceedings 13th Euromicro Conference on
// Real-Time Systems (pp. 59-66). IEEE.
pub fn is_schedulable_hyperbolic(taskset: &[RTTask]) -> Result<bool, Error> {
    assert_preconditions(taskset)?;
    
    // Theorem 1
    let bound: f64 =
        taskset.iter()
        .map(|task| task.get_utilization() + 1f64)
        .product();

    Ok(bound <= 2f64)
}

fn assert_preconditions(taskset: &[RTTask]) -> Result<(), Error> {
    AnalysisUtils::assert_implicit_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_period(taskset)?;

    Ok(())
}