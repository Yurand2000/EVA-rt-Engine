use eva_rt_engine::{common::taskset_serde::parse_taskset, prelude::*};

mod args;
use args::*;

fn main() {
    // Set exit code to 2 when there is a parse error.
    // This conforms to the 'quiet' flag specification:
    // - 0 -> schedulable
    // - 1 -> non-schedulable
    // - 2 -> other-error
    let args = match <Args as clap::Parser>::try_parse() {
        Ok(args) => args,
        Err(err) => {
            use clap::error::ErrorKind::*;

            let exit_code = match err.kind() {
                // display errors are ignored (exit-code = 0).
                DisplayHelp |
                DisplayHelpOnMissingArgumentOrSubcommand |
                DisplayVersion => 0,
                _ => 2,
            };

            err.print().unwrap();
            std::process::exit(exit_code);
        },
    };

    run_sched_analysis(args);
}

fn run_sched_analysis(args: Args) {
    let taskset =
        match parse_taskset(
            &args.taskset.taskset_file,
            args.taskset.taskset_file_ty
        ) {
            Ok(taskset) => taskset,
            Err(err) => {
                if !args.quiet {
                    eprintln!("{err}");
                }

                std::process::exit(2);
            },
        };

    let analyzers: [((PlatformModel, SchedulingAlgorithm, u64), fn(&[RTTask], Option<&str>) -> Vec<SchedResult<()>>); _] = [
        (
            (PlatformModel::FullyPreemptive, SchedulingAlgorithm::FixedPriority, 1),
            |taskset, specific_test| {
                let analyzer = eva_rt_engine::analysis::full_preemption::uniprocessor::fixed_priority::Analyzer;
                run_analyzer(analyzer, taskset, &(), specific_test)
            },
        ),
    ];

    let Some(results) =
        analyzers.into_iter()
        .find_map(|((platform, algorithm, num_processors), run_analyzer)| -> Option<Vec<SchedResult<()>>> {
            if args.scheduler.is_platform(platform, algorithm, num_processors) {
                Some(run_analyzer(&taskset, args.scheduler.specific_test.as_deref()))
            } else {
                None
            }
        })
        else {
            if !args.quiet {
                eprintln!(
                    "Unimplemented for Platform: {:?}, Algorithm: {:?}, Num Processors: {:?}",
                    args.scheduler.platform,
                    args.scheduler.algorithm,
                    args.scheduler.num_processors,
                );
            }

            std::process::exit(2);
        };

    if !args.quiet {
        results.iter()
            .for_each(|result| println!("{}", result) );
    }

    exit_on_results(&results);
}

fn run_analyzer<A, T, P>(analyzer: A, taskset: &[T], platform: &P, specific_test: Option<&str>) -> Vec<SchedResult<()>>
    where A: Analyzer<T, P>
{
    const DO_SHORT_CIRCUIT: bool = true;

    if let Some(test_name) = specific_test {
        vec![ analyzer.run_schedulability_test(taskset, &platform, &test_name) ]
    } else {
        analyzer.is_schedulable(taskset, &platform, DO_SHORT_CIRCUIT)
    }
}

fn exit_on_results(results: &[SchedResult<()>]) {
    let mut any_non_error = false;
    for result in results {
        if result.is_schedulable() {
            std::process::exit(0);
        } else if result.is_not_schedulable() {
            any_non_error = true;
        }
    }

    if any_non_error {
        std::process::exit(1);
    } else {
        std::process::exit(2);
    }
}