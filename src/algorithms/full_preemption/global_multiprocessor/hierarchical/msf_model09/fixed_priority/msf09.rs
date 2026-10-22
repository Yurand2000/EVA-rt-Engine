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

impl<S> SchedAnalysis<(), PriorityTask<LLTask>> for Analysis<S>
    where S: MSFServer
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        extra::AnalysisGeneric {
            model: &self.model,
            fnw: interference_upperbound,
            _marker: std::marker::PhantomData,
        }.run_test(taskset)
    }
}


fn workload_upperbound(task_i: &PriorityTask<LLTask>, task_k: &PriorityTask<LLTask>) -> Time
{
    let w = task_k.deadline + task_i.deadline - task_i.wcet;
    let n = f64::floor(w / task_i.period);
    n * task_i.wcet + Time::min(task_i.wcet, w - n * task_i.period)
}

fn interference_upperbound(taskset: &[PriorityTask<LLTask>], task_k: &PriorityTask<LLTask>) -> Time
{
    taskset.iter()
        .filter(|task_i| task_i.priority > task_k.priority )
        .map(|task_i| workload_upperbound(task_i, task_k))
        .sum()
}
