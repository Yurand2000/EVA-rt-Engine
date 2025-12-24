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
        SchedResult
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
    pub fn non_schedulable<T>(err: anyhow::Error) -> SchedResult<T> {
        Err(SchedError::NonSchedulable(err))
    }

    pub fn precondition<T>(err: anyhow::Error) -> SchedResult<T> {
        Err(SchedError::Precondition(err))
    }

    pub fn other<T>(err: anyhow::Error) -> SchedResult<T> {
        Err(SchedError::Other(err))
    }

    pub fn is_non_scheduable(&self) -> bool {
        match self {
            Self::NonSchedulable(_) => true,
            _ => false,
        }
    }

    pub fn precondition_error(&self) -> bool {
        match self {
            Self::Precondition(_) => true,
            _ => false,
        }
    }

    pub fn other_error(&self) -> bool {
        match self {
            Self::Other(_) => true,
            _ => false,
        }
    }
}