pub use crate::prelude::*;

pub fn new_ll_task(
    wcet: u64,
    deadline: u64,
    period: u64,
) -> LLTask {
    LLTask {
        wcet: Time::nanos(wcet as f64),
        deadline: Time::nanos(deadline as f64),
        period: Time::nanos(period as f64),
    }
}

pub fn new_prio_task(
    wcet: u64,
    deadline: u64,
    period: u64,
    priority: usize
) -> PriorityTask<LLTask> {
    PriorityTask {
        task: new_ll_task(wcet, deadline, period),
        priority,
    }
}
