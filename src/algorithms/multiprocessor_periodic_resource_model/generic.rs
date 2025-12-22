use crate::prelude::*;

use super::{
    MPRModel,
    sbf,
};

pub fn is_schedulable<FDem, FUb, FFilter>(
    taskset: &[RTTask],
    model: &MPRModel,
    demand_fn: FDem,
    arrival_k_upperbound_fn: FUb,
    filter_intervals_fn: FFilter,
) -> Result<bool, Error>
    where
        FDem: Fn(&[RTTask], usize, &RTTask, &MPRModel, Time) -> Time,
        FUb: Fn(&[RTTask], usize, &RTTask, &MPRModel) -> Result<Time, Error>,
        FFilter: Fn(&[RTTask], usize, &RTTask, &MPRModel, Time) -> bool,
{
    taskset.iter().enumerate().fold(Ok(true), |all, (k, task_k)| {
        match all {
            Ok(false) | Err(_) => { return all; },
            _ => (),
        };

        let ak_upperbound = arrival_k_upperbound_fn(taskset, k, task_k, model)?.ceil();

        Ok((0 ..= ak_upperbound.as_nanos() as usize)
            .map(|arrival_k| Time::nanos(arrival_k as f64))
            .filter(|arrival_k| filter_intervals_fn(taskset, k, task_k, model, *arrival_k))
            .all(|arrival_k| {
                demand_fn(taskset, k, task_k, model, arrival_k)
                    <=
                sbf(model, arrival_k + task_k.deadline)
            }))
    })
}

#[derive(Debug, Clone, Copy)]
pub enum GenerationStrategy {
    Naive,
    MonotoneLinearSearch,
    MonotoneBinarySearch,
}

#[derive(Debug, Clone)]
pub struct MPRModelSpecification {
    pub period: Time,
    pub concurrency: u64,
}

pub fn generate_interface<FProcLB, FProcUB, FResource>(
    taskset: &[RTTask],
    period: Time,
    generation_strategy: GenerationStrategy,
    num_processors_lower_bound: FProcLB,
    num_processors_upper_bound: FProcUB,
    minimum_required_resource_fn: FResource,
) -> Result<MPRModel, Error>
    where
        FProcLB: Fn(&[RTTask]) -> u64,
        FProcUB: Fn(&[RTTask]) -> u64,
        FResource: Fn(&[RTTask], &MPRModelSpecification) -> Result<Time, Error>
{
    use GenerationStrategy::*;
    use crate::common::binary_search::*;

    let lb = num_processors_lower_bound(taskset);
    let ub = num_processors_upper_bound(taskset);

    let model =
        match generation_strategy {
            Naive => {
                (lb ..= ub)
                .flat_map(|concurrency| {
                    let model = MPRModelSpecification {
                        period,
                        concurrency: concurrency as u64,
                    };

                    minimum_required_resource_fn(taskset, &model).ok()
                        .map(|res| (res, model).into())
                })
                .min_by_key(|model: &MPRModel| model.resource)
            },
            MonotoneLinearSearch => {
                (lb ..= ub)
                .flat_map(|concurrency| {
                    let model = MPRModelSpecification {
                        period,
                        concurrency: concurrency as u64,
                    };

                    minimum_required_resource_fn(taskset, &model).ok()
                        .map(|res| (res, model).into())
                })
                .next()
            },
            MonotoneBinarySearch => {
                binary_search_fn(
                    lb as usize ..= ub as usize,
                    |concurrency| {
                        let model = MPRModelSpecification {
                            period,
                            concurrency: concurrency as u64,
                        };

                        minimum_required_resource_fn(taskset, &model).ok()
                            .map(|res| (res, model).into())
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
    model: &MPRModelSpecification,
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
        min_feasible_resource_fn(taskset, model)?.floor().as_nanos() as usize;
    let max_feasible_resource =
        max_feasible_resource_fn(taskset, model)?.ceil().as_nanos() as usize;

    (min_feasible_resource ..= max_feasible_resource)
        .step_by(step_size.as_nanos() as usize)
        .map(|resource| Time::nanos(resource as f64))
        .find(|resource| {
            let model = (*resource, model.clone()).into();

            is_schedulable_fn(taskset, &model).unwrap_or(false)
        })
        .ok_or_else(|| Error::Generic(format!(
            "Cannot schedule taskset with period {}ns and {} CPUS",
            model.period.as_nanos(),
            model.concurrency,
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

impl Into<MPRModel> for (Time, MPRModelSpecification) {
    fn into(self) -> MPRModel {
        MPRModel {
            resource: self.0,
            period: self.1.period,
            concurrency: self.1.concurrency
        }
    }
}