use crate::prelude::*;
use super::{
    generic,
    MPRModel,
    num_processors_lower_bound,
    num_processors_upper_bound,
    resource_from_linear_sbf,
};

// demands for deadline monotonic and earliest deadline first, from [1]
use crate::analyses::smp_earliest_deadline_first::bcl_2009::{
    global_earliest_deadline_first_demand,
    global_fixed_priority_demand,
};

// Earliest Deadline First cluster-scheduling (using [1]) for MPR --------------
pub fn is_schedulable_edf(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
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

pub fn generate_interface_global_edf_simple(taskset: &[RTTask], period: Time, step_size: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::MonotoneLinearSearch,
        num_processors_lower_bound,
        num_processors_upper_bound, // ?
        |taskset, model|
            generic::minimum_required_resource(
                taskset,
                model,
                step_size,
                |taskset, model|
                    Ok(generic::minimum_resource_for_taskset(taskset, model.period)),
                |taskset, model|
                    generic::minimum_required_resource_inv(
                        taskset,
                        model,
                        |taskset, k, task_k, model, _|
                            demand_edf(taskset, k, task_k, model.concurrency),
                        |demand, interval, model|
                            resource_from_linear_sbf(demand, interval, model.period, model.concurrency),
                        |_, _, _, _| Ok(Time::zero()),
                        |_, _, _, _, _| true,
                    ),
                is_schedulable_edf
            )
    )
}

pub fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_earliest_deadline_first_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
}

// Fixed Priority cluster-scheduling (using [1]) for MPR -----------------------
pub fn is_schedulable_fp(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
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

pub fn generate_interface_global_fp(taskset: &[RTTask], period: Time, step_size: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::MonotoneLinearSearch,
        num_processors_lower_bound,
        num_processors_upper_bound, // ?
        |taskset, model|
            generic::minimum_required_resource(
                taskset,
                model,
                step_size,
                |taskset, model|
                    Ok(generic::minimum_resource_for_taskset(taskset, model.period)),
                |taskset, model|
                    generic::minimum_required_resource_inv(
                        taskset,
                        model,
                        |taskset, k, task_k, model, _|
                            demand_fp(taskset, k, task_k, model.concurrency),
                        |demand, interval, model|
                            resource_from_linear_sbf(demand, interval, model.period, model.concurrency),
                        |_, _, _, _| Ok(Time::zero()),
                        |_, _, _, _, _| true,
                    ),
                is_schedulable_fp
            )
    )
}

pub fn demand_fp(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_fixed_priority_demand(taskset, k, task_k)
        +
    concurrency as f64 * (task_k.wcet - Time::one())
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