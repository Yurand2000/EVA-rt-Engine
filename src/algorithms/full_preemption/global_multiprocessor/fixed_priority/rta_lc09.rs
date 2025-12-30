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
//! - [`is_schedulable`] \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. N. Guan, M. Stigge, W. Yi, and G. Yu, “New Response Time Bounds for Fixed
//!    Priority Multiprocessor Scheduling,” in 2009 30th IEEE Real-Time Systems
//!    Symposium, Dec. 2009, pp. 387–397. doi: 10.1109/RTSS.2009.11.

use crate::prelude::*;
use eva_rt_common::utils::RTUtils;

const ALGORITHM: &str = "Multiprocessor FP Response Time Analysis (Guan, Stigge, Yi, Yu 2009)";

/// Multiprocessor FP Response Time Analysis - Guan, Stigge, Yi, Yu 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], cpus: usize) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    let mut task_rts = vec![Time::zero(); taskset.len()];

    for (k, task_k) in taskset.iter().enumerate() {
        let task_k_rt = response_time(taskset, k, cpus, &task_rts[0..k]);
        if task_k_rt > task_k.deadline {
            return SchedResultFactory(ALGORITHM).non_schedulable();
        }

        task_rts[k] = task_k_rt;
    }

    SchedResultFactory(ALGORITHM).schedulable(())
}

// Equation 5 [1]
pub fn workload_non_carry_in(interval: Time, task: &RTTask) -> Time {
    (interval / task.period).floor() * task.wcet
        + Time::min(task.wcet, interval % task.period)
}

// Equation 6 [1]
pub fn workload_carry_in(interval: Time, task: &RTTask, task_rt: Time) -> Time {
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
pub fn interference_non_carry_in(interval: Time, task_k: &RTTask, task_i: &RTTask) -> Time {
    Time::clamp(
        workload_non_carry_in(interval, task_i),
        Time::zero(),
        Time::max(interval - task_k.wcet + Time::one(), Time::zero())
    )
}

// Equation 8 [1]
pub fn interference_carry_in(interval: Time, task_k: &RTTask, task_i: &RTTask, task_i_rt: Time) -> Time {
    Time::clamp(
        workload_carry_in(interval, task_i, task_i_rt),
        Time::zero(),
        Time::max(interval - task_k.wcet + Time::one(), Time::zero())
    )
}

// Equation 9 [1]
pub fn total_interference(interval: Time, cpus: usize, taskset: &[RTTask], k: usize, task_rts: &[Time]) -> Time {
    assert!(task_rts.len() == k);

    let interferences_non_carry_in: Vec<_> =
        taskset.iter().enumerate()
            .filter(|&(i, _)| i < k)
            .map(|(_, task_i)| interference_non_carry_in(interval, &taskset[k], task_i))
            .collect();

    let interferences_carry_in: Vec<_> =
        taskset.iter().zip(task_rts).enumerate()
            .filter(|&(i, _)| i < k)
            .map(|(_, (task_i, &task_i_rt))| interference_carry_in(interval, &taskset[k], task_i, task_i_rt))
            .collect();

    let mut interference_diffs: Vec<_> =
        interferences_carry_in.into_iter().zip(interferences_non_carry_in.iter())
            .map(|(itf_ci, &itf_nc)| itf_ci - itf_nc)
            .collect();

    interference_diffs.sort_unstable();

    interferences_non_carry_in.into_iter().sum::<Time>() +
        interference_diffs.into_iter().rev().take(cpus - 1).sum::<Time>()
}

// Equation 12 [1]
pub fn response_time(taskset: &[RTTask], k: usize, cpus: usize, task_rts: &[Time]) -> Time {
    let mut prev_x = taskset[k].wcet;
    let mut x;
    loop {
        x = Time::floor(total_interference(prev_x, cpus, taskset, k, task_rts) / cpus as f64) + taskset[k].wcet;
        if x == prev_x {
            return x;
        }

        debug_assert!(x > prev_x);
        prev_x = x;
    }
}