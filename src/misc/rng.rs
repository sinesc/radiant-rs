use prelude::*;

/// A very simple, seedable random number generator based on sin(). It is provided to simplify
/// experimentation with the library but not suitable for applications that require good quality
/// random numbers.
pub struct Rng (f64);

impl Rng {

    /// Creates a new instance with given seed.
    pub fn new<T>(seed: T) -> Rng where T: FromPrimitive + ToPrimitive + Copy {
        Rng(seed.to_f64().unwrap())
    }

    /// Returns a random number between 0.0 and non-inclusive 1.0
    pub fn get<T>(self: &mut Self) -> T where T: FromPrimitive + ToPrimitive + Copy {

        let large = self.0.sin() * 100000000.0;
        self.0 += 1.0;

        T::from_f64(large - large.floor()).unwrap()
    }

    /// Returns a random number between min and non-inclusive max.
    pub fn range<T>(self: &mut Self, min: T, max: T) -> T where T: FromPrimitive + ToPrimitive + Copy {

        let large = self.0.sin() * 100000000.0;
        self.0 += 1.0;

        let base = (large - large.floor()) as f64;
        let min = min.to_f64().unwrap();
        let max = max.to_f64().unwrap();
        T::from_f64(min + base * (max - min)).unwrap()
    }
}
