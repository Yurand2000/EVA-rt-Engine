//! ## Periodic Resource Model, Fixed Priority Local Scheduling - Shin & Lee 2003
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//! - [`DesignerLinear::design`] \
//!   | Generate the suitable interface given the taskset and the [`PRModel`]'s period. \
//!   | \
//!   | O(*n*) complexity
//!
//! ---
//! #### References:
//! 1. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;
use crate::algorithms::full_preemption::uniprocessor::hierarchical::pr_model03::*;

const ALGORITHM: &str = "Periodic Resource Model, Fixed Priority Local Scheduling (Shin & Lee 2003)";

/// Periodic Resource Model, Fixed Priority Local Scheduling - Shin & Lee 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub model: PRModel,
}

impl SchedAnalysis<Vec<Time>, &[RTTask]> for Analysis {
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[RTTask]) -> Result<Vec<Time>, SchedError> {
        // Equation 14 [1]
        is_schedulable_response(
            taskset,
            &self.model,
            rta,
        )
        .map_err(|err| SchedError::NonSchedulable(Some(err)))
    }
}

/// Periodic Resource Model, Fixed Priority Local Scheduling - Shin & Lee 2003 \[1\] \
/// Derive the best [`PRModel`] using demand analysis.
///
/// Refer to the [module](`self`) level documentation.
pub struct DesignerLinear {
    pub period: Time
}

impl SchedDesign<&[RTTask], PRModel> for DesignerLinear {
    fn designer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &&[RTTask]) -> Result<(), SchedError> {
        if !RTUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_designer(&self, taskset: &[RTTask]) -> Result<PRModel, SchedError> {
        // Equations 23, 24 [1]
        generate_model_from_response_linear(
            taskset,
            self.period,
            rta
        )
        .ok_or(SchedError::NonSchedulable(None))
    }
}

// Equation 10 [1]
fn rta(taskset: &[RTTask], k: usize, task_k: &RTTask, response: Time) -> Time {
    taskset.iter()
        .take(k)
        .map(|task_i| (response / task_i.period).ceil() * task_i.wcet)
        .sum::<Time>()
    +
        task_k.wcet
}