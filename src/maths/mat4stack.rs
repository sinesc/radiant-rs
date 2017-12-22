use prelude::*;
use maths::Mat4;

pub struct Mat4Stack<T: Debug + Float + NumCast = f32> (Vec<Mat4<T>>);

impl<T> Mat4Stack<T> where T: Debug + Float + NumCast {

    /// Creates a new matrix stack.
    pub fn new(matrix: Mat4<T>) -> Mat4Stack<T> {
        Mat4Stack(vec![ matrix ])
    }

    pub fn push(self: &mut Self) -> &mut Mat4<T> {
        let last = *self.0.last().unwrap();
        self.0.push(last);
        self.0.last_mut().unwrap()
    }

    pub fn pop(self: &mut Self) -> &mut Mat4<T> {
        self.0.pop().unwrap();
        self.0.last_mut().unwrap()
    }
}

impl<T: Debug + Float + NumCast> Deref for Mat4Stack<T> {
    type Target = Mat4<T>;

    fn deref(&self) -> &Mat4<T> {
        self.0.last().unwrap()
    }
}

impl<T: Debug + Float + NumCast> DerefMut for Mat4Stack<T> {
    fn deref_mut(&mut self) -> &mut Mat4<T> {
        self.0.last_mut().unwrap()
    }
}

impl<T: Debug + Float + NumCast> From<Mat4<T>> for Mat4Stack<T> {
    fn from(source: Mat4<T>) -> Self {
        Mat4Stack::new(source)
    }
}