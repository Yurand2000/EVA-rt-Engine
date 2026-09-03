//! ## Multiprocessor EDF - Baruah 2007
//!
//! #### Model:
//! - Periodic/Sporadic Task model
//! - Fully-Preemptive EDF scheduling
//!
//! #### Preconditions:
//! - Constrained Deadlines
//!
//! #### Implements:
//! - [`Analysis::is_schedulable`] \
//!   | pseudo-polynomial complexity
//! - [`AnalysisSimple::is_schedulable`] \
//!   | Worse runtime than [`Analysis::is_schedulable`], checks unnecessary timesteps. \
//!   | \
//!   | pseudo-polynomial complexity
//!
//! ---
//! #### References:
//! 1. S. Baruah, “Techniques for Multiprocessor Global Schedulability Analysis,”
//!    in 28th IEEE International Real-Time Systems Symposium (RTSS 2007),
//!    Tucson, AZ, USA: IEEE, Dec. 2007, pp. 119–128. doi: 10.1109/RTSS.2007.35.

use crate::prelude::*;
use crate::task::traits::LLTaskUtils as _;

const ALGORITHM: &str = "Multiprocessor EDF (Baruah 2007)";

/// Multiprocessor EDF - Baruah 2007 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct Analysis {
    pub num_processors: u64,
}

impl<T> SchedAnalysis<(), T> for Analysis
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        check_preconditions(taskset)
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {
        // As in [1]: It can also be shown that Condition 8 need only be tested at
        // those values of Ak at which DBF(tak_i, Ak
        // + Dk) (and DBF') changes for some task_i.
        //
        // Both functions change their output value on a periodic pattern: let C <=
        // D <= T, for task i where to compute the DBFs. The values change in the
        // range [0 + aT, C + aT] and at {D + aT} for all integers a. The union of
        // these ranges is the points where we actually need to perform the test.
        let schedulable =
            taskset.iter().enumerate().all(|(k, task_k)| {
                let ak_upperbound = arrival_k_upperbound(taskset, task_k, self.num_processors).ceil();

                (0 ..= ak_upperbound.ceil().as_nanos() as usize)
                    .map(|arrival_k| Time::nanos(arrival_k as f64))
                    .filter(|arrival_k| {
                        // Perform the test only where DBF/DBF' values change.
                        taskset.iter().any(|task_i| {
                            let interval = *arrival_k + task_k.get_deadline();
                            let modulus = interval % task_i.get_period();

                            modulus <= task_i.get_wcet() || modulus == task_i.get_deadline()
                        })
                    })
                    .all(|arrival_k| baruah_test_single(taskset, k, task_k, arrival_k, self.num_processors))
            });

        SchedError::result_from_schedulable(schedulable)
    }
}

/// Multiprocessor EDF - Baruah 2007 \[1\]
///
/// Refer to the [module](`self`) level documentation.
pub struct AnalysisSimple {
    pub num_processors: u64,
}

impl<T> SchedAnalysis<(), T> for AnalysisSimple
    where T: task_traits::LLTask
{
    fn analyzer_name(&self) -> &str { ALGORITHM }

    fn check_preconditions(&self, taskset: &[T]) -> Result<(), SchedError> {
        check_preconditions(taskset)
    }

    fn run_test(&self, taskset: &[T]) -> Result<(), SchedError> {

        let schedulable =
            taskset.iter().enumerate().all(|(k, task_k)| {
                let ak_upperbound = arrival_k_upperbound(taskset, task_k, self.num_processors).ceil();

                (0 ..= ak_upperbound.ceil().as_nanos() as usize)
                    .map(|arrival_k| Time::nanos(arrival_k as f64))
                    .all(|arrival_k| baruah_test_single(taskset, k, task_k, arrival_k, self.num_processors))
            });

        SchedError::result_from_schedulable(schedulable)
    }
}

fn check_preconditions<T>(taskset: &[T]) -> Result<(), SchedError>
    where T: task_traits::LLTask
{
    if !LLTasksetUtils::constrained_deadlines(taskset) {
        Err(SchedError::constrained_deadlines())
    } else {
        Ok(())
    }
}

