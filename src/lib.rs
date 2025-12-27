//! # eva-rt-engine
//!
//! The **Evaluation**, **Verification** and **Analysis Engine** for **Real-Time** applications
//! schedulability (short as *EVA-rt-Engine* or simply *EVA*) is a software created to perform
//! real-time schedulability analyses.
//!
//! **EVA** implements a variety of *state-of-the-art* tests to assert wheter a given taskset is
//! schedulable on a given platform. Additionally, it also implements designers that search for the
//! minimum required resources to schedule the given task on the given platform and scheduling
//! approach.
//!
//! **EVA** is distributed under the *GPL3* license, both as a standalone tool and as a Rust library
//! that can be easily integrated in other Rust-based projects.

/// Prelude module with commonly used exports.
pub mod prelude {
    pub use eva_rt_common::prelude::*;
    pub use eva_rt_common::utils::prelude::*;
    pub use super::algorithms::prelude::*;
    pub use super::analysis::prelude::*;
    pub use super::{
        SchedError,
        SchedErrorType,
        SchedResult,
        SchedErrors,
    };
}

pub mod analysis;
pub mod algorithms;
pub mod common;

/// Error for schedulability test results.
#[derive(Debug)]
pub struct SchedError {
    test_name: String,
    error_typ: SchedErrorType,
}

/// Error type for schedulability test results.
///
/// The error is [`SchedErrorType::NonSchedulable`] when the taskset is not
/// schedulable, [`SchedErrorType::Precondition`] when a schedulability test's
/// precondition is not met, or [`SchedErrorType::Other`] for other errors.
#[derive(Debug)]
pub enum SchedErrorType {
    NonSchedulable(Option<anyhow::Error>),
    Precondition(Option<anyhow::Error>),
    Other(anyhow::Error),
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SchedErrorType::*;

        match &self.error_typ {
            NonSchedulable(None) =>
                write!(f, "Sched Analysis \"{}\" error: non schedulable.", self.test_name),
            NonSchedulable(Some(error)) =>
                write!(f, "Sched Analysis \"{}\" error: non schedulable, reason: {}", self.test_name, error),
            Precondition(None) =>
                write!(f, "Sched Analysis \"{}\" precondition error.", self.test_name),
            Precondition(Some(error)) =>
                write!(f, "Sched Analysis \"{}\" precondition error, reason: {}", self.test_name, error),
            Other(error) =>
                write!(f, "Sched Analysis \"{}\" error: {}", self.test_name, error),
        }
    }
}

impl std::error::Error for SchedError {}

/// Type alias for a schedulability test results.
///
/// Is `Ok( T )` when the taskset is schedulable. \
/// Is `Err(`[`SchedError`]`)` when the taskset is not schedulable.
pub type SchedResult<T> = Result<T, SchedError>;

impl SchedError {
    pub fn non_schedulable(test_name: &str) -> Self {
        Self { test_name: test_name.to_owned(), error_typ: SchedErrorType::NonSchedulable(None) }
    }

    pub fn non_schedulable_reason(test_name: &str, error: anyhow::Error) -> Self {
        Self { test_name: test_name.to_owned(), error_typ: SchedErrorType::NonSchedulable(Some(error)) }
    }

    pub fn precondition(test_name: &str) -> Self {
        Self { test_name: test_name.to_owned(), error_typ: SchedErrorType::Precondition(None) }
    }

    pub fn precondition_reason(test_name: &str, error: anyhow::Error) -> Self {
        Self { test_name: test_name.to_owned(), error_typ: SchedErrorType::Precondition(Some(error)) }
    }

    pub fn other(test_name: &str, error: anyhow::Error) -> Self {
        Self { test_name: test_name.to_owned(), error_typ: SchedErrorType::Other(error) }
    }

    pub fn is_non_scheduable(&self) -> bool {
        match self.error_typ {
            SchedErrorType::NonSchedulable(_) => true,
            _ => false,
        }
    }

    pub fn is_precondition_error(&self) -> bool {
        match self.error_typ {
            SchedErrorType::Precondition(_) => true,
            _ => false,
        }
    }

    pub fn is_other_error(&self) -> bool {
        match self.error_typ {
            SchedErrorType::Other(_) => true,
            _ => false,
        }
    }
}

/// Helper factory for common schedulability test errors.
///
/// Takes the test's name as first parameter.
pub struct SchedErrors<'a>(&'a str);

impl<'a> SchedErrors<'a> {
    pub fn non_schedulable<T>(self) -> SchedResult<T> {
        Err(SchedError::non_schedulable(self.0))
    }

    pub fn non_schedulable_reason<T>(self, reason: anyhow::Error) -> SchedResult<T> {
        Err(SchedError::non_schedulable_reason(self.0, reason))
    }

    pub fn precondition<T>(self) -> SchedResult<T> {
        Err(SchedError::precondition(self.0))
    }

    pub fn precondition_reason<T>(self, reason: anyhow::Error) -> SchedResult<T> {
        Err(SchedError::precondition_reason(self.0, reason))
    }

    pub fn implicit_deadlines<T>(self) -> SchedResult<T> {
        Err(SchedError::precondition_reason(self.0,
            anyhow::format_err!("taskset must have implicit deadlines.")))
    }

    pub fn constrained_deadlines<T>(self) -> SchedResult<T> {
        Err(SchedError::precondition_reason(self.0,
            anyhow::format_err!("taskset must have constrained deadlines.")))
    }

    pub fn rate_monotonic<T>(self) -> SchedResult<T> {
        Err(SchedError::precondition_reason(self.0,
            anyhow::format_err!("taskset must be sorted by period.")))
    }

    pub fn deadline_monotonic<T>(self) -> SchedResult<T> {
        Err(SchedError::precondition_reason(self.0,
            anyhow::format_err!("taskset must be sorted by deadline.")))
    }
}