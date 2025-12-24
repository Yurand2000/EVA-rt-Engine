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
    pub use super::{
        SchedError,
        SchedResult,
        SchedErrors,
    };
}

pub mod analysis;
pub mod algorithms;
pub mod common;

#[derive(Debug)]
pub enum SchedError {
    NonSchedulable(anyhow::Error),
    Precondition(anyhow::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonSchedulable(error) =>
                write!(f, "Taskset non schedulable: {error}"),
            Self::Precondition(error) =>
                write!(f, "Sched Analysis Precondition error: {error}"),
            Self::Other(error) =>
                write!(f, "Sched Analysis error: {error}"),
        }
    }
}

impl std::error::Error for SchedError {}

pub type SchedResult<T> = Result<T, SchedError>;

impl SchedError {
    pub fn is_non_scheduable(&self) -> bool {
        match self {
            Self::NonSchedulable(_) => true,
            _ => false,
        }
    }

    pub fn is_precondition_error(&self) -> bool {
        match self {
            Self::Precondition(_) => true,
            _ => false,
        }
    }

    pub fn is_other_error(&self) -> bool {
        match self {
            Self::Other(_) => true,
            _ => false,
        }
    }
}

pub struct SchedErrors<'a>(&'a str);

impl<'a> SchedErrors<'a> {
    pub fn non_schedulable<T>(self) -> SchedResult<T> {
        Err(SchedError::NonSchedulable(anyhow::format_err!("{}, non schedulable.", self.0)))
    }

    pub fn non_schedulable_reason<T>(self, reason: std::fmt::Arguments) -> SchedResult<T> {
        Err(SchedError::NonSchedulable(anyhow::format_err!("{}, non schedulable; reason: {}", self.0, reason)))
    }

    pub fn precondition<T>(self) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!("{}, precondition unsatisfied.", self.0)))
    }

    pub fn precondition_reason<T>(self, reason: std::fmt::Arguments) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!("{}, precondition unsatisfied; reason: {}", self.0, reason)))
    }

    pub fn implicit_deadlines<T>(self) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!(
            "{}, taskset must have implicit deadlines.", self.0)))
    }

    pub fn constrained_deadlines<T>(self) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!(
            "{}, taskset must have constrained deadlines.", self.0)))
    }

    pub fn rate_monotonic<T>(self) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!(
            "{}, taskset must be sorted by period.", self.0)))
    }

    pub fn deadline_monotonic<T>(self) -> SchedResult<T> {
        Err(SchedError::Precondition(anyhow::format_err!(
            "{}, taskset must be sorted by deadline.", self.0)))
    }
}