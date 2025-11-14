use eva_engine::prelude::*;
use eva_engine::common::taskset_serde::prelude::*;

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
    let taskset = eva_engine::common::taskset_serde::plain_deserialize_taskset(
        &taskset_data,
        args.taskset_unit,
    ).unwrap();

    let min_period = args.min_period_us.unwrap_or(args.max_period_us);

    let model =
        (min_period ..= args.max_period_us)
        .step_by(args.period_step_us as usize)
        .map(|period_ns| {
            eva_engine::analyses::multiprocessor_periodic_resource_model::bcl_2009::
                generate_interface_global_fp(&taskset, Time::nanos(period_ns as f64), Time::nanos(args.step_size_ns as f64))
            .unwrap()
        })
        .min_by_key(|model| model.resource)
        .unwrap();

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