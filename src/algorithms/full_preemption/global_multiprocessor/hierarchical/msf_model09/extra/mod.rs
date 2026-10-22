//! ## Multi Supply Function Abstraction - Bini, Buttazzo, Bertogna 2009
//!
//! ---
//! #### References:
//! 1. E. Bini, G. Buttazzo and M. Bertogna, "The Multi Supply Function
//!    Abstraction for Multiprocessors," 2009 15th IEEE International
//!    Conference on Embedded and Real-Time Computing Systems and Applications,
//!    Beijing, China, 2009, pp. 294-302, doi: 10.1109/RTCSA.2009.39.

use crate::prelude::*;
use super::{MSFModel, MSFServer};

pub struct AnalysisGeneric<'a, Task, Server, FnW>
    where
        Task: std::borrow::Borrow<LLTask>,
        Server: MSFServer,
        FnW: Fn(&[Task], &Task) -> Time,
{
    pub model: &'a MSFModel<Server>,
    pub fnw: FnW,
    pub _marker: std::marker::PhantomData<Task>,
}

impl<'a, Task, Server, FnW> SchedAnalysis<(), Task> for AnalysisGeneric<'a, Task, Server, FnW>
    where
        Task: std::borrow::Borrow<LLTask>,
        Server: MSFServer,
        FnW: Fn(&[Task], &Task) -> Time,
{
    fn analyzer_name(&self) -> &str { panic!() }
    fn check_preconditions(&self, _: &[Task]) -> Result<(), SchedError> { panic!() }

    fn run_test(&self, taskset: &[Task]) -> Result<(), SchedError> {
        // Equation 33 [1]
        let schedulable =
            taskset.iter()
                .all(|__task_k| {
                    let task_k = __task_k.borrow();
                    let workload = (self.fnw)(taskset, __task_k);
                    let interference = self.model.interference(workload, task_k.deadline);

                    task_k.wcet + interference <= task_k.deadline
                });

        SchedError::result_from_schedulable(schedulable)
    }
}
