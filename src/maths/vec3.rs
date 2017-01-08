use prelude::*;
use maths::VecType;
use glium::uniforms::{AsUniformValue, UniformValue};

/// A 3-dimensional vector.
#[derive(Copy, Clone)]
pub struct Vec3<T: Copy + fmt::Debug + Float>(pub T, pub T, pub T);

impl<T: Copy + fmt::Debug + Float> Vec3<T> {
    /// Creates a new instances.
    pub fn new() -> Vec3<T> {
        Vec3::<T>(T::zero(), T::zero(), T::zero())
    }
    /// Returns the length of the vector
    pub fn len(self: &Self) -> T {
        (self.0*self.0 + self.1*self.1 + self.2*self.2).sqrt()
    }
}

impl<T: Copy + fmt::Debug + Float> VecType<T> for Vec3<T> {
    fn as_vec3(&self, _: T) -> Vec3<T> {
        *self
    }
}

impl<T: Copy + fmt::Debug + Float> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Vec3<T> {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}

impl<T: Copy + fmt::Debug + Float> Add for Vec3<T> {
    type Output = Vec3<T>;
    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl<T: Copy + fmt::Debug + Float> Mul<T> for Vec3<T> {
    type Output = Vec3<T>;
    fn mul(self, other: T) -> Vec3<T> {
        Vec3::<T>(self.0 * other, self.1 * other, self.2 * other)
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

impl<T: Copy + fmt::Debug + Float> fmt::Debug for Vec3<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3({:?}, {:?}, {:?})", self.0, self.1, self.2)
    }
}
