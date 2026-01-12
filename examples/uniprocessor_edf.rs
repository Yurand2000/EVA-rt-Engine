mod utils;

use utils::*;
use eva_rt_engine::algorithms::full_preemption::uniprocessor::earliest_deadline_first::*;

#[derive(clap::Parser, Debug,  Clone)]
pub struct Args {
    pub input_file: String,
}

fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let taskset = parse_taskset(&args.input_file, TasksetPlainUnit::Millis)?;

    run_analysis(edf73::Analysis, &taskset)?;

    Ok(())
}
