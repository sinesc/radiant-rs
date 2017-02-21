use prelude::*;
use maths::{Point2};

/// A rectangle.
#[derive(Copy, Clone)]
pub struct Rect<T: Debug + Float = f32>(pub Point2<T>, pub Point2<T>);

impl<T> Rect<T> where T: Debug + Float {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Rect(Point2(x1, y1), Point2(x2, y2))
    }
}

impl<T> From<(T, T, T, T)> for Rect<T> where T: Debug + Float {
    fn from(source: (T, T, T, T)) -> Self {
        Rect(Point2(source.0, source.1), Point2(source.2, source.3))
    }
}

impl<T> From<[ T; 4 ]> for Rect<T> where T: Debug + Float {
    fn from(source: [ T; 4 ]) -> Self {
        Rect(Point2(source[0], source[1]), Point2(source[2], source[3]))
    }
}
