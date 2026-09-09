//! ## MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009
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
//!   | O(*n^2*) complexity
//! - [`DesignerLinear::design`] \
//!   | O(*n^2*) complexity
//! - [`extra::DesignerPeriodConcurrency::design`] \
//!   | pseudo-polynomial complexity
//! - [`extra::DesignerFull::design`] \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of Global
//!    Scheduling Algorithms on Multiprocessor Platforms,” IEEE Transactions on
//!    Parallel and Distributed Systems, vol. 20, no. 4, pp. 553–566, Apr. 2009,
//!    doi: 10.1109/TPDS.2008.129.

use crate::prelude::*;
use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;
use crate::algorithms::full_preemption::global_multiprocessor::earliest_deadline_first::bcl09::global_earliest_deadline_first_demand;

const ALGORITHM: &str = "MPR Model, EDF Local Scheduler (*Derived from* Bertogna, Cirinei, Lipari 2009)";

/// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
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
        } else {
            Ok(())
        }
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        let schedulable =
            is_schedulable_demand(
                taskset,
                &self.model,
                |taskset, _, task_k, _, _|
                    demand_edf(taskset, task_k, self.model.concurrency),
                |_, _, _, _| Box::new(std::iter::once(Time::zero())),
            );

        SchedError::result_from_schedulable(schedulable)
    }
}

/// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
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
            |taskset, _, task_k, _, concurrency, _|
                demand_edf(taskset, task_k, concurrency),
            |_, _, _, _, _| Box::new(std::iter::once(Time::zero())),
        )
        .ok_or(SchedError::NonSchedulable(None))
    }
}

fn demand_edf<T>(taskset: &[T], task_k: &T, concurrency: u64) -> Time
    where T: task_traits::LLTask
{
    global_earliest_deadline_first_demand(taskset, task_k)
        +
    concurrency as f64 * task_k.get_wcet()
}

pub mod extra {
    use crate::prelude::*;
    use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

    /// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
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

    /// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
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

    fn num_processors_upper_bound<T>(taskset: &[T]) -> u64
        where T: task_traits::LLTask
    {
        taskset.len() as u64
    }
}
