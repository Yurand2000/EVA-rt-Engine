use crate::prelude::*;

#[derive(Clone)]
#[derive(Debug)]
pub enum Error {

}

pub struct MPRModel {
    pub resource: Time,
    pub period: Time,
    pub concurrency: i64,
}

// -----------------------------------------------------------------------------

impl MPRModel {
    pub fn is_feasible(&self) -> bool {
        self.resource <= self.concurrency as f64 * self.period
    }

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
pub fn resource_from_linear_supply_bound(lsbf: Time, interval: Time, period: Time, concurrency: i64) -> Time {
    // Note that this only works for positive values of the linear supply bound.
    // There is only one positive solution for a positive bound, but two
    // solutions or zero for a negative one.
    debug_assert!(lsbf >= Time::zero());

    // floating-point arithmetics formula
    let negb = 2.0 * period - interval;

    concurrency as f64 * (negb + Time::nanos( (negb * negb + 8.0 * period * lsbf / concurrency as f64).value().sqrt()) ) / 4.0
}

// global EDF for MPR ----------------------------------------------------------

// Section 5.1 [1]
pub fn generate_interface_global_edf(taskset: &[RTTask], period: Time) -> MPRModel {
    let v : Vec<_> = (
        num_processors_lower_bound(taskset) ..=
        num_processors_upper_bound(taskset)
    ).collect();

    todo!()
}

fn num_processors_lower_bound(taskset: &[RTTask]) -> i64 {
    f64::ceil(RTUtils::total_utilization(taskset)) as i64
}

// Section 5.1, Lemma 4 [1]
fn num_processors_upper_bound(taskset: &[RTTask]) -> i64 {
    debug_assert!(!taskset.is_empty());

    let n = taskset.len() as i64;

    let total_work: Time = taskset.iter()
        .map(|task| task.wcet)
        .sum();

    let den = taskset.iter()
        .map(|task| task.deadline - task.wcet)
        .min()
        .unwrap();

    (total_work / den).floor() as i64 + n
}

// Equation 3 [1]
fn workload_upperbound(task: &RTTask, time: Time) -> Time {
    activations_in_interval(task, time) * task.wcet + carry_in(task, time)
}

// Equation 3 [1]
fn activations_in_interval(task: &RTTask, time: Time) -> f64 {
    ((time + task.period - task.deadline) / task.period).floor()
}

// Equation 3 [1]
fn carry_in(task: &RTTask, time: Time) -> Time {
    Time::min(task.wcet, Time::max(Time::zero(), time - activations_in_interval(task, time) * task.period))
}

// Theorem 1, Equation 5 [1]
fn demand(taskset: &[RTTask], k: usize, release: Time, num_processors: i64) -> Time {
    let interference_hat = |taskset: &[RTTask], i: usize| -> Time {
        let task_i = &taskset[i];
        let task_k = &taskset[k];

        if i != k {
            Time::min(
                workload_upperbound(task_i, release + task_k.deadline)
                    - carry_in(task_i, release + task_k.deadline),
                release + task_k.deadline - task_k.wcet
            )
        } else {
            Time::min(
                workload_upperbound(task_k, release + task_k.deadline)
                    - task_k.wcet
                    - carry_in(task_k, release + task_k.deadline),
                release
            )
        }
    };

    let interference_flat = |taskset: &[RTTask], i: usize| -> Time {
        let task_i = &taskset[i];
        let task_k = &taskset[k];

        if i != k {
            Time::min(
                workload_upperbound(task_i, release + task_k.deadline),
                release + task_k.deadline - task_k.wcet
            )
        } else {
            Time::min(
                workload_upperbound(task_k, release + task_k.deadline) - task_k.wcet,
                release
            )
        }
    };

    let sum0 = (0..taskset.len())
        .map(|i| interference_hat(taskset, i))
        .sum::<Time>();

    // let mut vec1: Vec<_> = (0..taskset.len())
    //     .map(|i| interference_flat(taskset, i) - interference_hat(taskset, i))
    //     .collect();

    // vec1.sort_unstable();

    // let sum1 = vec1.into_iter().rev()
    //     .take(num_processors as usize - 1)
    //     .sum();

    let mut vec1: Vec<_> = (0..taskset.len())
        .map(|i| (i, carry_in(&taskset[i], release + taskset[k].deadline)))
        .collect();

    vec1.sort_unstable_by_key(|(_, ci)| *ci);

    // get the num_processors - 1 carry_ins and compute interferences for those
    let sum1 = vec1.into_iter().rev()
        .take(num_processors as usize - 1)
        .map(|(i, _)| interference_flat(taskset, i) - interference_hat(taskset, i))
        .sum();

    sum0 + sum1 + num_processors as f64 * taskset[k].wcet
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
                    if i <= k as i64 {
                        (self.resource / self.concurrency as f64).floor() + Time::one()
                    } else if i == k as i64 + 1 {
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

    pub fn to_periodic_tasks_simple(&self) -> (RTTask, i64) {
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
