use crate::prelude::*;

#[derive(Clone)]
#[derive(Debug)]
pub enum Error {
    NotOrderedByPeriod,
}

// Liu, C. L., & Layland, J. W. (1973). Scheduling algorithms for
// multiprogramming in a hard-real-time environment. Journal of the ACM (JACM),
// 20(1), 46-61.
pub fn is_schedulable(taskset: &[RTTask]) -> Result<bool, Error> {
    if !RTUtils::is_taskset_sorted_by_period(taskset) {
        return Err(Error::NotOrderedByPeriod);
    }

    // Theorem 5: let m = #Tasks, lub(Utilization) = m * (2^(1/m) - 1)
    let total_utilization = RTUtils::get_worst_case_utilization(taskset);
    let rate_monotonic_lub = (taskset.len() as f64) * (2f64.powf(taskset.len() as f64) - 1f64);

    Ok(total_utilization <= rate_monotonic_lub)
}

// Use the limit approximation for the least upper bound, i.e. #Tasks -> +inf
// Significant limit: forall a>0. lim x->0 ((a^x - 1) / x) = ln(a)
pub fn is_schedulable_simple(taskset: &[RTTask]) -> Result<bool, Error> {
    if !RTUtils::is_taskset_sorted_by_period(taskset) {
        return Err(Error::NotOrderedByPeriod);
    }

    // Theorem 5
    let total_utilization = RTUtils::get_worst_case_utilization(taskset);
    let rate_monotonic_lub = f64::ln(2f64);

    Ok(total_utilization <= rate_monotonic_lub)
}