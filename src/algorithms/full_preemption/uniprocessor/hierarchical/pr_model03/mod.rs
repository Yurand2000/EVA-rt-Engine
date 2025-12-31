//! ## Periodic Resource Model - Shin & Lee 2003
//!
//! Generic implementation for demand based analysis and response based analysis
//! for the Periodic Resource model. Can supply custom functions/formulas to
//! derive schedulability analyses for scheduling algorithms other than EDF and
//! Fixed Priority.
//!
//! #### Implements:
//! - [`PRModel`] \
//!   | Periodic Resource Model
//! - [`is_schedulable_demand`] \
//!   | Generic implementation for demand based analysis. \
//!   | \
//!   | O(*demand_fn*) \* O(*time_intervals*) complexity. \
//!   | pseudo-polynomial if the number time intervals to check depends on the taskset.
//! - [`is_schedulable_response`] \
//!   | Generic implementation for response time based analysis. \
//!   | \
//!   | pseudo-polynomial complexity \
//!   | (depends on the rate of convergence of the RTA analysis)
//!
//! ---
//! #### References:
//! 1. Shin and I. Lee, “Periodic resource model for compositional real-time
//!    guarantees,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003,
//!    Dec. 2003, pp. 2–13. doi: 10.1109/REAL.2003.1253249.

use crate::prelude::*;

// Local Scheduling Algorithms
pub mod earliest_deadline_first {
    pub mod shin_lee03;
}

pub mod fixed_priority {
    pub mod shin_lee03;
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\]
///
/// Refer to the [module](`self`) level documentation.
#[derive(Debug, Clone)]
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

    pub fn get_supply(&self, interval: Time) -> Time {
        // Equation 1 [1]
        let diff = self.period - self.resource;

        let base = ((interval - diff) / self.period).floor();

        base * self.resource
            +
        Time::max(interval - 2.0 * diff - self.period * base, Time::zero())
    }

    pub fn get_linear_supply(&self, interval: Time) -> Time {
        // Lemma 1 [1]
        self.capacity() * (interval - 2.0 * (self.period - self.resource))
    }

    pub fn get_interval_from_supply(&self, supply: Time) -> Time {
        // Equation 6, 7 [1]
        let diff = self.period - self.resource;

        diff + self.period * (supply / self.resource).floor()
            +
        Time::max(diff + supply - self.resource * (supply / self.resource), Time::zero())
    }

    pub fn get_linear_interval_from_supply(&self, supply: Time) -> Time {
        // Lemma 2 [1]
        (self.period / self.resource) * supply + 2.0 * (self.period - self.resource)
    }
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation for demand based analysis.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_demand<FDem, FTime>(
    test_name: &str,
    taskset: &[RTTask],
    model: &PRModel,
    demand_fn: FDem,
    time_intervals_fn: FTime,
) -> SchedResult<()>
    where
        FDem: Fn(&[RTTask], Time) -> Time,
        FTime: Fn(&[RTTask]) -> Box<dyn Iterator<Item = Time>>,
{
    let mut time_intervals = time_intervals_fn(taskset);

    let schedulable = time_intervals.all(|time|
        demand_fn(taskset, time) <= model.get_supply(time)
    );

    SchedResultFactory(test_name).is_schedulable(schedulable)
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation for response time based analysis.
///
/// Note: The response time analysis `rta_fn` function must be a **monotone**
/// function to guarantee that the computation terminates.
///
/// Refer to the [module](`self`) level documentation.
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