//! ## MPR Model, FP Local Scheduler - * 2026
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive Fixed Priority local scheduling
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
//! 1. Unpublished
//! 2. M. Bertogna and M. Cirinei, “Response-Time Analysis for Globally Scheduled
//!    Symmetric Multiprocessor Platforms,” in 28th IEEE International Real-Time
//!    Systems Symposium (RTSS 2007), Dec. 2007, pp. 149–160.
//!    doi: 10.1109/RTSS.2007.31.

use crate::prelude::*;
use crate::algorithms::full_preemption::global_multiprocessor::hierarchical::mpr_model09::*;

const ALGORITHM: &str = "MPR Model, FP Local Scheduler (* 2026)";

/// MPR Model, FP Local Scheduler - * 2026 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub fn is_schedulable(taskset: &[RTTask], model: &MPRModel) -> SchedResult<Vec<Time>> {
    if !RTUtils::constrained_deadlines(taskset) {
        return SchedResultFactory(ALGORITHM).constrained_deadlines();
    }

    let result: Result<Vec<_>, _> =
        taskset.iter().enumerate()
        .map(|(k, task_k)| {
            let response =
                fixpoint_search_with_limit(
                    task_k.wcet,
                    task_k.deadline + Time::nanos(1.0),
                    |response: &Time|
                        rta(taskset, k, task_k, *response, model)
                );

            if response > task_k.deadline {
                return Err(anyhow::format_err!("Response time of task {k} is greater than its deadline."));
            } else {
                return Ok(response);
            }
        })
        .collect();

    SchedResult {
        test_name: ALGORITHM.to_owned(),
        result: result.map_err(|err| SchedError::NonSchedulable(Some(err))),
    }
}

fn rta(taskset: &[RTTask], k: usize, task_k: &RTTask, response: Time, model: &MPRModel) -> Time {
    let server_tasks = model.to_periodic_tasks();
    println!("{:?}", server_tasks);

    // Equation 2 [2]
    task_k.wcet
        +
    Time::floor((
        taskset.iter()
            .take(k)
            .map(|task_i| (response / task_i.period).ceil() * task_i.wcet + task_i.wcet )
            .sum::<Time>()
        // Periodic servers' delay [1]
        +
        server_tasks.iter()
            .map(|server| ((response + server.wcet) / server.period).ceil() * (server.period - server.wcet) )
            .sum::<Time>()
    )
        /
            model.concurrency as f64
    )
}

#[test]
fn equal_to_bcl09_test()
{
    // TODO: add taskset generator
    let taskset = [
        RTTask::new_ns(2, 50, 100),
        RTTask::new_ns(49, 100, 100),
        // RTTask::new_ns(34, 100, 100),
    ];

    // let model = MPRModel { resource: Time::nanos(13.0), period: Time::nanos(22.0), concurrency: 1 };
    for period in time_range_iterator_w_step(Time::nanos(10.0), Time::nanos(100.0), Time::nanos(1.0)) {
        for resource in time_range_iterator_w_step(period / 10.0, period, Time::nanos(1.0)) {
            for concurrency in 1 ..= 6 {
                let model = MPRModel { resource, period, concurrency };

                let bcl09 = super::bcl09::is_schedulable(&taskset, &model);
                let new_test = is_schedulable(&taskset, &model);

                assert_eq!(bcl09.is_schedulable(), new_test.is_schedulable(), "{:?}", model);

                // if bcl09.is_schedulable() == new_test.is_schedulable() {
                //     continue;
                // }

                if new_test.is_schedulable() {
                    println!("{:?}\n{:?}", model, new_test.result.unwrap());
                    // assert!(false);
                }
            }
        }
    }
}