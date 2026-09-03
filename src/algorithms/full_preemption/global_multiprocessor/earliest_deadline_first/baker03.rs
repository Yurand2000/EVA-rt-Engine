//! ## Multiprocessor EDF - Baker 2003
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
//!   | O(*n^3*) complexity
//!
//! ---
//! #### References:
//! 1. T. P. Baker, “Multiprocessor EDF and deadline monotonic schedulability
//!    analysis,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 120–129. doi: 10.1109/REAL.2003.1253260.
//! 2. M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis
//!    of EDF on multiprocessor platforms,” in 17th Euromicro Conference on
//!    Real-Time Systems (ECRTS’05), July 2005, pp. 209–218.
//!    doi: 10.1109/ECRTS.2005.18.

use crate::prelude::*;
use crate::task::traits::LLTaskUtils as _;

const ALGORITHM: &str = "Multiprocessor EDF (Baker 2003)";

/// Multiprocessor EDF - Baker 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Theorem 5 [2]
        fn beta<T>(task_i: &T, task_k: &T) -> f64
            where T: task_traits::LLTask
        {
            let b0 = task_i.utilization() * (1.0 + (task_i.get_period() - task_i.get_deadline()) / task_k.get_deadline());

            if task_k.density() >= task_i.utilization() {
                b0
            } else {
                b0 + (task_i.get_wcet() - task_k.density() * task_i.get_period()) / task_k.get_deadline()
            }
        }

        // Theorem 5 [2]
        let schedulable =
            taskset.iter()
            .all(|task_k| {
                taskset.iter()
                    .map(|task_i| f64::min(1.0, beta(task_i, task_k)))
                    .sum::<f64>()
                <=
                self.num_processors as f64 * (1.0 - task_k.density()) + task_k.density()
            });

        SchedError::result_from_schedulable(schedulable)
    }
}

#[test]
// Example in Section 3.3 [2]
fn gfb_bak_example() {
    use crate::task::test_utils::*;

    let taskset = [
        new_ll_task(49, 100, 100),
        new_ll_task(49, 100, 100),
        new_ll_task(2, 50, 100),
    ];

    let num_processors = 2;

    assert!(super::gbf03::AnalysisSporadic { num_processors }.is_schedulable(&taskset).is_ok());
    assert!(Analysis { num_processors }.is_schedulable(&taskset).is_err());
}
