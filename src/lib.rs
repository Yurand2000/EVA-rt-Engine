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
    pub use super::utils::{
        sched_result::*,
        design_result::*,
    };
    pub use super::{
        fixpoint_search_with_limit,
    };
}

pub mod analysis;
pub mod algorithms;
pub mod common;

pub mod utils {
    pub mod sched_result;
    pub mod design_result;
}

/// Apply the given function recursively until a fix point or an upper limit is
/// reached. Convergence is guaranteed if the provided function is monotone.
pub fn fixpoint_search_with_limit<T, F>(
    init: T,
    limit: T,
    fun: F
) -> T
    where
        T: PartialOrd + PartialEq,
        F: Fn(&T) -> T,
{
    let mut value = init;

    loop {
        let new_value = fun(&value);

        if new_value > limit {
            return limit;
        } else if new_value == value {
            return new_value;
        }

        value = new_value;
    }
}