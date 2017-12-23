use prelude::*;
use maths::{Vec2, Vec3, VecType, Rect, Point2};
use core::{Uniform, AsUniform};

/// A 4x4 matrix.
#[derive(Copy, Clone)]
pub struct Mat4<T: Debug + Float + NumCast = f32> (pub [ [ T; 4 ]; 4 ]);

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
        let mut matrix = Mat4::<T>::identity();
        let two = T::one() + T::one();
        *matrix
            .translate(Vec3::<T>(-T::one(), T::one(), T::zero()))
            .scale(Vec3(two / width, -two / height, T::one()))
    }

    /// Creates a look-at matrix with the given eye position, target position, and up-vector.
    pub fn look_at<E, C, U>(eye: E, target: C, up: U) -> Mat4<T> where E: VecType<T>, C: VecType<T>, U: VecType<T> {

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
        let Rect(Point2(left, top), Point2(right, bottom)) = rectangle.into();
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
        let Rect(Point2(left, top), Point2(right, bottom)) = rectangle.into();
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

    /// Translate matrix by given vector.
    pub fn translate<V>(&mut self, translation_vector: V) -> &mut Self where V: VecType<T> {

        let Vec3::<T>(x, y, z) = translation_vector.as_vec3(T::zero());
        {
            let a = &mut self.0;
            a[3][0] = a[0][0]* x + a[1][0] * y + a[2][0] * z + a[3][0];
            a[3][1] = a[0][1]* x + a[1][1] * y + a[2][1] * z + a[3][1];
            a[3][2] = a[0][2]* x + a[1][2] * y + a[2][2] * z + a[3][2];
            a[3][3] = a[0][3]* x + a[1][3] * y + a[2][3] * z + a[3][3];
        }
        self
    }

    /// Scale matrix by given vector.
    pub fn scale<V>(&mut self, scaling_vector: V) -> &mut Self where V: VecType<T> {

        let Vec3::<T>(x, y, z) = scaling_vector.as_vec3(T::one());
        {
            let a = &mut self.0;
            a[0][0] = a[0][0] * x;
            a[0][1] = a[0][1] * x;
            a[0][2] = a[0][2] * x;
            a[0][3] = a[0][3] * x;
            a[1][0] = a[1][0] * y;
            a[1][1] = a[1][1] * y;
            a[1][2] = a[1][2] * y;
            a[1][3] = a[1][3] * y;
            a[2][0] = a[2][0] * z;
            a[2][1] = a[2][1] * z;
            a[2][2] = a[2][2] * z;
            a[2][3] = a[2][3] * z;
        }
        self
    }

    /// Scales at given position.
    pub fn scale_at<P, V>(&mut self, position: P, scaling_vector: V) -> &mut Self where P: VecType<T>, V: VecType<T> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .scale(scaling_vector)
            .translate(-position)
    }

    /// Rotates the origin around z.
    pub fn rotate(&mut self, radians: T) -> &mut Self {

        let s = radians.sin();
        let c = radians.cos();
        let a00 = self.0[0][0];
        let a01 = self.0[0][1];
        let a02 = self.0[0][2];
        let a03 = self.0[0][3];
        let a10 = self.0[1][0];
        let a11 = self.0[1][1];
        let a12 = self.0[1][2];
        let a13 = self.0[1][3];

        // Perform axis-specific matrix multiplication
        self.0[0][0] = a00 * c + a10 * s;
        self.0[0][1] = a01 * c + a11 * s;
        self.0[0][2] = a02 * c + a12 * s;
        self.0[0][3] = a03 * c + a13 * s;
        self.0[1][0] = a10 * c - a00 * s;
        self.0[1][1] = a11 * c - a01 * s;
        self.0[1][2] = a12 * c - a02 * s;
        self.0[1][3] = a13 * c - a03 * s;

        self
    }

    /// Rotates around z at given position.
    pub fn rotate_at<P>(&mut self, position: P, radians: T) -> &mut Self where P: VecType<T> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .rotate(radians)
            .translate(-position)
    }

    /// Rotates around axis.
    pub fn rotate_axis<V>(&mut self, radians: T, axis: V) -> &mut Self where Vec3<T>: From<V> {

        let Vec3::<T>(mut x, mut y, mut z) = axis.into();
        let mut len: T = (x * x + y * y + z * z).sqrt();

        if len == T::zero() || len.is_normal() == false {
            return self;
        }

        len = T::one() / len;
        x = x * len;
        y = y * len;
        z = z * len;

        let s: T = radians.sin();
        let c: T = radians.cos();
        let t: T = T::one() - c;

        let a00 = self.0[0][0];
        let a01 = self.0[0][1];
        let a02 = self.0[0][2];
        let a03 = self.0[0][3];
        let a10 = self.0[1][0];
        let a11 = self.0[1][1];
        let a12 = self.0[1][2];
        let a13 = self.0[1][3];
        let a20 = self.0[2][0];
        let a21 = self.0[2][1];
        let a22 = self.0[2][2];
        let a23 = self.0[2][3];

        // Construct the elements of the rotation matrix
        let b00 = x * x * t + c;
        let b01 = y * x * t + z * s;
        let b02 = z * x * t - y * s;
        let b10 = x * y * t - z * s;
        let b11 = y * y * t + c;
        let b12 = z * y * t + x * s;
        let b20 = x * z * t + y * s;
        let b21 = y * z * t - x * s;
        let b22 = z * z * t + c;

        // Perform rotation-specific matrix multiplication
        self.0[0][0] = a00 * b00 + a10 * b01 + a20 * b02;
        self.0[0][1] = a01 * b00 + a11 * b01 + a21 * b02;
        self.0[0][2] = a02 * b00 + a12 * b01 + a22 * b02;
        self.0[0][3] = a03 * b00 + a13 * b01 + a23 * b02;
        self.0[1][0] = a00 * b10 + a10 * b11 + a20 * b12;
        self.0[1][1] = a01 * b10 + a11 * b11 + a21 * b12;
        self.0[1][2] = a02 * b10 + a12 * b11 + a22 * b12;
        self.0[1][3] = a03 * b10 + a13 * b11 + a23 * b12;
        self.0[2][0] = a00 * b20 + a10 * b21 + a20 * b22;
        self.0[2][1] = a01 * b20 + a11 * b21 + a21 * b22;
        self.0[2][2] = a02 * b20 + a12 * b21 + a22 * b22;
        self.0[2][3] = a03 * b20 + a13 * b21 + a23 * b22;

        self
    }

    /// Rotates around axis at given position.
    pub fn rotate_axis_at<P, V>(&mut self, position: P, radians: T, axis: V) -> &mut Self where P: VecType<T>, Vec3<T>: From<V> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .rotate_axis(radians, axis)
            .translate(-position)
    }

    /// Sets the matrix value from another matrix.
    pub fn set(&mut self, other: Mat4<T>) -> &mut Self {
        for (k, v) in other.0.iter().enumerate() {
            self.0[k] = *v;
        }
        self
    }

    /// Returns a pure rotation matrix for given matrix
    pub fn get_rotation(self: &Self) -> Mat4<T> {
        let Vec3(x, y, z) = self.get_scale();
        let a = &self.0;
        Mat4([
            [ a[0][0] / x, a[0][1] / x, a[0][2] / x, T::zero()  ],
            [ a[1][0] / y, a[1][1] / y, a[1][2] / y, T::zero(), ],
            [ a[2][0] / z, a[2][1] / z, a[2][2] / z, T::zero(), ],
            [ T::zero(),   T::zero(),   T::zero(),   T::one(),  ],
        ])
    }

    /// Returns the matrix's translation vector.
    pub fn get_translation(self: &Self) -> Vec3<T> {
        let a = &self.0;
        Vec3(a[3][0], a[3][1], a[3][2])
    }

    /// Returns the matrix's scaling vector.
    pub fn get_scale(self: &Self) -> Vec3<T> {
        let a = &self.0;
        let x = Vec3(a[0][0], a[0][1], a[0][2]);
        let y = Vec3(a[1][0], a[1][1], a[1][2]);
        let z = Vec3(a[2][0], a[2][1], a[2][2]);
        Vec3(x.len(), y.len(), z.len())
    }

    /// Get rotation matrix euler angles
    #[allow(non_snake_case)]
    pub fn get_euler(self: &Self) -> Vec3<T> {
        let a = &self.0;
        let y: T;
        let z: T;
        let x: T;

        let half_PI = NumCast::from(f64::consts::PI / 2.0).unwrap();

    	if a[0][1] > NumCast::from(0.998).unwrap() { // singularity at north pole
    		y = a[2][0].atan2(a[2][2]);
    		z = half_PI;
    		x = T::zero();
    	} else if a[0][1] < NumCast::from(-0.998).unwrap() { // singularity at south pole
    		y = a[2][0].atan2(a[2][2]);
    		z = -half_PI;
    		x = T::zero();
    	} else {
        	y = (-a[0][2]).atan2(a[0][0]);
            x = (-a[2][1]).atan2(a[1][1]);
        	z = a[0][1].asin();
        }

        Vec3(x, y, z)
    }
}

