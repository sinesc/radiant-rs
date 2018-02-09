use radiant_rs::{Uniform, AsUniform};
use prelude::*;
use super::{Vec3, Vector, Angle, Rect, Point2};

/// A 2-dimensional vector.
#[derive(Copy, Clone)]
pub struct Vec2<T = f32>(pub T, pub T);

impl<T> Vec2<T> where T: Float {
    /// Creates a new instances.
    pub fn new() -> Self {
        Vec2::<T>(T::zero(), T::zero())
    }
    /// Returns the length of the vector
    pub fn len(self: &Self) -> T {
        (self.0*self.0 + self.1*self.1).sqrt()
    }
    /// Returns the dot-product of the vectors.
    pub fn dot(self: &Self, other: &Self) -> T {
        self.0 * other.0 + self.1 * other.1
    }
    /// Returns the direction of the vector in radians.
    pub fn to_radians(self: &Self) -> T {
        self.1.atan2(self.0)
    }
    /// Returns the direction of the vector in degrees.
    pub fn to_degrees(self: &Self) -> T {
        self.to_radians().to_degrees()
    }
    /// Returns the direction of the vector as an angle instance.
    pub fn to_angle(self: &Self) -> Angle<T> {
        Angle(self.to_radians())
    }
    /// Creates a unit-vector from the angle given in radians.
    pub fn from_radians(radians: T) -> Self {
        Vec2::<T>(radians.cos(), radians.sin())
    }
    /// Creates a unit-vector from the angle given in degrees.
    pub fn from_degrees(degrees: T) -> Self {
        Self::from_radians(degrees.to_radians())
    }
    /// Creates a unit-vector from given angle.
    pub fn from_angle(angle: Angle<T>) -> Self {
        Self::from_radians(angle.to_radians())
    }
    /// Normalizes the vector.
    pub fn normalize(mut self: Self) -> Self {
        let len = self.len();
        if len > T::zero() {
            self.0 = self.0 / len;
            self.1 = self.1 / len;
        }
        self
    }
    /// The left pointing normal of the vector.
    pub fn left(mut self: Self) -> Self {
        let x = self.0;
        self.0 = -self.1;
        self.1 = x;
        self
    }
    /// The right pointing normal of the vector.
    pub fn right(mut self: Self) -> Self {
        let x = self.0;
        self.0 = self.1;
        self.1 = -x;
        self
    }
    /// Extends the vector by given length.
    pub fn extend(mut self: Self, extension_len: T) -> Self {
        let base_len = self.len();
        if base_len > T::zero() {
            let new_len = base_len + extension_len;
            let factor = new_len / base_len;
            self.0 = self.0 * factor;
            self.1 = self.1 * factor;
        }
        self
    }
    /// Returns outbound vector for this point and given bounding box. Subtracting
    /// it from this point will result in a point on the bounding box.
    pub fn outbound(self: &Self, bounding: Rect<T>) -> Option<Self> {
        let min = bounding.0;
        let max = bounding.1;
        let outside = Vec2(
            if self.0 < min.0 { self.0 - min.0 } else if self.0 > max.0 { self.0 - max.0 } else { T::zero() },
            if self.1 < min.1 { self.1 - min.1 } else if self.1 > max.1 { self.1 - max.1 } else { T::zero() }
        );
        if (outside.0 != T::zero()) || (outside.1 != T::zero()) { Some(outside) } else { None }
    }
    /// Returns true if the vecor is a zero-vector.
    pub fn is_zero(self: &Self) -> bool {
        self.0 == T::zero() && self.1 == T::zero()
    }
    /// Returns distance to other point.
    pub fn distance(self: &Self, other: &Self) -> T{
        let dv = *self - *other;
        (dv.0 * dv.0 + dv.1 * dv.1).sqrt()
    }
}

impl<T> Vector<T> for Vec2<T> where T: Copy {
    fn as_vec3(&self, neutral: T) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, neutral)
    }
}

// from/to array

impl<T> From<[ T; 2 ]> for Vec2<T> where T: Copy {
    fn from(source: [ T; 2 ]) -> Self {
        Vec2(source[0], source[1])
    }
}

