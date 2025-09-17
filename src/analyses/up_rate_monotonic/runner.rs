use crate::prelude::*;

#[derive(clap::Parser, Debug)]
#[derive(serde::Deserialize)]
pub struct Args {
    /// Analysis to run
    /// 
    /// classic:    Liu & Layland 1973
    /// simple:     Liu & Layland 1973
    ///             upper bound for any number of tasks
    /// hyperbolic: Bini, Buttazzo and Buttazzo 2001
    #[arg(value_enum, value_name= "type", default_value="classic", verbatim_doc_comment)]
    #[serde(default)]
    typ: Type,
}

#[derive(Clone)]
#[derive(clap::ValueEnum, Debug)]
#[derive(serde::Deserialize)]
pub enum Type {
    #[serde(rename = "classic")]
    Classic,
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "hyperbolic")]
    Hyperbolic,
}

pub fn main(taskset: &[RTTask], args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(match args.typ {
        Type::Classic => super::is_schedulable(taskset)?,
        Type::Simple => super::is_schedulable_simple(taskset)?,
        Type::Hyperbolic => super::is_schedulable_hyperbolic(taskset)?,
    })
}

impl Default for Type {
    fn default() -> Self {
        Self::Classic
    }
}