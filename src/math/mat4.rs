use prelude::*;
use super::{Vec2, Vec3, Rect, Vector, Matrix};
use super::matrix::Mat4 as Mat4Type;

/// A 4x4 matrix.
#[derive(Copy, Clone)]
pub struct Mat4<T: Copy + Debug + Float + NumCast = f32>(pub [ [ T; 4 ]; 4 ]);

impl<T> Mat4<T> where T: Debug + Float + NumCast {
    /// Creates a zero matrix.
    pub fn new() -> Mat4<T> {
        Mat4([ [ T::zero(); 4]; 4 ])
    }
    /// Creates an identity matrix.
    pub fn identity() -> Mat4<T> {
        Mat4([
            [ T::one(),  T::zero(), T::zero(), T::zero()  ],
            [ T::zero(), T::one(),  T::zero(), T::zero(), ],
            [ T::zero(), T::zero(), T::one(),  T::zero(), ],
            [ T::zero(), T::zero(), T::zero(), T::one(),  ],
        ])
    }
    /// Creates viewport matrix mapping viewport top left to (0.0, 0.0) and bottom right to (width, height)
    pub fn viewport(width: T, height: T) -> Mat4<T> {
        //const TWO: T = T::one() + T::one(); // TODO: compiler whining
        #[allow(non_snake_case)]
        let TWO = T::one() + T::one();
        Mat4([
            [ TWO/width, T::zero(), T::zero(), T::zero() ],
            [ T::zero(), -TWO/height, T::zero(), T::zero() ],
            [ T::zero(), T::zero(), T::one(), T::zero() ],
            [ -T::one(), T::one(), T::zero(), T::one() ],
        ])
    }
    /// Creates a look-at matrix with the given eye position, target position, and up-vector.
    pub fn look_at<E, C, U>(eye: E, target: C, up: U) -> Mat4<T> where E: Vector<T>, C: Vector<T>, U: Vector<T> {

        let eye = eye.as_vec3(T::zero());
        let target = target.as_vec3(T::zero());
        let up = up.as_vec3(T::zero());
        let zero = T::zero();
        let one = T::one();

        if (eye.0 - target.0).abs() < T::epsilon() && (eye.1 - target.1).abs() < T::epsilon() && (eye.2 - target.2).abs() < T::epsilon() {
            return Self::identity();
        }

        let mut z0 = eye.0 - target.0;
        let mut z1 = eye.1 - target.1;
        let mut z2 = eye.2 - target.2;

        let recip_len = (z0 * z0 + z1 * z1 + z2 * z2).sqrt().recip();
        z0 = z0 * recip_len;
        z1 = z1 * recip_len;
        z2 = z2 * recip_len;

        let mut x0 = up.1 * z2 - up.2 * z1;
        let mut x1 = up.2 * z0 - up.0 * z2;
        let mut x2 = up.0 * z1 - up.1 * z0;
        let len = (x0 * x0 + x1 * x1 + x2 * x2).sqrt();

        if len < T::epsilon() {
            x0 = zero;
            x1 = zero;
            x2 = zero;
        } else {
            let recip_len = len.recip();
            x0 = x0 * recip_len;
            x1 = x1 * recip_len;
            x2 = x2 * recip_len;
        }

        let mut y0 = z1 * x2 - z2 * x1;
        let mut y1 = z2 * x0 - z0 * x2;
        let mut y2 = z0 * x1 - z1 * x0;
        let len = (y0 * y0 + y1 * y1 + y2 * y2).sqrt();

        if len < T::epsilon() {
            y0 = zero;
            y1 = zero;
            y2 = zero;
        } else {
            let recip_len = len.recip();
            y0 = y0 * recip_len;
            y1 = y1 * recip_len;
            y2 = y2 * recip_len;
        }

        Mat4([
            [ x0, y0, z0, zero ],
            [ x1, y1, z1, zero ],
            [ x2, y2, z2, zero ],
            [
                -(x0 * eye.0 + x1 * eye.1 + x2 * eye.2),
                -(y0 * eye.0 + y1 * eye.1 + y2 * eye.2),
                -(z0 * eye.0 + z1 * eye.1 + z2 * eye.2),
                one
            ]
        ])
    }
    /// Creates an orthogonal projection matrix with the given rectangular bounds at the
    /// near and far clipping plane.
    pub fn ortho<R>(rectangle: R, near: T, far: T) -> Mat4<T> where Rect<T>: From<R> {
        let ((left, top), (right, bottom)) = rectangle.into();
        let two = T::one() + T::one();
        let zero = T::zero();
        let lr = T::one() / (left - right);
        let bt = T::one() / (bottom - top);
        let nf = T::one() / (near - far);
        Mat4([
            [ -two * lr,            zero,               zero,               zero     ],
            [ zero,                 -two * bt,          zero,               zero     ],
            [ zero,                 zero,               two * nf,           zero     ],
            [ (left + right) * lr, (top + bottom) * bt, (far + near) * nf,  T::one() ],
        ])
    }
    /// Creates a frustum projection matrix with the given rectangular bounds at the
    /// near clipping plane and rectangle * (far / near) at the far clipping plane.
    pub fn frustum<R>(rectangle: R, near: T, far: T) -> Mat4<T> where Rect<T>: From<R> {
        let ((left, top), (right, bottom)) = rectangle.into();
        let two = T::one() + T::one();
        let zero = T::zero();
        let rl = (right - left).recip();
        let tb = (top - bottom).recip();
        let nf = (near - far).recip();
        Mat4([
            [ (near * two) * rl, zero, zero, zero ],
            [ zero, (near * two) * tb, zero, zero ],
            [ (right + left) * rl, (top + bottom) * tb, (far + near) * nf, -T::one() ],
            [ zero, zero, (far * near * two) * nf, zero ]
        ])
    }
    /// Creates a perspective projection matrix with the given field of view, aspect and
    /// near/far clipping planes.
    pub fn perspective(fov_y: T, aspect: T, near: T, far: T) -> Mat4<T> {
        let two = T::one() + T::one();
        let f = (fov_y / two).tan().recip();
        let nf = (near - far).recip();
        Mat4([
            [ f / aspect,   T::zero(),  T::zero(),              T::zero() ],
            [ T::zero(),    f,          T::zero(),              T::zero() ],
            [ T::zero(),    T::zero(),  (far + near) * nf,      -T::one() ],
            [ T::zero(),    T::zero(),  two * far * near * nf,  T::zero() ],
        ])
    }
}

