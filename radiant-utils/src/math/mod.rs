mod angle;
mod vec2;
mod vec3;
mod mat4;
pub mod matrix;

use crate::prelude::*;
pub use self::angle::Angle;
pub use self::vec2::Vec2;
pub use self::vec3::Vec3;
pub use self::mat4::Mat4;
pub use self::matrix::Matrix;

pub type Point2<T = f32> = (T, T);
pub type Rect<T = f32> = (Point2<T>, Point2<T>);

/// Values that can be converted to a vector.
pub trait Vector<T> where T: Copy {
    /// Returns the given value as a Vec3
    fn as_vec3(self: &Self, neutral: T) -> Vec3<T>;
}

impl<T> Vector<T> for (T, T, T) where T: Copy {
    fn as_vec3(self: &Self, _: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, self.2)
    }
}

impl<T> Vector<T> for (T, T) where T: Copy {
    fn as_vec3(self: &Self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}

impl<T> Vector<T> for T where T: Copy {
    fn as_vec3(self: &Self, _: T) -> Vec3<T> {
        Vec3::<T>(*self, *self, *self)
    }
}

/// Interpolates between values. Returns source_value for ratio = 0.0 and target_value for ratio = 1.0.
pub fn lerp<T, S>(source_value: &T, target_value: &T, ratio: S) -> T
    where T: Add + Mul<S> + From<<<T as Mul<S>>::Output as Add>::Output> + Copy, S: Float, <T as Mul<S>>::Output: Add
{
    T::from( (*source_value) * (S::one() - ratio) + (*target_value * ratio) )
}

/// Mutates source_value to approach target_value at the rate_of_change. Effectively a lerp that writes to source.
pub fn approach<T, S>(source_value: &mut T, target_value: &T, rate_of_change: S)
    where T: Add + Mul<S> + From<<<T as Mul<S>>::Output as Add>::Output> + Copy, S: Float, <T as Mul<S>>::Output: Add
{
    *source_value = T::from( (*source_value) * (S::one() - rate_of_change) + (*target_value * rate_of_change) );
}

/// Returns the smaller of the two given values.
pub fn min<T>(a: T, b: T) -> T where T: PartialOrd {
    if a.lt(&b) { a } else { b }
}

/// Returns the greater of the two given values.
pub fn max<T>(a: T, b: T) -> T where T: PartialOrd {
    if a.gt(&b) { a } else { b }
}

/// Returns the given value limited to the bounds min and max.
pub fn clamp<T>(v: T, min: T, max: T) -> T where T: PartialOrd {
    if v.lt(&min) { min } else if v.gt(&max) { max } else { v }
}
