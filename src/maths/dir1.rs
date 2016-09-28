use prelude::*;
use maths::Vec2;

const RAD_TO_DEG32: f32 = 180.0 / f32::consts::PI;
const RAD_TO_DEG64: f64 = 180.0 / f64::consts::PI;

#[derive(Copy, Clone, Debug)]
pub struct Dir1<T: Copy + fmt::Display + Float>(pub T);

impl<T: Copy + fmt::Display + Float> Dir1<T> {
    //const RAD2DEG: T = 180.0 / T::PI;
    pub fn new() -> Dir1<T> {
        Dir1::<T>(T::zero())
    }
    pub fn from_vec2(vec: Vec2<T>) -> Dir1<T> {
        Dir1::<T>(vec.1.atan2(vec.0))
    }
    pub fn to_vec2(&self) -> Vec2<T> {
        Vec2::<T>(self.0.cos(), self.0.sin())
    }
    pub fn to_rad(&self) -> T {
        self.0
    }
}

impl Dir1<f32> {
    pub fn to_deg(&self) -> f32 {
        self.0 * RAD_TO_DEG32
    }
}

impl Dir1<f64> {
    pub fn to_deg(&self) -> f64 {
        self.0 * RAD_TO_DEG64
    }
}

impl<T: Copy + fmt::Display + Float> fmt::Display for Dir1<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Dir1({})", self.0)
    }
}
