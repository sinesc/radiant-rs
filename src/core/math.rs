use prelude::*;
use core::{AsUniform, Uniform};

/// A 4x4 matrix.
pub type Mat4<T = f32> = [ [ T; 4 ]; 4 ];

pub trait Mat4Trait<T> {
    fn identity() -> Mat4<T>;
    fn viewport(width: T, height: T) -> Mat4<T>;
    fn set(self: &mut Self, other: &Mat4<T>);
}

impl Mat4Trait<f32> for Mat4<f32> {
    fn identity() -> Mat4<f32> {
        [
            [ 1., 0., 0., 0. ],
            [ 0., 1., 0., 0. ],
            [ 0., 0., 1., 0. ],
            [ 0., 0., 0., 1. ],
        ]
    }
    fn viewport(width: f32, height: f32) -> Mat4<f32> {
        [
            [ 2. / width, 0., 0., 0. ],
            [ 0., -2. / height, 0., 0. ],
            [ 0., 0., 1., 0. ],
            [ -1., 1., 0., 1. ],
        ]
    }
    fn set(self: &mut Self, other: &Mat4<f32>) {
        *self = *other;
    }
}

impl AsUniform for Mat4<f32> {
    fn as_uniform(&self) -> Uniform {
        let a = &self;
        Uniform::Mat4([
            [ a[0][0], a[0][1], a[0][2], a[0][3] ],
            [ a[1][0], a[1][1], a[1][2], a[1][3] ],
            [ a[2][0], a[2][1], a[2][2], a[2][3] ],
            [ a[3][0], a[3][1], a[3][2], a[3][3] ],
        ])
    }
}

impl AsUniform for Mat4<f64> {
    fn as_uniform(&self) -> Uniform {
        let a = &self;
        Uniform::DoubleMat4([
            [ a[0][0], a[0][1], a[0][2], a[0][3] ],
            [ a[1][0], a[1][1], a[1][2], a[1][3] ],
            [ a[2][0], a[2][1], a[2][2], a[2][3] ],
            [ a[3][0], a[3][1], a[3][2], a[3][3] ],
        ])
    }
}

/// A stack of 4x4 matrices.
#[derive(Debug)]
pub struct Mat4Stack<T = f32> (Vec<Mat4<T>>);

impl<T> Mat4Stack<T> where T: Copy {

    /// Creates a new matrix stack.
    pub fn new(matrix: Mat4<T>) -> Mat4Stack<T> {
        Mat4Stack(vec![ matrix ])
    }

    /// Pushes a copy of the current matrix on the stack and returns a reference to it.
    pub fn push(self: &mut Self) -> &mut Mat4<T> {
        let last = *self.0.last().unwrap();
        self.0.push(last);
        self.0.last_mut().unwrap()
    }

    /// Removes the top matrix from the stack and replaces the current matrix with it.
    pub fn pop(self: &mut Self) -> &mut Mat4<T> {
        self.0.pop().unwrap();
        self.0.last_mut().unwrap()
    }
}

impl<T> Deref for Mat4Stack<T> where T: Copy {
    type Target = Mat4<T>;

    fn deref(&self) -> &Mat4<T> {
        self.0.last().unwrap()
    }
}

impl<T> DerefMut for Mat4Stack<T> where T: Copy {
    fn deref_mut(&mut self) -> &mut Mat4<T> {
        self.0.last_mut().unwrap()
    }
}

impl<T> From<Mat4<T>> for Mat4Stack<T> where T: Copy {
    fn from(source: Mat4<T>) -> Self {
        Mat4Stack::new(source)
    }
}

/// A point in 2d space.
pub type Point2<T = f32> = (T, T);

pub trait Point2Trait<T> {
    fn x(self: &Self) -> T;
    fn y(self: &Self) -> T;
    fn as_array(self: &Self) -> [ T; 2 ];
}

impl<T> Point2Trait<T> for Point2<T> where T: Copy {
    #[inline]
    fn x(self: &Self) -> T {
        self.0
    }
    #[inline]
    fn y(self: &Self) -> T {
        self.1
    }
    #[inline]
    fn as_array(self: &Self) -> [ T; 2 ] {
        [ self.0, self.1 ]
    }
}

/// A rectangle.
pub type Rect<T = f32> = (Point2<T>, Point2<T>);

pub trait RectTrait<T> {
    fn top_left(self: &Self) -> Point2<T>;
    fn top_right(self: &Self) -> Point2<T>;
    fn bottom_left(self: &Self) -> Point2<T>;
    fn bottom_right(self: &Self) -> Point2<T>;
    fn as_array(self: &Self) -> [ T; 4 ];
}

impl<T> RectTrait<T> for Rect<T> where T: Copy {
    #[inline]
    fn top_left(self: &Self) -> Point2<T> {
        ((self.0).0, (self.0).1)
    }
    #[inline]
    fn top_right(self: &Self) -> Point2<T> {
        ((self.1).0, (self.0).1)
    }
    #[inline]
    fn bottom_left(self: &Self) -> Point2<T> {
        ((self.0).0, (self.1).1)
    }
    #[inline]
    fn bottom_right(self: &Self) -> Point2<T> {
        ((self.1).0, (self.1).1)
    }
    #[inline]
    fn as_array(self: &Self) -> [ T; 4 ] {
        [ (self.0).0, (self.0).1, (self.1).0, (self.1).1 ]
    }
}