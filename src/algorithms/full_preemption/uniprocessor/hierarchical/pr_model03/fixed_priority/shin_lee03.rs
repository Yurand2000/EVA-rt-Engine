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

impl<T> SchedAnalysis<Vec<Time>, T> for Analysis
    where T: task_traits::LLTask + task_traits::Priority
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<Vec<Time>, SchedError> {
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

impl<T> SchedDesign<&[T], PRModel> for DesignerLinear
    where T: task_traits::LLTask + task_traits::Priority
{
    fn designer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_designer(&self, taskset: &[T]) -> Result<PRModel, SchedError> {
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
fn rta<T>(taskset: &[T], task_k: &T, response: Time) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    taskset.iter()
        .filter(|task_i| task_i.get_priority() > task_k.get_priority())
        .map(|task_i| (response / task_i.get_period()).ceil() * task_i.get_wcet())
        .sum::<Time>()
    +
        task_k.get_wcet()
}
