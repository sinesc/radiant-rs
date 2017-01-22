use prelude::*;
mod loops;
mod rng;

pub use misc::loops::{renderloop, mainloop};
pub use misc::rng::Rng;

/// Mutates source_value to approach target_value at the rate_of_change
pub fn approach<T, S>(source_value: &mut T, target_value: T, rate_of_change: S)
    where T: FromPrimitive + ToPrimitive + Clone, S: ToPrimitive
{
    let rate_of_change = rate_of_change.to_f64().unwrap();
    let new_source_value = (*source_value).to_f64().unwrap() * (1.0 - rate_of_change) + (target_value.to_f64().unwrap() * rate_of_change);
    *source_value = T::from_f64(new_source_value).unwrap();
}
