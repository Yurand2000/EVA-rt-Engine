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

    #[allow(non_snake_case)]
    Ok(taskset.iter().enumerate().all(|(k, task_k)| {
        let Ak_upperbound = Ak_upperbound(taskset, task_k, num_processors).ceil();

        let mut last_dbfs = vec![Time::nanos(f64::NAN); taskset.len()];

        (Time::zero() ..= Ak_upperbound).into_iter()
            .all(|Ak| {
                let mut skip = true;

                let next_dbfs = taskset.iter()
                    .map(|task_i| demand_bound_function(task_i, Ak + task_k.deadline));

                // Section 6, Corollary 2 [1]
                for (last_dbf, next_dbf) in last_dbfs.iter_mut().zip(next_dbfs) {
                    if *last_dbf != next_dbf {
                        *last_dbf = next_dbf;
                        skip = false;
                    }
                }

                if skip {
                    return true;
                }

                baruah_test_single(taskset, k, task_k, Ak, num_processors, &last_dbfs)
            })
    }))
}

#[allow(non_snake_case)]
// Section 5, Theorem 2, Equation 8 [1]
fn baruah_test_single(taskset: &[RTTask], k: usize, task_k: &RTTask, Ak: Time, num_processors: u64, dbfs: &[Time]) -> bool {
    let interferences_1: Vec<_> = taskset.iter().enumerate()
        .map(|(i, task_i)| interference_1(i, task_i, k, task_k, Ak, dbfs)).collect();
    let mut interferences_diff: Vec<_> = taskset.iter().enumerate()
        .zip(interferences_1.iter())
        .map(|((i, task_i), i1)| interference_2(i, task_i, k, task_k, Ak) - *i1).collect();
    interferences_diff.sort_unstable();

    interferences_1.into_iter().sum::<Time>() + interferences_diff.into_iter().rev().take((num_processors - 1) as usize).sum::<Time>()
        <= num_processors as f64 * (Ak + task_k.deadline - task_k.wcet)
}

// Section 2 [1]
fn demand_bound_function(task: &RTTask, interval: Time) -> Time {
    Time::max(Time::zero(), ((interval - task.deadline) / task.period).floor() * task.wcet + task.wcet)
}

#[allow(non_snake_case)]
// Section 6, Equation 3 [1]
fn interference_1(i: usize, task_i: &RTTask, k: usize, task_k: &RTTask, Ak: Time, dbfs: &[Time]) -> Time {
    // let dbf = demand_bound_function(task_i, Ak + task_k.deadline);
    let dbf = dbfs[i];

    if i != k {
        Time::min(dbf, Ak + task_k.deadline - task_k.wcet)
    } else {
        Time::min(dbf - task_k.wcet, Ak)
    }
}

// Section 6, Equation 4 [1]
fn demand_bound_function_2(task: &RTTask, interval: Time) -> Time {
    (interval / task.period).floor() * task.wcet + Time::min(task.wcet, Time::nanos(interval % task.period))
}

#[allow(non_snake_case)]
// Section 6, Equation 5 [1]
fn interference_2(i: usize, task_i: &RTTask, k: usize, task_k: &RTTask, Ak: Time) -> Time {
    let dbf2 = demand_bound_function_2(task_i, Ak + task_k.deadline);

    if i != k {
        Time::min(dbf2, Ak + task_k.deadline - task_k.wcet)
    } else {
        Time::min(dbf2 - task_k.wcet, Ak)
    }
}


#[allow(non_snake_case)]
// Section 6, Equation 9 [1]
fn Ak_upperbound(taskset: &[RTTask], task_k: &RTTask, num_processors: u64) -> Time {
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

/* -----------------------------------------------------------------------------
References:
[1] S. Baruah, “Techniques for Multiprocessor Global Schedulability Analysis,”
in 28th IEEE International Real-Time Systems Symposium (RTSS 2007), Tucson, AZ,
USA: IEEE, Dec. 2007, pp. 119–128. doi: 10.1109/RTSS.2007.35.
*/