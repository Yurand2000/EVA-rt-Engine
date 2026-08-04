use crate::prelude::*;

pub mod prelude {
    pub use super::traits as task_traits;
    pub use super::{
        LLTask,
        PriorityTask,
    };
}

pub mod traits {
    use crate::prelude::Time;

    pub trait LLTask {
        fn get_wcet(&self) -> Time;
        fn get_deadline(&self) -> Time;
        fn get_period(&self) -> Time;
    }

    pub trait Priority {
        fn get_priority(&self) -> usize;
    }

    pub trait LLTaskUtils {
        fn has_implicit_deadline(&self) -> bool;
        fn has_constrained_deadline(&self) -> bool;
        fn utilization(&self) -> f64;
        fn density(&self) -> f64;
    }

    impl<T: LLTask> LLTaskUtils for T {
        fn has_implicit_deadline(&self) -> bool {
            self.get_deadline() == self.get_period()
        }

        fn has_constrained_deadline(&self) -> bool {
            self.get_deadline() <= self.get_period()
        }

        fn utilization(&self) -> f64 {
            self.get_wcet() / self.get_period()
        }

        fn density(&self) -> f64 {
            self.get_wcet() / self.get_deadline()
        }
    }
}

pub struct LLTask {
    pub wcet: Time,
    pub deadline: Time,
    pub period: Time,
}

impl traits::LLTask for LLTask {
    fn get_wcet(&self) -> Time {
        self.wcet
    }

    fn get_deadline(&self) -> Time {
        self.deadline
    }

    fn get_period(&self) -> Time {
        self.period
    }
}


pub struct PriorityTask {
    pub task: LLTask,
    pub priority: usize,
}

impl std::ops::Deref for PriorityTask {
    type Target = LLTask;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}

impl std::ops::DerefMut for PriorityTask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}

impl traits::LLTask for PriorityTask {
    fn get_wcet(&self) -> Time {
        self.wcet
    }

    fn get_deadline(&self) -> Time {
        self.deadline
    }

    fn get_period(&self) -> Time {
        self.period
    }
}

impl traits::Priority for PriorityTask {
    fn get_priority(&self) -> usize {
        self.priority
    }
}
