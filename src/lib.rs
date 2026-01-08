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
    pub use super::utils::{
        binary_search::*,
        fixpoint_search::*,
        sched_result::*,
        design_result::*,
        time_iterators::*,
    };
}

pub mod algorithms;

/// Utility Functions
pub mod utils {
    pub mod binary_search;
    pub mod fixpoint_search;
    pub mod sched_result;
    pub mod design_result;
    pub mod time_iterators;
}
