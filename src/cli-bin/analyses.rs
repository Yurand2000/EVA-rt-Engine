use eva_engine::prelude::*;

pub fn uniprocessor_edf(taskset: &[RTTask], quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::up_earliest_deadline_first::*;

    print_test_result("Liu & Layland 1973", liu_layland_73(taskset), quiet)
}

pub fn uniprocessor_fp(taskset: &[RTTask], quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::up_fixed_priority::*;

    let tests: &[(&'static str, fn(&[RTTask]) -> Result<bool, Error> )] = &[
        ("Rate Monotonic - Liu & Layland 1973 (Simplified)", rate_monotonic::is_schedulable_simple),
        ("Rate Monotonic - Liu & Layland 1973", rate_monotonic::is_schedulable),
        ("Rate Monotonic - Bini, Buttazzo, Buttazzo 2001", rate_monotonic::is_schedulable_hyperbolic),
        ("Deadline Monotonic - Leung & Whitehead 1982 (Pessimistic)", deadline_monotonic::is_schedulable_pessimistic),
        ("Deadline Monotonic - Leung & Whitehead 1982", deadline_monotonic::is_schedulable),
    ];

    for (test_name, test_fn) in tests {
        if print_test_result(test_name, test_fn(taskset), quiet)? { return Ok(true) };
    }

    Ok(false)
}

pub fn global_earliest_deadline_first(taskset: &[RTTask], num_processors: u64, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::smp_earliest_deadline_first::*;

    let tests: &[(&'static str, fn(&[RTTask], u64) -> Result<bool, Error> )] = &[
        ("GBF Test", gfb_test_sporadic),
        ("BAK Test", bak_test),
        ("BCL Test", bcl_edf),
        ("Baruah Test", baruah_2007::baruah_test),
    ];

    for (test_name, test_fn) in tests {
        if print_test_result(test_name, test_fn(taskset, num_processors), quiet)? { return Ok(true) };
    }

    Ok(false)
}

pub fn global_fixed_priority(taskset: &[RTTask], num_processors: u64, quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    use eva_engine::analyses::smp_fixed_priority::*;

    let tests: &[(&'static str, fn(&[RTTask], u64) -> Result<bool, Error> )] = &[
        ("Deadline Monotonic - BCL Test", deadline_monotonic::is_schedulable),
    ];

    for (test_name, test_fn) in tests {
        if print_test_result(test_name, test_fn(taskset, num_processors), quiet)? { return Ok(true) };
    }

    Ok(false)
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