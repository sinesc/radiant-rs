mod loops;
mod rng;
mod periodic;

use prelude::*;

pub use self::loops::{renderloop, mainloop};
pub use self::rng::Rng;
pub use self::periodic::Periodic;

/// Mutates source_value to approach target_value at the rate_of_change.
pub fn approach<T, S>(source_value: &mut T, target_value: T, rate_of_change: S)
    where T: Add + Mul<S> + From<<<T as Mul<S>>::Output as Add>::Output> + Copy, S: Float, <T as Mul<S>>::Output: Add
{
    *source_value = T::from( (*source_value) * (S::one() - rate_of_change) + (target_value * rate_of_change) );
}

/// Returns the smaller of the two given values, ignoring edge-cases like NaN.
pub fn min<T>(a: T, b: T) -> T where T: PartialOrd {
    if a.lt(&b) { a } else { b }
}

/// Returns the greater of the two given values, ignoring edge-cases like NaN.
pub fn max<T>(a: T, b: T) -> T where T: PartialOrd {
    if a.gt(&b) { a } else { b }
}
