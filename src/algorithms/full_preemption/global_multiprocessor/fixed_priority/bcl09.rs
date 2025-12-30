//! ## Multiprocessor Fixed Priority - Bertogna, Cirinei, Lipari 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed Priority scheduling
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
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of
//!    Global Scheduling Algorithms on Multiprocessor Platforms,” IEEE
//!    Transactions on Parallel and Distributed Systems, vol. 20, no. 4, pp.
//!    553–566, Apr. 2009, doi: 10.1109/TPDS.2008.129.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Multiprocessor Fixed Priority (Bertogna, Cirinei, Lipari 2009)";

/// Multiprocessor Fixed Priority - Bertogna, Cirinei, Lipari 2009
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Theorem 8 [1]
    // Section 4 Equation 10
    let schedulable =
        taskset.iter().enumerate()
        .all(|(k, task_k)|
            global_fixed_priority_demand(taskset, k, task_k)
                <
            num_processors as f64 * (task_k.laxity() + Time::one())
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

pub fn global_fixed_priority_demand(taskset: &[RTTask], k: usize, task_k: &RTTask) -> Time {
    use crate::algorithms::full_preemption::global_multiprocessor
             ::earliest_deadline_first::bcl09::workload_upperbound;

    taskset.iter()
        .enumerate()
        .filter(|(i, _)| *i < k)
        .map(|(_, task_i)| {
            Time::min(
                workload_upperbound(task_k.deadline, task_i),
                task_k.laxity() + Time::one(),
            )
        })
        .sum()
}