/// 4x4 Matrices.
impl<T> Matrix<T> for Mat4<T> where T: Debug + Float + NumCast {
    fn set<M>(self: &mut Self, other: M) -> &mut Self where Mat4Type<T>: From<M> {
        self.0 = other.into();
        self
    }
    fn translate<V>(self: &mut Self, translation_vector: V) -> &mut Self where V: Vector<T> {
        self.0.translate(translation_vector);
        self
    }
    fn scale<V>(self: &mut Self, scaling_vector: V) -> &mut Self where V: Vector<T> {
        self.0.scale(scaling_vector);
        self
    }
    fn scale_at<P, V>(self: &mut Self, position: P, scaling_vector: V) -> &mut Self where P: Vector<T>, V: Vector<T> {
        self.0.scale_at(position, scaling_vector);
        self
    }
    fn rotate(self: &mut Self, radians: T) -> &mut Self {
        self.0.rotate(radians);
        self
    }
    fn rotate_at<P>(self: &mut Self, position: P, radians: T) -> &mut Self where P: Vector<T> {
        self.0.rotate_at(position, radians);
        self
    }
    fn rotate_axis<V>(self: &mut Self, radians: T, axis: V) -> &mut Self where Vec3<T>: From<V> {
        self.0.rotate_axis(radians, axis);
        self
    }
    fn rotate_axis_at<P, V>(self: &mut Self, position: P, radians: T, axis: V) -> &mut Self where P: Vector<T>, Vec3<T>: From<V> {
        self.0.rotate_axis_at(position, radians, axis);
        self
    }
    fn get_rotation(self: &Self) -> Self {
        let rot = self.0.get_rotation();
        Mat4(rot)
    }
    fn get_translation(self: &Self) -> Vec3<T> {
        self.0.get_translation()
    }
    fn get_scale(self: &Self) -> Vec3<T> {
        self.0.get_scale()
    }
    fn get_euler(self: &Self) -> Vec3<T> {
        self.0.get_euler()
    }
}

impl<T: Debug + Float> Mul<T> for Mat4<T> {
    type Output = Mat4<T>;
    fn mul(self, other: T) -> Mat4<T> {
        let a = self.0;
        let b = other;
        Mat4([
            [ a[0][0] * b, a[0][1] * b, a[0][2] * b, a[0][3] * b ],
            [ a[1][0] * b, a[1][1] * b, a[1][2] * b, a[1][3] * b ],
            [ a[2][0] * b, a[2][1] * b, a[2][2] * b, a[2][3] * b ],
            [ a[3][0] * b, a[3][1] * b, a[3][2] * b, a[3][3] * b ],
        ])
    }
}

