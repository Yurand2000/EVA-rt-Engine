//! ## Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Implicit Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//! - [`DesignerLinear::design`] \
//!   | Generate the suitable interface given the taskset and the [`PRModel`]'s period. \
//!   | \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;
use crate::algorithms::full_preemption::uniprocessor::hierarchical::pr_model03::*;

const ALGORITHM: &str = "Periodic Resource Model, EDF Local Scheduling (Shin & Lee 2003)";

/// Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub model: PRModel,
}

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Equation 9 [1]
        let schedulable =
            is_schedulable_demand(
                taskset,
                &self.model,
                demand,
                time_intervals
            );

        SchedError::result_from_schedulable(schedulable)
    }
}

/// Periodic Resource Model, EDF Local Scheduling - Shin & Lee 2003 \[1\] \
/// Derive the best [`PRModel`] using demand analysis.
///
/// Refer to the [module](`self`) level documentation.
pub struct DesignerLinear {
    pub period: Time
}

impl<T> SchedDesign<PRModel, T> for DesignerLinear
    where T: task_traits::LLTask
{
    fn designer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::implicit_deadlines(taskset) {
            Err(SchedError::implicit_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_designer(&self, taskset: &[T]) -> Result<PRModel, SchedError> {
    // Equation 16 [1]
        generate_model_from_demand_linear(
            taskset,
            self.period,
            demand,
            time_intervals
        )
        .ok_or(SchedError::NonSchedulable(None))
    }
}

// Section 4.1 [1]
fn demand<T>(taskset: &[T], interval: Time) -> Time
    where T: task_traits::LLTask
{
    taskset.iter()
        .map(|task| (interval / task.get_period()).floor() * task.get_wcet())
        .sum()
}

// Theorem 1 [1]
fn time_intervals<T>(taskset: &[T]) -> Box<dyn Iterator<Item = Time>>
    where T: task_traits::LLTask
{
    let max_time = LLTasksetUtils::hyperperiod(taskset) * 2.0;

    Box::new(TimeIterators::range(Time::zero(), max_time))
}
