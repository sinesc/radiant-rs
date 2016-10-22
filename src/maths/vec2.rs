use prelude::*;
use maths::{Vec3, VecType};
use glium::uniforms::{AsUniformValue, UniformValue};

const RAD_TO_DEG32: f32 = 180.0 / f32::consts::PI;
const RAD_TO_DEG64: f64 = 180.0 / f64::consts::PI;
const DEG_TO_RAD32: f32 = f32::consts::PI / 180.0;
const DEG_TO_RAD64: f64 = f64::consts::PI / 180.0;

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T: Copy + fmt::Display + Float>(pub T, pub T);

impl<T: Copy + fmt::Display + Float> Vec2<T> {
    pub fn new() -> Vec2<T> {
        Vec2::<T>(T::zero(), T::zero())
    }
    pub fn to_rad(self: &Self) -> T {
        self.1.atan2(self.0)
    }
    pub fn from_rad(radians: T) -> Vec2<T> {
        Vec2::<T>(radians.cos(), radians.sin())
    }
}

impl Vec2<f64> {
    pub fn to_deg(self: &Self) -> f64 {
        self.to_rad() * RAD_TO_DEG64
    }
    pub fn from_deg(degrees: f64) -> Vec2<f64> {
        let radians = degrees * DEG_TO_RAD64;
        Self::from_rad(radians)
    }
}

impl Vec2<f32> {
    pub fn to_deg(self: &Self) -> f32 {
        self.to_rad() * RAD_TO_DEG32
    }
    pub fn from_deg(degrees: f32) -> Vec2<f32> {
        let radians = degrees * DEG_TO_RAD32;
        Self::from_rad(radians)
    }
}

impl<T: Copy + fmt::Display + Float> VecType<T> for Vec2<T> {
    fn as_vec3(&self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}

impl<T: Copy + fmt::Display + Float> Neg for Vec2<T> {
    type Output = Vec2<T>;

    fn neg(self) -> Vec2<T> {
        Vec2::<T>(-self.0, -self.1)
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

#[doc(hidden)]
impl AsUniformValue for Vec2<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec2([ self.0, self.1 ])
    }
}

#[doc(hidden)]
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
