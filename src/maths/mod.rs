mod mat4;
mod vec2;
mod vec3;
mod dir1;

pub use maths::mat4::Mat4;
pub use maths::vec2::Vec2;
pub use maths::vec3::Vec3;
pub use maths::dir1::Dir1;

use prelude::*;

pub trait VecType<T: Copy + fmt::Display + Float> {
    fn as_vec3(&self, neutral: T) -> Vec3<T>;
}

impl<T: Copy + fmt::Display + Float> VecType<T> for (T, T, T) {
    fn as_vec3(&self, _: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, self.2)
    }
}

impl<T: Copy + fmt::Display + Float> VecType<T> for (T, T) {
    fn as_vec3(&self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}
