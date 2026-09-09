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
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//! - [`DesignerLinear::design`] \
//!   | pseudo-polynomial complexity
//! - [`extra::DesignerPeriodConcurrency::design`] \
//!   | pseudo-polynomial complexity
//! - [`extra::DesignerFull::design`] \
//!   | pseudo-polynomial complexity
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
use task_traits::LLTaskUtils as _;

const ALGORITHM: &str = "MPR Model, EDF Local Scheduler (Shin, Easwaran, Lee 2009)";

/// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub model: MPRModel,
}

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else if f64::abs(self.model.utilization() - LLTasksetUtils::total_utilization(taskset)) < 0.01 {
            Err(SchedError::Other(
                anyhow::format_err!("Arrival times upperbound tends to infinity, the computation becomes intractable.")
            ))
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Section 4.2, Theorem 1 [1]
        let schedulable =
            is_schedulable_demand(
                taskset,
                &self.model,
                |taskset, _, task_k, model, arrival_k|
                    demand_edf(taskset, task_k, model.concurrency, arrival_k),
                |taskset, _, task_k, model| -> Box<dyn Iterator<Item = Time>> {
                    let arrival_k_upperbound =
                        arrival_k_upperbound_edf(taskset, task_k, model);

                    Box::new(
                        TimeIterators::range(Time::zero(), arrival_k_upperbound)
                        .filter(|arrival_k| filter_intervals_edf(taskset, task_k, model, *arrival_k))
                    )
                }
            );

        SchedError::result_from_schedulable(schedulable)
    }
}


/// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct AnalysisSimple {
    pub model: MPRModel,
}

impl<T> SchedAnalysis<(), T> for AnalysisSimple
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // Section 4.2, Theorem 1 [1]
        let schedulable =
            is_schedulable_demand(
                taskset,
                &self.model,
                |taskset, _, task_k, model, arrival_k|
                    demand_edf(taskset, task_k, model.concurrency, arrival_k),
                |taskset, _, task_k, model|  {
                    let arrival_k_upperbound =
                        arrival_k_upperbound_edf(taskset, task_k, model);

                    Box::new(TimeIterators::range(Time::zero(), arrival_k_upperbound))
                }
            );

        SchedError::result_from_schedulable(schedulable)
    }
}

/// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
///
/// Generate the best MPRModel for the given taskset. This requires the model's
/// period and maxmimum cuncurrency.
///
/// Refer to the [module](`self`) level documentation.
pub struct DesignerLinear {
    pub period: Time,
    pub concurrency: u64,
}

impl<T> SchedDesign<MPRModel, T> for DesignerLinear
    where T: task_traits::LLTask
{
    fn designer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        if !LLTasksetUtils::constrained_deadlines(taskset) {
            Err(SchedError::constrained_deadlines())
        } else {
            Ok(())
        }
    }

    fn run_designer(&self, taskset: &[T]) -> Result<MPRModel, SchedError> {
        generate_model_from_demand_linear(
            taskset,
            self.period,
            self.concurrency,
            |taskset, _, task_k, _, concurrency, arrival_k|
                demand_edf(taskset, task_k, concurrency, arrival_k),
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
                        let interval = *arrival_k + task_k.get_deadline();

                        // Perform the test only where I_hat/I_flat values change.
                        taskset.iter().any(|task_i| {
                            let modulus = *arrival_k % task_i.get_period();

                            interval <= task_i.get_wcet() || modulus == Time::zero()
                        })
                    })
                )
            },
        )
        .ok_or(SchedError::NonSchedulable(None))
    }
}

fn filter_intervals_edf<T>(
    taskset: &[T],
    task_k: &T,
    model: &MPRModel,
    arrival_k: Time
) -> bool
    where T: task_traits::LLTask
{
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
    let interval = arrival_k + task_k.get_deadline();

    // Perform the test only where SBF changes
    let floor = (model.resource / model.concurrency as f64).floor();
    let modulus = (interval + floor) % model.period;
    if modulus >= model.period - floor || modulus == Time::zero() {
        return true;
    }

    // Perform the test only where I_hat/I_flat values change.
    taskset.iter().any(|task_i| {
        let modulus = arrival_k % task_i.get_period();

        interval <= task_i.get_wcet() || modulus == Time::zero()
    })
}

