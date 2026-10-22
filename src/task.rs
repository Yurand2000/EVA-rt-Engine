use crate::prelude::*;

pub mod prelude {
    pub use super::{
        PriorityTask,
        LLTask,
    };
}

pub struct PriorityTask<Task> {
    pub task: Task,
    pub priority: usize,
}

impl<Task> std::ops::Deref for PriorityTask<Task> {
    type Target = Task;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}

impl<Task> std::ops::DerefMut for PriorityTask<Task> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}

pub struct LLTask {
    pub wcet: Time,
    pub deadline: Time,
    pub period: Time,
}

impl LLTask {
    pub fn has_implicit_deadline(&self) -> bool {
        self.deadline == self.period
    }

    pub fn has_constrained_deadline(&self) -> bool {
        self.deadline <= self.period
    }

    pub fn utilization(&self) -> f64 {
        self.wcet / self.period
    }

    pub fn density(&self) -> f64 {
        self.wcet / self.deadline
    }
}

pub struct Taskset<T>(std::marker::PhantomData<T>);

impl Taskset<LLTask> {
    pub fn implicit_deadlines<'a, Taskset, Task>(mut taskset: Taskset) -> bool
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.all(|t| LLTask::has_implicit_deadline(t.deref()))
    }

    pub fn constrained_deadlines<'a, Taskset, Task>(mut taskset: Taskset) -> bool
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.all(|t| LLTask::has_constrained_deadline(t.deref()))
    }

    pub fn total_utilization<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.map(|t| LLTask::utilization(t.deref())).sum()
    }

    pub fn total_density<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.map(|t| LLTask::density(t.deref())).sum()
    }

    pub fn largest_utilization<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.map(|t| LLTask::utilization(t.deref())).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn largest_density<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        taskset.map(|t| LLTask::density(t.deref())).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn hyperperiod<'a, Taskset, Task>(taskset: Taskset) -> Time
        where Taskset: Iterator<Item = &'a Task>,
              Task: std::ops::Deref<Target = LLTask> + 'static
    {
        let hyperperiod =
            taskset
            .map(|t| t.period.as_nanos() as i64)
            .fold(1, |lcm, period| num::integer::lcm(lcm, period));

        Time::nanos(hyperperiod as f64)
    }
}

impl Taskset<PriorityTask<LLTask>> {
    pub fn rate_monotonic_priority_assignment<'a, Taskset>(mut taskset: Taskset) -> bool
        where Taskset: Iterator<Item = &'a PriorityTask<LLTask>> + Clone,
    {
        while let Some(task_i) = taskset.next() {
            if taskset.clone().any(|task_j|
                (task_i.priority == task_j.priority) ||
                (task_i.period < task_j.period && task_i.priority > task_j.priority) ||
                (task_i.period > task_j.period && task_i.priority < task_j.priority)
            ) {
                return false;
            }
        }

        return true;
    }

    pub fn deadline_monotonic_priority_assignment<'a, Taskset>(mut taskset: Taskset) -> bool
        where Taskset: Iterator<Item = &'a PriorityTask<LLTask>> + Clone,
    {
        while let Some(task_i) = taskset.next() {
            if taskset.clone().any(|task_j|
                (task_i.priority == task_j.priority) ||
                (task_i.deadline < task_j.deadline && task_i.priority > task_j.priority) ||
                (task_i.deadline > task_j.deadline && task_i.priority < task_j.priority)
            ) {
                return false;
            }
        }

        return true;
    }
}

