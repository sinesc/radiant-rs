use prelude::*;
use maths::vec3::Vec3;
use maths::VecType;
use core::{Uniform, AsUniform};

/// A 4x4 matrix.
#[derive(Copy, Clone)]
pub struct Mat4<T: Debug + Float + NumCast = f32> ([ [ T; 4 ]; 4 ]);

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

    /// Translate matrix by given vector.
    pub fn translate<Vector: VecType<T>>(&mut self, v: Vector) -> &mut Self {

        let Vec3::<T>(x, y, z) = v.as_vec3(T::zero());
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
    pub fn scale<Vector: VecType<T>>(&mut self, v: Vector) -> &mut Self {

        let Vec3::<T>(x, y, z) = v.as_vec3(T::one());
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

    /// Rotates the origin around given vector.
    pub fn rotate_axis<Vector: VecType<T>>(&mut self, rad: T, axis: Vector) -> &mut Self {

        let Vec3::<T>(mut x, mut y, mut z) = axis.as_vec3(T::zero());

        let mut len: T = (x * x + y * y + z * z).sqrt();

        if len == T::zero() || len.is_normal() == false {
            return self;
        }

        len = T::one() / len;
        x = x * len;
        y = y * len;
        z = z * len;

        let s: T = rad.sin();
        let c: T = rad.cos();
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

    /// Rotates the origin around z.
    pub fn rotate(&mut self, rad: T) -> &mut Self {

        let s = rad.sin();
        let c = rad.cos();
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
    pub fn rotate_at<Vector: VecType<T>>(&mut self, position: Vector, radians: T) -> &mut Self {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .rotate(radians)
            .translate(-position);

        self
    }

    /// Scales at given position.
    pub fn scale_at<Vector: VecType<T>>(&mut self, position: Vector, scaling: Vector) -> &mut Self {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .scale(scaling)
            .translate(-position);

        self
    }

    /// Sets the matrix value from another matrix.
    pub fn set(&mut self, other: Mat4<T>) {
        for (k, v) in other.0.iter().enumerate() {
            self.0[k] = *v;
        }
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

    /// Returns the matrix as an array of 4 arrays of 4 Ts.
    pub fn as_array(self: &Self) -> [ [T; 4]; 4 ] {
        self.0
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
