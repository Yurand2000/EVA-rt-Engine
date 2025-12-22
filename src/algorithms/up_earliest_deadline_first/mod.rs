use crate::prelude::*;

/// Liu, C. L., & Layland, J. W. (1973). Scheduling algorithms for
/// multiprogramming in a hard-real-time environment. Journal of the ACM (JACM),
/// 20(1), 46-61.
///
/// **Prerequisites:**
/// - Periodic tasks.
/// - Implicit deadlines.
///
/// **Worst-Case Complexity:** *O(n)*
pub fn liu_layland_73(taskset: &[RTTask]) -> Result<bool, Error> {
    AnalysisUtils::assert_implicit_deadlines(taskset)?;

    let total_utilization = RTUtils::total_utilization(taskset);

    Ok(total_utilization <= 1f64)
}