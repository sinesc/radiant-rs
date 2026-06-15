use std::sync::atomic::{AtomicUsize, Ordering};

/// A very simple, seedable atomic random number generator based on sin().
pub struct ARng (AtomicUsize);

impl ARng {

    /// Creates a new instance with given seed.
    pub fn new(seed: usize) -> ARng {
        ARng(AtomicUsize::new(seed))
    }

    /// Returns a random number between 0.0 and non-inclusive 1.0
    pub fn get(self: &Self) -> f64 {
        let pos = self.0.fetch_add(1, Ordering::SeqCst);
        let large = (pos as f64).sin() * 100000000.0;
        large - large.floor()
    }

    /// Returns a random number between min and non-inclusive max.
    pub fn range(self: &Self, min: f64, max: f64) -> f64 {
        let pos = self.0.fetch_add(1, Ordering::SeqCst);
        let large = (pos as f64).sin() * 100000000.0;
        let base = (large - large.floor()) as f64;
        min + base * (max - min)
    }

    /// Returns a random item from given slice.
    pub fn chose<'a, T>(self: &Self, source: &'a [ T ]) -> &'a T {
        let index = self.range(0 as f64, source.len() as f64) as usize;
        &source[index]
    }
}