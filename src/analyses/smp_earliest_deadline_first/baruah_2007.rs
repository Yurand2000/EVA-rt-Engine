use std::marker::PhantomData;

use crate::prelude::*;

/// S. Baruah, “Techniques for Multiprocessor Global Schedulability Analysis,”
/// in 28th IEEE International Real-Time Systems Symposium (RTSS 2007), Tucson,
/// AZ, USA: IEEE, Dec. 2007, pp. 119–128. doi: 10.1109/RTSS.2007.35.
/// *Section 5, Theorem 2, Equation 8*
///
/// **Notes:**
/// - Sporadic tasks.
/// - Constrained deadlines.
pub fn baruah_test(taskset: &[RTTask], num_processors: u64) -> Result<bool, Error> {
    AnalysisUtils::assert_constrained_deadlines(taskset)?;
    AnalysisUtils::assert_integer_times(taskset)?;

    // As in [1]: It can also be shown that Condition 8 need only be tested at
    // those values of Ak at which DBF(tak_i, Ak + Dk) changes for some task_i.
    //
    // Given the defintion of DBF, its values may change only every period of
    // task_i. Given that we take all tasks, we then step by the greatest common
    // divisor of all the periods.
    let step_size = taskset.iter().map(|task| task.period)
        .fold(0, |acc, period| num::integer::gcd(acc, period.as_nanos() as usize));

    Ok(taskset.iter().enumerate().all(|(k, task_k)| {
        let ak_upperbound = arrival_k_upperbound(taskset, task_k, num_processors).ceil();

        let mut dbfs_generators: Vec<_> =
            taskset.iter().map(|task_i| {
                dbf_periodic_generator(task_i, task_k, step_size)
            }).collect();

        (0 ..= ak_upperbound.ceil().as_nanos() as usize).step_by(step_size)
            .all(|arrival_k| {
                baruah_test_single(
                    taskset, k, task_k, Time::nanos(arrival_k as f64), num_processors,
                    dbfs_generators.as_mut_slice(),
                )
            })
    }))
}

// Section 5, Theorem 2, Equation 8 [1]
fn baruah_test_single(taskset: &[RTTask], k: usize, task_k: &RTTask, arrival_k: Time, num_processors: u64,
    dbfs_generators: &mut [PeriodicGenerator<impl Fn(Time) -> Time, Time, Time>],
) -> bool {
    let interferences_1: Vec<_> = taskset.iter().enumerate()
        .map(|(i, task_i)| interference_1(i, task_i, k, task_k, arrival_k, dbfs_generators)).collect();
    let mut interferences_diff: Vec<_> = taskset.iter().enumerate()
        .zip(interferences_1.iter())
        .map(|((i, task_i), i1)| interference_2(i, task_i, k, task_k, arrival_k) - *i1).collect();
    interferences_diff.sort_unstable();

    interferences_1.into_iter().sum::<Time>() + interferences_diff.into_iter().rev().take((num_processors - 1) as usize).sum::<Time>()
        <= num_processors as f64 * (arrival_k + task_k.deadline - task_k.wcet)
}

// Section 2 [1]
fn demand_bound_function(task: &RTTask, interval: Time) -> Time {
    Time::max(Time::zero(), ((interval - task.deadline) / task.period).floor() * task.wcet + task.wcet)
}

fn dbf_periodic_generator(task_i: &RTTask, task_k: &RTTask, periods_gcd: usize)
    -> PeriodicGenerator<impl Fn(Time) -> Time, Time, Time>
{
    PeriodicGenerator::new(
        |arrival_k: Time| demand_bound_function(task_i, arrival_k + task_k.deadline),
        task_i.period.as_nanos() as usize / periods_gcd,
        task_i.deadline.as_nanos() as usize
    )
}

// Section 6, Equation 3 [1]
fn interference_1(i: usize, _task_i: &RTTask, k: usize, task_k: &RTTask, arrival_k: Time,
    dbfs_generators: &mut [PeriodicGenerator<impl Fn(Time) -> Time, Time, Time>],
) -> Time {
    // let dbf = demand_bound_function(task_i, Ak + task_k.deadline);
    let dbf = dbfs_generators[i].next(arrival_k);

    if i != k {
        Time::min(dbf, arrival_k + task_k.deadline - task_k.wcet)
    } else {
        Time::min(dbf - task_k.wcet, arrival_k)
    }
}

