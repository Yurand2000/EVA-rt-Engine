use std::ops::Add;

pub mod prelude {
    pub use super::{
        Time,
        RTTask,
        RTUtils,
        RTUtilization
    };
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub value_ns: u64
}

pub type RTUtilization = f64;

#[derive(Clone)]
#[derive(Debug)]
pub struct RTTask {
    pub wcet: Time,
    pub deadline: Time,
    pub period: Time,
}

pub struct RTUtils;

// =============================================================================

impl Time {
    const MICRO_TO_NANO: u64 = 1000;
    const MILLI_TO_NANO: u64 = 1000_000;

    pub fn zero() -> Self {
        Self { value_ns: 0 }
    }

    pub fn nanos(time_ns: u64) -> Self {
        Self { value_ns: time_ns }
    }

    pub fn micros(time_us: u64) -> Self {
        Self { value_ns: time_us * Self::MICRO_TO_NANO }
    }

    pub fn millis(time_ms: u64) -> Self {
        Self { value_ns: time_ms * Self::MILLI_TO_NANO }
    }

    pub fn as_nanos(&self) -> u64 {
        self.value_ns
    }

    pub fn as_micros(&self) -> u64 {
        self.value_ns / Self::MICRO_TO_NANO
    }

    pub fn as_millis(&self) -> u64 {
        self.value_ns / Self::MILLI_TO_NANO
    }
}

impl Add for Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { value_ns: (self.value_ns + rhs.value_ns) }
    }
}

impl RTTask {
    pub fn new_ns(wcet: u64, deadline: u64, period: u64) -> Self {
        Self {
            wcet: Time::nanos(wcet),
            deadline: Time::nanos(deadline),
            period: Time::nanos(period),
        }
    }

    pub fn get_utilization(&self) -> RTUtilization {
        (self.wcet.as_nanos() as RTUtilization) / (self.period.as_nanos() as RTUtilization)
    }
}

impl RTUtils {
    pub fn is_taskset_sorted_by_period(taskset: &[RTTask]) -> bool {
        taskset.windows(2).all(|w| w[0].period.value_ns < w[1].period.value_ns)
    }

    pub fn get_worst_case_utilization(taskset: &[RTTask]) -> RTUtilization {
        taskset.iter()
            .map(|task| task.get_utilization())
            .sum()
    }
}