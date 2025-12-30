//! ## Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
//!
//! This module provides a more general version of the MPR framework. Instead
//! of using the actual definitions on the paper for the demand function, these
//! functions require their definitions as input and provide ways to build new
//! schedulability tests and model designers for other local scheduling policies.
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive generic Work Conserving scheduling policy
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | O(*n^2*) complexity
//! - [`generate_interface`] \
//!   | O(*n^2*) complexity
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

use super::{
    MPRModel,
    MPRModelSpecification,
};

// -----------------------------------------------------------------------------

/// Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
///
/// Generic implementation for the MPRModel schedulability test.
/// Requires:
/// - A demand function, which describes the workload demand of the taskset at a given timepoint.
/// - A function which provides the set of arrival times of a task to check.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable<FDem, FAk>(
    test_name: &str,
    taskset: &[RTTask],
    model: &MPRModel,
    demand_fn: FDem,
    arrival_times_fn: FAk,
) -> SchedResult<()>
    where
        FDem: Fn(&[RTTask], usize, &RTTask, &MPRModel, Time) -> Result<Time, SchedError>,
        FAk: Fn(&[RTTask], usize, &RTTask, &MPRModel) -> Result<Box<dyn Iterator<Item = Time>>, SchedError>,
{
    for (k, task_k) in taskset.iter().enumerate() {
        let arrival_times_iter =
            match arrival_times_fn(taskset, k, task_k, model) {
                Ok(iter) => iter,
                Err(err) => return SchedResultFactory(test_name).from_err(err),
            };

        for arrival_k in arrival_times_iter {
            let demand =
                match demand_fn(taskset, k, task_k, model, arrival_k) {
                    Ok(demand) => demand,
                    Err(err) => return SchedResultFactory(test_name).from_err(err),
                };

            let supply =
                model.get_supply(arrival_k + task_k.deadline);

            if demand > supply {
                return SchedResultFactory(test_name).non_schedulable();
            }
        }
    }

    SchedResultFactory(test_name).schedulable(())
}

#[derive(Debug, Clone, Copy)]
pub enum GenerationStrategy {
    Naive,
    MonotoneLinearSearch,
    MonotoneBinarySearch,
}

pub fn generate_interface<FProcs, FResource>(
    test_name: &str,
    taskset: &[RTTask],
    period: Time,
    generation_strategy: GenerationStrategy,
    num_processors_fn: FProcs,
    minimum_required_resource_fn: FResource,
) -> SchedResult<MPRModel>
    where
        FProcs: Fn(&[RTTask]) -> Result<(u64, u64), SchedError>,
        FResource: Fn(&[RTTask], &MPRModelSpecification) -> Result<Time, SchedError>
{
    use GenerationStrategy::*;
    use crate::common::binary_search::*;

    let (lb, ub) =
        match num_processors_fn(taskset) {
            Ok(range) => range,
            Err(err) => return SchedResultFactory(test_name).from_err(err),
        };

    let model =
        match generation_strategy {
            Naive => {
                (lb ..= ub)
                .flat_map(|concurrency| {
                    let spec = MPRModelSpecification {
                        period,
                        concurrency: concurrency as u64,
                    };

                    minimum_required_resource_fn(taskset, &spec).ok()
                        .map(|r| spec.into_model(r))
                })
                .min_by_key(|model: &MPRModel| model.resource)
            },
            MonotoneLinearSearch => {
                (lb ..= ub)
                .flat_map(|concurrency| {
                    let spec = MPRModelSpecification {
                        period,
                        concurrency: concurrency as u64,
                    };

                    minimum_required_resource_fn(taskset, &spec).ok()
                        .map(|r| spec.into_model(r))
                })
                .next()
            },
            MonotoneBinarySearch => {
                binary_search_fn(
                    lb as usize ..= ub as usize,
                    |concurrency| {
                        let spec = MPRModelSpecification {
                            period,
                            concurrency: concurrency as u64,
                        };

                        minimum_required_resource_fn(taskset, &spec).ok()
                            .map(|r| spec.into_model(r))
                    })
            },
        };

    model.ok_or_else(|| Error::Generic(format!(
        "Cannot schedule taskset with period {}ns",
        period.as_nanos()
    )))
}