// Section 6, Equation 4 [1]
fn demand_bound_function_2(task: &RTTask, interval: Time) -> Time {
    (interval / task.period).floor() * task.wcet + Time::min(task.wcet, Time::nanos(interval % task.period))
}

// Section 6, Equation 5 [1]
fn interference_2(i: usize, task_i: &RTTask, k: usize, task_k: &RTTask, arrival_k: Time) -> Time {
    let dbf2 = demand_bound_function_2(task_i, arrival_k + task_k.deadline);

    if i != k {
        Time::min(dbf2, arrival_k + task_k.deadline - task_k.wcet)
    } else {
        Time::min(dbf2 - task_k.wcet, arrival_k)
    }
}


// Section 6, Equation 9 [1]
fn arrival_k_upperbound(taskset: &[RTTask], task_k: &RTTask, num_processors: u64) -> Time {
    let mut wcets: Vec<_> = taskset.iter().map(|task| task.wcet).collect();
    wcets.sort_unstable();
    let csum: Time = wcets.into_iter().rev().take((num_processors - 1) as usize).sum();

    let total_utilization = RTUtils::total_utilization(taskset);

   (csum - task_k.deadline * (num_processors as f64 - total_utilization)
        + taskset.iter().map(|task| (task.period - task.deadline) * task.utilization() ).sum()
        + num_processors as f64 * task_k.wcet)
    /
        (num_processors as f64 - total_utilization)
}

// -----------------------------------------------------------------------------
struct PeriodicGenerator<F: Fn(T) -> U, T, U> {
    function: F,
    last_result: Option<U>,
    last_step: usize,
    num_steps: usize,
    _phantom_t: PhantomData<T>,
}

impl<F: Fn(T) -> U, T, U: Clone> PeriodicGenerator<F, T, U> {
    pub fn new(fun: F, period: usize, offset: usize) -> Self {
        Self {
            function: fun,
            last_result: None,
            last_step: offset,
            num_steps: period,
            _phantom_t: PhantomData,
        }
    }

    pub fn next(&mut self, data: T) -> U {
        if let Some(result) = self.last_result.as_mut() {
            if self.last_step % self.num_steps == 0 {
                let new_result = (self.function)(data);
                self.last_result = Some(new_result.clone());
                self.last_step += 1;
                new_result
            } else {
                self.last_step += 1;
                result.clone()
            }
        } else {
            let new_result = (self.function)(data);
            self.last_result = Some(new_result.clone());
            new_result
        }
    }
}

#[test]
fn periodic_generator_dbf_test() {
    let task_i = RTTask::new_ns(10, 20, 20);
    let task_k = RTTask::new_ns(5, 20, 40);

    let periods_gcd = num::integer::gcd(20, 40);

    let mut generator = dbf_periodic_generator(&task_i, &task_k, periods_gcd);

    for time in (0 .. 1000).step_by(periods_gcd) {
        let arrival_k = Time::nanos(time as f64);

        assert_eq!(generator.next(arrival_k), demand_bound_function(&task_i, arrival_k + task_k.deadline));
    }
}

#[test]
fn periodic_generator_dbf_test_2() {
    let task_i = RTTask::new_ns(10, 20, 20);
    let task_k = RTTask::new_ns(5, 20, 40);

    for period in 0..100 {
        let dbf_at_zero = demand_bound_function(&task_i, task_i.deadline + (period as f64 * task_i.period) + task_k.deadline);
        for time in 0..task_i.period.as_nanos() as usize {
            let arrival_k = task_i.deadline + Time::nanos(time as f64) + (period as f64 * task_i.period);

            assert_eq!(dbf_at_zero, demand_bound_function(&task_i, arrival_k + task_k.deadline));
        }
    }
}

/* -----------------------------------------------------------------------------
References:
[1] S. Baruah, “Techniques for Multiprocessor Global Schedulability Analysis,”
in 28th IEEE International Real-Time Systems Symposium (RTSS 2007), Tucson, AZ,
USA: IEEE, Dec. 2007, pp. 119–128. doi: 10.1109/RTSS.2007.35.
*/