use crate::prelude::*;

pub mod bcl_2009;
pub mod generic;

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
pub fn sbf(model: &MPRModel, time: Time) -> Time {
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
        k(model, time) * model.resource
            +
        Time::max(
            Time::zero(),
            (I(model, time) - k(model, time) * model.period)
                * model.concurrency as f64 + model.resource
        )
    } else {
        Time::zero()
    }
}

// Equation 2 [1]
pub fn linear_sbf(model: &MPRModel, interval: Time) -> Time {
    let (resource, period, concurrency) = (model.resource, model.period, model.concurrency);

    resource * (interval - 2.0 * (period - resource / concurrency as f64)) / period
}

// Extracted Theta from Equation 2 [1]
pub fn resource_from_linear_sbf(lsbf: Time, interval: Time, period: Time, concurrency: u64) -> Time {
    // Note that this only works for positive values of the linear supply bound.
    // There is only one positive solution for a positive bound, but two
    // solutions or zero for a negative one.
    debug_assert!(lsbf >= Time::zero());

    let concurrency = concurrency as f64;
    let negb = 2.0 * period - interval;
    let bsqr = negb * negb;

    concurrency * (negb + Time2::sqrt(bsqr + 8.0 * period * lsbf / concurrency) ) / 4.0
}

// global EDF for MPR ----------------------------------------------------------

// Section 4.2, Theorem 1 [1]
pub fn is_schedulable_edf_simple(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    taskset.iter().enumerate().fold(Ok(true), |all, (k, task_k)| {
        match all {
            Ok(false) | Err(_) => { return all; },
            _ => (),
        };

        let ak_upperbound = arrival_k_upperbound_edf(taskset, task_k, model)?.ceil();

        // naive implementation
        Ok((0 ..= ak_upperbound.ceil().as_nanos() as usize)
            .map(|arrival_k| Time::nanos(arrival_k as f64))
            .all(|arrival_k| {
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k)
                    <=
                sbf(model, arrival_k + task_k.deadline)
            }))
    })
}

pub fn is_schedulable_edf(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    taskset.iter().enumerate().fold(Ok(true), |all, (k, task_k)| {
        match all {
            Ok(false) | Err(_) => { return all; },
            _ => (),
        };

        let ak_upperbound = arrival_k_upperbound_edf(taskset, task_k, model)?.ceil();

        // It is also easy to show that Equation (5) only needs to be evaluated
        // at those values of Ak for which at least one  of I_hat, I_flat, or
        // sbf change. [1]
        //
        // Both functions I_hat and I_flat change their value based on Wi and
        // CIi, on a periodic basis: their values are the same every interval of
        // the form [D_i + aT_i, D_i + T_I + aT_i] for all a >= 0. The I_hat
        // function also changes in the interval [0, C_i].
        // While the linear supply bound function changes at every interval, the
        // non-linear sbf is constant for values in the range [-floor(Theta/m) +
        // a*Pi, Pi - 2floor(Theta/m) + a*Pi] for all a >= 0.
        Ok((0 ..= ak_upperbound.ceil().as_nanos() as usize)
            .map(|arrival_k| Time::nanos(arrival_k as f64))
            .filter(|arrival_k| {
                let interval = *arrival_k + task_k.deadline;

                // Perform the test only where SBF changes
                let floor = (model.resource / model.concurrency as f64).floor();
                let modulus = (interval + floor) % model.period;
                if modulus >= model.period - floor || modulus == Time::zero() {
                    return true;
                }

                // Perform the test only where I_hat/I_flat values change.
                taskset.iter().any(|task_i| {
                    let modulus = *arrival_k % task_i.period;

                    interval <= task_i.wcet || modulus == Time::zero()
                })
            })
            .all(|arrival_k| {
                demand_edf(taskset, k, task_k, model.concurrency, arrival_k)
                    <=
                sbf(model, arrival_k + task_k.deadline)
            }))
    })
}

// Section 4.2, Theorem 2 [1]
fn arrival_k_upperbound_edf(taskset: &[RTTask], task_k: &RTTask, model: &MPRModel) -> Result<Time, Error> {
    let taskset_utilization = RTUtils::total_utilization(taskset);
    if f64::abs(model.utilization() - taskset_utilization) < 0.01 {
        return Err(Error::Generic(
            format!("Arrival times upperbound tends to infinity, the computation becomes intractable.")
        ));
    }

    let mut wcets: Vec<_> =
        taskset.iter().map(|task| task.wcet).collect();
    wcets.sort_unstable();

    let c_sum: Time = wcets.into_iter().rev().take(model.concurrency as usize - 1).sum();

    let u_sum: Time = taskset.iter()
        .map(|task| (task.period - task.deadline) * task.utilization()).sum();
    let b_sum: Time =
        model.resource * (2.0 - (2.0 * model.resource) / (model.concurrency as f64 * model.period));

    Ok((
        c_sum
        - model.concurrency as f64 * task_k.wcet
        - task_k.deadline * (model.utilization() - taskset_utilization)
        + u_sum + b_sum
    ) / (
        model.utilization() - taskset_utilization
    ))
}

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

