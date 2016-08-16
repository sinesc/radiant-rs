use num::traits::Float;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;
use glium::uniforms::{AsUniformValue, UniformValue};

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T: Copy + fmt::Display + Float>(pub T, pub T);

impl<T: Copy + fmt::Display + Float> Vec2<T> {
    pub fn new() -> Vec2<T> {
        Vec2::<T>(T::zero(), T::zero())
    }
}

impl<T: Copy + fmt::Display + Float> Add for Vec2<T> {
    type Output = Vec2<T>;
    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 + other.0, self.1 + other.1)
    }
}

impl<T: Copy + fmt::Display + Float> Mul<T> for Vec2<T> {
    type Output = Vec2<T>;
    fn mul(self, other: T) -> Vec2<T> {
        Vec2::<T>(self.0 * other, self.1 * other)
    }
}

impl AsUniformValue for Vec2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2([ self.0, self.1 ])
    }
}

impl AsUniformValue for Vec2<f64> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec2([ self.0, self.1 ])
    }
}

impl<T: Copy + fmt::Display + Float> fmt::Display for Vec2<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec2({}, {})", self.0, self.1)
    }
}
