//! Root Module containing all the implemented analyses

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        Error,
        AnalysisUtils,
    };
}

pub mod full_preemption {
    pub mod uniprocessor {
        pub mod earliest_deadline_first {
            pub mod edf73;
        }

        pub mod fixed_priority {
            pub mod rate_monotonic73;
            pub mod deadline_monotonic90;
            pub mod rta86;
            pub mod hyperbolic01;
        }

        pub mod hierarchical {

        }
    }

    pub mod global_multiprocessor {
        pub mod earliest_deadline_first {
            pub mod baruah07;
        }

        pub mod fixed_priority {
            pub mod deadline_monotonic_bcl05;
            pub mod rta_lc09;
        }

        pub mod generic_work_conserving {

        }

        pub mod hierarchical {

        }
    }
}

// Multi Processor Global scheduling
pub mod smp_earliest_deadline_first;
pub mod smp_generic;

// Multi Processor Hierarchical scheduling
pub mod multiprocessor_periodic_resource_model;

#[derive(Debug, Clone)]
pub enum Error {
    Generic(String),
    Precondition(String),
    NotOrderedByPeriod,
    NotOrderedByDeadline,
    NonImplicitDeadlines,
    NonConstrainedDeadlines,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(err) =>
                write!(f, "Analysis error: {err}"),
            Error::Precondition(err) =>
                write!(f, "Analysis precondition error: {err}"),
            Error::NotOrderedByPeriod =>
                write!(f, "Analysis precondition error: taskset not ordered by period"),
            Error::NotOrderedByDeadline =>
                write!(f, "Analysis precondition error: taskset not ordered by deadline"),
            Error::NonImplicitDeadlines =>
                write!(f, "Analysis precondition error: taskset must have implicit deadlines"),
            Error::NonConstrainedDeadlines =>
                write!(f, "Analysis precondition error: taskset must have constrained deadlines"),
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

    pub fn assert_integer_times(taskset: &[RTTask]) -> Result<(), Error> {
        if taskset.iter().all(|task| {
            task.wcet.as_nanos().fract() < 1e-10 &&
            task.deadline.as_nanos().fract() < 1e-10 &&
            task.period.as_nanos().fract() < 1e-10
        }) {
            Ok(())
        } else {
            Err(Error::Precondition(format!("Required integer times for task's parameters")))
        }
    }
}