// global EDF for MPR, inverse -------------------------------------------------

// Section 5.1 [1]
pub fn generate_interface_global_edf_simple(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    let Some((resource, concurrency)) =
        // naive implementation (can do binary search: Section 5.1, Lemma 3 [1])
        // we can stop as soon as we have a non-None value, as the resource is
        // monotonically increasing with the number of CPUs.
        (num_processors_lower_bound(taskset) ..= num_processors_upper_bound(taskset))
        .fold(None, |acc, concurrency: u64| {
            if acc.is_some() {
                acc
            } else {
                best_required_resource_edf(taskset, period, concurrency).ok()
                    .map(|res| (res, concurrency))
            }
        })
    else { panic!("unexpected"); };

    Ok(MPRModel {
        resource,
        period,
        concurrency,
    })
}

pub fn generate_interface_global_edf(taskset: &[RTTask], period: Time) -> Result<MPRModel, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    let concurrency_range =
        num_processors_lower_bound(taskset) as usize
            ..=
        num_processors_upper_bound(taskset) as usize;

    let Some((resource, concurrency)) =
        // binary search: Section 5.1, Lemma 3 [1]
        binary_search_fn(
            concurrency_range,
            |concurrency| {
                best_required_resource_edf(taskset, period, concurrency as u64).ok()
                    .map(|res| (res, concurrency as u64))
            })
    else { panic!("unexpected"); };

    Ok(MPRModel {
        resource,
        period,
        concurrency,
    })
}

#[inline(always)]
fn num_processors_lower_bound(taskset: &[RTTask]) -> u64 {
    f64::ceil(RTUtils::total_utilization(taskset)) as u64
}

// Section 5.1, Lemma 4 [1]
#[inline(always)]
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

pub fn best_required_resource_edf(taskset: &[RTTask], period: Time, concurrency: u64) -> Result<Time, Error> {
    let max_feasible_resource = period * concurrency as f64;

    taskset.iter().enumerate().fold(Ok(Time::zero()), |acc, (k, task_k)| {
        if acc.is_err() {
            return acc;
        }

        // To bound Ak as in Theorem 2 we must know the value of Theta. However,
        // since Theta is being computed, we use its smallest (0) and largest
        // (mPi) possible values to bound Ak. [1]
        let ak_upperbound = concurrency as f64 * period;

        // It is also easy to show that Equation (5) only needs to be evaluated
        // at those values of Ak for which at least one  of I_hat, I_flat, or
        // sbf change. [1]
        //
        // Both functions I_hat and I_flat change their value based on Wi and
        // CIi, on a periodic basis: their values are the same every interval of
        // the form [D_i + aT_i, D_i + T_I + aT_i] for all a >= 0. The I_hat
        // function also changes in the interval [0, C_i]. The linear supply
        // bound function changes at every interval, but we can consider only
        // the intervals where I_hat and I_flat change, as it is a monotone
        // function (i.e., if it's satisfied between those intervals, it will be
        // also satisfied outside because of monotonicity).
        let best_resource_k =
            (0 ..= ak_upperbound.ceil().as_nanos() as usize)
            .map(|arrival_k| Time::nanos(arrival_k as f64))
            .filter(|arrival_k| {
                let interval = *arrival_k + task_k.deadline;

                // Perform the test only where I_hat/I_flat values change.
                taskset.iter().any(|task_i| {
                    let modulus = *arrival_k % task_i.period;

                    interval <= task_i.wcet || modulus == Time::zero()
                })
            })
            .fold(Ok(Time::zero()), |acc, arrival_k| {
                if acc.is_err() {
                    return acc;
                }

                let interval = arrival_k + task_k.deadline;
                let demand = demand_edf(taskset, k, task_k, concurrency, arrival_k);

                // For each  value of number of processors m, we compute the
                // smallest value of Theta that satisfies Equation (5) in
                // Theorem 1. However, Theta appears inside floor and ceiling
                // functions in sbf, and hence these computations may be
                // intractable. Therefore, we replace sbf in this equation with
                // lsbf given in Equation (2). [1]
                let resource_at_arrival_k =
                    resource_from_linear_sbf(demand, interval, period, concurrency);

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
            })?;

        Ok(Time::max(
            acc.unwrap(),
            best_resource_k
        ))
    })
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
            model.resource - model.concurrency as f64 *
                (model.resource / model.concurrency as f64).floor()
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

        let lsbf = linear_sbf(
            &MPRModel { resource, period, concurrency },
            interval
        );

        // skip negative supply values
        if lsbf < Time::zero() {
            continue;
        }

        let inverse = resource_from_linear_sbf(lsbf, interval, period, concurrency);
        assert_eq!(resource, inverse);
    }}}}
}

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

    assert_eq!(
        is_schedulable_edf(&taskset, &model).unwrap(),
        is_schedulable_edf_simple(&taskset, &model).unwrap()
    );
}

// References ------------------------------------------------------------------
// [1] I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
//     Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
//     Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
