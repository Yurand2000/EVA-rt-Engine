use crate::prelude::*;

// Bertogna, M., Cirinei, M. and Lipari, G., 2005, December. New schedulability
// tests for real-time task sets scheduled by deadline monotonic on
// multiprocessors. In International Conference on Principles of Distributed
// Systems (pp. 306-321). Berlin, Heidelberg: Springer Berlin Heidelberg.
// Theorem 5
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;

    if num_processors < 2 {
        return Err(Error::Generic(format!("Number of processors must be greater of equal to 2")));
    }

    let d_tot = RTUtils::total_density(taskset);
    let d_max = RTUtils::largest_density(taskset);

    Ok(d_tot <= (num_processors as f64 / 2f64) * (1f64 - d_max) + d_max)
}