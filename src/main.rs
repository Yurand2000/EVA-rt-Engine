use analyzer::prelude::*;

const DEFAULT_AFTER_HELP: &str = "Refer to README.md files for further documentation";

#[derive(clap::Parser, Debug)]
#[command(after_help=DEFAULT_AFTER_HELP)]
struct Args {
    #[command(about, subcommand)]
    analysis: Analyses,

    /// Taskset data file
    #[arg(short='i', value_name="filename")]
    taskset_file: String,

    /// Taskset file type
    #[arg(value_enum, short='f', long="format", value_name="format", default_value="auto")]
    taskset_file_ty: TasksetFileType,

    /// Quiet mode / Exit code as analysis result
    /// 
    /// When enabled, a zero exit code means analysis success, a one means
    /// failure, any other code means that an error has happened.
    #[arg(short='q', default_value="false", action=clap::ArgAction::SetTrue)]
    quiet: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Analyses {
    /// UniProcessor Rate Monotonic
    /// 
    /// Preconditions:
    /// - Taskset sorted by period.
    /// - Implicit deadlines.
    #[command(name = "rate-monotonic", after_help=DEFAULT_AFTER_HELP, verbatim_doc_comment)]
    RateMonotonic(analyzer::analyses::up_rate_monotonic::runner::Args),
}

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

    let quiet = args.quiet;
    match main_w_exit_code(args) {
        Ok(success) => {
            if !quiet {
                if success {
                    println!("Analysis Output: Schedulable")
                } else {
                    println!("Analysis Output: NON Schedulable");
                }
            } else {
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

fn main_w_exit_code(args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    let taskset = parse_taskset(&args.taskset_file, args.taskset_file_ty)?;

    let result = match args.analysis {
        Analyses::RateMonotonic(args) => up_rate_monotonic::runner::main(&taskset, args)?,
    };
    
    Ok(result)
}
