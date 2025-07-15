use crate::prelude::*;

#[derive(Clone)]
#[derive(Debug)]
pub enum Error { }

// Liu, C. L., & Layland, J. W. (1973). Scheduling algorithms for
// multiprogramming in a hard-real-time environment. Journal of the ACM (JACM),
// 20(1), 46-61.
pub fn is_schedulable(taskset: &[RTTask]) -> Result<bool, Error> {
    let total_utilization = RTUtils::get_worst_case_utilization(taskset);

    Ok(total_utilization <= 1f64)
}