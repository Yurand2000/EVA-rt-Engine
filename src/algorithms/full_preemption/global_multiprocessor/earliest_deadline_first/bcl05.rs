//! ## Multiprocessor EDF - Bertogna, Cirinei, Lipari 2005
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | O(*n^2*) complexity
//!
//! ---
//! #### References:
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis
//!    of EDF on multiprocessor platforms,” in 17th Euromicro Conference on
//!    Real-Time Systems (ECRTS’05), July 2005, pp. 209–218.
//!    doi: 10.1109/ECRTS.2005.18.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Multiprocessor EDF (Bertogna, Cirinei, Lipari 2005)";

/// Multiprocessor EDF - Bertogna, Cirinei, Lipari 2005 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    #[inline(always)]
    fn num_jobs(task_i: &RTTask, task_k: &RTTask) -> f64 {
        ((task_k.deadline - task_i.deadline) / (task_i.period)).floor() + 1.0
    }

    #[inline(always)]
    // Theorem 7 [1]
    fn beta(task_i: &RTTask, task_k: &RTTask) -> f64 {
        let n_jobs = num_jobs(task_i, task_k);

        (
            n_jobs * task_i.wcet
                +
            Time::min(
                task_i.wcet,
                Time::max(
                    Time::zero(),
                    task_k.deadline - n_jobs * task_i.period
                )
            )
        ) / task_k.deadline
    }

    // Theorem 7 [1]
    let schedulable =
        taskset.iter().enumerate()
        .all(|(k, task_k)| {
            let mut beta_in_range = false;

            let sum = taskset.iter().enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, task_i)| beta(task_i, task_k))
                .inspect(|beta_i| {
                    if *beta_i > 0.0 && *beta_i <= 1.0 - task_k.density() {
                        beta_in_range = true;
                    }
                })
                .map(|beta_i| f64::min(beta_i, 1.0 - task_k.density()))
                .sum::<f64>();

            let cmp = num_processors as f64 * (1.0 - task_k.density());

            sum < cmp || (sum == cmp && beta_in_range)
        });

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}