// Given a taskset and a MPR Model's Period and number of CPUS, compute the
// model's minimum resource by stepping through all the possible resource values
// and checking for schedulability.
pub fn minimum_required_resource<FResMin, FResMax, FSched>(
    taskset: &[RTTask],
    spec: &MPRModelSpecification,
    step_size: Time,
    min_feasible_resource_fn: FResMin,
    max_feasible_resource_fn: FResMax,
    is_schedulable_fn: FSched,
) -> Result<Time, Error>
    where
        FResMin: Fn(&[RTTask], &MPRModelSpecification) -> Result<Time, Error>,
        FResMax: Fn(&[RTTask], &MPRModelSpecification) -> Result<Time, Error>,
        FSched: Fn(&[RTTask], &MPRModel) -> Result<bool, Error>,
{
    let min_feasible_resource =
        min_feasible_resource_fn(taskset, spec)?.floor().as_nanos() as usize;
    let max_feasible_resource =
        max_feasible_resource_fn(taskset, spec)?.ceil().as_nanos() as usize;

    (min_feasible_resource ..= max_feasible_resource)
        .step_by(step_size.as_nanos() as usize)
        .map(|resource| Time::nanos(resource as f64))
        .find(|resource| {
            let model = spec.clone().into_model(*resource);

            is_schedulable_fn(taskset, &model).unwrap_or(false)
        })
        .ok_or_else(|| Error::Generic(format!(
            "Cannot schedule taskset with period {}ns and {} CPUS",
            spec.period.as_nanos(),
            spec.concurrency,
        )))
}

// Given a taskset and a MPR Model's Period and number of CPUS, compute the
// model's minimum resource. This needs the demand function (usually based on
// the algorithm), a function which specified which intervals to check, and the
// inverse of the Supply Bound Function that models your supply. It is also
// possible to filter out some of the intervals, useful if it is possible to
// compute which intervals have the same demand.
pub fn minimum_required_resource_inv<FDem, FRSbf, FUb, FFilter>(
    taskset: &[RTTask],
    model: &MPRModelSpecification,
    demand_fn: FDem,
    resource_from_sbf_fn: FRSbf,
    arrival_k_upperbound_fn: FUb,
    filter_intervals_fn: FFilter,
) -> Result<Time, Error>
    where
        FDem: Fn(&[RTTask], usize, &RTTask, &MPRModelSpecification, Time) -> Time,
        FRSbf: Fn(Time, Time, &MPRModelSpecification) -> Time,
        FUb: Fn(&[RTTask], usize, &RTTask, &MPRModelSpecification) -> Result<Time, Error>,
        FFilter: Fn(&[RTTask], usize, &RTTask, &MPRModelSpecification, Time) -> bool,
{
    let max_feasible_resource = model.period * model.concurrency as f64;

    taskset.iter().enumerate().fold(Ok(Time::zero()), |acc, (k, task_k)| {
        if acc.is_err() {
            return acc;
        }

        let ak_upperbound = arrival_k_upperbound_fn(taskset, k, task_k, model)?.ceil();

        let best_resource_k =
            (0 ..= ak_upperbound.as_nanos() as usize)
            .map(|arrival_k| Time::nanos(arrival_k as f64))
            .filter(|arrival_k| filter_intervals_fn(taskset, k, task_k, model, *arrival_k))
            .fold(Ok(Time::zero()), |acc, arrival_k| {
                if acc.is_err() {
                    return acc;
                }

                let interval = arrival_k + task_k.deadline;
                let demand = demand_fn(taskset, k, task_k, model, arrival_k);

                let resource_at_arrival_k =
                    resource_from_sbf_fn(demand, interval, model);

                if resource_at_arrival_k > max_feasible_resource {
                    Err(Error::Generic(format!(
                        "Cannot schedule taskset with period {}ns and {} CPUS",
                        model.period.as_nanos(),
                        model.concurrency,
                    )))
                } else {
                    Ok(Time::max(
                        acc.unwrap(),
                        resource_at_arrival_k
                    ))
                }
            })?;

        Ok(Time::max(
            acc.unwrap(),
            best_resource_k
        ))
    })
}

pub fn minimum_resource_for_taskset(taskset: &[RTTask], model_period: Time) -> Time {
    RTUtils::total_utilization(taskset) * model_period
}