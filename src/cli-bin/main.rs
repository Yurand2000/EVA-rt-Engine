use crate::prelude::*;
use eva_engine::prelude::*;

pub mod prelude {
    pub use super::args::prelude::*;
}

pub mod analyses;
pub mod args;

fn main() {
    let args = match <Args as clap::Parser>::try_parse() {
        Ok(args) => args,
        Err(err) => {
            use clap::error::ErrorKind::*;

            let exit_code = match err.kind() {
                DisplayHelp |
                DisplayHelpOnMissingArgumentOrSubcommand |
                DisplayVersion => 0,
                _ => 2,
            };

            err.print().unwrap();
            std::process::exit(exit_code);
        },
    };

    check_args(&args).unwrap();

    let quiet = args.quiet;
    match main_wo_exit_code(args) {
        Ok(success) => {
            if quiet {
                if success {
                    std::process::exit(0);
                } else {
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        },
    };
}

fn check_args(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    match (
        args.scheduler_specification.algorithm.is_some(),
        args.scheduler_specification.num_processors.is_some(),
        args.scheduler_specification.config_file.is_some()
    ) {
        (true, true, false) | (false, false, true) => Ok(()),
        _ => Err(format!(
            "Either specify an Algorithm and the # of CPUs or a config file (run with -h for help)"
        ).into())
    }
}

fn check_analysis_args(
    algorithm: &SchedulingAlgorithm,
    num_cpus: u64
) -> Result<(), Box<dyn std::error::Error>> {
    use SchedulingAlgorithm::*;

    if num_cpus == 0 {
        return Err(format!("Must specify a positive number of CPUs").into());
    }

    let up_num_cpu_error = "UniProcessor Algorithms can run on only one CPU";
    match algorithm {
        UpEDF if num_cpus != 1 => Err(format!("{up_num_cpu_error}").into()),
        UpFP if num_cpus != 1 => Err(format!("{up_num_cpu_error}").into()),
        _ => Ok(()),
    }
}

#[derive(serde::Deserialize)]
struct SchedulerSpecification {
    pub algorithm: SchedulingAlgorithm,
    pub num_processors: u64,
    pub specific_test: Option<String>,
}

fn main_wo_exit_code(args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    use SchedulingAlgorithm::*;

    let taskset = parse_taskset(
        &args.taskset_args.taskset_file,
        args.taskset_args.taskset_file_ty
    )?;

    let scheduler: SchedulerSpecification =
        args.scheduler_specification.algorithm
            .and_then(|algo| -> Option<Result<_, Box<dyn std::error::Error>>> {
                Some( Ok(SchedulerSpecification {
                    algorithm: algo,
                    num_processors: args.scheduler_specification.num_processors.unwrap(),
                    specific_test: args.scheduler_specification.specific_test,
                } ))
            })
            .unwrap_or_else(|| {
                std::fs::read_to_string(args.scheduler_specification.config_file.unwrap())
                    .map_err(|err| format!("Config parse error: {err}").into())
                    .and_then(|config_data| {
                        serde_json::from_str(&config_data)
                            .map_err(|err| format!("Config parse error: {err}").into())
                    })
            })?;

    check_analysis_args(&scheduler.algorithm, scheduler.num_processors)?;

    let single_test = scheduler.specific_test.as_deref();
    let num_cpus = scheduler.num_processors;
    match scheduler.algorithm {
        UpEDF =>
            analyses::uniprocessor_edf(&taskset, single_test, args.quiet),
        UpFP =>
            analyses::uniprocessor_fp(&taskset, single_test, args.quiet),
        GlobalEDF =>
            analyses::global_earliest_deadline_first(&taskset, num_cpus, single_test, args.quiet),
        GlobalFP =>
            analyses::global_fixed_priority(&taskset, num_cpus, single_test, args.quiet),
    }
}
