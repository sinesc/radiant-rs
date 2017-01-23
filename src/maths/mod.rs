mod mat4;
mod vec2;
mod vec3;
mod angle;
use prelude::*;

pub use maths::mat4::Mat4;
pub use maths::vec2::Vec2;
pub use maths::vec3::Vec3;
pub use maths::angle::Angle;

/// A 2-dimensional point.
pub type Point2<T = f32> = Vec2<T>;

// required due to #26953
#[allow(non_snake_case)]
pub fn Point2<T: Debug + Float>(x: T, y: T) -> Point2<T> {
    Vec2(x, y)
}

/// A 3-dimensional point.
pub type Point3<T = f32> = Vec3<T>;

// required due to #26953
#[allow(non_snake_case)]
pub fn Point3<T: Debug + Float>(x: T, y: T, z: T) -> Point3<T> {
    Vec3(x, y, z)
}

#[derive(Copy, Clone)]
pub struct Rect<T: Debug + Float = f32>(pub Point2<T>, pub Point2<T>);

impl<T> Rect<T> where T: Debug + Float {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Rect(Point2(x1, y1), Point2(x2, y2))
    }
}

/// Trait for values that can be converted to a vector.
pub trait VecType<T: Copy + fmt::Debug + Float> {
    /// Returns the given value as a Vec3
    fn as_vec3(self: &Self, neutral: T) -> Vec3<T>;
}

impl<T: Copy + fmt::Debug + Float> VecType<T> for (T, T, T) {
    fn as_vec3(self: &Self, _: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, self.2)
    }
}

impl<T: Copy + fmt::Debug + Float> VecType<T> for (T, T) {
    fn as_vec3(self: &Self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}

impl<T: Copy + fmt::Debug + Float> VecType<T> for T {
    fn as_vec3(self: &Self, _: T) -> Vec3<T> {
        Vec3::<T>(*self, *self, *self)
    }
}
