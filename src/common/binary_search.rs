pub mod prelude {
    pub use super::{
        binary_search_fn,
    };
}

/// perform a binary search for the least non-None function value on a bounded
/// range. The function, when succeeds, must be ascending monotonic.
pub fn binary_search_fn<R, F, T>(range: R, fun: F) -> Option<T>
    where R: std::ops::RangeBounds<usize>, F: Fn(usize) -> Option<T>,
{
    let mut left =
        match range.start_bound() {
            std::ops::Bound::Included(left) => *left,
            std::ops::Bound::Excluded(left) => *left + 1,
            std::ops::Bound::Unbounded => { return None; },
        };

    let mut right =
        match range.end_bound() {
            std::ops::Bound::Included(left) => *left,
            std::ops::Bound::Excluded(left) => *left + 1,
            std::ops::Bound::Unbounded => { return None; },
        };

    let mut best = None;
    while left < right {
        let mid = left + (right - left) / 2;
        best = fun(mid);

        if best.is_some() {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    best
}