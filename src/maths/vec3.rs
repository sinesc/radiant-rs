use num::traits::Float;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Neg;
use maths::VecType;
use glium::uniforms::{AsUniformValue, UniformValue};

#[derive(Copy, Clone, Debug)]
pub struct Vec3<T: Copy + fmt::Display + Float>(pub T, pub T, pub T);

impl<T: Copy + fmt::Display + Float> Vec3<T> {
    pub fn new() -> Vec3<T> {
        Vec3::<T>(T::zero(), T::zero(), T::zero())
    }
}

impl<T: Copy + fmt::Display + Float> VecType<T> for Vec3<T> {
    fn as_vec3(&self, _: T) -> Vec3<T> {
        *self
    }
}

impl<T: Copy + fmt::Display + Float> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Vec3<T> {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}

impl<T: Copy + fmt::Display + Float> Add for Vec3<T> {
    type Output = Vec3<T>;
    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl<T: Copy + fmt::Display + Float> Mul<T> for Vec3<T> {
    type Output = Vec3<T>;
    fn mul(self, other: T) -> Vec3<T> {
        Vec3::<T>(self.0 * other, self.1 * other, self.2 * other)
    }
}

impl AsUniformValue for Vec3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3([ self.0, self.1, self.2 ])
    }
}

impl AsUniformValue for Vec3<f64> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec3([ self.0, self.1, self.2 ])
    }
}

impl<T: Copy + fmt::Display + Float> fmt::Display for Vec3<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3({}, {}, {})", self.0, self.1, self.2)
    }
}