// Section 5, Theorem 2, Equation 8 [1]
fn baruah_test_single<T>(taskset: &[T], k: usize, task_k: &T, arrival_k: Time, num_processors: u64) -> bool
    where T: task_traits::LLTask
{

    let interferences_1: Vec<_> = taskset.iter().enumerate()
        .map(|(i, task_i)| interference_1(i, task_i, k, task_k, arrival_k))
        .collect();

    let mut interferences_diff: Vec<_> = taskset.iter().enumerate()
        .zip(interferences_1.iter())
        .map(|((i, task_i), i1)| interference_2(i, task_i, k, task_k, arrival_k) - *i1)
        .collect();
    interferences_diff.sort_unstable();

    let i1_sum = interferences_1.into_iter().sum::<Time>();
    let idiff_sum = interferences_diff.into_iter()
        .rev().take((num_processors - 1) as usize).sum::<Time>();

    i1_sum + idiff_sum <= num_processors as f64 * (arrival_k + task_k.get_deadline() - task_k.get_wcet())
}

// Section 2 [1]
fn demand_bound_function<T>(task: &T, interval: Time) -> Time
    where T: task_traits::LLTask
{
    Time::max(Time::zero(), (((interval - task.get_deadline()) / task.get_period()).floor() + 1.0) * task.get_wcet())
}

// Section 6, Equation 3 [1]
fn interference_1<T>(i: usize, task_i: &T, k: usize, task_k: &T, arrival_k: Time) -> Time
    where T: task_traits::LLTask
{
    let dbf = demand_bound_function(task_i, arrival_k + task_k.get_deadline());
    if i != k {
        Time::min(dbf, arrival_k + task_k.get_deadline() - task_k.get_wcet())
    } else {
        Time::min(dbf - task_k.get_wcet(), arrival_k)
    }
}

// Section 6, Equation 4 [1]
fn demand_bound_function_2<T>(task: &T, interval: Time) -> Time
    where T: task_traits::LLTask
{
    (interval / task.get_period()).floor() * task.get_wcet() + Time::min(task.get_wcet(), interval % task.get_period())
}

// Section 6, Equation 5 [1]
fn interference_2<T>(i: usize, task_i: &T, k: usize, task_k: &T, arrival_k: Time) -> Time
    where T: task_traits::LLTask
{
    let dbf2 = demand_bound_function_2(task_i, arrival_k + task_k.get_deadline());

    if i != k {
        Time::min(dbf2, arrival_k + task_k.get_deadline() - task_k.get_wcet())
    } else {
        Time::min(dbf2 - task_k.get_wcet(), arrival_k)
    }
}


// Section 6, Equation 9 [1]
fn arrival_k_upperbound<T>(taskset: &[T], task_k: &T, num_processors: u64) -> Time
    where T: task_traits::LLTask
{
    let mut wcets: Vec<_> = taskset.iter().map(|task| task.get_wcet()).collect();
    wcets.sort_unstable();
    let csum: Time = wcets.into_iter().rev().take((num_processors - 1) as usize).sum();

    let total_utilization = LLTasksetUtils::total_utilization(taskset);

   (csum - task_k.get_deadline() * (num_processors as f64 - total_utilization)
        + taskset.iter().map(|task| (task.get_period() - task.get_deadline()) * task.utilization() ).sum()
        + num_processors as f64 * task_k.get_wcet())
    /
        (num_processors as f64 - total_utilization)
}

#[test]
pub fn simple_vs_optimized() {
    use crate::task::test_utils::*;

    let taskset = [
        new_ll_task(35, 90, 160),
        new_ll_task(70, 115, 160),
        new_ll_task(30, 50, 75),
    ];

    let num_processors = 1;

    assert_eq!(
        Analysis { num_processors }.is_schedulable(&taskset).is_ok(),
        AnalysisSimple { num_processors }.is_schedulable(&taskset).is_ok()
    );
}
