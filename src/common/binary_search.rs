pub mod prelude {
    pub use super::{
        binary_search_fn,
        bin_search,
        exp_search,
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

pub fn bin_search<R, F, T>(range: R, fun: F, value: T) -> Option<usize>
    where R: std::ops::RangeBounds<usize>, F: Fn(usize) -> T, T: PartialEq + Eq + PartialOrd + Ord
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

    while left < right {
        let mid = left + (right - left) / 2;
        let val_m = fun(mid);

        match val_m.cmp(&value) {
            std::cmp::Ordering::Less => {
                left = mid + 1;
            },
            std::cmp::Ordering::Equal => { return Some(mid); },
            std::cmp::Ordering::Greater => {
                right = mid;
            },
        }
    }

    let left_v = fun(left);

    if left_v >= value {
        Some(left)
    } else {
        None
    }
}

// perform an exponential search to find, given an output o, the smallest input
// i such that fun(i) >= o.
pub fn exp_search<R, F, T>(range: R, fun: F, value: T) -> Option<usize>
    where R: std::ops::RangeBounds<usize>, F: Fn(usize) -> T, T: PartialEq + Eq + PartialOrd + Ord
{
    let std::ops::Bound::Unbounded = range.end_bound() else { return None; };
    let mut left =
        match range.start_bound() {
            std::ops::Bound::Included(left) => *left,
            std::ops::Bound::Excluded(left) => *left + 1,
            std::ops::Bound::Unbounded => { return None; },
        };

    if fun(left) == value {
        Some(left)
    } else {
        let mut right = left + 1;

        loop {
            let val_r = fun(right);

            match val_r.cmp(&value) {
                std::cmp::Ordering::Less => {
                    left = right;
                    right *= 2;
                },
                std::cmp::Ordering::Equal =>
                    { return Some(right); },
                std::cmp::Ordering::Greater =>
                    { return bin_search(left ..= right, fun, value); },
            }
        }
    }
}