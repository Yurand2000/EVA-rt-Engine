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
    value_ns_w_precision: i64
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
    pub const MICRO_TO_NANO: i64 = 1000;
    pub const MILLI_TO_NANO: i64 = 1000_000;
    pub const SECS_TO_NANO: i64 = 1000_000_000;
    const PRECISION_BITS: i64 = 10;
    pub const PRECISION: i64 = 1 << Self::PRECISION_BITS;
    const PRECISION_MASK: i64 = Self::PRECISION - 1;

    pub fn zero() -> Self {
        Self { value_ns_w_precision: 0 }
    }

    pub fn one() -> Self {
        Self { value_ns_w_precision: Self::PRECISION }
    }

    pub fn nanos_f64(time_ns: f64) -> Self {
        Self { value_ns_w_precision: (time_ns * Self::PRECISION as f64) as i64 }
    }

    pub fn nanos(time_ns: i64) -> Self {
        Self { value_ns_w_precision: time_ns * Self::PRECISION }
    }

    pub fn micros(time_us: i64) -> Self {
        Self { value_ns_w_precision: time_us * Self::MICRO_TO_NANO * Self::PRECISION }
    }

    pub fn millis(time_ms: i64) -> Self {
        Self { value_ns_w_precision: time_ms * Self::MILLI_TO_NANO * Self::PRECISION }
    }

    pub fn raw(time_raw: i64) -> Self {
        Self { value_ns_w_precision: time_raw }
    }

    pub fn raw128(time_raw: i128) -> Self {
        Self { value_ns_w_precision: time_raw as i64 }
    }

    pub fn as_nanos_f64(&self) -> f64 {        
        self.value_ns_w_precision as f64 / Self::PRECISION as f64 
    }

    pub fn as_nanos(&self) -> i64 {
        rounded_div::i64(self.value_ns_w_precision, Self::PRECISION)        
    }

    pub fn as_micros(&self) -> i64 {
        rounded_div::i64(self.value_ns_w_precision, Self::MICRO_TO_NANO * Self::PRECISION)
    }

    pub fn as_millis(&self) -> i64 {
        rounded_div::i64(self.value_ns_w_precision, Self::MILLI_TO_NANO * Self::PRECISION)
    }

    pub fn as_raw(&self) -> i64 {
        self.value_ns_w_precision
    }

    pub fn as_raw_128(&self) -> i128 {
        self.value_ns_w_precision as i128
    }

    pub fn div_floor_i64(self, rhs: i64) -> Self {
        Time::raw(i64::div_floor(self.value_ns_w_precision, rhs))
    }

    pub fn div_ceil_i64(self, rhs: i64) -> Self {
        Time::raw(i64::div_ceil(self.value_ns_w_precision, rhs))
    }

    pub fn div_floor_time(self, rhs: Self) -> i64 {
        i64::div_floor(self.value_ns_w_precision, rhs.value_ns_w_precision)
    }

    pub fn div_ceil_time(self, rhs: Self) -> i64 {
        i64::div_ceil(self.value_ns_w_precision, rhs.value_ns_w_precision)
    }

    pub fn floor(self) -> Self {
        Self { value_ns_w_precision: self.value_ns_w_precision & (!Self::PRECISION_MASK) }
    }

    pub fn ceil(self) -> Self {
        if self.precision_bits() > 0 {
            self.floor() + Time::one()
        } else {
            self.floor()
        }
    }

    pub fn round(self) -> Self {
        if 2 * self.precision_bits() >= Self::PRECISION {
            self.floor() + Time::one()
        } else {
            self.floor()
        }
    }

    #[inline(always)]
    fn precision_bits(&self) -> i64 {
        self.value_ns_w_precision & Self::PRECISION_MASK
    }

    pub fn positive_or_zero(self) -> Self {
        Time::max(self, Time::zero())
    }
}

impl std::ops::Add for Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { value_ns_w_precision: (self.value_ns_w_precision + rhs.value_ns_w_precision) }
    }
}

impl std::ops::Sub for Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output { value_ns_w_precision: (self.value_ns_w_precision - rhs.value_ns_w_precision) }
    }
}

impl std::ops::Mul<i64> for Time {
    type Output = Time;

    fn mul(self, rhs: i64) -> Self::Output {
        Self::Output { value_ns_w_precision: (self.value_ns_w_precision * rhs) }
    }
}

impl std::ops::Mul<Time> for i64 {
    type Output = Time;

    fn mul(self, rhs: Time) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Div for Time {
    type Output = i64;

    fn div(self, rhs: Self) -> Self::Output {
        self.value_ns_w_precision / rhs.value_ns_w_precision
    }
}

impl std::ops::Div<i64> for Time {
    type Output = Time;

    fn div(self, rhs: i64) -> Self::Output {
        Time { value_ns_w_precision: self.value_ns_w_precision / rhs }
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
        format!("{} ns", self.value_ns_w_precision).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let time_string = String::deserialize(deserializer)?;
        
        let pieces: Vec<_> = time_string.trim().split_whitespace().collect();
        if pieces.len() == 1 {
            let time: i64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;

            Ok(Time { value_ns_w_precision: time })
        } else if pieces.len() == 2 {
            let time: i64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;
            let unit = match pieces[1] {
                "s" => Time::SECS_TO_NANO,
                "ms" => Time::MILLI_TO_NANO,
                "us" => Time::MICRO_TO_NANO,
                "ns" => 1,
                u => { return Err(serde::de::Error::custom(format!("Unknown time unit: {u}"))); }
            };

            Ok(Time { value_ns_w_precision: time * unit * Self::PRECISION })
        } else {
            return Err(serde::de::Error::custom("Parsing error, unknown format"));
        }
    }
}

impl RTTask {
    pub fn new_ns(wcet: u64, deadline: u64, period: u64) -> Self {
        Self {
            wcet: Time::nanos(wcet as i64),
            deadline: Time::nanos(deadline as i64),
            period: Time::nanos(period as i64),
        }
    }

    pub fn get_utilization(&self) -> RTBandwidth {
        (self.wcet.value_ns_w_precision as RTBandwidth) / (self.period.value_ns_w_precision as RTBandwidth)
    }

    pub fn get_density(&self) -> RTBandwidth {
        (self.wcet.value_ns_w_precision as RTBandwidth) / (self.deadline.value_ns_w_precision as RTBandwidth)
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
        let hyperperiod =
            taskset.iter()
            .map(|task| task.period.value_ns_w_precision)
            .fold(Time::one().value_ns_w_precision, |lcm, period| num::integer::lcm(lcm, period));

        Time { value_ns_w_precision: hyperperiod }
    }
}