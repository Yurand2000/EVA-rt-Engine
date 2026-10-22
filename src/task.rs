use crate::prelude::*;

pub mod prelude {
    pub use super::{
        PriorityTask,
        Taskset,
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

impl<Task> std::borrow::Borrow<Task> for PriorityTask<Task> {
    fn borrow(&self) -> &Task {
        &self.task
    }
}

impl<Task> std::borrow::BorrowMut<Task> for PriorityTask<Task> {
    fn borrow_mut(&mut self) -> &mut Task {
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
    pub fn implicit_deadlines<'a, Taskset, Task>(taskset: Taskset) -> bool
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().all(|t| LLTask::has_implicit_deadline(t.borrow()))
    }

    pub fn constrained_deadlines<'a, Taskset, Task>(taskset: Taskset) -> bool
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().all(|t| LLTask::has_constrained_deadline(t.borrow()))
    }

    pub fn total_utilization<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().map(|t| LLTask::utilization(t.borrow())).sum()
    }

    pub fn total_density<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().map(|t| LLTask::density(t.borrow())).sum()
    }

    pub fn largest_utilization<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().map(|t| LLTask::utilization(t.borrow())).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn largest_density<'a, Taskset, Task>(taskset: Taskset) -> f64
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        taskset.into_iter().map(|t| LLTask::density(t.borrow())).max_by(f64::total_cmp).unwrap_or(0.0)
    }

    pub fn hyperperiod<'a, Taskset, Task>(taskset: Taskset) -> Time
        where Taskset: IntoIterator<Item = &'a Task>,
              Task: std::borrow::Borrow<LLTask> + 'static,
    {
        let hyperperiod =
            taskset.into_iter()
            .map(|t| t.borrow().period.as_nanos() as i64)
            .fold(1, |lcm, period| num::integer::lcm(lcm, period));

        Time::nanos(hyperperiod as f64)
    }
}

impl Taskset<PriorityTask<LLTask>> {
    pub fn rate_monotonic_priority_assignment<'a, Taskset, Iter>(taskset: Taskset) -> bool
        where Taskset: IntoIterator<Item = &'a PriorityTask<LLTask>, IntoIter = Iter>,
              Iter: Iterator<Item = &'a PriorityTask<LLTask>> + Clone,
    {
        let mut taskset = taskset.into_iter();
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

    pub fn deadline_monotonic_priority_assignment<'a, Taskset, Iter>(taskset: Taskset) -> bool
        where Taskset: IntoIterator<Item = &'a PriorityTask<LLTask>, IntoIter = Iter>,
              Iter: Iterator<Item = &'a PriorityTask<LLTask>> + Clone,
    {
        let mut taskset = taskset.into_iter();
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

