use crate::prelude::*;
use super::{
    generic,
    MPRModel,
    num_processors_lower_bound,
    num_processors_upper_bound,
};

// demands for deadline monotonic and earliest deadline first, from [1]
use crate::analyses::smp_earliest_deadline_first::bcl_2009::{
    global_earliest_deadline_first_demand,
    global_fixed_priority_demand,
};

// Earliest Deadline First cluster-scheduling (using [1]) for MPR --------------
pub fn is_schedulable_edf_simple(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::is_schedulable(
        taskset,
        model,
        |taskset, k, task_k, model, _|
            demand_edf(taskset, k, task_k, model.concurrency),
        |_, _, _, _| Ok(Time::zero()),
        |_, _, _, _, _| true,
    )
}

pub fn generate_interface_global_edf_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::Naive,
        num_processors_lower_bound, // ?
        num_processors_upper_bound, // ?
        |taskset, model|
            generic::minimum_required_resource(
                taskset,
                model,
                |taskset, k, task_k, model, _|
                    demand_edf(taskset, k, task_k, model.concurrency),
                |_, _, _, _| Ok(Time::zero()),
                |_, _, _, _, _| true,
            ),
    )
}

fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_earliest_deadline_first_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
}

// Fixed Priority cluster-scheduling (using [1]) for MPR -------------------
pub fn is_schedulable_fp_simple(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::is_schedulable(
        taskset,
        model,
        |taskset, k, task_k, model, _|
            demand_fp(taskset, k, task_k, model.concurrency),
        |_, _, _, _| Ok(Time::zero()),
        |_, _, _, _, _| true,
    )
}

pub fn generate_interface_global_fp_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::Naive,
        num_processors_lower_bound, // ?
        num_processors_upper_bound, // ?
        |taskset, model|
            generic::minimum_required_resource(
                taskset,
                model,
                |taskset, k, task_k, model, _|
                    demand_fp(taskset, k, task_k, model.concurrency),
                |_, _, _, _| Ok(Time::zero()),
                |_, _, _, _, _| true,
            ),
    )
}

pub fn generate_interface_global_fp2_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_ordered_by_deadline(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::Naive,
        num_processors_lower_bound, // ?
        num_processors_upper_bound, // ?
        |taskset, model| {
            let min_feasible_resource = taskset.iter().map(|task| task.wcet).sum::<Time>().as_nanos() as usize;
            let max_feasible_resource = (model.period * model.concurrency as f64).as_nanos() as usize;

            (min_feasible_resource ..= max_feasible_resource)
                .map(|resource| Time::nanos(resource as f64))
                .filter(|resource|
                    is_schedulable_fp_simple(taskset, &(*resource, model.clone()).into()).unwrap_or(false)
                )
                .next()
                .ok_or_else(|| Error::Generic(format!(
                    "Cannot schedule taskset with period {}ns and {} CPUS",
                    model.period.as_nanos(),
                    model.concurrency,
                )))
        }
    )
}

fn demand_fp(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_fixed_priority_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
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