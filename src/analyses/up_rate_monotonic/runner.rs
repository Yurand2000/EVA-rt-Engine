use crate::prelude::*;

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// Analysis to run
    /// 
    /// classic: Liu & Layland 1973
    #[arg(value_enum, value_name= "type", default_value="classic", verbatim_doc_comment)]
    typ: Type,
}

#[derive(clap::ValueEnum, Debug)]
#[derive(Clone)]
pub enum Type {
    Classic,
    Simple,
    Hyperbolic,
}

pub fn main(taskset: &[RTTask], args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(match args.typ {
        Type::Classic => super::is_schedulable(taskset)?,
        Type::Simple => super::is_schedulable_simple(taskset)?,
        Type::Hyperbolic => super::is_schedulable_hyperbolic(taskset)?,
    })
}