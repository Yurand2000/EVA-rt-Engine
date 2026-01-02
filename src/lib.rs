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
        binary_search_fn,
        time_range_iterator_w_step,
        time_range_iterator,
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
    mut fun: F
) -> T
    where
        T: PartialOrd + PartialEq,
        F: FnMut(&T) -> T,
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

pub fn binary_search_fn<T, FVal, FCmp>(
    (mut left, mut right): (usize, usize),
    mut fun: FVal,
    mut cmp: FCmp
) -> T
    where
        FVal: FnMut(usize) -> T,
        FCmp: FnMut(&T) -> std::cmp::Ordering,
{
    use std::cmp::Ordering::*;

    assert!(left <= right);

    loop {
        let mid = left + (right - left) / 2;
        let mid_value = fun(mid);

        match cmp(&mid_value) {
            Less | Equal => { left = mid + 1; },
            Greater => { right = mid; },
        }

        if left >= right {
            return mid_value;
        }
    }
}

use crate::prelude::*;

pub fn time_range_iterator(start: Time, end: Time) -> impl Iterator<Item = Time> {
    (start.value_ns as usize ..= end.value_ns as usize)
        .map(|time_ns| Time { value_ns: time_ns as f64 })
}

pub fn time_range_iterator_w_step(start: Time, end: Time, step: Time) -> impl Iterator<Item = Time> {
    (start.value_ns as usize ..= end.value_ns as usize)
        .step_by(step.value_ns as usize)
        .map(|time_ns| Time { value_ns: time_ns as f64 })
}