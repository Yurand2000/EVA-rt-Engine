//! ## Multiprocessor EDF - Goossens, Funk, Baruah 2003
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Implicit/Constrained Deadlines
//!
//! #### Implements:
//! - [`gfb_test_periodic`] \
//!   | Periodic Task Model \
//!   | Implicit Deadlines \
//!   | linear O(*n*) complexity
//! - [`gfb_test_sporadic`] \
//!   | Sporadic Task Model \
//!   | Constrained Deadlines \
//!   | linear O(*n*) complexity
//!
//! ---
//! #### References:
//! 1. J. Goossens, S. Funk, and S. Baruah, “Priority-Driven Scheduling of
//!    Periodic Task Systems on Multiprocessors,” Real-Time Systems, vol. 25,
//!    no. 2, pp. 187–205, Sept. 2003, doi: 10.1023/A:1025120124771.
//! 2. M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis
//!    of EDF on multiprocessor platforms,” in 17th Euromicro Conference on
//!    Real-Time Systems (ECRTS’05), July 2005, pp. 209–218.
//!    doi: 10.1109/ECRTS.2005.18.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Multiprocessor EDF (Goossens, Funk, Baruah 2003)";

/// Multiprocessor EDF, Periodic tasks - Goossens, Funk, Baruah 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_periodic(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::implicit_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).implicit_deadlines();
    }

    let u_tot = RTUtils::total_utilization(taskset);
    let u_max = RTUtils::largest_utilization(taskset);

    // Theorem 3 [2]
    let schedulable =
        u_tot <= (num_processors as f64) - u_max * (num_processors as f64 - 1f64);

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

/// Multiprocessor EDF, Sporadic tasks - Goossens, Funk, Baruah 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_sporadic(taskset: &[RTTask], num_processors: u64) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    let d_tot = RTUtils::total_density(taskset);
    let d_max = RTUtils::largest_density(taskset);

    // Theorem 4 [2]
    let schedulable =
        d_tot <= (num_processors as f64) - d_max * (num_processors as f64 - 1f64);

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}