use prelude::*;
use maths::Vec2;

/// A 2-dimensional point.
#[derive(Copy, Clone, Debug)]
pub struct Point2<T = f32>(pub T, pub T) where T: Copy + Clone + Debug;

impl<T> From<(T, T)> for Point2<T> where T: Copy + Clone + Debug {
    fn from(source: (T, T)) -> Self {
        Point2(source.0, source.1)
    }
}

impl<T> From<Vec2<T>> for Point2<T> where T: Float + Copy + Clone + Debug {
    fn from(source: Vec2<T>) -> Self {
        Point2(source.0, source.1)
    }
}

impl From<Point2<f32>> for [ f32; 2 ] {
    fn from(source: Point2<f32>) -> Self {
        [ source.0, source.1 ]
    }
}

impl From<Point2<f32>> for (f32, f32) {
    fn from(source: Point2<f32>) -> Self {
        (source.0, source.1)
    }
}

/// A rectangle.
#[derive(Copy, Clone)]
pub struct Rect<T: Copy + Clone + Debug = f32>(pub Point2<T>, pub Point2<T>);

impl<T> Rect<T> where T: Copy + Clone + Debug {
    /// Creates a new rectangle with given dimensions.
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Rect(Point2(x1, y1), Point2(x2, y2))
    }
}

impl<T> From<(T, T, T, T)> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: (T, T, T, T)) -> Self {
        Rect(Point2(source.0, source.1), Point2(source.2, source.3))
    }
}

impl<T> From<((T, T), (T, T))> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: ((T, T), (T, T))) -> Self {
        Rect(Point2((source.0).0, (source.0).1), Point2((source.1).0, (source.1).1))
    }
}

impl<T> From<(Point2<T>, (T, T))> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: (Point2<T>, (T, T))) -> Self {
        Rect(source.0, Point2((source.1).0, (source.1).1))
    }
}

impl<T> From<((T, T), Point2<T>)> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: ((T, T), Point2<T>)) -> Self {
        Rect(Point2((source.0).0, (source.0).1), source.1)
    }
}

impl<T> From<(Point2<T>, Point2<T>)> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: (Point2<T>, Point2<T>)) -> Self {
        Rect(source.0, source.1)
    }
}

impl<T> From<[ T; 4 ]> for Rect<T> where T: Copy + Clone + Debug {
    fn from(source: [ T; 4 ]) -> Self {
        Rect(Point2(source[0], source[1]), Point2(source[2], source[3]))
    }
}
