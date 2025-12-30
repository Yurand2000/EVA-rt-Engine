//! ## Periodic Resource Model - Shin & Lee 2003
//!
//! #### Implements:
//! - [`is_schedulable_demand`] \
//!   | pseudo-polynomial complexity
//! - [`is_schedulable_response`] \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;

pub struct PRModel {
    pub resource: Time,
    pub period: Time,
}

impl PRModel {
    pub fn is_feasible(&self) -> bool {
        self.resource <= self.period
    }

    pub fn capacity(&self) -> f64 {
        self.resource / self.period
    }

    // Equation 1
    pub fn get_supply(&self, interval: Time) -> Time {
        let diff = self.period - self.resource;

        let base = ((interval - diff) / self.period).floor();

        base * self.resource
            +
        Time::max(interval - 2.0 * diff - self.period * base, Time::zero())
    }

    // Lemma 1
    pub fn get_linear_supply(&self, interval: Time) -> Time {
        self.capacity() * (interval - 2.0 * (self.period - self.resource))
    }

    // Equation 6, 7
    pub fn get_interval_from_supply(&self, supply: Time) -> Time {
        let diff = self.period - self.resource;

        diff + self.period * (supply / self.resource).floor()
            +
        Time::max(diff + supply - self.resource * (supply / self.resource), Time::zero())
    }

    // Lemma 2
    pub fn get_linear_interval_from_supply(&self, supply: Time) -> Time {
        (self.period / self.resource) * supply + 2.0 * (self.period - self.resource)
    }
}

pub fn is_schedulable_demand<FDem>(
    test_name: &str,
    taskset: &[RTTask],
    model: &PRModel,
    demand_fn: FDem,
) -> SchedResult<()>
    where
        FDem: Fn(&[RTTask], Time) -> Time,
{
    let max_time = 2.0 * RTUtils::hyperperiod(taskset);

    let schedulable =
        (0 ..= max_time.as_nanos() as u64)
        .map(|time| Time::nanos(time as f64))
        .all(|time| demand_fn(taskset, time) <= model.get_supply(time) );

    SchedResultFactory(test_name).is_schedulable(schedulable)
}

pub fn is_schedulable_response<FRTA>(
    test_name: &str,
    taskset: &[RTTask],
    model: &PRModel,
    rta_fn: FRTA,
) -> SchedResult<Vec<Time>>
    where
        FRTA: Fn(&[RTTask], usize, &RTTask, Time) -> Time,
{
    let schedulable: Result<Vec<Time>, usize> =
        taskset.iter().enumerate()
        .map(|(k, task_k)| {
            let mut response = task_k.wcet;

            loop {
                let new_response =
                    model.get_interval_from_supply(
                        rta_fn(taskset, k, task_k, response)
                    );

                if new_response > task_k.deadline {
                    return Err(k);
                } else if new_response == response {
                    return Ok(new_response);
                }

                response = new_response;
            }
        })
        .collect();

    match schedulable {
        Ok(respose_times) => SchedResultFactory(test_name).schedulable(respose_times),
        Err(k) => SchedResultFactory(test_name).non_schedulable_reason(
            anyhow::format_err!("Response time of task {k} is greater than its deadline.")),
    }
}