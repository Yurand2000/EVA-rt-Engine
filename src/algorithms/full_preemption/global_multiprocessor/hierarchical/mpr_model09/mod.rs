//! ## Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive generic Work Conserving scheduling policy
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable_edf`] \
//!   | O(*n^2*) complexity
//! - [`is_schedulable_edf_simple`] \
//!   | O(*n^2*) complexity
//! - [`generate_interface_global_edf_simple`] \
//!   | O(*n^2*) complexity
//! - [`generate_interface_global_edf`] \
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

// Local Scheduling Algorithms
pub mod earliest_deadline_first {
    pub mod shin_easwaran_lee09;
}

pub mod fixed_priority {
    pub mod bcl09;
}

pub mod bcl_2009;
pub mod generic;
pub mod model;

use model::*;

/// Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
///
/// Generic implementation for the MPRModel schedulability test.
/// Requires:
/// - A demand function, which describes the workload demand of the taskset at a given timepoint.
/// - A function which provides the set of arrival times of a task to check.
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable<'a, 'b, 'c, FDem, FAk>(
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

pub fn generate_interface_naive_search<FProcs, FModel>(
    taskset: &[RTTask],
    period: Time,
    (min_concurrency, max_concurrency): (u64, u64),
    mut best_model_fn: FModel,
) -> Option<MPRModel>
    where
        FModel: FnMut(&[RTTask], Time, u64) -> Option<MPRModel>
{
    (min_concurrency ..= max_concurrency)
        .filter_map(|concurrency| best_model_fn(taskset, period, concurrency))
        .min_by_key(|model: &MPRModel| model.resource)
}

pub fn generate_interface_linear_search<FProcs, FModel>(
    taskset: &[RTTask],
    period: Time,
    (min_concurrency, max_concurrency): (u64, u64),
    mut best_model_fn: FModel,
) -> Option<MPRModel>
    where
        FModel: FnMut(&[RTTask], Time, u64) -> Option<MPRModel>
{
    (min_concurrency ..= max_concurrency)
        .filter_map(|concurrency| best_model_fn(taskset, period, concurrency))
        .next()
}

pub fn generate_interface_binary_search<FProcs, FModel>(
    taskset: &[RTTask],
    period: Time,
    (min_concurrency, max_concurrency): (u64, u64),
    mut best_model_fn: FModel,
) -> Option<MPRModel>
    where
        FModel: FnMut(&[RTTask], Time, u64) -> Option<MPRModel>
{
    binary_search_fn(
        (min_concurrency as usize, max_concurrency as usize),
        |concurrency|
            best_model_fn(taskset, period, concurrency as u64),
        |model: &Option<MPRModel>| {
            if model.is_some() {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }
    )
    .filter(|model: &MPRModel| model.is_feasible())
}

// global EDF for MPR, inverse -------------------------------------------------

// Section 5.1 [1]
pub fn generate_interface_global_edf_simple(taskset: &[RTTask], period: Time, step_size: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::MonotoneLinearSearch,
        num_processors_lower_bound,
        num_processors_upper_bound,
        |taskset, model|
            minimum_required_resource_edf(taskset, model, step_size),
    )
}

pub fn generate_interface_global_edf(taskset: &[RTTask], period: Time, step_size: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::generate_interface(
        taskset,
        period,
        generic::GenerationStrategy::MonotoneBinarySearch,
        num_processors_lower_bound,
        num_processors_upper_bound,
        |taskset, model|
            minimum_required_resource_edf(taskset, model, step_size),
    )
}

fn minimum_required_resource_edf(
    taskset: &[RTTask],
    model: &MPRModelSpecification,
    step_size: Time,
) -> Result<Time, Error> {
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
                |taskset, k, task_k, model, arrival_k|
                    demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
                |demand, interval, model|
                    MPRModel::resource_from_supply_linear(demand, interval, model.period, model.concurrency),

                // To bound Ak as in Theorem 2 we must know the value of Theta.
                // However, since Theta is being computed, we use its smallest
                // (0) and largest (mPi) possible values to bound Ak. [1]
                |_, _, _, model|
                    Ok(model.concurrency as f64 * model.period),

                // It is also easy to show that Equation (5) only needs to be
                // evaluated at those values of Ak for which at least one  of
                // I_hat, I_flat, or sbf change. [1]
                //
                // Both functions I_hat and I_flat change their value based on
                // Wi and CIi, on a periodic basis: their values are the same
                // every interval of the form [D_i + aT_i, D_i + T_I + aT_i] for
                // all a >= 0. The I_hat function also changes in the interval
                // [0, C_i]. The linear supply bound function changes at every
                // interval, but we can consider only the intervals where I_hat
                // and I_flat change, as it is a monotone function (i.e., if
                // it's satisfied between those intervals, it will be also
                // satisfied outside because of monotonicity).
                |taskset, _, task_k, _, arrival_k| {
                    let interval = arrival_k + task_k.deadline;

                    // Perform the test only where I_hat/I_flat values change.
                    taskset.iter().any(|task_i| {
                        let modulus = arrival_k % task_i.period;

                        interval <= task_i.wcet || modulus == Time::zero()
                    })
                },
            ),
        is_schedulable_edf,
    )
}

#[inline(always)]
pub fn num_processors_lower_bound(taskset: &[RTTask]) -> u64 {
    f64::ceil(RTUtils::total_utilization(taskset)) as u64
}

// Section 5.1, Lemma 4 [1]
#[inline(always)]
pub fn num_processors_upper_bound(taskset: &[RTTask]) -> u64 {
    debug_assert!(!taskset.is_empty());

    let n = taskset.len() as u64;

    let den = taskset.iter()
        .map(|task| task.laxity())
        .min()
        .unwrap();

    if den == Time::zero() {
        todo!("unexpected");
    }

    let total_work: Time = taskset.iter()
        .map(|task| task.wcet)
        .sum();

    (total_work / den).ceil() as u64 + n
}

// Equation 3 [1]
fn workload_upperbound_2_edf(task: &RTTask, time: Time) -> Time {
    activations_in_interval_edf(task, time) * task.wcet + carry_in_edf(task, time)
}

fn workload_upperbound_edf(task: &RTTask, time: Time) -> Time {
    activations_in_interval_edf(task, time) * task.wcet
}

// Equation 3 [1]
#[inline(always)]
fn activations_in_interval_edf(task: &RTTask, time: Time) -> f64 {
    ((time + task.period - task.deadline) / task.period).floor()
}

// Equation 3 [1]
#[inline(always)]
fn carry_in_edf(task: &RTTask, time: Time) -> Time {
    Time::min(
        task.wcet,
        Time::max(
            Time::zero(),
            time - activations_in_interval_edf(task, time) * task.period
        )
    )
}

// Tests -----------------------------------------------------------------------
#[test]
fn test_lsbf() {
    for resource in    (10 .. 1000).step_by(10).map(|ms| Time::millis(ms as f64)) {
    for period in      (10 .. 1000).step_by(10).map(|ms| Time::millis(ms as f64)) {
    for interval in    (10 .. 1000).step_by(10).map(|ms| Time::millis(ms as f64)) {
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