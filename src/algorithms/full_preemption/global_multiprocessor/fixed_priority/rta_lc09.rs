//! ## Multiprocessor FP Response Time Analysis - Guan, Stigge, Yi, Yu 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed-Priority scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. N. Guan, M. Stigge, W. Yi, and G. Yu, “New Response Time Bounds for Fixed
//!    Priority Multiprocessor Scheduling,” in 2009 30th IEEE Real-Time Systems
//!    Symposium, Dec. 2009, pp. 387–397. doi: 10.1109/RTSS.2009.11.

use crate::prelude::*;

const ALGORITHM: &str = "Multiprocessor FP Response Time Analysis (Guan, Stigge, Yi, Yu 2009)";

/// Multiprocessor FP Response Time Analysis - Guan, Stigge, Yi, Yu 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl SchedAnalysis<(), PriorityTask<LLTask>> for Analysis
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        if !Taskset::<LLTask>::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[PriorityTask<LLTask>]) -> Result<(), SchedError> {
        let mut task_rts = vec![Time::zero(); taskset.len()];

        for (k, task_k) in taskset.iter().enumerate() {
            let task_k_rt = response_time(taskset, task_k, self.num_processors, &task_rts);
            if task_k_rt > task_k.deadline {
                return Err(SchedError::NonSchedulable(Some(
                    anyhow::format_err!("task {k} misses its deadline.")
                )));
            }

            task_rts[k] = task_k_rt;
        }

        Ok(())
    }
}

// Equation 5 [1]
fn workload_non_carry_in(interval: Time, task: &PriorityTask<LLTask>) -> Time
{
    (interval / task.period).floor() * task.wcet
        + Time::min(task.wcet, interval % task.period)
}

// Equation 6 [1]
fn workload_carry_in(interval: Time, task: &PriorityTask<LLTask>, task_rt: Time) -> Time
{
    let work_interval = Time::max(Time::zero(), interval - task.wcet);

    (work_interval / task.period).floor() * task.wcet
        + task.wcet
        + Time::clamp(
            work_interval % task.period - (task.period - task_rt),
            Time::zero(),
            Time::max(task.wcet - Time::one(), Time::zero()),
        )
}

// Equation 7 [1]
fn interference_non_carry_in(interval: Time, task_k: &PriorityTask<LLTask>, task_i: &PriorityTask<LLTask>) -> Time
{
    Time::clamp(
        workload_non_carry_in(interval, task_i),
        Time::zero(),
        Time::max(interval - task_k.wcet + Time::one(), Time::zero())
    )
}

// Equation 8 [1]
fn interference_carry_in(interval: Time, task_k: &PriorityTask<LLTask>, task_i: &PriorityTask<LLTask>, task_i_rt: Time) -> Time
{
    Time::clamp(
        workload_carry_in(interval, task_i, task_i_rt),
        Time::zero(),
        Time::max(interval - task_k.wcet + Time::one(), Time::zero())
    )
}

// Equation 9 [1]
fn total_interference(interval: Time, cpus: u64, taskset: &[PriorityTask<LLTask>], task_k: &PriorityTask<LLTask>, task_rts: &[Time]) -> Time
{
    let interferences_non_carry_in: Vec<_> =
        taskset.iter()
            .filter(|task_i| task_i.priority > task_k.priority)
            .map(|task_i| interference_non_carry_in(interval, task_k, task_i))
            .collect();

    let interferences_carry_in: Vec<_> =
        taskset.iter().zip(task_rts)
            .filter(|(task_i, _)| task_i.priority > task_k.priority)
            .map(|(task_i, &task_i_rt)| interference_carry_in(interval, task_k, task_i, task_i_rt))
            .collect();

    let mut interference_diffs: Vec<_> =
        interferences_carry_in.into_iter().zip(interferences_non_carry_in.iter())
            .map(|(itf_ci, &itf_nc)| itf_ci - itf_nc)
            .collect();

    interference_diffs.sort_unstable();

    interferences_non_carry_in.into_iter().sum::<Time>() +
        interference_diffs.into_iter().rev().take(cpus as usize - 1).sum::<Time>()
}

// Equation 12 [1]
fn response_time(taskset: &[PriorityTask<LLTask>], task_k: &PriorityTask<LLTask>, cpus: u64, task_rts: &[Time]) -> Time
{
    let mut prev_x = task_k.wcet;
    let mut x;
    loop {
        x = Time::floor(total_interference(prev_x, cpus, taskset, task_k, task_rts) / cpus as f64) + task_k.wcet;
        if x == prev_x {
            return x;
        }

        debug_assert!(x > prev_x);
        prev_x = x;
    }
}