impl<T> From<[ [T; 4]; 4 ]> for Mat4<T> where T: Debug + Float {
    fn from(source: [ [T; 4]; 4 ]) -> Self {
        Mat4(source)
    }
}

impl From<Mat4<f32>> for [ [f32; 4]; 4 ] {
    fn from(source: Mat4<f32>) -> Self {
        source.0
    }
}

impl From<Mat4<f64>> for [ [f64; 4]; 4 ] {
    fn from(source: Mat4<f64>) -> Self {
        source.0
    }
}

impl<'a> From<&'a Mat4<f32>> for [ [f32; 4]; 4 ] {
    fn from(source: &'a Mat4<f32>) -> Self {
        source.0
    }
}

impl<'a> From<&'a Mat4<f64>> for [ [f64; 4]; 4 ] {
    fn from(source: &'a Mat4<f64>) -> Self {
        source.0
    }
}

impl<T: Debug + Float> Mul<T> for Mat4<T> {
    type Output = Mat4<T>;
    fn mul(self, other: T) -> Mat4<T> {
        let a = &self.0;
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
        let a = &self.0;
        let b = &other.0;
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

impl AsUniform for Mat4<f32> {
    fn as_uniform(&self) -> Uniform {
        let a = &self.0;
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
        let a = &self.0;
        Uniform::DoubleMat4([
            [ a[0][0], a[0][1], a[0][2], a[0][3] ],
            [ a[1][0], a[1][1], a[1][2], a[1][3] ],
            [ a[2][0], a[2][1], a[2][2], a[2][3] ],
            [ a[3][0], a[3][1], a[3][2], a[3][3] ],
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
