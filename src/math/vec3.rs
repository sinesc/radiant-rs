use prelude::*;
use super::Vector;
use radiant_rs::{Uniform, AsUniform};

/// A 3-dimensional vector.
#[derive(Copy, Clone)]
pub struct Vec3<T: Debug + Float = f32>(pub T, pub T, pub T);

impl<T> Vec3<T> where T: Debug + Float {
    /// Creates a new instances.
    pub fn new() -> Vec3<T> {
        Vec3::<T>(T::zero(), T::zero(), T::zero())
    }
    /// Returns the length of the vector
    pub fn len(self: &Self) -> T {
        (self.0*self.0 + self.1*self.1 + self.2*self.2).sqrt()
    }
    /// Returns the dot-product of the vectors.
    pub fn dot(self: &Self, other: &Vec3<T>) -> T {
        self.0 * other.0 + self.1 * other.1 + self.1 * other.2
    }
}

impl<T> Vector<T> for Vec3<T> where T: Debug + Float {
    fn as_vec3(&self, _: T) -> Vec3<T> {
        *self
    }
}

impl<T> From<(T, T, T)> for Vec3<T> where T: Debug + Float {
    fn from(source: (T, T, T)) -> Self {
        Vec3(source.0, source.1, source.2)
    }
}

impl<T> From<[ T; 3 ]> for Vec3<T> where T: Debug + Float {
    fn from(source: [ T; 3 ]) -> Self {
        Vec3(source[0], source[1], source[2])
    }
}

impl From<Vec3<f32>> for [ f32; 3 ] {
    fn from(source: Vec3<f32>) -> Self {
        [ source.0, source.1, source.2 ]
    }
}

impl From<Vec3<f64>> for [ f64; 3 ] {
    fn from(source: Vec3<f64>) -> Self {
        [ source.0, source.1, source.2 ]
    }
}

impl<T> Neg for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    fn neg(self) -> Vec3<T> {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}

impl<T> Add for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl<T> AddAssign for Vec3<T> where T: Debug + Float {
    fn add_assign(self: &mut Self, other: Vec3<T>) {
        *self = Vec3::<T> (
            self.0 + other.0,
            self.1 + other.1,
            self.2 + other.2
        )
    }
}

impl<T> Sub for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    fn sub(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl<T> SubAssign for Vec3<T> where T: Debug + Float {
    fn sub_assign(self: &mut Self, other: Vec3<T>) {
        *self = Vec3::<T> (
            self.0 - other.0,
            self.1 - other.1,
            self.2 - other.2,
        )
    }
}

impl<T> Mul<Vec3<T>> for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    /// Multiplies individual vector components with those of the given vector.
    fn mul(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl<T> MulAssign<Vec3<T>> for Vec3<T> where T: Debug + Float {
    /// Mutates the vector by multiplying its components with those of the given vector.
    fn mul_assign(&mut self, other: Vec3<T>) {
        *self = Vec3::<T>(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl<T> Mul<T> for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    /// Multiplies the vector with given scalar operand.
    fn mul(self, other: T) -> Vec3<T> {
        Vec3::<T>(self.0 * other, self.1 * other, self.2 * other)
    }
}

impl<T> MulAssign<T> for Vec3<T> where T: Debug + Float {
    /// Mutates the vector by multiplying it with the scalar operand.
    fn mul_assign(&mut self, other: T) {
        *self = Vec3::<T>(self.0 * other, self.1 * other, self.2 * other)
    }
}

impl<T> Div<Vec3<T>> for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    /// Divides individual vector components with those of the given vector.
    fn div(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T>(self.0 / other.0, self.1 / other.1, self.2 / other.2)
    }
}

impl<T> DivAssign<Vec3<T>> for Vec3<T> where T: Debug + Float {
    /// Mutates the vector by dividing its components with those of the given vector.
    fn div_assign(&mut self, other: Vec3<T>) {
        *self = Vec3::<T>(self.0 / other.0, self.1 / other.1, self.2 / other.2)
    }
}

impl<T> Div<T> for Vec3<T> where T: Debug + Float {
    type Output = Vec3<T>;
    /// Divides the vector by given scalar operand.
    fn div(self, other: T) -> Vec3<T> {
        Vec3::<T>(self.0 / other, self.1 / other, self.2 / other)
    }
}

impl<T> DivAssign<T> for Vec3<T> where T: Debug + Float {
    /// Mutates the vector by dividing it by given scalar.
    fn div_assign(&mut self, other: T) {
        *self = Vec3::<T>(self.0 / other, self.1 / other, self.2 / other)
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

impl AsUniform for Vec3<f32> {
    fn as_uniform(&self) -> Uniform {
        Uniform::Vec3([ self.0, self.1, self.2 ])
    }
}

impl AsUniform for Vec3<f64> {
    fn as_uniform(&self) -> Uniform {
        Uniform::DoubleVec3([ self.0, self.1, self.2 ])
    }
}

impl<T> Debug for Vec3<T> where T: Debug + Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3({:?}, {:?}, {:?})", self.0, self.1, self.2)
    }
}
