use crate::prelude::*;

pub mod prelude {
    pub use super::{
        Error,
        AnalysisUtils,
    };
}

// Single Processor analyses
pub mod rate_monotonic;
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