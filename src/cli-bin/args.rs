use eva_rt_engine::common::taskset_serde::prelude::*;

pub mod prelude {
    pub use super::{
        Args,
        SchedulerSpecification,
        SchedulingAlgorithm,
        TasksetArgs,
        CacheOptions,
    };
}

const DEFAULT_AFTER_HELP: &str = "Refer to README.md files for further documentation";

#[derive(clap::Parser, Debug)]
#[command(after_help=DEFAULT_AFTER_HELP)]
pub struct Args {
    /// Quiet mode / Exit code as analysis result
    ///
    /// When enabled, a zero exit code means analysis success, a one means
    /// failure, any other code means that an error has happened.
    #[arg(short='q', default_value="false", action=clap::ArgAction::SetTrue)]
    pub quiet: bool,

    #[command(flatten, next_help_heading="Scheduling Algorithm Specification")]
    pub scheduler_specification: SchedulerSpecification,

    #[command(flatten, next_help_heading="Taskset Specification")]
    pub taskset_args: TasksetArgs,

    #[command(flatten, next_help_heading="Cache Options")]
    pub cache_options: CacheOptions,
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = true)]
pub struct SchedulerSpecification {
    /// Scheduling Algorithm
    #[arg(short='a', value_name="ALGORITHM")]
    pub algorithm: Option<SchedulingAlgorithm>,

    /// Number of processors
    #[arg(short='n', value_name="# CPUs")]
    pub num_processors: Option<u64>,

    /// Specific Test
    #[arg(long="test", value_name="TEST NAME")]
    pub specific_test: Option<String>,

    /// Config file
    #[arg(short='c', value_name="CONFIG FILE")]
    pub config_file: Option<String>,
}

#[derive(Debug, Clone)]
#[derive(clap::ValueEnum)]
#[derive(serde::Deserialize)]
pub enum SchedulingAlgorithm {
    UpEDF,
    UpFP,
    GlobalEDF,
    GlobalFP,
}

#[derive(clap::Args, Debug)]
pub struct TasksetArgs {
    /// Taskset data file
    #[arg(short='i', value_name="TASKSET FILE")]
    pub taskset_file: String,

    /// Taskset file type
    #[arg(value_enum, short='f', long="format", value_name="FORMAT", default_value="auto")]
    pub taskset_file_ty: TasksetFileType,
}

#[derive(clap::Args, Debug)]
pub struct CacheOptions {
    /// Cache directory (enabled only if supplied)
    ///
    /// When supplied, the software will save analysis results in the specified
    /// folder and will not re-run analyses on the same taskset and analysis
    /// combination, caching outputs for re-runs on the same inputs.
    #[arg(long="cache-dir")]
    pub dir: Option<String>,

    /// Cache max size in Mb
    #[arg(long="cache-size", default_value="10")]
    pub size: u64,
}