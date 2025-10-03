use crate::prelude::*;

pub mod bcl_2009;
pub mod baruah_2007;

/// Goossens, J., Funk, S. and Baruah, S., 2003. Priority-driven scheduling of
/// periodic task systems on multiprocessors. Real-time systems, 25(2),
/// pp.187-205. *Theorem 5*
///
/// **Prerequisites:**
/// - Periodic tasks.
/// - Implicit deadlines.
///
/// **Worst-Case Complexity:** *O(n)*
pub fn gfb_test_periodic(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_implicit_deadlines(taskset)?;

    let u_tot = RTUtils::total_utilization(taskset);
    let u_max = RTUtils::largest_utilization(taskset);

    Ok(u_tot <= (num_processors as f64) - u_max * (num_processors as f64 - 1f64))
}

/// M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis of
/// EDF on multiprocessor platforms,” in 17th Euromicro Conference on Real-Time
/// Systems (ECRTS’05), July 2005, pp. 209–218. doi: 10.1109/ECRTS.2005.18.
/// *Theorem 4, Equation 5*
///
/// **Prerequisites:**
/// - Sporadic tasks.
/// - Constrained deadlines.
///
/// **Worst-Case Complexity:** *O(n)*
pub fn gfb_test_sporadic(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    let d_tot = RTUtils::total_density(taskset);
    let d_max = RTUtils::largest_density(taskset);

    Ok(d_tot <= (num_processors as f64) - d_max * (num_processors as f64 - 1f64))
}

/// M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis of
/// EDF on multiprocessor platforms,” in 17th Euromicro Conference on Real-Time
/// Systems (ECRTS’05), July 2005, pp. 209–218. doi: 10.1109/ECRTS.2005.18.
/// *Theorem 5, Equation 6*
///
/// **Prerequisites:**
/// - Sporadic tasks.
/// - Constrained deadlines.
///
/// **Worst-Case Complexity:** *O(n^3)*
pub fn bak_test(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    fn beta(task_i: &RTTask, task_k: &RTTask) -> f64 {
        let b0 = task_i.utilization() * (1.0 + (task_i.period - task_i.deadline) / task_k.deadline);

        if task_k.density() >= task_i.utilization() {
            b0
        } else {
            b0 + (task_i.wcet - task_k.density() * task_i.period) / task_k.deadline
        }
    }

    Ok(taskset.iter()
        .all(|task_k| {
            taskset.iter()
                .map(|task_i| f64::min(1.0, beta(task_i, task_k)))
                .sum::<f64>()
            <=
            num_processors as f64 * (1.0 - task_k.density()) + task_k.density()
        }))
}

/// M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis of
/// EDF on multiprocessor platforms,” in 17th Euromicro Conference on Real-Time
/// Systems (ECRTS’05), July 2005, pp. 209–218. doi: 10.1109/ECRTS.2005.18.
/// *Theorem 7, Equation 8*
///
/// **Prerequisites:**
/// - Sporadic tasks.
/// - Constrained deadlines.
///
/// **Worst-Case Complexity:** *O(n^2)*
pub fn bcl_edf(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    #[inline(always)]
    fn num_jobs(task_i: &RTTask, task_k: &RTTask) -> f64 {
        ((task_k.deadline - task_i.deadline) / (task_i.period)).floor() + 1.0
    }

    #[inline(always)]
    fn beta(task_i: &RTTask, task_k: &RTTask) -> f64 {
        (num_jobs(task_i, task_k) * task_i.wcet + Time::min(task_i.wcet, (task_k.deadline - num_jobs(task_i, task_k) * task_i.period).positive_or_zero())) / task_k.deadline
    }

    Ok(taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let mut beta_in_range = false;

            let sum = taskset.iter().enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, task_i)| {
                    let beta = beta(task_i, task_k);

                    if beta > 0.0 && beta <= 1.0 - task_k.density() {
                        beta_in_range = true;
                    }

                    beta
                })
                .map(|beta_i| f64::min(beta_i, 1.0 - task_k.density()))
                .sum::<f64>();

            let cmp = num_processors as f64 * (1.0 - task_k.density());

            sum < cmp || (sum == cmp && beta_in_range)
        }))
}

#[test]
// Example in Section 3.3 [2]
fn gfb_bak_example() {
    let taskset = [
        RTTask::new_ns(49, 100, 100),
        RTTask::new_ns(49, 100, 100),
        RTTask::new_ns(2, 50, 100),
    ];

    let num_processors = 2;

    assert!(gfb_test_sporadic(&taskset, num_processors).unwrap());
    assert!(!bak_test(&taskset, num_processors).unwrap());
}

/* -----------------------------------------------------------------------------
References:
[1]: Goossens, J., Funk, S. and Baruah, S., 2003. Priority-driven scheduling of
periodic task systems on multiprocessors. Real-time systems, 25(2), pp.187-205.

[2]: M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis
of EDF on multiprocessor platforms,” in 17th Euromicro Conference on Real-Time
Systems (ECRTS’05), July 2005, pp. 209–218. doi: 10.1109/ECRTS.2005.18.

[3]: Bertogna, M., Cirinei, M. and Lipari, G., 2008. Schedulability analysis of
global scheduling algorithms on multiprocessor platforms. IEEE Transactions on
parallel and distributed systems, 20(4), pp.553-566.
*/