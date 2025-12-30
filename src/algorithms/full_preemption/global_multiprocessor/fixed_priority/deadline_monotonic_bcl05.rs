//! ## Multiprocessor Fixed Priority DM - Bertogna, Cirinei, Lipari 2005
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed-Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//! - Deadline Monotonic priority assigment
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | linear *O(n)* complexity
//!
//! ---
//! #### References:
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “New Schedulability Tests for
//!    Real-Time Task Sets Scheduled by Deadline Monotonic on Multiprocessors,”
//!    in Principles of Distributed Systems, J. H. Anderson, G. Prencipe, and R.
//!    Wattenhofer, Eds., Berlin, Heidelberg: Springer, Dec. 2005, pp. 306–321.
//!    doi: 10.1007/11795490_24.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Fixed Priority DM (Bertogna, Cirinei, Lipari 2005)";

/// Multiprocessor Fixed Priority DM - Bertogna, Cirinei, Lipari 2005 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    if !RTUtils::is_taskset_sorted_by_deadline(taskset) {
        return SchedResultFactory(ALGORITHM).deadline_monotonic();
    }

    // Theorem 5 [1]
    let d_tot = RTUtils::total_density(taskset);
    let d_max = RTUtils::largest_density(taskset);

    let schedulable =
        d_tot <= (num_processors as f64 / 2f64) * (1f64 - d_max) + d_max;

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}