//! ## MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009
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
use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

const ALGORITHM: &str = "MPR Model, EDF Local Scheduler (Shin, Easwaran, Lee 2009)";

/// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable_edf(taskset: &[RTTask], model: &MPRModel) -> SchedResult<()> {
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
        is_schedulable(
            taskset,
            model,
            |taskset, k, task_k, model, arrival_k|
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
            |taskset, _, task_k, model| -> Box<dyn Iterator<Item = Time>> {
                let arrival_k_upperbound =
                    arrival_k_upperbound_edf(taskset, task_k, model);

                Box::new(
                    (0 ..= arrival_k_upperbound.as_nanos() as u64)
                    .map(|time_ns| Time::nanos(time_ns as f64))
                    .filter(|arrival_k| filter_intervals_edf(taskset, task_k, model, *arrival_k))
                )
            }
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

fn is_schedulable_edf_simple(taskset: &[RTTask], model: &MPRModel) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    // Section 4.2, Theorem 1 [1]
    let schedulable =
        is_schedulable(
            taskset,
            model,
            |taskset, k, task_k, model, arrival_k|
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
            |taskset, _, task_k, model| -> Box<dyn Iterator<Item = Time>> {
                let arrival_k_upperbound =
                    arrival_k_upperbound_edf(taskset, task_k, model);

                Box::new(
                    (0 ..= arrival_k_upperbound.as_nanos() as u64)
                    .map(|time_ns| Time::nanos(time_ns as f64))
                )
            }
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
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

    assert!(is_schedulable_edf(&taskset, &model).is_schedulable());
    assert!(is_schedulable_edf_simple(&taskset, &model).is_schedulable());
}