// Section 4.2, Theorem 2 [1]
fn arrival_k_upperbound_edf<T>(taskset: &[T], task_k: &T, model: &MPRModel) -> Time
    where T: task_traits::LLTask
{
    let taskset_utilization = LLTasksetUtils::total_utilization(taskset);

    let mut wcets: Vec<_> =
        taskset.iter().map(|task| task.get_wcet()).collect();
    wcets.sort_unstable();

    let c_sum: Time = wcets.into_iter().rev().take(model.concurrency as usize - 1).sum();

    let u_sum: Time = taskset.iter()
        .map(|task| (task.get_period() - task.get_deadline()) * task.utilization()).sum();
    let b_sum: Time =
        model.resource * (2.0 - (2.0 * model.resource) / (model.concurrency as f64 * model.period));

    (
        c_sum
        - model.concurrency as f64 * task_k.get_wcet()
        - task_k.get_deadline() * (model.utilization() - taskset_utilization)
        + u_sum + b_sum
    ) / (
        model.utilization() - taskset_utilization
    )
}

// Section 4.2, Theorem 1 [1]
fn demand_edf<T>(taskset: &[T], task_k: &T, concurrency: u64, arrival_k: Time) -> Time
    where T: task_traits::LLTask
{
    let interference_hat: Vec<_> =
        taskset.iter()
            .map(|task_i| interference_hat(task_i, task_k, arrival_k))
            .collect();

    let mut interference_diff: Vec<_> =
        taskset.iter().zip(interference_hat.iter())
            .map(|(task_i, interference_hat_i)| interference_flat(task_i, task_k, arrival_k) - *interference_hat_i)
            .collect();

    interference_diff.sort_unstable();

    let sum_interference_hat: Time = interference_hat.into_iter().sum();
    let sum_interference_diff: Time = interference_diff.into_iter().rev()
        .take(concurrency as usize - 1).sum();

    sum_interference_hat + sum_interference_diff + concurrency as f64 * task_k.get_wcet()
}

// Section 4.2, Theorem 1 [1]
fn interference_flat<T>(task_i: &T, task_k: &T, arrival_k: Time) -> Time
    where T: task_traits::LLTask
{
    let workload_upperbound = workload_upperbound_2_edf(task_i, arrival_k + task_k.get_deadline());

    if std::ptr::eq(task_i, task_k) {
        Time::min(workload_upperbound - task_k.get_wcet(), arrival_k)
    } else {
        Time::min(workload_upperbound, arrival_k + task_k.get_deadline() - task_k.get_wcet())
    }
}

// Section 4.2, Theorem 1 [1]
fn interference_hat<T>(task_i: &T, task_k: &T, arrival_k: Time) -> Time
    where T: task_traits::LLTask
{
    let workload_upperbound = workload_upperbound_edf(task_i, arrival_k + task_k.get_deadline());

    if std::ptr::eq(task_i, task_k) {
        Time::min(workload_upperbound - task_k.get_wcet(), arrival_k)
    } else {
        Time::min(workload_upperbound, arrival_k + task_k.get_deadline() - task_k.get_wcet())
    }
}

// Equation 3 [1]
fn workload_upperbound_2_edf<T>(task: &T, time: Time) -> Time
    where T: task_traits::LLTask
{
    activations_in_interval_edf(task, time) * task.get_wcet() + carry_in_edf(task, time)
}

fn workload_upperbound_edf<T>(task: &T, time: Time) -> Time
    where T: task_traits::LLTask
{
    activations_in_interval_edf(task, time) * task.get_wcet()
}

// Equation 3 [1]
#[inline(always)]
fn activations_in_interval_edf<T>(task: &T, time: Time) -> f64
    where T: task_traits::LLTask
{
    ((time + task.get_period() - task.get_deadline()) / task.get_period()).floor()
}

// Equation 3 [1]
#[inline(always)]
fn carry_in_edf<T>(task: &T, time: Time) -> Time
    where T: task_traits::LLTask
{
    Time::min(
        task.get_wcet(),
        Time::max(
            Time::zero(),
            time - activations_in_interval_edf(task, time) * task.get_period()
        )
    )
}

pub mod extra {
    use crate::prelude::*;
    use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

    /// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
    ///
    /// Generate the best MPRModel for the given taskset. This requires the model's
    /// period and maxmimum cuncurrency.
    ///
    /// Refer to the [module](`self`) level documentation.
    pub struct DesignerPeriodConcurrency {
        pub period: Time,
        pub concurrency: u64,
        pub resource_step: Time,
    }

    impl<T> SchedDesign<MPRModel, T> for DesignerPeriodConcurrency
        where T: task_traits::LLTask
    {
        fn designer_name(&self) -> &str { super::ALGORITHM }

        fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
            if !LLTasksetUtils::constrained_deadlines(taskset) {
                Err(SchedError::constrained_deadlines())
            } else {
                Ok(())
            }
        }

