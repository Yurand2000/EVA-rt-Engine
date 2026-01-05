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