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
//! - [`is_schedulable`] \
//!   | O(*n^2*) complexity
//! - [`generate_model_linear`] \
//!   | O(*n^2*) complexity
//! - [`extra::generate_model`] \
//!   | pseudo-polynomial complexity
//! - [`extra::generate_best_model`] \
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
pub fn is_schedulable(taskset: &[RTTask], model: &MPRModel) -> SchedResult<()> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    let schedulable =
        is_schedulable_demand(
            taskset,
            model,
            |taskset, k, task_k, _, _|
                demand_edf(taskset, k, task_k, model.concurrency),
            |_, _, _, _| Box::new(std::iter::once(Time::zero())),
        );

    SchedResultFactory(ALGORITHM).is_schedulable(schedulable)
}

/// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
///
/// Generate the best MPRModel for the given taskset. This requires the model's
/// period and maxmimum cuncurrency.
///
/// Refer to the [module](`self`) level documentation.
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
            |taskset, k, task_k, _, concurrency, _|
                demand_edf(taskset, k, task_k, concurrency),
            |_, _, _, _, _| Box::new(std::iter::once(Time::zero())),
        );

    DesignResultFactory(ALGORITHM).from_option(model)
}

fn demand_edf(taskset: &[RTTask], k: usize, task_k: &RTTask, concurrency: u64) -> Time {
    global_earliest_deadline_first_demand(taskset, k, task_k)
        +
    concurrency as f64 * task_k.wcet
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

    /// MPR Model, EDF Local Scheduler - *Derived from* Bertogna, Cirinei, Lipari 2009 \[1\]
    ///
    /// Generate the best MPRModel for the given taskset. Searches the space of
    /// possible MPRModels given a range of valid periods.
    ///
    /// Refer to the [module](`self`) level documentation.
    pub fn generate_best_model(
        taskset: &[RTTask],
        (min_period, max_period, period_step): (Time, Time, Time),
        resource_step: Time,
    ) -> DesignResult<MPRModel> {
        if !RTUtils::constrained_deadlines(taskset) {
            return DesignResultFactory(super::ALGORITHM).constrained_deadlines();
        }

        let min_processors =
            num_processors_lower_bound(taskset);

        let max_processors =
            num_processors_upper_bound(taskset);

        let best_model =
            time_range_iterator_w_step(min_period, max_period, period_step)
            .flat_map(|period| {
                (min_processors ..= max_processors)
                .find_map(|concurrency| {
                    let best_model =
                        __generate_model(
                            taskset,
                            period,
                            concurrency,
                            resource_step
                        );

                    best_model.result.ok()
                })

            })
            .min_by_key(|model| model.resource);

        DesignResultFactory(super::ALGORITHM).from_option(best_model)
    }


    fn num_processors_lower_bound(taskset: &[RTTask]) -> u64 {
        f64::ceil(RTUtils::total_utilization(taskset)) as u64
    }

    fn num_processors_upper_bound(taskset: &[RTTask]) -> u64 {
        taskset.len() as u64
    }
}