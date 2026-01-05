/// Perform binary search on the given range. It requires a value function
/// and a unary comparison function that tells if the generated value compares
/// to the serach's output.
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
            Less => { left = mid + 1; },
            Equal => { return mid_value; },
            Greater => { right = mid; },
        }

        if left >= right {
            return mid_value;
        }
    }
}
/// Perform exponential search starting at the given location. It requires a
/// value function and a unary comparison function that tells if the generated
/// value compares to the serach's output.
pub fn exponential_search_fn<T, FVal, FCmp> (
    mut left: usize,
    mut fun: FVal,
    mut cmp: FCmp
) -> T
    where
        FVal: FnMut(usize) -> T,
        FCmp: FnMut(&T) -> std::cmp::Ordering,
{
    use std::cmp::Ordering::*;

    let left_value = fun(left);
    if let std::cmp::Ordering::Equal = cmp(&left_value) {
        left_value
    } else {
        let mut right = left + 1;

        loop {
            let right_value = fun(right);

            match cmp(&right_value) {
                Less =>
                    { return binary_search_fn((left, right), fun, cmp); },
                Equal =>
                    { return right_value; },
                Greater => {
                    left = right;
                    right *= 2;
                },
            }
        }
    }
}