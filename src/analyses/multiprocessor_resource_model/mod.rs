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
        self.resource <= self.concurrency * self.period
    }

    pub fn utilization(&self) -> f64 {
        self.resource.as_raw() as f64 / self.period.as_raw() as f64
    }
}

// Equation 1 [1]
pub fn supply_bound_function(model: &MPRModel, time: Time) -> Time {
    #[inline(always)]
    fn k(model: &MPRModel, time: Time) -> i64 {
        Time::div_floor_time(
            time - model.period + Time::div_ceil_i64(model.resource, model.concurrency),
            model.period
        )
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    fn I(model: &MPRModel, time: Time) -> Time {
        time - 2 * model.period + Time::div_ceil_i64(model.resource, model.concurrency)
    }

    // sbf conditions
    if time >= model.period - Time::div_ceil_i64(model.resource, model.concurrency) {
        k(model, time) * model.resource + Time::max(
            Time::zero(),
            (I(model, time) - k(model, time) * model.period) * model.concurrency + model.resource
        )
    } else {
        Time::zero()
    }
}

// Equation 2 [1]
pub fn linear_supply_bound_function(model: &MPRModel, interval: Time) -> Time {
    let (resource, period, concurrency) = (model.resource, model.period, model.concurrency);
    
    // integer arithmetics formula
    Time::raw128(
        resource.as_raw_128() * (interval - 2 * (period - resource / concurrency)).as_raw_128() / period.as_raw_128()
    )

    // floating-point arithmetics formula
    // let (resource, interval, period, concurrency) =
    //     (resource.as_nanos_f64(), interval.as_nanos_f64(), period.as_nanos_f64(), concurrency as f64);
    // Time::nanos_f64(
    //     resource * (interval - 2.0 * (period - resource / concurrency)) / period
    // )
}

// Extracted Theta from Equation 2 [1]
pub fn resource_from_linear_supply_bound(lsbf: Time, interval: Time, period: Time, concurrency: i64) -> Time {
    // Note that this only works for positive values of the linear supply bound.
    // There is only one positive solution for a positive bound, but two
    // solutions or zero for a negative one.
    debug_assert!(lsbf >= Time::zero());

    // integer arithmetics formula
    let (lsbf, interval, period, cpus) =
        (lsbf.as_raw_128(), interval.as_raw_128(), period.as_raw_128(), concurrency as i128);
    let negb = 2 * period - interval;

    Time::raw128(
        cpus * (negb + num::integer::sqrt(negb*negb + 8 * period * lsbf / cpus)) / 4
    )

    // floating-point arithmetics formula
    // let (lsbf, interval, period, cpus) =
    //     (lsbf.as_nanos_f64(), interval.as_nanos_f64(), period.as_nanos_f64(), concurrency as f64);
    // let negb = 2.0 * period - interval;

    // Time::nanos_f64(
    //     cpus * (negb + f64::sqrt(negb*negb + 8.0 * period * lsbf / cpus)) / 4.0
    // )
}

// global EDF for MPR ----------------------------------------------------------

// Section 5.1 [1]
pub fn generate_interface_global_edf(taskset: &[RTTask], period: Time) -> MPRModel {
    let v : Vec<_> = (
        num_processors_lower_bound(taskset) ..=
        num_processors_upper_bound(taskset)
    ).collect();

    let num_processors = 
        v.binary_search_by_key(|i: &i64| -> std::cmp::Ordering {
            todo!()
        })
        .unwrap() as i64; // the upperbound number of processors guarantees schedulability

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

    total_work / den + n
}

// Equation 3 [1]
fn workload_upperbound(task: &RTTask, time: Time) -> Time {
    activations_in_interval(task, time) * task.wcet + carry_in(task, time)
}

// Equation 3 [1]
fn activations_in_interval(task: &RTTask, time: Time) -> i64 {
    Time::div_floor_time(time + task.period - task.deadline, task.period)
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
   
    sum0 + sum1 + num_processors * taskset[k].wcet
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
            model.resource - model.concurrency * Time::div_floor_i64(model.resource, model.concurrency)
        }

        let k = psi(&self).as_nanos();

        (0..self.concurrency)
            .map(|i| {
                let wcet =
                    if i <= k {
                        Time::div_floor_i64(self.resource, self.concurrency) + Time::one()
                    } else if i == k + 1 {
                        Time::div_floor_i64(self.resource, self.concurrency)
                            + psi(&self) - k * Time::div_floor_i64(psi(&self), k)
                    } else {
                        Time::div_floor_i64(self.resource, self.concurrency)
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
                wcet: Time::div_floor_i64(self.resource, self.concurrency) + Time::one(),
                deadline: self.period,
                period: self.period,
            };

        (task, self.concurrency)
    }
}

// Tests -----------------------------------------------------------------------
#[test]
fn test_lsbf() {
    for resource in    (10 .. 1000).step_by(10).map(|ms| Time::millis(ms)) {
    for period in      (10 .. 1000).step_by(10).map(|ms| Time::millis(ms)) {
    for interval in    (10 .. 1000).step_by(10).map(|ms| Time::millis(ms)) {
    for concurrency in   1 .. 10 {
        // skip unfeasible models
        if resource >= concurrency * period {
            continue;
        }

        let lsbf = linear_supply_bound_function(&MPRModel { resource, period, concurrency }, interval);
        // skip negative supply values
        if lsbf < Time::zero() {
            continue;
        }

        let inverse = resource_from_linear_supply_bound(lsbf, interval, period, concurrency);
        assert_eq!(resource.as_nanos(), inverse.as_nanos());
    }}}}
}

// References ------------------------------------------------------------------
// [1] I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
//     Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
//     Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
