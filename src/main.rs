#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    analysis: Analyses,

    /// taskset data, either .json or plain .txt file
    #[arg(short='i', value_name="filename")]
    taskset_file: String,
}

#[derive(clap::Subcommand, Debug)]
enum Analyses {
    /// UP Rate monotonic
    /// 
    /// UniProcessor Rate Monotonic test.
    /// Preconditions:
    /// - Taskset sorted by period.
    /// - Implicit deadlines.
    #[command(name = "rate-monotonic", verbatim_doc_comment)]
    RateMonotonic(analyzer::analyses::up_rate_monotonic::runner::Args),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();

    println!("Hello, world!");

    Ok(())
}
