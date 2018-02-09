mod angle;
mod vec2;
mod vec3;
mod mat4;
pub mod matrix;

use prelude::*;
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
