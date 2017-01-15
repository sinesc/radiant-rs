use prelude::*;
use maths::{Vec3, VecType};
use glium::uniforms::{AsUniformValue, UniformValue};

/// A 2-dimensional vector.
#[derive(Copy, Clone)]
pub struct Vec2<T: Copy + fmt::Debug + Float = f32>(pub T, pub T);

impl<T> Vec2<T> where T: Copy + fmt::Debug + Float {
    /// Creates a new instances.
    pub fn new() -> Vec2<T> {
        Vec2::<T>(T::zero(), T::zero())
    }
    /// Returns the length of the vector
    pub fn len(self: &Self) -> T {
        (self.0*self.0 + self.1*self.1).sqrt()
    }
    /// Returns the direction of the vector in radians.
    pub fn to_rad(self: &Self) -> T {
        self.1.atan2(self.0)
    }
    /// Returns the direction of the vector in degrees.
    pub fn to_deg(self: &Self) -> T {
        self.to_rad().to_degrees()
    }
    /// Creates a unit-vector from the angle given in radians.
    pub fn from_rad(radians: T) -> Vec2<T> {
        Vec2::<T>(radians.cos(), radians.sin())
    }
    /// Creates a unit-vector from the angle given in degrees.
    pub fn from_deg(degrees: T) -> Vec2<T> {
        Self::from_rad(degrees.to_radians())
    }
}

impl<T> VecType<T> for Vec2<T> where T: Copy + fmt::Debug + Float{
    fn as_vec3(&self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}

impl<T> Neg for Vec2<T> where T: Copy + fmt::Debug + Float{
    type Output = Vec2<T>;

    fn neg(self) -> Vec2<T> {
        Vec2::<T>(-self.0, -self.1)
    }
}

impl<T> Add for Vec2<T> where T: Copy + fmt::Debug + Float{
    type Output = Vec2<T>;
    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 + other.0, self.1 + other.1)
    }
}

impl<T> Sub for Vec2<T> where T: Copy + fmt::Debug + Float{
    type Output = Vec2<T>;
    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 - other.0, self.1 - other.1)
    }
}

impl<T> AddAssign for Vec2<T> where T: Copy + fmt::Debug + Float {
    fn add_assign(self: &mut Self, other: Vec2<T>) {
        *self = Vec2::<T> (
            self.0 + other.0,
            self.1 + other.1
        )
    }
}

impl<T> SubAssign for Vec2<T> where T: Copy + fmt::Debug + Float{
    fn sub_assign(self: &mut Self, other: Vec2<T>) {
        *self = Vec2::<T> (
            self.0 - other.0,
            self.1 - other.1
        )
    }
}

impl<T> Mul<T> for Vec2<T> where T: Copy + fmt::Debug + Float {
    type Output = Vec2<T>;
    fn mul(self, other: T) -> Vec2<T> {
        Vec2::<T>(self.0 * other, self.1 * other)
    }
}

impl<T> Mul<Vec2<T>> for Vec2<T> where T: Copy + fmt::Debug + Float{
    type Output = Vec2<T>;
    fn mul(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 * other.0, self.1 * other.1)
    }
}

impl Mul<Vec2<f32>> for f32 {
    type Output = Vec2<f32>;
    fn mul(self, other: Vec2<f32>) -> Vec2<f32> {
        Vec2::<f32>(self * other.0, self * other.1)
    }
}

impl Mul<Vec2<f64>> for f64 {
    type Output = Vec2<f64>;
    fn mul(self, other: Vec2<f64>) -> Vec2<f64> {
        Vec2::<f64>(self * other.0, self * other.1)
    }
}

/*
impl<T> Mul<Vec2<T>> for T where T: Copy + fmt::Debug + Float {
    type Output = Vec2<T>;
    fn mul(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self * other.0, self * other.1)
    }
}
*/

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

impl<T> fmt::Debug for Vec2<T> where T: Copy + fmt::Debug + Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec2({:?}, {:?})", self.0, self.1)
    }
}
