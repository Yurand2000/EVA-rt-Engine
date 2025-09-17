pub mod prelude {
    pub use super::taskset_serde::prelude::*;
    pub use super::{
        Time,
        RTTask,
        RTUtils,
        RTBandwidth
    };
}

pub mod taskset_serde;

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub value_ns: u64
}

pub type RTBandwidth = f64;

#[derive(Clone)]
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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
    const SECS_TO_NANO: u64 = 1000_000_000;

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

impl std::ops::Add for Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { value_ns: (self.value_ns + rhs.value_ns) }
    }
}

impl std::ops::Sub for Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output { value_ns: (self.value_ns - rhs.value_ns) }
    }
}

impl std::ops::Mul<u64> for Time {
    type Output = Time;

    fn mul(self, rhs: u64) -> Self::Output {
        Self::Output { value_ns: (self.value_ns * rhs) }
    }
}

impl std::ops::Mul<Time> for u64 {
    type Output = Time;

    fn mul(self, rhs: Time) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Div for Time {
    type Output = u64;

    fn div(self, rhs: Self) -> Self::Output {
        self.value_ns / rhs.value_ns
    }
}

impl std::ops::Div<u64> for Time {
    type Output = Time;

    fn div(self, rhs: u64) -> Self::Output {
        Time { value_ns: self.value_ns / rhs }
    }
}

impl std::iter::Sum for Time {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Time::zero(), |acc, val| acc + val)
    }
}

impl serde::Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        format!("{} ns", self.value_ns).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let time_string = String::deserialize(deserializer)?;
        
        let pieces: Vec<_> = time_string.trim().split_whitespace().collect();
        if pieces.len() == 1 {
            let time: u64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;

            Ok(Time { value_ns: time })
        } else if pieces.len() == 2 {
            let time: u64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;
            let unit = match pieces[1] {
                "s" => Time::SECS_TO_NANO,
                "ms" => Time::MILLI_TO_NANO,
                "us" => Time::MICRO_TO_NANO,
                "ns" => 1,
                u => { return Err(serde::de::Error::custom(format!("Unknown time unit: {u}"))); }
            };

            Ok(Time { value_ns: time * unit })
        } else {
            return Err(serde::de::Error::custom("Parsing error, unknown format"));
        }
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

    pub fn get_utilization(&self) -> RTBandwidth {
        (self.wcet.value_ns as RTBandwidth) / (self.period.value_ns as RTBandwidth)
    }

    pub fn get_density(&self) -> RTBandwidth {
        (self.wcet.value_ns as RTBandwidth) / (self.deadline.value_ns as RTBandwidth)
    }

    pub fn laxity(&self) -> Time {
        self.deadline - self.wcet
    }

    pub fn has_implicit_deadline(&self) -> bool {
        self.deadline == self.period
    }

    pub fn has_constrained_deadline(&self) -> bool {
        self.deadline <= self.period
    }
}

impl RTUtils {
    pub fn is_taskset_sorted_by_period(taskset: &[RTTask]) -> bool {
        taskset.windows(2).all(|w| w[0].period < w[1].period)
    }

    pub fn is_taskset_sorted_by_deadline(taskset: &[RTTask]) -> bool {
        taskset.windows(2).all(|w| w[0].deadline < w[1].deadline)
    }

    pub fn implicit_deadlines(taskset: &[RTTask]) -> bool {
        taskset.iter().all(RTTask::has_implicit_deadline)
    }

    pub fn constrained_deadlines(taskset: &[RTTask]) -> bool {
        taskset.iter().all(RTTask::has_constrained_deadline)
    }

    pub fn total_utilization(taskset: &[RTTask]) -> RTBandwidth {
        taskset.iter()
            .map(RTTask::get_utilization)
            .sum()
    }

    pub fn largest_utilization(taskset: &[RTTask]) -> RTBandwidth {
        let max = taskset.iter()
            .map(|t| ordered_float::OrderedFloat(RTTask::get_utilization(t)))
            .max();

        match max {
            Some(max) => *max,
            None => 0f64,
        }
    }

    pub fn total_density(taskset: &[RTTask]) -> RTBandwidth {
        taskset.iter()
            .map(RTTask::get_density)
            .sum()
    }

    pub fn largest_density(taskset: &[RTTask]) -> RTBandwidth {
        let max = taskset.iter()
            .map(|t| ordered_float::OrderedFloat(RTTask::get_density(t)))
            .max();

        match max {
            Some(max) => *max,
            None => 0f64,
        }
    }

    pub fn hyperperiod(taskset: &[RTTask]) -> Time {
        let hyperperiod_ns =
            taskset.iter()
            .map(|task| task.period.value_ns)
            .fold(1, |lcm, period| num::integer::lcm(lcm, period));

        Time { value_ns: hyperperiod_ns }
    }
}