mod utils;

use utils::*;
use eva_rt_engine::algorithms::full_preemption::uniprocessor::fixed_priority::*;

#[derive(clap::Parser, Debug,  Clone)]
pub struct Args {
    pub input_file: String,
}

fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();

    let taskset = parse_taskset(&args.input_file, TasksetPlainUnit::Millis)?;

    run_analysis(rate_monotonic73::Analysis, &taskset)?;
    run_analysis(rate_monotonic73::AnalysisSimple, &taskset)?;
    run_analysis(hyperbolic01::Analysis, &taskset)?;
    run_analysis(deadline_monotonic90::Analysis, &taskset)?;
    run_analysis(rta86::Analysis, &taskset)?;

    Ok(())
}
