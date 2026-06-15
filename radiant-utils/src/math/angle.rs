use crate::prelude::*;
use super::Vec2;

/// An Angle between -PI and PI.
#[cfg_attr(feature = "serialize-serde", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Angle<T = f32>(pub T);

impl<T> Angle<T> where T: Float {
    /// Returns the angle's value in radians.
    pub fn to_radians(self: &Self) -> T {
        self.0
    }
    /// Returns the angle's value in degrees.
    pub fn to_degrees(self: &Self) -> T {
        self.0.to_degrees()
    }
    /// Returns a vector pointing in the direction of the angle.
    #[deprecated(since = "0.2.2", note = "use Vec2::from() instead")]
    pub fn to_vec2(self: &Self) -> Vec2<T> {
        Vec2::from(*self)
    }
    /// Creates an angle from a radians value.
    pub fn from_radians(radians: T) -> Self {
        Angle::<T>(radians)
    }
    /// Creates an angle from a degrees value.
    pub fn from_degrees(degrees: T) -> Self {
        Self::from_radians(degrees.to_radians())
    }
    /// Returns a normalized version of the angle.
    #[allow(non_snake_case)]
    pub fn normalize(self: &Self) -> Self {
        let PI = NumCast::from(f64::consts::PI).unwrap();
        if self.0.abs() > PI {
            let two_PI = NumCast::from(f64::consts::PI * 2.0).unwrap();
            Angle(self.0 - two_PI * ((self.0 + PI) / two_PI).floor())
        } else {
            *self
        }
    }
    /// Returns smallest directional angle between self and target.
    pub fn diff(self: &Self, other: Self) -> Self {
        let mut this = *self;
        this.align_with(other);
        this - other
    }
    /// Mutates self so that subtracting the target will yield the smallest directional angle between self and target.
    #[allow(non_snake_case)]
    pub fn align_with(self: &mut Self, target: Self) -> &mut Self {

        let PI = NumCast::from(f64::consts::PI).unwrap();
        let two_PI = NumCast::from(f64::consts::PI * 2.0).unwrap();

        // normalize

        if self.0.abs() > PI {
            self.0 = self.0 - two_PI * ((self.0 + PI) / two_PI).floor();
        }

        let target_radians = if target.0.abs() > PI {
            target.0 - two_PI * ((target.0 + PI) / two_PI).floor()
        } else {
            target.0
        };

        // adjust self if self-other would exceend +/-PI

        let diff = self.0 - target_radians;

        if diff.abs() > PI {
            self.0 = self.0 - diff.signum() * two_PI;
        }

        self
    }
}

// from vector

impl<T> From<Vec2<T>> for Angle<T> where T: Float {
    fn from(vec: Vec2<T>) -> Angle<T> {
        Angle(vec.1.atan2(vec.0))
    }
}

// from/to float

impl<T> From<T> for Angle<T> where T: Float {
    fn from(other: T) -> Angle<T> {
        Angle(other)
    }
}

impl From<Angle<f64>> for f64 {
    fn from(other: Angle<f64>) -> f64 {
        other.to_radians() as f64
    }
}

impl From<Angle<f32>> for f32 {
    fn from(other: Angle<f32>) -> f32 {
        other.to_radians() as f32
    }
}

impl<'a> From<&'a Angle<f64>> for f64 {
    fn from(other: &'a Angle<f64>) -> f64 {
        other.to_radians() as f64
    }
}

impl<'a> From<&'a Angle<f32>> for f32 {
    fn from(other: &'a Angle<f32>) -> f32 {
        other.to_radians() as f32
    }
}

// Default

impl<T> Default for Angle<T> where T: Float {
    fn default() -> Self {
        Angle(T::zero())
    }
}

// TODO: why?

impl<T> ToPrimitive for Angle<T> where T: Float {
    fn to_f64(self: &Self) -> Option<f64> {
        NumCast::from(self.to_radians())
    }
    fn to_f32(self: &Self) -> Option<f32> {
        NumCast::from(self.to_radians())
    }
    fn to_i64(self: &Self) -> Option<i64> {
        NumCast::from(self.to_radians())
    }
    fn to_u64(self: &Self) -> Option<u64> {
        let value = self.to_radians();
        if value >= T::zero() { NumCast::from(self.to_radians()) } else { None }
    }
}

impl<T> FromPrimitive for Angle<T> where T: Float {
    fn from_f64(n: f64) -> Option<Angle<T>> {
        Some(Angle(NumCast::from(n).unwrap()))
    }
    fn from_f32(n: f32) -> Option<Angle<T>> {
        Some(Angle(NumCast::from(n).unwrap()))
    }
    fn from_i64(n: i64) -> Option<Angle<T>> {
        Some(Angle(NumCast::from(n).unwrap()))
    }
    fn from_u64(n: u64) -> Option<Angle<T>> {
        Some(Angle(NumCast::from(n).unwrap()))
    }
}

// operators

impl<T> Neg for Angle<T> where T: Float {
    type Output = Angle<T>;

    fn neg(self) -> Angle<T> {
        Angle::<T>(-self.0)
    }
}

impl<T> Add for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn add(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 + other.0)
    }
}

impl<T> AddAssign for Angle<T> where T: Float {
    fn add_assign(self: &mut Self, other: Angle<T>) {
        *self = Angle::<T>(self.0 + other.0)
    }
}

impl<T> Sub for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn sub(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 - other.0)
    }
}

impl<T> SubAssign for Angle<T> where T: Float {
    fn sub_assign(self: &mut Self, other: Angle<T>) {
        *self = Angle::<T>(self.0 - other.0)
    }
}

impl<T> Mul<Angle<T>> for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn mul(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 * other.0)
    }
}

impl<T> MulAssign for Angle<T> where T: Float {
    fn mul_assign(&mut self, other: Angle<T>) {
        *self = Angle::<T>(self.0 * other.0)
    }
}

impl<T> Mul<T> for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn mul(self, other: T) -> Angle<T> {
        Angle::<T>(self.0 * other)
    }
}

impl<T> Div<Angle<T>> for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn div(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 / other.0)
    }
}

impl<T> DivAssign for Angle<T> where T: Float {
    fn div_assign(&mut self, other: Angle<T>) {
        *self = Angle::<T>(self.0 / other.0)
    }
}

impl<T> Div<T> for Angle<T> where T: Float {
    type Output = Angle<T>;
    fn div(self, other: T) -> Angle<T> {
        Angle::<T>(self.0 / other)
    }
}

// as radiant uniform

#[cfg(feature = "uniforms")]
use radiant_rs::{Uniform, AsUniform};

#[cfg(feature = "uniforms")]
impl AsUniform for Angle<f32> {
    fn as_uniform(&self) -> Uniform {
        Uniform::Float(self.0)
    }
}

#[cfg(feature = "uniforms")]
impl AsUniform for Angle<f64> {
    fn as_uniform(&self) -> Uniform {
        Uniform::Double(self.0)
    }
}

// debug print

impl<T> Debug for Angle<T> where T: Debug + Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Angle({:?})", self.0)
    }
}
