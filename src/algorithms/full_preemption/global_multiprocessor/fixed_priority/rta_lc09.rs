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

impl<T> SchedAnalysis<(), T> for Analysis
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

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        let mut task_rts = vec![Time::zero(); taskset.len()];

        for (k, task_k) in taskset.iter().enumerate() {
            let task_k_rt = response_time(taskset, task_k, self.num_processors, &task_rts);
            if task_k_rt > task_k.get_deadline() {
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
fn workload_non_carry_in<T>(interval: Time, task: &T) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    (interval / task.get_period()).floor() * task.get_wcet()
        + Time::min(task.get_wcet(), interval % task.get_period())
}

// Equation 6 [1]
fn workload_carry_in<T>(interval: Time, task: &T, task_rt: Time) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    let work_interval = Time::max(Time::zero(), interval - task.get_wcet());

    (work_interval / task.get_period()).floor() * task.get_wcet()
        + task.get_wcet()
        + Time::clamp(
            work_interval % task.get_period() - (task.get_period() - task_rt),
            Time::zero(),
            Time::max(task.get_wcet() - Time::one(), Time::zero()),
        )
}

// Equation 7 [1]
fn interference_non_carry_in<T>(interval: Time, task_k: &T, task_i: &T) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    Time::clamp(
        workload_non_carry_in(interval, task_i),
        Time::zero(),
        Time::max(interval - task_k.get_wcet() + Time::one(), Time::zero())
    )
}

// Equation 8 [1]
fn interference_carry_in<T>(interval: Time, task_k: &T, task_i: &T, task_i_rt: Time) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    Time::clamp(
        workload_carry_in(interval, task_i, task_i_rt),
        Time::zero(),
        Time::max(interval - task_k.get_wcet() + Time::one(), Time::zero())
    )
}

// Equation 9 [1]
fn total_interference<T>(interval: Time, cpus: u64, taskset: &[T], task_k: &T, task_rts: &[Time]) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    let interferences_non_carry_in: Vec<_> =
        taskset.iter()
            .filter(|task_i| task_i.get_priority() > task_k.get_priority())
            .map(|task_i| interference_non_carry_in(interval, task_k, task_i))
            .collect();

    let interferences_carry_in: Vec<_> =
        taskset.iter().zip(task_rts)
            .filter(|(task_i, _)| task_i.get_priority() > task_k.get_priority())
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
fn response_time<T>(taskset: &[T], task_k: &T, cpus: u64, task_rts: &[Time]) -> Time
    where T: task_traits::LLTask + task_traits::Priority
{
    let mut prev_x = task_k.get_wcet();
    let mut x;
    loop {
        x = Time::floor(total_interference(prev_x, cpus, taskset, task_k, task_rts) / cpus as f64) + task_k.get_wcet();
        if x == prev_x {
            return x;
        }

        debug_assert!(x > prev_x);
        prev_x = x;
    }
}
