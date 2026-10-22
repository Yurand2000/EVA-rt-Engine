//! ## Multi Supply Function Abstraction - Bini, Buttazzo, Bertogna 2009
//!
//! #### Implements:
//!
//! ---
//! #### References:
//! 1. E. Bini, G. Buttazzo and M. Bertogna, "The Multi Supply Function
//!    Abstraction for Multiprocessors," 2009 15th IEEE International
//!    Conference on Embedded and Real-Time Computing Systems and Applications,
//!    Beijing, China, 2009, pp. 294-302, doi: 10.1109/RTCSA.2009.39.

use crate::prelude::*;
use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::msf_model09::*;

const ALGORITHM: &str = "MSF Model, FP Local Scheduler (Bini, Buttazzo, Bertogna 2009)";

/// MPR Model, FP Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis<S>
    where S: MSFServer
{
    pub model: MSFModel<S>,
}

impl<T, S> SchedAnalysis<(), T> for Analysis<S>
    where T: task_traits::LLTask + task_traits::Priority,
          S: MSFServer
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        extra::AnalysisGeneric {
            model: &self.model,
            fnw: interference_upperbound,
            _marker: std::marker::PhantomData,
        }.run_test(taskset)
    }
}


fn workload_upperbound<T>(task_i: &T, task_k: &T) -> Time
    where T: task_traits::LLTask
{
    let w = task_k.get_deadline() + task_i.get_deadline() - task_i.get_wcet();
    let n = f64::floor(w / task_i.get_period());
    n * task_i.get_wcet() + Time::min(task_i.get_wcet(), w - n * task_i.get_period())
}

fn interference_upperbound<T>(taskset: &[T], task_k: &T) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    taskset.iter()
        .filter(|task_i| task_i.get_priority() > task_k.get_priority() )
        .map(|task_i| workload_upperbound(task_i, task_k))
        .sum()
}
