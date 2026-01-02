use eva_rt_engine::prelude::*;
use eva_rt_engine::common::taskset_serde::prelude::*;

#[derive(clap::Parser)]
pub struct Args {
    /// Taskset data file
    #[arg(short='i', value_name="TASKSET FILE")]
    pub taskset_file: String,

    #[arg(short='u', value_name="TASKSET UNIT", default_value="millis")]
    pub taskset_unit: TasksetPlainUnit,

    /// MPR Period Min (optional)
    #[arg(short='t', value_name="Min PERIOD us")]
    pub min_period_us: Option<u64>,

    /// MPR Period (Max)
    ///
    /// If minimum period is unspecified, it equals to the max period
    #[arg(short='T', value_name="(Max) PERIOD us")]
    pub max_period_us: u64,

    /// MPR Period step
    #[arg(long="tstep", value_name="PERIOD STEP us", default_value= "100")]
    pub period_step_us: u64,

    /// Interface search step
    #[arg(short='S', default_value="100", value_name="STEP SIZE ns")]
    pub step_size_ns: u64
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let taskset_data = std::fs::read_to_string(&args.taskset_file).unwrap();
    let taskset = eva_rt_engine::common::taskset_serde::plain_deserialize_taskset(
        &taskset_data,
        args.taskset_unit,
    ).unwrap();

    let min_period = Time::micros(args.min_period_us.unwrap_or(args.max_period_us) as f64);
    let max_period = Time::micros(args.max_period_us as f64);
    let period_step = Time::micros(args.period_step_us as f64);

    let resource_step = Time::nanos(args.step_size_ns as f64);

    let model =
        eva_rt_engine::algorithms::full_preemption::global_multiprocessor::hierarchical::
        mpr_model09::fixed_priority::bcl09::extra::generate_best_model(
            &taskset,
            (min_period, max_period, period_step),
            resource_step
        );

    let model = model.result.unwrap();

    let scale =
        match args.taskset_unit {
            TasksetPlainUnit::Millis => Time::MILLI_TO_NANO,
            TasksetPlainUnit::Micros => Time::MICRO_TO_NANO,
            TasksetPlainUnit::Nanos => 1.0,
        };

    println!("{} {:.0} {:.0}",
        model.concurrency,
        (model.resource / (model.concurrency as f64 * scale)).ceil().as_nanos(),
        (model.period / scale).ceil().as_nanos()
    );
}