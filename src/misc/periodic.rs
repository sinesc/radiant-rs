use prelude::*;
use super::min;

/// A generic interval, testable for elapsation. !todo Is that even a word?
#[derive(Copy, Clone)]
pub struct Periodic<S = f32, T = S> {
    interval: T,
    next: S,
}

impl<S, T> Periodic<S, T>
    where
        S: Copy + PartialOrd + Add<T> + Sub<T> + Sub<S> +
            From<<S as Add<T>>::Output> +
            From<<S as Sub<T>>::Output>,
        T: Copy + PartialOrd + Add<T> +
            From<<S as Sub<S>>::Output>
{
    /// Returns a new Periodic instance.
    pub fn new(current: S, interval: T) -> Periodic<S, T> {
        Periodic {
            interval: interval,
            next: S::from(current + interval),
        }
    }
    /// Returns true if the interval has elapsed and subtracts the interval
    /// since then from the next interval.
    pub fn elapsed(self: &mut Self, current: S) -> bool {
        if self.next <= current {
            self.next = S::from(S::from(current + self.interval) - min(T::from(current - self.next), self.interval));
            true
        } else {
            false
        }
    }
    /// Returns true if the interval has elapsed and advances by one interval.
    /// This function will return true for each interval that has elapsed.
    pub fn accumulated(self: &mut Self, current: S) -> bool {
        if self.next <= current {
            self.next = S::from(self.next + self.interval);
            true
        } else {
            false
        }
    }
    /// Returns true if the interval has elapsed
    pub fn peek(self: &mut Self, current: S) -> bool {
        self.next <= current
    }
}

impl<S, T> Debug for Periodic<S, T> where S: Debug, T: Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Periodic(interval: {:?}, next: {:?})", self.interval, self.next)
    }
}
