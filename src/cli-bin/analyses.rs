use eva_engine::prelude::*;

pub fn uniprocessor_edf(taskset: &[RTTask], single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::up_earliest_deadline_first::*;

    let tests: &[(_, _, fn(&[RTTask]) -> Result<bool, Error> )] = &[
        ("classic", "Rate Monotonic - Liu & Layland 1973", liu_layland_73),
    ];

    test_runner_up(tests, taskset, single_test, quiet)
}

pub fn uniprocessor_fp(taskset: &[RTTask], single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::up_fixed_priority::*;

    let tests: &[(_, _, fn(&[RTTask]) -> Result<bool, Error> )] = &[
        ("rm-simplified", "Rate Monotonic - Liu & Layland 1973 (Simplified)", rate_monotonic::is_schedulable_simple),
        ("rm-classic", "Rate Monotonic - Liu & Layland 1973", rate_monotonic::is_schedulable),
        ("rm-hyperbolic", "Rate Monotonic - Bini, Buttazzo, Buttazzo 2001", rate_monotonic::is_schedulable_hyperbolic),
        ("dm-pessimistic", "Deadline Monotonic - Leung & Whitehead 1982 (Pessimistic)", deadline_monotonic::is_schedulable_pessimistic),
        ("dm-classic", "Deadline Monotonic - Leung & Whitehead 1982", deadline_monotonic::is_schedulable),
    ];

    test_runner_up(tests, taskset, single_test, quiet)
}

pub fn global_earliest_deadline_first(taskset: &[RTTask], num_processors: u64, single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::smp_earliest_deadline_first::*;

    let tests: &[(_, _, fn(&[RTTask], u64) -> Result<bool, Error> )] = &[
        ("gbf", "GBF Test", gfb_test_sporadic),
        ("bak", "BAK Test", bak_test),
        ("bcl", "BCL Test", bcl_edf),
        ("baruah", "Baruah Test", baruah_2007::baruah_test),
    ];

    test_runner_smp(tests, taskset, num_processors, single_test, quiet)
}

pub fn global_fixed_priority(taskset: &[RTTask], num_processors: u64, single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::smp_fixed_priority::*;

    let tests: &[(_, _, fn(&[RTTask], u64) -> Result<bool, Error> )] = &[
        ("bcl", "Deadline Monotonic - BCL Test", deadline_monotonic::is_schedulable),
    ];

    test_runner_smp(tests, taskset, num_processors, single_test, quiet)
}

fn print_test_result(test_name: &str, res: Result<bool, eva_engine::analyses::Error>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    match res {
        Ok(success) => {
            if !quiet {
                if success {
                    println!("{test_name}: Pass");
                } else {
                    println!("{test_name}: Failure");
                }
            }

            Ok(success)
        },
        Err(err @ eva_engine::analyses::Error::Generic(_)) => Err(err.into()),
        Err(err) => {
            if !quiet {
                println!("{test_name}: Error\n    {err}");
            }

            Ok(false)
        }
    }
}

fn test_runner_up(tests: &[(&'static str, &'static str, fn(&[RTTask]) -> Result<bool, Error>)],
    taskset: &[RTTask], single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>>
{
    match single_test {
        Some(single_test) => {
            tests.iter().find(|(test_id, _, _)| *test_id == single_test)
                .map(|(_, test_name, test_fn)| {
                    print_test_result(test_name, test_fn(taskset), quiet)
                }).unwrap_or_else(|| {
                    Err(format!("Single Test \"{single_test}\" not found").into())
                })
        },
        None => {
            for (_, test_name, test_fn) in tests {
                    if print_test_result(test_name, test_fn(taskset), quiet)? { return Ok(true) };
                }

            Ok(false)
        },
    }
}

fn test_runner_smp(tests: &[(&'static str, &'static str, fn(&[RTTask], u64) -> Result<bool, Error>)],
    taskset: &[RTTask], num_processors: u64, single_test: Option<&str>, quiet: bool) -> Result<bool, Box<dyn std::error::Error>>
{
    match single_test {
        Some(single_test) => {
            tests.iter().find(|(test_id, _, _)| *test_id == single_test)
                .map(|(_, test_name, test_fn)| {
                    print_test_result(test_name, test_fn(taskset, num_processors), quiet)
                }).unwrap_or_else(|| {
                    Err(format!("Single Test \"{single_test}\" not found").into())
                })
        },
        None => {
            for (_, test_name, test_fn) in tests {
                    if print_test_result(test_name, test_fn(taskset, num_processors), quiet)? { return Ok(true) };
                }

            Ok(false)
        },
    }
}