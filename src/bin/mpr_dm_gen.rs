use eva_engine::prelude::*;

#[derive(clap::Parser)]
pub struct Args {
    /// Taskset data file
    #[arg(short='i', value_name="TASKSET FILE")]
    pub taskset_file: String,

    /// MPR Period
    #[arg(short='T', value_name="PERIOD ms")]
    pub period_ms: u64,
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let taskset = parse_taskset(
        &args.taskset_file,
        TasksetFileType::Plain
    ).unwrap();

    let model =
        eva_engine::analyses::multiprocessor_periodic_resource_model::bcl_2009::
            // generate_interface_global_dm_simple(&taskset, Time::millis(args.period_ms as f64))
            generate_interface_global_dm2_simple(&taskset, Time::millis(args.period_ms as f64))
        .unwrap();

    println!("{} {:.0} {:.0}",
        model.concurrency,
        (model.resource / model.concurrency as f64).as_millis().ceil(),
        model.period.as_millis().ceil()
    );
}