use crate::prelude::*;
use super::{
    MPRModel,
    supply_bound_function
};

// demands for deadline monotonic and earliest deadline first, from [1]
use crate::analyses::smp_earliest_deadline_first::bcl_2009::{
    global_earliest_deadline_first_demand,
    global_deadline_monotonic_demand,
};

// Earliest Deadline First cluster-scheduling (using [1]) for MPR --------------
pub fn is_schedulable_edf_simple(taskset: &[RTTask], model: &super::MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    is_schedulable_simple(taskset, demand_edf, model)
}

pub fn generate_interface_global_edf_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generate_interface_simple(taskset, |taskset, period, concurrency| {
        best_required_resource(taskset, demand_edf, period, concurrency)
    }, period)
}

fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_earliest_deadline_first_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
}

// Deadline Monotonic cluster-scheduling (using [1]) for MPR -------------------
pub fn is_schedulable_dm_simple(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    is_schedulable_simple(taskset, demand_dm, model)
}

pub fn generate_interface_global_dm_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generate_interface_simple(taskset, |taskset, period, concurrency| {
        best_required_resource(taskset, demand_dm, period, concurrency)
    }, period)
}

fn demand_dm(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_deadline_monotonic_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
}

// Generic routines ------------------------------------------------------------
fn is_schedulable_simple<F>(taskset: &[RTTask], demand_fn: F, model: &MPRModel) -> Result<bool, Error>
    where F: Fn(&[RTTask], usize, &RTTask, u64) -> Time
{
    // bertogna tests extended to MPR don't run on all the possible arrival
    // times, it's just one check per task.
    Ok(taskset.iter().enumerate().all(|(k, task_k)| {
        demand_fn(taskset, k, task_k, model.concurrency)
            <=
        supply_bound_function(model, task_k.deadline)
    }))
}

fn generate_interface_simple<F>(taskset: &[RTTask], best_resource_fn: F, period: Time) -> Result<MPRModel, Error>
    where F: Fn(&[RTTask], Time, u64) -> Result<Time, Error>
{
    let concurrency_range =
        super::num_processors_lower_bound(taskset)
            ..=
        super::num_processors_upper_bound(taskset);

    // scan over all the possible processors and take the max, as we don't know
    // if the best_resource_fn is monotone.
    let Some((resource, concurrency)) =
        concurrency_range
        .flat_map(|concurrency: u64|
            best_resource_fn(taskset, period, concurrency)
                .ok().map(|res| (res, concurrency))
        )
        .max()
    else { panic!("unexpected"); };

    Ok(MPRModel {
        resource,
        period,
        concurrency,
    })
}

fn best_required_resource<F>(taskset: &[RTTask], demand_fn: F, period: Time, concurrency: u64) -> Result<Time, Error>
    where F: Fn(&[RTTask], usize, &RTTask, u64) -> Time
{
    let max_feasible_resource = period * concurrency as f64;

    taskset.iter().enumerate().fold(Ok(Time::zero()), |acc, (k, task_k)| {
        if acc.is_err() {
            return acc;
        }

        let demand = demand_fn(taskset, k, task_k, concurrency);
        let resource_at_arrival_k =
            super::resource_from_linear_supply_bound(demand, task_k.deadline, period, concurrency);

        if resource_at_arrival_k > max_feasible_resource {
            Err(Error::Generic(format!(
                "Cannot schedule taskset with period {}ns and {concurrency} CPUS",
                period.as_nanos()
            )))
        } else {
            Ok(Time::max(
                acc.unwrap(),
                resource_at_arrival_k
            ))
        }
    })
}

/* -----------------------------------------------------------------------------
References:
[1]: Bertogna, M., Cirinei, M. and Lipari, G., 2008. Schedulability analysis of
global scheduling algorithms on multiprocessor platforms. IEEE Transactions on
parallel and distributed systems, 20(4), pp.553-566.
[2]: I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
*/