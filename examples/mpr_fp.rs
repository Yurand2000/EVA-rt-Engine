mod utils;

use eva_rt_common::time::Time;
use utils::*;
use eva_rt_engine::{algorithms::full_preemption::global_multiprocessor::
    hierarchical::mpr_model09::fixed_priority::bcl09::*, prelude::SchedDesign as _};

#[derive(clap::Parser, Debug,  Clone)]
pub struct Args {
    /// Taskset file
    pub input_file: String,

    /// Minimum MPR Period, milliseconds
    #[arg(short='m')]
    pub min_period_ms: u64,

    /// Maximum MPR Period, milliseconds
    #[arg(short='M')]
    pub max_period_ms: u64,

    /// Period Search Step, milliseconds
    #[arg(long="period-step", default_value="1")]
    pub period_step_ms: u64,

    /// Resource Search Step, nanoseconds
    #[arg(long="resource-step", default_value="100")]
    pub resource_step_ns: u64,
}

fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let taskset = parse_taskset(&args.input_file, TasksetPlainUnit::Millis)?;

    let designer =
        extra::DesignerFull {
            period_range: (
                Time::millis(args.min_period_ms as f64),
                Time::millis(args.max_period_ms as f64),
                Time::millis(args.period_step_ms as f64),
            ),
            resource_step: Time::nanos(args.resource_step_ns as f64),
        };

    let best_model =
        designer.design(&taskset)?;

    println!("{} {:.0} {:.0}",
        best_model.concurrency,
        (best_model.resource / best_model.concurrency as f64).as_millis(),
        best_model.period.as_millis(),
    );

    Ok(())
}