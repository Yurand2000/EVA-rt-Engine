//! ## MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF local scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`is_schedulable`] \
//!   | ?? complexity
//! - [`generate_model_linear`] \
//!   | ?? complexity
//! - [`extra::generate_model`] \
//!   | ?? complexity
//! - [`extra::generate_best_model`] \
//!   | ?? complexity
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
use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

const ALGORITHM: &str = "MPR Model, EDF Local Scheduler (Shin, Easwaran, Lee 2009)";

/// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], model: &MPRModel) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    if f64::abs(model.utilization() - RTUtils::total_utilization(taskset)) < 0.01 {
        return SchedResultFactory(ALGORITHM).other(
            anyhow::format_err!("Arrival times upperbound tends to infinity, the computation becomes intractable.")
        );
    }

    // Section 4.2, Theorem 1 [1]
    let schedulable =
        is_schedulable_demand(
            taskset,
            model,
            |taskset, k, task_k, model, arrival_k|
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
            |taskset, _, task_k, model| -> Box<dyn Iterator<Item = Time>> {
                let arrival_k_upperbound =
                    arrival_k_upperbound_edf(taskset, task_k, model);

                Box::new(
                    time_range_iterator(Time::zero(), arrival_k_upperbound)
                    .filter(|arrival_k| filter_intervals_edf(taskset, task_k, model, *arrival_k))
                )
            }
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

#[allow(unused)]
fn is_schedulable_simple(taskset: &[RTTask], model: &MPRModel) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Section 4.2, Theorem 1 [1]
    let schedulable =
        is_schedulable_demand(
            taskset,
            model,
            |taskset, k, task_k, model, arrival_k|
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
            |taskset, _, task_k, model|  {
                let arrival_k_upperbound =
                    arrival_k_upperbound_edf(taskset, task_k, model);

                Box::new(time_range_iterator(Time::zero(), arrival_k_upperbound))
            }
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

pub fn generate_model_linear(
    taskset: &[RTTask],
    model_period: Time,
    model_concurrency: u64,
) -> DesignResult<MPRModel> {
    if !RTUtils::constrained_deadlines(taskset) {
        return DesignResultFactory(ALGORITHM).constrained_deadlines();
    }

    let model =
        generate_model_from_demand_linear(
            taskset,
            model_period,
            model_concurrency,
            |taskset, k, task_k, _, concurrency, arrival_k|
                demand_edf(taskset, k, task_k, concurrency, arrival_k),
            |taskset, _, task_k, period, concurrency| -> Box<dyn Iterator<Item = Time>> {
                // To bound Ak as in Theorem 2 we must know the value of Theta.
                // However, since Theta is being computed, we use its smallest
                // (0) and largest (mPi) possible values to bound Ak. [1]
                let arrival_k_upperbound = concurrency as f64 * period;

                Box::new(
                    (0 ..= arrival_k_upperbound.as_nanos() as u64)
                    .map(|time_ns| Time::nanos(time_ns as f64))
                    .filter(|arrival_k| {
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
                        let interval = *arrival_k + task_k.deadline;

                        // Perform the test only where I_hat/I_flat values change.
                        taskset.iter().any(|task_i| {
                            let modulus = *arrival_k % task_i.period;

                            interval <= task_i.wcet || modulus == Time::zero()
                        })
                    })
                )
            },
        );

    DesignResultFactory(ALGORITHM).from_option(model)
}

fn filter_intervals_edf(
    taskset: &[RTTask],
    task_k: &RTTask,
    model: &MPRModel,
    arrival_k: Time
) -> bool {
    // It is also easy to show that Equation (5) only needs to be evaluated at
    // those values of Ak for which at least one  of I_hat, I_flat, or sbf
    // change. [1]
    //
    // Both functions I_hat and I_flat change their value based on Wi and CIi,
    // on a periodic basis: their values are the same every interval of the form
    // [D_i + aT_i, D_i + T_I + aT_i] for all a >= 0. The I_hat function also
    // changes in the interval [0, C_i]. While the linear supply bound function
    // changes at every interval, the non-linear sbf is constant for values in
    // the range [-floor(Theta/m) + a*Pi, Pi - 2floor(Theta/m) + a*Pi] for all a
    // >= 0.
    let interval = arrival_k + task_k.deadline;

    // Perform the test only where SBF changes
    let floor = (model.resource / model.concurrency as f64).floor();
    let modulus = (interval + floor) % model.period;
    if modulus >= model.period - floor || modulus == Time::zero() {
        return true;
    }

    // Perform the test only where I_hat/I_flat values change.
    taskset.iter().any(|task_i| {
        let modulus = arrival_k % task_i.period;

        interval <= task_i.wcet || modulus == Time::zero()
    })
}

// Section 4.2, Theorem 2 [1]
fn arrival_k_upperbound_edf(taskset: &[RTTask], task_k: &RTTask, model: &MPRModel) -> Time {
    let taskset_utilization = RTUtils::total_utilization(taskset);

    let mut wcets: Vec<_> =
        taskset.iter().map(|task| task.wcet).collect();
    wcets.sort_unstable();

    let c_sum: Time = wcets.into_iter().rev().take(model.concurrency as usize - 1).sum();

    let u_sum: Time = taskset.iter()
        .map(|task| (task.period - task.deadline) * task.utilization()).sum();
    let b_sum: Time =
        model.resource * (2.0 - (2.0 * model.resource) / (model.concurrency as f64 * model.period));

    (
        c_sum
        - model.concurrency as f64 * task_k.wcet
        - task_k.deadline * (model.utilization() - taskset_utilization)
        + u_sum + b_sum
    ) / (
        model.utilization() - taskset_utilization
    )
}

// Section 4.2, Theorem 1 [1]
fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64, arrival_k: Time) -> Time {
    let interference_hat: Vec<_> =
        taskset.iter().enumerate()
            .map(|(i, task_i)| interference_hat(i, task_i, k, task_k, arrival_k))
            .collect();

    let mut interference_diff: Vec<_> =
        taskset.iter().enumerate()
            .map(|(i, task_i)| interference_flat(i, task_i, k, task_k, arrival_k) - interference_hat[i])
            .collect();

    interference_diff.sort_unstable();

    let sum_interference_hat: Time = interference_hat.into_iter().sum();
    let sum_interference_diff: Time = interference_diff.into_iter().rev()
        .take(concurrency as usize - 1).sum();

    sum_interference_hat + sum_interference_diff + concurrency as f64 * task_k.wcet
}

// Section 4.2, Theorem 1 [1]
fn interference_flat(i: usize, task_i: &RTTask, k: usize, task_k: &RTTask, arrival_k: Time) -> Time {
    let workload_upperbound = workload_upperbound_2_edf(task_i, arrival_k + task_k.deadline);

    if i == k {
        Time::min(workload_upperbound - task_k.wcet, arrival_k)
    } else {
        Time::min(workload_upperbound, arrival_k + task_k.deadline - task_k.wcet)
    }
}

// Section 4.2, Theorem 1 [1]
fn interference_hat(i: usize, task_i: &RTTask, k: usize, task_k: &RTTask, arrival_k: Time) -> Time {
    let workload_upperbound = workload_upperbound_edf(task_i, arrival_k + task_k.deadline);

    if i == k {
        Time::min(workload_upperbound - task_k.wcet, arrival_k)
    } else {
        Time::min(workload_upperbound, arrival_k + task_k.deadline - task_k.wcet)
    }
}

fn num_processors_lower_bound(taskset: &[RTTask]) -> u64 {
    f64::ceil(RTUtils::total_utilization(taskset)) as u64
}

// Section 5.1, Lemma 4 [1]
fn num_processors_upper_bound(taskset: &[RTTask]) -> u64 {
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

pub mod extra {
    use crate::prelude::*;
    use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

    pub fn generate_model(
        taskset: &[RTTask],
        model_period: Time,
        model_concurrency: u64,
        resource_step: Time,
    ) -> DesignResult<MPRModel> {
        if !RTUtils::constrained_deadlines(taskset) {
            return DesignResultFactory(super::ALGORITHM).constrained_deadlines();
        }

        __generate_model(taskset, model_period, model_concurrency, resource_step)
    }

    fn __generate_model(
        taskset: &[RTTask],
        model_period: Time,
        model_concurrency: u64,
        resource_step: Time,
    ) -> DesignResult<MPRModel> {
        let min_feasible_resource =
            RTUtils::total_utilization(taskset) * model_period;
        let max_feasible_resource_model =
            super::generate_model_linear(taskset, model_period, model_concurrency);

        if !max_feasible_resource_model.is_successful() {
            return max_feasible_resource_model;
        }

        let max_feasible_resource = max_feasible_resource_model.result.unwrap().resource;

        let best_model =
            time_range_iterator_w_step(min_feasible_resource, max_feasible_resource, resource_step)
            .find_map(|resource| {
                let model = MPRModel {
                    resource,
                    period: model_period,
                    concurrency: model_concurrency,
                };

                if super::is_schedulable(taskset, &model).is_schedulable() {
                    Some(model)
                } else {
                    None
                }
            });

        DesignResultFactory(super::ALGORITHM).from_option(best_model)
    }

    pub fn generate_best_model(
        taskset: &[RTTask],
        (min_period, max_period, period_step): (Time, Time, Time),
        resource_step: Time,
    ) -> DesignResult<MPRModel> {
        if !RTUtils::constrained_deadlines(taskset) {
            return DesignResultFactory(super::ALGORITHM).constrained_deadlines();
        }

        let min_processors =
            super::num_processors_lower_bound(taskset) as usize;

        let max_processors =
            super::num_processors_upper_bound(taskset) as usize;

        let best_model =
            time_range_iterator_w_step(min_period, max_period, period_step)
            .flat_map(|period| {
                let best_model =
                    binary_search_fn(
                        (min_processors, max_processors),
                        |concurrency|
                            __generate_model(
                                taskset,
                                period,
                                concurrency as u64,
                                resource_step
                            ),
                        |model| -> std::cmp::Ordering {
                            if model.is_successful() {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        }
                    );

                best_model.result.ok()
            })
            .min_by_key(|model| model.resource);

        DesignResultFactory(super::ALGORITHM).from_option(best_model)
    }
}

// -----------------------------------------------------------------------------

#[test]
pub fn simple_vs_optimized() {
    let taskset = [
        RTTask::new_ns(35, 90, 160),
        RTTask::new_ns(70, 115, 160),
        RTTask::new_ns(30, 50, 75),
    ];

    let model = MPRModel {
        resource: Time::nanos(75.0),
        period: Time::nanos(50.0),
        concurrency: 2,
    };

    assert!(is_schedulable(&taskset, &model).is_schedulable());
    assert!(is_schedulable_simple(&taskset, &model).is_schedulable());
}