impl<T> Mul<Vec2<T>> for Mat4<T> where T: Debug + Float {
    type Output = Vec2<T>;
    /// Multiplies the matrix with given vector operand, using 0 as z-component and 1 as w-component of the vector.
    fn mul(self, other: Vec2<T>) -> Vec2<T> {
        let mat = self.0;
        Vec2(
            other.0 * mat[0][0] + other.1 * mat[1][0] + mat[2][0] + mat[3][0],
            other.0 * mat[0][1] + other.1 * mat[1][1] + mat[2][0] + mat[3][1]
        )
    }
}

impl<T> Mul<Vec3<T>> for Mat4<T> where T: Debug + Float {
    type Output = Vec3<T>;
    /// Multiplies the matrix with given vector operand using 1 as w-component of the vector.
    fn mul(self, other: Vec3<T>) -> Vec3<T> {
        let mat = self.0;
        Vec3::<T>(
            other.0 * mat[0][0] + other.1 * mat[1][0] + other.2 * mat[2][0] + mat[3][0],
            other.0 * mat[0][1] + other.1 * mat[1][1] + other.2 * mat[2][1] + mat[3][1],
            other.0 * mat[0][2] + other.1 * mat[1][2] + other.2 * mat[2][2] + mat[3][2]
        )
    }
}

impl<T: Debug + Float> Mul<Mat4<T>> for Mat4<T> {
    type Output = Mat4<T>;
    fn mul(self, other: Mat4<T>) -> Mat4<T> {
        let a = self.0;
        let b = other.0;
        Mat4([
            [
                a[0][0]*b[0][0] + a[1][0]*b[0][1] + a[2][0]*b[0][2] + a[3][0]*b[0][3],
                a[0][1]*b[0][0] + a[1][1]*b[0][1] + a[2][1]*b[0][2] + a[3][1]*b[0][3],
                a[0][2]*b[0][0] + a[1][2]*b[0][1] + a[2][2]*b[0][2] + a[3][2]*b[0][3],
                a[0][3]*b[0][0] + a[1][3]*b[0][1] + a[2][3]*b[0][2] + a[3][3]*b[0][3],
            ],
            [
                a[0][0]*b[1][0] + a[1][0]*b[1][1] + a[2][0]*b[1][2] + a[3][0]*b[1][3],
                a[0][1]*b[1][0] + a[1][1]*b[1][1] + a[2][1]*b[1][2] + a[3][1]*b[1][3],
                a[0][2]*b[1][0] + a[1][2]*b[1][1] + a[2][2]*b[1][2] + a[3][2]*b[1][3],
                a[0][3]*b[1][0] + a[1][3]*b[1][1] + a[2][3]*b[1][2] + a[3][3]*b[1][3],
            ],
            [
                a[0][0]*b[2][0] + a[1][0]*b[2][1] + a[2][0]*b[2][2] + a[3][0]*b[2][3],
                a[0][1]*b[2][0] + a[1][1]*b[2][1] + a[2][1]*b[2][2] + a[3][1]*b[2][3],
                a[0][2]*b[2][0] + a[1][2]*b[2][1] + a[2][2]*b[2][2] + a[3][2]*b[2][3],
                a[0][3]*b[2][0] + a[1][3]*b[2][1] + a[2][3]*b[2][2] + a[3][3]*b[2][3],
            ],
            [
                a[0][0]*b[3][0] + a[1][0]*b[3][1] + a[2][0]*b[3][2] + a[3][0]*b[3][3],
                a[0][1]*b[3][0] + a[1][1]*b[3][1] + a[2][1]*b[3][2] + a[3][1]*b[3][3],
                a[0][2]*b[3][0] + a[1][2]*b[3][1] + a[2][2]*b[3][2] + a[3][2]*b[3][3],
                a[0][3]*b[3][0] + a[1][3]*b[3][1] + a[2][3]*b[3][2] + a[3][3]*b[3][3],
            ],
        ])
    }
}

impl<T: Debug + Float> Debug for Mat4<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = self.0;
        write!(f, "Mat4(
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
)",
a[0][0], a[0][1], a[0][2], a[0][3],
a[1][0], a[1][1], a[1][2], a[1][3],
a[2][0], a[2][1], a[2][2], a[2][3],
a[3][0], a[3][1], a[3][2], a[3][3])
    }
}