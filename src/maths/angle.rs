use prelude::*;
use super::vec2::Vec2;

/// An Angle between -PI and PI.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Angle<T: Debug + Float + NumCast = f32>(pub T);

impl<T> Angle<T> where T: Debug + Float + NumCast {
    /// Returns the angle's value in radians.
    pub fn to_radians(self: &Self) -> T {
        self.0
    }
    /// Returns the angle's value in degrees.
    pub fn to_degrees(self: &Self) -> T {
        self.0.to_degrees()
    }
    /// Returns a vector pointing in the direction of the angle.
    pub fn to_vec2(self: &Self) -> Vec2<T> {
        Vec2::from_angle(*self)
    }
    /// Creates an angle from a radians value.
    pub fn from_radians(radians: T) -> Angle<T> {
        Angle::<T>(radians)
    }
    /// Creates an angle from a degrees value.
    pub fn from_degrees(degrees: T) -> Angle<T> {
        Self::from_radians(degrees.to_radians())
    }
    /// Mutates self to its normalized representation.
    #[allow(non_snake_case)]
    pub fn normalize(self: &mut Self) -> &mut Self {
        let PI = NumCast::from(std::f64::consts::PI).unwrap();
        let two_PI = NumCast::from(std::f64::consts::PI * 2.0).unwrap();
        if self.0.abs() > PI {
            self.0 = self.0 - two_PI * ((self.0 + PI) / two_PI).floor();
        }
        self
    }
    /// Mutates self so that subtracting the target will yield the smallest directional angle between self and target.
    #[allow(non_snake_case)]
    pub fn align_with(self: &mut Self, target: &Angle<T>) -> &mut Self {

        let PI = NumCast::from(std::f64::consts::PI).unwrap();
        let two_PI = NumCast::from(std::f64::consts::PI * 2.0).unwrap();

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

impl<T> From<T> for Angle<T> where T: Debug + Float {
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

impl<T> ToPrimitive for Angle<T> where T: Debug + Float {
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

impl<T> FromPrimitive for Angle<T> where T: Debug + Float {
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

impl<T> Neg for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;

    fn neg(self) -> Angle<T> {
        Angle::<T>(-self.0)
    }
}

impl<T> Add for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn add(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 + other.0)
    }
}

impl<T> AddAssign for Angle<T> where T: Debug + Float {
    fn add_assign(self: &mut Self, other: Angle<T>) {
        *self = Angle::<T>(self.0 + other.0)
    }
}

impl<T> Sub for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn sub(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 - other.0)
    }
}

impl<T> SubAssign for Angle<T> where T: Debug + Float {
    fn sub_assign(self: &mut Self, other: Angle<T>) {
        *self = Angle::<T>(self.0 - other.0)
    }
}

impl<T> Mul<Angle<T>> for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn mul(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 * other.0)
    }
}

impl<T> MulAssign for Angle<T> where T: Debug + Float {
    fn mul_assign(&mut self, other: Angle<T>) {
        *self = Angle::<T>(self.0 * other.0)
    }
}

impl<T> Mul<T> for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn mul(self, other: T) -> Angle<T> {
        Angle::<T>(self.0 * other)
    }
}

impl<T> Div<Angle<T>> for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn div(self, other: Angle<T>) -> Angle<T> {
        Angle::<T>(self.0 / other.0)
    }
}

impl<T> DivAssign for Angle<T> where T: Debug + Float {
    fn div_assign(&mut self, other: Angle<T>) {
        *self = Angle::<T>(self.0 / other.0)
    }
}

impl<T> Div<T> for Angle<T> where T: Debug + Float {
    type Output = Angle<T>;
    fn div(self, other: T) -> Angle<T> {
        Angle::<T>(self.0 / other)
    }
}

impl<T> Debug for Angle<T> where T: Debug + Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Angle({:?})", self.0)
    }
}
