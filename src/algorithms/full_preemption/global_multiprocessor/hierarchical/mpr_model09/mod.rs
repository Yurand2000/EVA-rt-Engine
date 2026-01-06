//! ## Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
//!
//! #### Implements:
//! - [`MPRModel`] \
//!   | Multi-Processor Periodic Resource Model
//! - [`is_schedulable_demand`] \
//!   | O(*taskset_size*) * O(*arrival_times*) * O(*demand_fn) complexity
//! - [`generate_model_from_demand_linear`] \
//!   | O(*taskset_size*) * O(*arrival_times*) * O(*demand_fn) complexity
//!
//! ---
//! #### References:
//! 1. I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
//!    Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
//!    Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
//! 2. A. Easwaran, I. Shin, and I. Lee, “Optimal virtual cluster-based
//!    multiprocessor scheduling,” Real-Time Syst, vol. 43, no. 1, pp. 25–59,
//!    Sept. 2009, doi: 10.1007/s11241-009-9073-x.

use crate::prelude::*;

// Local Scheduling Algorithms
pub mod earliest_deadline_first {
    pub mod shin_easwaran_lee09;
    pub mod bcl09;
}

pub mod fixed_priority {
    pub mod bcl09;
    pub mod yurand26;
}

/// Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
///
/// Refer to the [module](`self`) level documentation.
#[derive(Debug, Clone)]
pub struct MPRModel {
    // Section 3.2 [1]
    pub resource: Time,
    pub period: Time,
    pub concurrency: u64,
}

impl MPRModel {
    pub fn is_feasible(&self) -> bool {
        self.resource <= self.concurrency as f64 * self.period
    }

    /// resource / period
    pub fn utilization(&self) -> f64 {
        self.resource / self.period
    }

    /// Get the total supply the model provides in the given time interval.
    pub fn get_supply(&self, interval: Time) -> Time {
        // Definition 1 [2]
        // Supply Bound Function for a MPRModel
        let m = self.concurrency as f64;
        let a = Time::floor(self.resource / m);
        let b = self.resource - m * a;
        let t1 = interval - (self.period - Time::ceil(self.resource / m));
        let x = t1 - self.period * f64::floor(t1 / self.period);
        let y = self.period - a;

        if t1 < Time::zero() {
            Time::zero()
        } else {
            f64::floor(t1 / self.period) * self.resource
                +
            Time::max(Time::zero(), m*x - m*self.period + self.resource)
                -
            if x >= Time::one() && x < y {
                Time::zero()
            } else {
                Time::nanos(m) - b
            }
        }
    }

    /// Get the total supply the model provides in the given time interval (linear version).
    ///
    /// Note: for_all time. linear_supply(model, time) <= supply(model, time)
    pub fn get_supply_linear(&self, interval: Time) -> Time {
        let (resource, period, concurrency) = (self.resource, self.period, self.concurrency);

        // Equation 2 [2]
        resource / period * (interval - 2.0 * (period - resource / concurrency as f64) + Time::nanos(2.0))
    }

    /// Get the resource of the model which provides the given (linear) supply in the given time interval.
    pub fn resource_from_supply_linear(lsbf: Time, interval: Time, period: Time, concurrency: u64) -> Time {
        // Note that this only works for positive values of the linear supply bound.
        // There is only one positive solution for a positive bound, but two
        // solutions or zero for a negative one.
        debug_assert!(lsbf >= Time::zero());

        let cpus = concurrency as f64;
        let negb = 2.0 * period - interval - Time::nanos(2.0);
        let bsqr = negb * negb;

        // Extracted Theta from Equation 2 [2]
        cpus * (negb + Time2::sqrt(bsqr + 8.0 * period * lsbf / cpus) ) / 4.0
    }
}

// -----------------------------------------------------------------------------
// Convert each MPRModel to a set of periodic tasks (with implicit deadline)
// that represent the high-level requirements for the scheduled taskset. This
// set of server tasks can be scheduled with uniprocessor algorithms, as they
// are meant to be pinned to invididual CPUs.
impl MPRModel {
    // Section 5.2, Definition 1 [1]
    pub fn to_periodic_tasks(&self) -> Vec<RTTask> {
        #[inline(always)]
        fn psi(model: &MPRModel) -> Time {
            model.resource % Time::nanos(model.concurrency as f64)
        }

        if self.concurrency == 1 {
            return vec![ RTTask { wcet: self.resource, deadline: self.period, period: self.period } ];
        }

        let psi = psi(&self);
        let k = psi.as_nanos().floor() as u64;

        (0..self.concurrency)
            .map(|i| {
                let wcet =
                    if i < k {
                        (self.resource / self.concurrency as f64).floor() + Time::one()
                    } else if i == k {
                        (self.resource / self.concurrency as f64).floor()
                            + (psi % Time::nanos(k as f64)).floor()
                    } else {
                        (self.resource / self.concurrency as f64).floor()
                    };

                RTTask {
                    wcet: wcet,
                    deadline: self.period,
                    period: self.period,
                }
            })
            .collect()
    }