        fn run_designer(&self, taskset: &[T]) -> Result<MPRModel, SchedError> {
            let min_resource =
                LLTasksetUtils::total_utilization(taskset) * self.period;
            let max_resource = {
                let designer = super::DesignerLinear { period: self.period, concurrency: self.concurrency };

                designer.check_preconditions(&taskset)?;
                designer.run_designer(taskset)?.resource
            };

            (extra::DesignerPeriodConcurrencyNaive {
                period: self.period,
                concurrency: self.concurrency,
                resource_iter_fn: |_, _| Ok(Box::new(TimeIterators::range_w_step(min_resource, max_resource, self.resource_step))),
                analysis_gen_fn: |resource, period, concurrency|
                    super::Analysis { model: MPRModel { resource, period, concurrency }},
                marker: std::marker::PhantomData,
            })
            .run_designer(taskset)
        }
    }

    /// MPR Model, EDF Local Scheduler - Shin, Easwaran, Lee 2009 \[1\]
    ///
    /// Generate the best MPRModel for the given taskset. Searches the space of
    /// possible MPRModels given a range of valid periods.
    ///
    /// Refer to the [module](`self`) level documentation.
    pub struct DesignerFull {
        pub period_range: (Time, Time, Time),
        pub resource_step: Time,
    }

    impl<T> SchedDesign<MPRModel, T> for DesignerFull
        where T: task_traits::LLTask
    {
        fn designer_name(&self) -> &str { super::ALGORITHM }

        fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
            if !LLTasksetUtils::constrained_deadlines(taskset) {
                Err(SchedError::constrained_deadlines())
            } else {
                Ok(())
            }
        }

        fn run_designer(&self, taskset: &[T]) -> Result<MPRModel, SchedError> {
            let min_processors =
                num_processors_lower_bound(taskset);

            let max_processors =
                num_processors_upper_bound(taskset);

            let designer = extra::DesignerNaive {
                period_iter_fn: || Ok(Box::new(TimeIterators::range_w_step(self.period_range.0, self.period_range.1, self.period_range.2))),
                concurrency_iter_fn: |_| Ok(Box::new(min_processors ..= max_processors)),
                resource_iter_fn: |period, concurrency| {
                    let min_resource =
                        LLTasksetUtils::total_utilization(taskset) * period;
                    let max_resource = {
                        let designer = super::DesignerLinear { period, concurrency };

                        designer.check_preconditions(&taskset)?;
                        designer.run_designer(taskset)?.resource
                    };

                    Ok(Box::new(TimeIterators::range_w_step(min_resource, max_resource, self.resource_step)))
                },
                analysis_gen_fn: |resource, period, concurrency|
                    super::Analysis { model: MPRModel { resource, period, concurrency }},
                marker: std::marker::PhantomData,
            };

            designer.run_designer(taskset)
        }
    }

    fn num_processors_lower_bound<T>(taskset: &[T]) -> u64
        where T: task_traits::LLTask
    {
        f64::ceil(LLTasksetUtils::total_utilization(taskset)) as u64
    }

    // Section 5.1, Lemma 4 [1]
    fn num_processors_upper_bound<T>(taskset: &[T]) -> u64
        where T: task_traits::LLTask
    {
        debug_assert!(!taskset.is_empty());

        let n = taskset.len() as u64;

        let den = taskset.iter()
            .map(|task| task.get_deadline() - task.get_wcet())
            .min()
            .unwrap();

        if den == Time::zero() {
            todo!("unexpected");
        }

        let total_work: Time = taskset.iter()
            .map(|task| task.get_wcet())
            .sum();

        (total_work / den).ceil() as u64 + n
    }
}

// -----------------------------------------------------------------------------

#[test]
pub fn simple_vs_optimized() {
    use crate::task::test_utils::*;

    let taskset = [
        new_ll_task(35, 90, 160),
        new_ll_task(70, 115, 160),
        new_ll_task(30, 50, 75),
    ];

    let model = MPRModel {
        resource: Time::nanos(75.0),
        period: Time::nanos(50.0),
        concurrency: 2,
    };

    let optimized_test = Analysis { model: model.clone() }.is_schedulable(&taskset);
    let simple_test = AnalysisSimple { model: model }.is_schedulable(&taskset);
    assert_eq!(optimized_test.is_ok(), simple_test.is_ok());
}
