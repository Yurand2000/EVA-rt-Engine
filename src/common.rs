pub mod prelude {
    pub use super::{
        Time,
        Time2,
        RTTask,
        RTUtils,
    };
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Time {
    value_ns: f64
}

#[derive(Clone, Copy)]
pub struct Time2 {
    value: f64
}

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
    pub const MICRO_TO_NANO: f64 = 1000.0;
    pub const MILLI_TO_NANO: f64 = 1000_000.0;
    pub const SECS_TO_NANO: f64 = 1000_000_000.0;

    pub fn zero() -> Self {
        Self { value_ns: 0.0 }
    }

    pub fn one() -> Self {
        Self { value_ns: 1.0 }
    }

    pub fn nanos(time_ns: f64) -> Self {
        Self { value_ns: time_ns }
    }

    pub fn micros(time_us: f64) -> Self {
        Self { value_ns: time_us * Self::MICRO_TO_NANO }
    }

    pub fn millis(time_ms: f64) -> Self {
        Self { value_ns: time_ms * Self::MILLI_TO_NANO }
    }
    pub fn as_nanos(&self) -> f64 {
        self.value_ns
    }

    pub fn as_micros(&self) -> f64 {
        self.value_ns / Self::MICRO_TO_NANO
    }

    pub fn as_millis(&self) -> f64 {
        self.value_ns / Self::MILLI_TO_NANO
    }

    pub fn floor(self) -> Self {
        Self { value_ns: f64::floor(self.value_ns) }
    }

    pub fn ceil(self) -> Self {
        Self { value_ns: f64::ceil(self.value_ns) }
    }

    pub fn round(self) -> Self {
        Self { value_ns: f64::round(self.value_ns) }
    }

    pub fn positive_or_zero(self) -> Self {
        Time::max(self, Time::zero())
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        let error = 0.5;

        f64::abs(self.value_ns - other.value_ns) < error
    }
}

impl Eq for Time { }

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ordered_float::OrderedFloat(self.value_ns)
            .partial_cmp(&ordered_float::OrderedFloat(other.value_ns))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ordered_float::OrderedFloat(self.value_ns)
            .cmp(&ordered_float::OrderedFloat(other.value_ns))
    }
}

impl std::ops::Neg for Time {
    type Output = Time;

    fn neg(self) -> Self::Output {
        Self::Output { value_ns: -self.value_ns }
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

impl std::ops::Mul<f64> for Time {
    type Output = Time;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output { value_ns: (self.value_ns * rhs) }
    }
}

impl std::ops::Mul<Time> for f64 {
    type Output = Time;

    fn mul(self, rhs: Time) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Div for Time {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.value_ns / rhs.value_ns
    }
}

impl std::ops::Div<f64> for Time {
    type Output = Time;

    fn div(self, rhs: f64) -> Self::Output {
        Time { value_ns: self.value_ns / rhs }
    }
}

impl std::ops::Rem for Time {
    type Output = f64;

    fn rem(self, rhs: Self) -> Self::Output {
        self.value_ns.floor() % rhs.value_ns.floor()
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
            let time: f64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;

            Ok(Time { value_ns: time })
        } else if pieces.len() == 2 {
            let time: f64 = pieces[0].parse()
                .map_err(|err| serde::de::Error::custom(format!("Invalid time: {err}")))?;
            let unit = match pieces[1] {
                "s" => Time::SECS_TO_NANO,
                "ms" => Time::MILLI_TO_NANO,
                "us" => Time::MICRO_TO_NANO,
                "ns" => 1.0,
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
            wcet: Time::nanos(wcet as f64),
            deadline: Time::nanos(deadline as f64),
            period: Time::nanos(period as f64),
        }
    }

    pub fn utilization(&self) -> f64 {
        self.wcet.value_ns / self.period.value_ns
    }

    pub fn density(&self) -> f64 {
        self.wcet.value_ns / self.deadline.value_ns
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

    pub fn total_utilization(taskset: &[RTTask]) -> f64 {
        taskset.iter()
            .map(RTTask::utilization)
            .sum()
    }

    pub fn largest_utilization(taskset: &[RTTask]) -> f64 {
        let max = taskset.iter()
            .map(|t| ordered_float::OrderedFloat(RTTask::utilization(t)))
            .max();

        match max {
            Some(max) => *max,
            None => 0f64,
        }
    }

    pub fn total_density(taskset: &[RTTask]) -> f64 {
        taskset.iter()
            .map(RTTask::density)
            .sum()
    }

    pub fn largest_density(taskset: &[RTTask]) -> f64 {
        let max = taskset.iter()
            .map(|t| ordered_float::OrderedFloat(RTTask::density(t)))
            .max();

        match max {
            Some(max) => *max,
            None => 0f64,
        }
    }

    pub fn hyperperiod(taskset: &[RTTask]) -> Time {
        let hyperperiod =
            taskset.iter()
            .map(|task| task.period.as_nanos().floor() as i64)
            .fold(1, |lcm, period| num::integer::lcm(lcm, period));

        Time { value_ns: hyperperiod as f64 }
    }
}

// -----------------------------------------------------------------------------

impl Time2 {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl std::ops::Neg for Time2 {
    type Output = Time2;

    fn neg(self) -> Self::Output {
        Self::Output { value: -self.value }
    }
}

impl std::ops::Add for Time2 {
    type Output = Time2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { value: (self.value + rhs.value) }
    }
}

impl std::ops::Sub for Time2 {
    type Output = Time2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output { value: (self.value - rhs.value) }
    }
}

impl std::ops::Mul<Time> for Time {
    type Output = Time2;

    fn mul(self, rhs: Time) -> Self::Output {
        Self::Output { value: (self.value_ns * rhs.value_ns) }
    }
}

impl std::ops::Mul<f64> for Time2 {
    type Output = Time2;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output { value: (self.value * rhs) }
    }
}

impl std::ops::Mul<Time2> for f64 {
    type Output = Time2;

    fn mul(self, rhs: Time2) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Div<Time> for Time2 {
    type Output = Time;

    fn div(self, rhs: Time) -> Self::Output {
        Self::Output { value_ns: self.value / rhs.value_ns }
    }
}

impl std::ops::Div<f64> for Time2 {
    type Output = Time2;

    fn div(self, rhs: f64) -> Self::Output {
        Self::Output { value: self.value / rhs }
    }
}