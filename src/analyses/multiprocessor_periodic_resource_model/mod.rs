use crate::prelude::*;

pub mod bcl_2009;
pub mod generic;

// Section 3.2 [1]
#[derive(Debug, Clone)]
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

// Definition 1 [2]
pub fn sbf(model: &MPRModel, time: Time) -> Time {
    let m = model.concurrency as f64;
    let a = Time::floor(model.resource / m);
    let b = model.resource - m * a;
    let t1 = time - (model.period - Time::ceil(model.resource / m));
    let x = t1 - model.period * f64::floor(t1 / model.period);
    let y = model.period - a;

    if t1 < Time::zero() {
        Time::zero()
    } else {
        f64::floor(t1 / model.period) * model.resource
            +
        Time::max(Time::zero(), m*x - m*model.period + model.resource)
            -
        if x >= Time::one() && x < y {
            Time::zero()
        } else {
            Time::nanos(m) - b
        }
    }
}

#[deprecated]
// Easwaran's SBF, from [2], is not monotone, thus exponential search cannot be applied.
pub fn resource_from_sbf(sbf_v: Time, interval: Time, period: Time, concurrency: u64) -> Time {
    exp_search( 0.. , |resource| {
        let model = MPRModel {
            resource: Time::nanos(resource as f64),
            period: period,
            concurrency: concurrency,
        };

        sbf(&model, interval)
    }, sbf_v)
        .map(|resource| Time::nanos(resource as f64))
        .unwrap_or(Time::zero())
}

// Equation 2 [2]
pub fn linear_sbf(model: &MPRModel, interval: Time) -> Time {
    let (resource, period, concurrency) = (model.resource, model.period, model.concurrency);

    resource / period * (interval - 2.0 * (period - resource / concurrency as f64) + Time::nanos(2.0))
}

// Extracted Theta from Equation 2 [2]
pub fn resource_from_linear_sbf(lsbf: Time, interval: Time, period: Time, concurrency: u64) -> Time {
    // Note that this only works for positive values of the linear supply bound.
    // There is only one positive solution for a positive bound, but two
    // solutions or zero for a negative one.
    debug_assert!(lsbf >= Time::zero());

    let concurrency = concurrency as f64;
    let negb = 2.0 * period - interval + Time::nanos(2.0);
    let bsqr = negb * negb;

    concurrency * (negb + Time2::sqrt(bsqr + 8.0 * period * lsbf / concurrency) ) / 4.0
}

// global EDF for MPR ----------------------------------------------------------

// Section 4.2, Theorem 1 [1]
pub fn is_schedulable_edf_simple(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::is_schedulable(
        taskset,
        model,
        |taskset, k, task_k, model, arrival_k|
            demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
        |taskset, _, task_k, model|
            arrival_k_upperbound_edf(taskset, task_k, model),
        |_, _, _, _, _| true,
    )
}

pub fn is_schedulable_edf(taskset: &[RTTask], model: &MPRModel) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    generic::is_schedulable(
        taskset,
        model,
        |taskset, k, task_k, model, arrival_k|
            demand_edf(taskset, k, task_k, model.concurrency, arrival_k),
        |taskset, _, task_k, model|
            arrival_k_upperbound_edf(taskset, task_k, model),
        |taskset, _, task_k, model, arrival_k|
            filter_intervals_edf(taskset, task_k, model, arrival_k),
    )
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
    model: &generic::MPRModelSpecification,
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
                    resource_from_linear_sbf(demand, interval, model.period, model.concurrency),

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
// [2] A. Easwaran, I. Shin, and I. Lee, “Optimal Virtual Cluster-based
//     Multiprocessor Scheduling,” Real-Time Syst, vol. 43, no. 1, pp. 25–59,
//     Sept. 2009, doi: 10.1007/s11241-009-9073-x.