    pub fn to_periodic_tasks_simple(&self) -> (RTTask, u64) {
        let task =
            RTTask {
                wcet: (self.resource / self.concurrency as f64).floor() + Time::one(),
                deadline: self.period,
                period: self.period,
            };

        (task, self.concurrency)
    }
}

/// Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
///
/// Generic implementation for the MPRModel schedulability test.
/// Requires:
/// - A demand function, which describes the workload demand of the taskset at a given timepoint.
/// - A function which provides the set of arrival times of a task to check.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_demand<'a, 'b, 'c, FDem, FAk>(
    taskset: &'a [RTTask],
    model: &'b MPRModel,
    mut demand_fn: FDem,
    mut arrival_times_fn: FAk,
) -> bool
    where
        'a: 'c, 'b: 'c,
        FDem: FnMut(&'a [RTTask], usize, &'a RTTask, &'b MPRModel, Time) -> Time,
        FAk: FnMut(&'a [RTTask], usize, &'a RTTask, &'b MPRModel) -> Box<dyn Iterator<Item = Time> + 'c> ,
{
    taskset.iter().enumerate()
    .all(|(k, task_k)| {
        arrival_times_fn(taskset, k, task_k, model)
        .all(|arrival_k| {
            let demand =
                demand_fn(taskset, k, task_k, model, arrival_k);

            let supply =
                model.get_supply(arrival_k + task_k.deadline);

            demand <= supply
        })
    })
}

/// Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
///
/// Generic implementation for the generation of the MPRModel.
/// Requires:
/// - MPRModel's period and maximum concurrency.
/// - A demand function, which describes the workload demand of the taskset at a given timepoint.
/// - A function which provides the set of arrival times of a task to check.
///
/// Given a taskset and a MPR Model's Period and number of CPUS, compute the
/// model's minimum resource. This needs the demand function (usually based on
/// the algorithm), a function which specified which intervals to check, and the
/// inverse of the Supply Bound Function that models your supply. It is also
/// possible to filter out some of the intervals, useful if it is possible to
/// compute which intervals have the same demand.
///
/// Refer to the [module](`self`) level documentation.
pub fn generate_model_from_demand_linear<'a, 'b, FDem, FTime>(
    taskset: &'a [RTTask],
    model_period: Time,
    model_concurrency: u64,
    mut demand_fn: FDem,
    mut time_intervals_fn: FTime,
) -> Option<MPRModel>
    where
        'a: 'b,
        FDem: FnMut(&'a [RTTask], usize, &'a RTTask, Time, u64, Time) -> Time,
        FTime: FnMut(&'a [RTTask], usize, &'a RTTask, Time, u64) -> Box<dyn Iterator<Item = Time> + 'b>,
{
    let max_feasible_resource = model_period * model_concurrency as f64;

    taskset.iter().enumerate()
    .map(|(k, task_k)| {
        let time_intervals =
            time_intervals_fn(taskset, k, task_k, model_period, model_concurrency);

        time_intervals
            .map(|arrival_k| {
                let interval = arrival_k + task_k.deadline;
                let demand = demand_fn(taskset, k, task_k, model_period, model_concurrency, arrival_k);

                // move this function as static method of
                let resource = MPRModel::resource_from_supply_linear(
                    demand, interval, model_period, model_concurrency
                );

                if resource > max_feasible_resource {
                    None
                } else {
                    Some(resource)
                }
            })
            // try_fold instead of max to short-circuit on the minimum resource
            // being greater than the max feasible resource.
            .try_fold(Time::zero(), |acc: Time, res| Some(acc.max(res?)))
    })
    .try_fold(Time::zero(), |acc: Time, res| Some(acc.max(res?)))
    .map(|best_resource| MPRModel {
        resource: best_resource,
        period: model_period,
        concurrency: model_concurrency,
    })
}

// Tests -----------------------------------------------------------------------
#[test]
fn test_lsbf() {
    for resource in    time_range_iterator_w_step(Time::millis(10.0), Time::millis(1000.0), Time::millis(10.0)) {
    for period in      time_range_iterator_w_step(Time::millis(10.0), Time::millis(1000.0), Time::millis(10.0)) {
    for interval in    time_range_iterator_w_step(Time::millis(10.0), Time::millis(1000.0), Time::millis(10.0)) {
    for concurrency in   1 .. 10 {
        // skip unfeasible models
        if resource >= concurrency as f64 * period {
            continue;
        }

        let lsbf = (MPRModel { resource, period, concurrency }).get_supply_linear(interval);

        // skip negative supply values
        if lsbf < Time::zero() {
            continue;
        }

        let inverse = MPRModel::resource_from_supply_linear(lsbf, interval, period, concurrency);
        assert_eq!(resource, inverse);
    }}}}
}