use crate::prelude::*;

pub mod prelude {
    pub use super::{
        Error,
        AnalysisUtils,
    };
}

// Single Processor analyses
pub mod up_rate_monotonic;
pub mod earliest_deadline_first;

pub mod deadline_monotonic;

pub mod response_time_analysis;

// Multi Processor analyses
pub mod smp_edf;
pub mod smp_dm;

#[derive(Clone)]
#[derive(Debug)]
pub enum Error {
    Generic(String),
    NotOrderedByPeriod,
    NotOrderedByDeadline,
    NonImplicitDeadlines,
    NonConstrainedDeadlines,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(err) => write!(f, "Analysis error: {err}"),
            Error::NotOrderedByPeriod => write!(f, "Analysis error: taskset not ordered by period"),
            Error::NotOrderedByDeadline => write!(f, "Analysis error: taskset not ordered by deadline"),
            Error::NonImplicitDeadlines => write!(f, "Analysis error: taskset must have implicit deadlines"),
            Error::NonConstrainedDeadlines => write!(f, "Analysis error: taskset must have constrained deadlines"),
        }
    }
}

impl std::error::Error for Error {}

pub struct AnalysisUtils;

impl AnalysisUtils {
    pub fn assert_ordered_by_period(taskset: &[RTTask]) -> Result<(), Error> {
        if RTUtils::is_taskset_sorted_by_period(taskset) {
            Ok(())
        } else {
            Err(Error::NotOrderedByPeriod)
        }
    }

    pub fn assert_ordered_by_deadline(taskset: &[RTTask]) -> Result<(), Error> {
        if RTUtils::is_taskset_sorted_by_deadline(taskset) {
            Ok(())
        } else {
            Err(Error::NotOrderedByDeadline)
        }
    }

    pub fn assert_implicit_deadlines(taskset: &[RTTask]) -> Result<(), Error> {
        if RTUtils::implicit_deadlines(taskset) {
            Ok(())
        } else {
            Err(Error::NonImplicitDeadlines)
        }
    }

    pub fn assert_constrained_deadlines(taskset: &[RTTask]) -> Result<(), Error> {
        if RTUtils::constrained_deadlines(taskset) {
            Ok(())
        } else {
            Err(Error::NonConstrainedDeadlines)
        }
    }
}