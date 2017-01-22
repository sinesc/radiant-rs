mod mat4;
mod vec2;
mod vec3;
mod angle;

pub use maths::mat4::Mat4;
pub use maths::vec2::Vec2;
pub use maths::vec3::Vec3;
pub use maths::angle::Angle;

use prelude::*;

pub trait VecType<T: Copy + fmt::Debug + Float> {
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
