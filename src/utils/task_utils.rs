pub use crate::prelude::*;
use crate::task::traits::LLTaskUtils;

pub mod prelude {
    pub use super::{
        LLTasksetUtils,
    };
}

pub struct LLTasksetUtils<T>(std::marker::PhantomData<T>);

impl<T> LLTasksetUtils<T>
    where T: task_traits::LLTask
{
    pub fn implicit_deadlines(taskset: &[T]) -> bool {
        taskset.iter().all(LLTaskUtils::has_implicit_deadline)
    }

    pub fn constrained_deadlines(taskset: &[T]) -> bool {
        taskset.iter().all(LLTaskUtils::has_constrained_deadline)
    }

    pub fn total_utilization(taskset: &[T]) -> f64 {
        taskset.iter().map(LLTaskUtils::utilization).sum()
    }

    pub fn total_density(taskset: &[T]) -> f64 {
        taskset.iter().map(LLTaskUtils::density).sum()
    }

    pub fn largest_utilization(taskset: &[T]) -> f64 {
        taskset.iter().map(LLTaskUtils::utilization).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn largest_density(taskset: &[T]) -> f64 {
        taskset.iter().map(LLTaskUtils::density).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn hyperperiod(taskset: &[T]) -> Time {
        let hyperperiod =
            taskset.iter()
            .map(|t| t.get_period().as_nanos() as i64)
            .fold(1, |lcm, period| num::integer::lcm(lcm, period));

        Time::nanos(hyperperiod as f64)
    }
}

impl<T> LLTasksetUtils<T>
    where T: task_traits::LLTask + task_traits::Priority
{
    pub fn rate_monotonic_priority_assignment(taskset: &[T]) -> bool {
        let n = taskset.len();
        for i in 0 .. n {
            let task_i = &taskset[i];
            for j in i+1 .. n {
                let task_j = &taskset[j];
                if (task_i.get_priority() == task_j.get_priority()) ||
                   (task_i.get_period() < task_j.get_period() && task_i.get_priority() > task_j.get_priority()) ||
                   (task_i.get_period() > task_j.get_period() && task_i.get_priority() < task_j.get_priority()) {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn deadline_monotonic_priority_assignment(taskset: &[T]) -> bool {
        let n = taskset.len();
        for i in 0 .. n {
            let task_i = &taskset[i];
            for j in i+1 .. n {
                let task_j = &taskset[j];
                if (task_i.get_priority() == task_j.get_priority()) ||
                   (task_i.get_deadline() < task_j.get_deadline() && task_i.get_priority() > task_j.get_priority()) ||
                   (task_i.get_deadline() > task_j.get_deadline() && task_i.get_priority() < task_j.get_priority()) {
                    return false;
                }
            }
        }

        return true;
    }

}
