use eva_rt_engine::prelude::*;

pub mod taskset_serde;

pub use taskset_serde::*;

pub fn run_analysis<A, T, Taskset>(analysis: A, taskset: Taskset) -> anyhow::Result<()>
    where
        A: SchedAnalysis<T, Taskset>
{
    use SchedError as Err;

    print!("Running \"{}\":\n\t", analysis.analyzer_name());

    match analysis.is_schedulable(taskset) {
        Ok(_) => {
            println!("schedulable");
            Ok(())
        },
        Err(err) => {
            let Some(ref_err) = err.downcast_ref::<Err>()
                else { panic!("unexpected"); };

            match ref_err {
                Err::Other(_) => Err(err),
                _ => {
                    println!("{ref_err}");
                    Ok(())
                }
            }
        },
    }
}