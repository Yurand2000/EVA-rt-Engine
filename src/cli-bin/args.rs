const DEFAULT_AFTER_HELP: &str = "Refer to the crate's documentation for further help";

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
    pub scheduler: SchedulingArgs,

    #[command(flatten, next_help_heading="Taskset Specification")]
    pub taskset: TasksetArgs,
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = true)]
pub struct SchedulingArgs {
    /// Platform model
    #[arg(short='p', value_name="PLATFORM", default_value="fully-preemptive")]
    pub platform: PlatformModel,

    /// Scheduling algorithm
    #[arg(short='a', value_name="ALGORITHM")]
    pub algorithm: SchedulingAlgorithm,

    /// Number of processors
    #[arg(short='n', value_name="# CPUs", default_value="1")]
    pub num_processors: u64,

    /// Execute specific schedulability test
    #[arg(long="test", value_name="TEST NAME")]
    pub specific_test: Option<String>,
}

impl SchedulingArgs {
    pub fn is_platform(&self,
        platform: PlatformModel,
        algorithm: SchedulingAlgorithm,
        num_processors: u64
    ) -> bool {
        self.platform == platform &&
        self.algorithm == algorithm &&
        self.num_processors == num_processors
    }
}

#[derive(Debug, Clone)]
#[derive(PartialEq, Eq)]
#[derive(clap::ValueEnum)]
pub enum PlatformModel {
    #[value(name = "fully-preemptive")]
    FullyPreemptive,
}

#[derive(Debug, Clone)]
#[derive(PartialEq, Eq)]
#[derive(clap::ValueEnum)]
#[derive(serde::Deserialize)]
pub enum SchedulingAlgorithm {
    #[value(name = "fixed-priority", alias("fp"))]
    FixedPriority,
    #[value(name = "earliest-deadline-first", alias("edf"))]
    EarliestDeadlineFirst,
}

#[derive(clap::Args, Debug)]
pub struct TasksetArgs {
    /// Taskset data file
    #[arg(short='i', value_name="TASKSET FILE")]
    pub taskset_file: String,

    /// Taskset file type
    #[arg(value_enum, short='f', long="format", value_name="FORMAT", default_value="auto")]
    pub taskset_file_ty: eva_rt_engine::prelude::TasksetFileType,
}