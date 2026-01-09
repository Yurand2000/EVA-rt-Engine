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
//! - [`AnalysisPeriodic::is_schedulable`] \
//!   | Periodic Task Model \
//!   | Implicit Deadlines \
//!   | linear O(*n*) complexity
//! - [`AnalysisSporadic::is_schedulable`] \
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

const ALGORITHM: &str = "Multiprocessor EDF (Goossens, Funk, Baruah 2003)";

/// Multiprocessor EDF, Periodic tasks - Goossens, Funk, Baruah 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct AnalysisPeriodic {
    pub num_processors: u64,
}

impl SchedAnalysis<(), &[RTTask]> for AnalysisPeriodic {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<(), SchedError> {
        let u_tot = RTUtils::total_utilization(taskset);
        let u_max = RTUtils::largest_utilization(taskset);

        // Theorem 3 [2]
        let schedulable =
            u_tot <= (self.num_processors as f64) - u_max * (self.num_processors as f64 - 1f64);

        SchedError::result_from_schedulable(schedulable)
    }
}

/// Multiprocessor EDF, Sporadic tasks - Goossens, Funk, Baruah 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct AnalysisSporadic {
    pub num_processors: u64,
}

impl SchedAnalysis<(), &[RTTask]> for AnalysisSporadic {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<(), SchedError> {
        let d_tot = RTUtils::total_density(taskset);
        let d_max = RTUtils::largest_density(taskset);

        // Theorem 4 [2]
        let schedulable =
            d_tot <= (self.num_processors as f64) - d_max * (self.num_processors as f64 - 1f64);

        SchedError::result_from_schedulable(schedulable)
    }
}