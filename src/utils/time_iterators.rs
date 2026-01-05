//! Custom Iterators for Time ranges.

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