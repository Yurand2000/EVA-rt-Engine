use crate::prelude::*;

// Section 3.2 [1]
pub struct MPRModel {
    pub resource: Time,
    pub period: Time,
    pub concurrency: u64,
}

// -----------------------------------------------------------------------------

impl MPRModel {
    pub fn is_feasible(&self) -> bool {
        self.resource <= self.concurrency as f64 * self.period
    }

    /// resource / period
    pub fn utilization(&self) -> f64 {
        self.resource / self.period
    }
}

// Equation 1 [1]
pub fn supply_bound_function(model: &MPRModel, time: Time) -> Time {
    #[inline(always)]
    fn k(model: &MPRModel, time: Time) -> f64 {
        (
            (time - model.period + (model.resource / model.concurrency as f64).ceil()) /
            model.period
        ).floor()
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    fn I(model: &MPRModel, time: Time) -> Time {
        time - 2.0 * model.period + (model.resource / model.concurrency as f64).ceil()
    }

    // sbf conditions
    if time >= model.period - (model.resource / model.concurrency as f64).ceil() {
        k(model, time) * model.resource + Time::max(
            Time::zero(),
            (I(model, time) - k(model, time) * model.period) * model.concurrency as f64 + model.resource
        )
    } else {
        Time::zero()
    }
}

// Equation 2 [1]
pub fn linear_supply_bound_function(model: &MPRModel, interval: Time) -> Time {
    let (resource, period, concurrency) = (model.resource, model.period, model.concurrency);

    resource * (interval - 2.0 * (period - resource / concurrency as f64)) / period
}

// Extracted Theta from Equation 2 [1]
pub fn resource_from_linear_supply_bound(lsbf: Time, interval: Time, period: Time, concurrency: u64) -> Time {
    // Note that this only works for positive values of the linear supply bound.
    // There is only one positive solution for a positive bound, but two
    // solutions or zero for a negative one.
    debug_assert!(lsbf >= Time::zero());

    // floating-point arithmetics formula
    let negb = 2.0 * period - interval;

    concurrency as f64 * (negb + Time::nanos( (negb * negb + 8.0 * period * lsbf / concurrency as f64).value().sqrt()) ) / 4.0
}

// global EDF for MPR ----------------------------------------------------------

// Section 4.2, Theorem 1 [1]
pub fn is_schedulable_edf(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    Ok(taskset.iter().enumerate().all(|(k, task_k)| {
        let ak_upperbound = arrival_k_upperbound_edf(taskset, task_k, model).ceil();

        // naive implementation
        (0 ..= ak_upperbound.ceil().as_nanos() as usize)
            .all(|arrival_k| {
                let arrival_k = Time::nanos(arrival_k as f64);

                demand_edf(taskset, k, task_k, model, arrival_k)
                    <=
                linear_supply_bound_function(model, arrival_k + task_k.deadline)
            })
    }))
}

// Section 4.2, Theorem 2 [1]
fn arrival_k_upperbound_edf(taskset: &[RTTask], task_k: &RTTask, model: &MPRModel) -> Time {
    let mut wcets: Vec<_> =
        taskset.iter().map(|task| task.wcet).collect();
    wcets.sort_unstable();

    let c_sum: Time = wcets.into_iter().rev().take(model.concurrency as usize - 1).sum();
    let taskset_utilization = RTUtils::total_utilization(taskset);

    let u_sum: Time = taskset.iter().map(|task| (task.period - task.deadline) * task.utilization()).sum();
    let b_sum: Time = model.resource * (2.0 - (2.0 * model.resource) / (model.concurrency as f64 * model.period));

    (
        c_sum
        - model.concurrency as f64 * task_k.wcet
        - task_k.deadline * (model.utilization() - taskset_utilization)
        + u_sum + b_sum
    ) / (
        model.utilization() - taskset_utilization
    )
}

fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, model: &MPRModel, arrival_k: Time) -> Time {
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
        .take(model.concurrency as usize - 1).sum();

    sum_interference_hat + sum_interference_diff + model.concurrency as f64 * task_k.wcet
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

// Section 5.1 [1]
pub fn generate_interface_global_edf(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    let Some((resource, concurrency)) =
        (num_processors_lower_bound_edf(taskset) ..= num_processors_upper_bound_edf(taskset))
        .filter_map(|concurrency| {
            best_required_resource_edf(taskset, period, concurrency).ok()
                .map(|res| (res, concurrency))
        })
        .min_by_key(|(resource, _)| *resource)
    else { panic!("unexpected"); };

    Ok(MPRModel {
        resource,
        period,
        concurrency,
    })
}

#[inline(always)]
fn num_processors_lower_bound_edf(taskset: &[RTTask]) -> u64 {
    f64::ceil(RTUtils::total_utilization(taskset)) as u64
}

// Section 5.1, Lemma 4 [1]
#[inline(always)]
fn num_processors_upper_bound_edf(taskset: &[RTTask]) -> u64 {
    debug_assert!(!taskset.is_empty());

    let n = taskset.len() as u64;

    let total_work: Time = taskset.iter()
        .map(|task| task.wcet)
        .sum();

    let den = taskset.iter()
        .map(|task| task.laxity())
        .min()
        .unwrap();

    (total_work / den).ceil() as u64 + n
}

pub fn best_required_resource_edf(taskset: &[RTTask], period: Time, concurrency: u64) -> Result<Time, Error> {
    todo!()
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
    Time::min(task.wcet, Time::max(Time::zero(), time - activations_in_interval_edf(task, time) * task.period))
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
            model.resource - model.concurrency as f64 * (model.resource / model.concurrency as f64).floor()
        }

        let k = psi(&self).as_nanos();

        (0..self.concurrency)
            .map(|i| {
                let wcet =
                    if i <= k as u64 {
                        (self.resource / self.concurrency as f64).floor() + Time::one()
                    } else if i == k as u64 + 1 {
                        (self.resource / self.concurrency as f64).floor()
                            + psi(&self) - k * (psi(&self) / k).floor()
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

        let lsbf = linear_supply_bound_function(&MPRModel { resource, period, concurrency }, interval);
        // skip negative supply values
        if lsbf < Time::zero() {
            continue;
        }

        let inverse = resource_from_linear_supply_bound(lsbf, interval, period, concurrency);
        assert_eq!(resource, inverse);
    }}}}
}

// References ------------------------------------------------------------------
// [1] I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
//     Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
//     Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
