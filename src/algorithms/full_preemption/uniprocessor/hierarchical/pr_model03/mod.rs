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
//! - [`generate_model_demand_linear`] \
//!   | Generic implementation for deriving the appropriate PRModel for a given
//!     taskset and model's period using demand analysis. Uses the linear
//!     approximation functions to derive the resource requirements. \
//!   | \
//!   | O(*demand_fn*) \* O(*time_intervals*) complexity. \
//!   | pseudo-polynomial if the number time intervals to check depends on the taskset.
//! - [`generate_model_response_linear`] \
//!   | Generic implementation for deriving the appropriate PRModel for a given
//!     taskset and model's period using response time analysis. Uses the linear
//!     approximation functions to derive the resource requirements. \
//!   | \
//!   | O(*n*) * O(rta_fn) complexity
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

    pub fn get_supply_linear(&self, interval: Time) -> Time {
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

    pub fn get_interval_from_supply_linear(&self, supply: Time) -> Time {
        // Lemma 2 [1]
        (self.period / self.resource) * supply + 2.0 * (self.period - self.resource)
    }

    /// Gets the resource necessary to provide a given supply in the given interval.
    ///
    /// Inverts [`PRModel::get_linear_supply`] on the resource field of the [`PRModel`]
    /// or equivalently inverts [`PRModel::get_linear_interval_from_supply`].
    pub fn get_resource_linear(supply: Time, interval: Time, period: Time) -> Time {
        let b = interval - 2.0 * period;

        (- b + Time2::sqrt(b * b + 8.0 * period * supply)) / 4.0
    }
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation for demand based analysis.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_demand<FDem, FTime>(
    taskset: &[RTTask],
    model: &PRModel,
    demand_fn: FDem,
    time_intervals_fn: FTime,
) -> bool
    where
        FDem: Fn(&[RTTask], Time) -> Time,
        FTime: Fn(&[RTTask]) -> Box<dyn Iterator<Item = Time>>,
{
    let mut time_intervals = time_intervals_fn(taskset);

    time_intervals.all(|time|
        demand_fn(taskset, time) <= model.get_supply(time)
    )
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation for response time based analysis.
///
/// Note: The response time analysis `rta_fn` function must be a **monotone**
/// function to guarantee that the computation terminates.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_response<FRTA>(
    taskset: &[RTTask],
    model: &PRModel,
    rta_fn: FRTA,
) -> Result<Vec<Time>, anyhow::Error>
    where
        FRTA: Fn(&[RTTask], usize, &RTTask, Time) -> Time,
{
    taskset.iter().enumerate()
        .map(|(k, task_k)| {
            let response =
                fixpoint_search_with_limit(
                    task_k.wcet,
                    task_k.deadline,
                    |response: &Time|
                        model.get_interval_from_supply(
                            rta_fn(taskset, k, task_k, *response)
                        )
                );

            if response > task_k.deadline {
                return Err(anyhow::format_err!("Response time of task {k} is greater than its deadline."));
            } else {
                return Ok(response);
            }
        })
        .collect()
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation generating the best [`PRModel`] using demand analysis.
///
/// Refer to the [module](`self`) level documentation.
pub fn generate_model_demand_linear<FDem, FTime>(
    taskset: &[RTTask],
    model_period: Time,
    demand_fn: FDem,
    time_intervals_fn: FTime,
) -> Option<PRModel>
    where
        FDem: Fn(&[RTTask], Time) -> Time,
        FTime: Fn(&[RTTask]) -> Box<dyn Iterator<Item = Time>>,
{
    let time_intervals = time_intervals_fn(taskset);

    let min_resource =
        time_intervals
        .map(|time| {
            let supply = demand_fn(taskset, time);
            PRModel::get_resource_linear(supply, time, model_period)
        })
        .max()?;

    let model = PRModel {
        resource: min_resource,
        period: model_period,
    };

    if model.is_feasible() {
        Some(model)
    } else {
        None
    }
}

/// Periodic Resource Model - Shin & Lee 2003 \[1\] \
/// Generic implementation generating the best [`PRModel`] using response time analysis.
///
/// Refer to the [module](`self`) level documentation.
pub fn generate_model_response_linear<FRTA>(
    taskset: &[RTTask],
    model_period: Time,
    rta_fn: FRTA,
) -> Option<PRModel>
    where
        FRTA: Fn(&[RTTask], usize, &RTTask, Time) -> Time,
{
    let min_resource =
        taskset.iter().enumerate()
        .map(|(k, task_k)| {
            let supply = rta_fn(taskset, k, task_k, task_k.period);
            PRModel::get_resource_linear(supply, task_k.period, model_period)
        })
        .max()?;

    let model = PRModel {
        resource: min_resource,
        period: model_period,
    };

    if model.is_feasible() {
        Some(model)
    } else {
        None
    }
}