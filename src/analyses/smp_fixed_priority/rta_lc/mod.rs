use crate::prelude::*;

pub mod prelude {
    pub use super::{
        is_schedulable,
    };
}

pub fn is_schedulable(taskset: &[RTTask], cpus: usize) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;

    let mut task_rts = vec![Time::zero(); taskset.len()];

    for (k, task_k) in taskset.iter().enumerate() {
        let task_k_rt = response_time(taskset, k, cpus, &task_rts);
        if task_k_rt > task_k.deadline {
            return Ok(false);
        }

        task_rts[k] = task_k_rt;
    }

    Ok(true)
}

// Equation 5, [1]
pub fn workload_non_carry_in(interval: Time, task: &RTTask) -> Time {
    (interval / task.period).floor() * task.wcet
        + Time::min(task.wcet, interval % task.period)
}

// Equation 6, [1]
pub fn workload_carry_in(interval: Time, task: &RTTask, task_rt: Time) -> Time {
    let work_interval = Time::max(Time::zero(), interval - task.wcet);

    (work_interval / task.period).floor() * task.wcet
        + task.wcet
        + Time::clamp(
            work_interval % task.period - (task.period - task_rt),
            Time::zero(),
            task.wcet - Time::one(),
        )
}

// Equation 7, [1]
pub fn interference_non_carry_in(interval: Time, task_k: &RTTask, task_i: &RTTask) -> Time {
    Time::clamp(
        workload_non_carry_in(interval, task_i),
        Time::zero(),
        interval - task_k.wcet + Time::one()
    )
}

// Equation 8, [1]
pub fn interference_carry_in(interval: Time, task_k: &RTTask, task_i: &RTTask, task_i_rt: Time) -> Time {
    Time::clamp(
        workload_carry_in(interval, task_i, task_i_rt),
        Time::zero(),
        interval - task_k.wcet + Time::one()
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
        interference_diffs.into_iter().take(cpus - 1).sum::<Time>()
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

        prev_x = x;
    }
}

/* -----------------------------------------------------------------------------
References:
[1] N. Guan, M. Stigge, W. Yi, and G. Yu, “New Response Time Bounds for Fixed
Priority Multiprocessor Scheduling,” in 2009 30th IEEE Real-Time Systems
Symposium, Dec. 2009, pp. 387–397. doi: 10.1109/RTSS.2009.11.
*/