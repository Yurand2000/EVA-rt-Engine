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
//! - [`Analysis::is_schedulable`] \
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

const ALGORITHM: &str = "Fixed Priority DM (Bertogna, Cirinei, Lipari 2005)";

/// Multiprocessor Fixed Priority DM - Bertogna, Cirinei, Lipari 2005 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl SchedAnalysis<(), &[RTTask]> for Analysis {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if !RTUtils::is_taskset_sorted_by_deadline(taskset) {
            Err(SchedError::deadline_monotonic())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<(), SchedError> {
        // Theorem 5 [1]
        let d_tot = RTUtils::total_density(taskset);
        let d_max = RTUtils::largest_density(taskset);

        let schedulable =
            d_tot <= (self.num_processors as f64 / 2f64) * (1f64 - d_max) + d_max;

        SchedError::result_from_schedulable(schedulable)
    }
}