impl From<Vec2<f32>> for [ f32; 2 ] {
    fn from(source: Vec2<f32>) -> Self {
        [ source.0, source.1 ]
    }
}

impl From<Vec2<f64>> for [ f64; 2 ] {
    fn from(source: Vec2<f64>) -> Self {
        [ source.0, source.1 ]
    }
}

// from/to tuple struct

impl<T> From<Point2<T>> for Vec2<T> {
    fn from(source: Point2<T>) -> Self {
        Vec2(source.0, source.1)
    }
}

/*impl From<Point2<i32>> for Vec2<f32> {
    fn from(source: Point2<i32>) -> Self {
        Vec2(source.0 as f32, source.1 as f32)
    }
}*/

impl From<Vec2<f32>> for Point2<f32> {
    fn from(source: Vec2<f32>) -> Self {
        (source.0, source.1)
    }
}

impl From<Vec2<f64>> for Point2<f64> {
    fn from(source: Vec2<f64>) -> Self {
        (source.0, source.1)
    }
}

// operators

impl<T> Neg for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    fn neg(self) -> Vec2<T> {
        Vec2::<T>(-self.0, -self.1)
    }
}

impl<T> Add for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 + other.0, self.1 + other.1)
    }
}

impl<T> AddAssign for Vec2<T> where T: Float {
    fn add_assign(self: &mut Self, other: Vec2<T>) {
        *self = Vec2::<T> (
            self.0 + other.0,
            self.1 + other.1
        )
    }
}

impl<T> Sub for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 - other.0, self.1 - other.1)
    }
}

impl<T> SubAssign for Vec2<T> where T: Float {
    fn sub_assign(self: &mut Self, other: Vec2<T>) {
        *self = Vec2::<T> (
            self.0 - other.0,
            self.1 - other.1
        )
    }
}

impl<T> Mul<Vec2<T>> for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    /// Multiplies individual vector components with those of the given vector.
    fn mul(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 * other.0, self.1 * other.1)
    }
}

impl<T> MulAssign<Vec2<T>> for Vec2<T> where T: Float {
    /// Mutates the vector by multiplying its components with those of the given vector.
    fn mul_assign(&mut self, other: Vec2<T>) {
        *self = Vec2::<T>(self.0 * other.0, self.1 * other.1)
    }
}

impl<T> Mul<T> for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    /// Multiplies the vector with given scalar operand.
    fn mul(self, other: T) -> Vec2<T> {
        Vec2::<T>(self.0 * other, self.1 * other)
    }
}

impl<T> MulAssign<T> for Vec2<T> where T: Float {
    /// Mutates the vector by multiplying it with the scalar operand.
    fn mul_assign(&mut self, other: T) {
        *self = Vec2::<T>(self.0 * other, self.1 * other)
    }
}

impl<T> Div<Vec2<T>> for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    /// Divides individual vector components with those of the given vector.
    fn div(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T>(self.0 / other.0, self.1 / other.1)
    }
}

impl<T> DivAssign<Vec2<T>> for Vec2<T> where T: Float {
    /// Mutates the vector by dividing its components with those of the given vector.
    fn div_assign(&mut self, other: Vec2<T>) {
        *self = Vec2::<T>(self.0 / other.0, self.1 / other.1)
    }
}

impl<T> Div<T> for Vec2<T> where T: Float {
    type Output = Vec2<T>;
    /// Divides the vector by given scalar operand.
    fn div(self, other: T) -> Vec2<T> {
        Vec2::<T>(self.0 / other, self.1 / other)
    }
}

impl<T> DivAssign<T> for Vec2<T> where T: Float {
    /// Mutates the vector by dividing it by given scalar.
    fn div_assign(&mut self, other: T) {
        *self = Vec2::<T>(self.0 / other, self.1 / other)
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

// as radiant uniform

impl AsUniform for Vec2<f32> {
    fn as_uniform(&self) -> Uniform {
        Uniform::Vec2([ self.0, self.1 ])
    }
}

impl AsUniform for Vec2<f64> {
    fn as_uniform(&self) -> Uniform {
        Uniform::DoubleVec2([ self.0, self.1 ])
    }
}

// debug print

impl<T> Debug for Vec2<T> where T: Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec2({:?}, {:?})", self.0, self.1)
    }
}
