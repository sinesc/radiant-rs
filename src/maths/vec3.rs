use prelude::*;
use maths::VecType;
use glium::uniforms::{AsUniformValue, UniformValue};

/// A 3-dimensional vector.
#[derive(Copy, Clone)]
pub struct Vec3<T: Copy + fmt::Debug + Float = f32>(pub T, pub T, pub T);

impl<T> Vec3<T> where T: Copy + fmt::Debug + Float {
    /// Creates a new instances.
    pub fn new() -> Vec3<T> {
        Vec3::<T>(T::zero(), T::zero(), T::zero())
    }
    /// Returns the length of the vector
    pub fn len(self: &Self) -> T {
        (self.0*self.0 + self.1*self.1 + self.2*self.2).sqrt()
    }
}

impl<T> VecType<T> for Vec3<T> where T: Copy + fmt::Debug + Float{
    fn as_vec3(&self, _: T) -> Vec3<T> {
        *self
    }
}

impl<T> Neg for Vec3<T> where T: Copy + fmt::Debug + Float {
    type Output = Vec3<T>;

    fn neg(self) -> Vec3<T> {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}

impl<T> Add for Vec3<T> where T: Copy + fmt::Debug + Float {
    type Output = Vec3<T>;
    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl<T> AddAssign for Vec3<T> where T: Copy + fmt::Debug + Float{
    fn add_assign(self: &mut Self, other: Vec3<T>) {
        *self = Vec3::<T> (
            self.0 + other.0,
            self.1 + other.1,
            self.2 + other.2
        )
    }
}

impl<T> Mul<T> for Vec3<T> where T: Copy + fmt::Debug + Float {
    type Output = Vec3<T>;
    fn mul(self, other: T) -> Vec3<T> {
        Vec3::<T>(self.0 * other, self.1 * other, self.2 * other)
    }
}

impl<T> Mul<Vec3<T>> for Vec3<T> where T: Copy + fmt::Debug + Float {
    type Output = Vec3<T>;
    fn mul(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 * other.0, self.1 * other.1, self.1 * other.2)
    }
}

impl Mul<Vec3<f32>> for f32 {
    type Output = Vec3<f32>;
    fn mul(self, other: Vec3<f32>) -> Vec3<f32> {
        Vec3::<f32>(self * other.0, self * other.1, self * other.2)
    }
}

impl Mul<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;
    fn mul(self, other: Vec3<f64>) -> Vec3<f64> {
        Vec3::<f64>(self * other.0, self * other.1, self * other.2)
    }
}

#[doc(hidden)]
impl AsUniformValue for Vec3<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3([ self.0, self.1, self.2 ])
    }
}

#[doc(hidden)]
impl AsUniformValue for Vec3<f64> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleVec3([ self.0, self.1, self.2 ])
    }
}

impl<T> fmt::Debug for Vec3<T> where T: Copy + fmt::Debug + Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3({:?}, {:?}, {:?})", self.0, self.1, self.2)
    }
}
