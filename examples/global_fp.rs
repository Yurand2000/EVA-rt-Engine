mod utils;

use utils::*;
use eva_rt_engine::algorithms::full_preemption::global_multiprocessor::fixed_priority::*;

#[derive(clap::Parser, Debug,  Clone)]
pub struct Args {
    pub input_file: String,

    #[arg(short='c')]
    pub cpus: u64,
}

fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let taskset = parse_taskset(&args.input_file, TasksetPlainUnit::Millis)?;

    run_analysis(deadline_monotonic_bcl05::Analysis { num_processors: args.cpus }, &taskset)?;
    run_analysis(bcl09::Analysis { num_processors: args.cpus }, &taskset)?;
    run_analysis(rta_lc09::Analysis { num_processors: args.cpus }, &taskset)?;

    Ok(())
}
