use prelude::*;
use maths::vec3::Vec3;
use maths::VecType;
use glium::uniforms::{AsUniformValue, UniformValue};

/// A 4x4 matrix.
#[derive(Copy, Clone)]
pub struct Mat4<T: Copy + fmt::Debug + Float + FromPrimitive = f32> {
    data: [ T; 16 ],
}

const E00: usize = 0;
const E01: usize = 1;
const E02: usize = 2;
const E03: usize = 3;
const E10: usize = 4;
const E11: usize = 5;
const E12: usize = 6;
const E13: usize = 7;
const E20: usize = 8;
const E21: usize = 9;
const E22: usize = 10;
const E23: usize = 11;
const E30: usize = 12;
const E31: usize = 13;
const E32: usize = 14;
const E33: usize = 15;

impl<T: Copy + fmt::Debug + Float + FromPrimitive> Mat4<T> {

    /// Creates a zero matrix.
    pub fn new() -> Mat4<T> {
        Mat4::<T> {
            data: [ T::zero(); 16 ]
        }
    }

    /// Creates an identity matrix.
    pub fn identity() -> Mat4<T> {
        Mat4::<T> {
            data: [
                T::one(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::one(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::one(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::zero(),
                T::one(),
            ]
        }
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
            let a = &mut self.data;
            a[12] = a[0] * x + a[4] * y + a[8] * z + a[12];
            a[13] = a[1] * x + a[5] * y + a[9] * z + a[13];
            a[14] = a[2] * x + a[6] * y + a[10] * z + a[14];
            a[15] = a[3] * x + a[7] * y + a[11] * z + a[15];
        }
        self
    }

    /// Scale matrix by given vector.
    pub fn scale<Vector: VecType<T>>(&mut self, v: Vector) -> &mut Self {

        let Vec3::<T>(x, y, z) = v.as_vec3(T::one());
        {
            let a = &mut self.data;
            a[0]  = a[0]  * x;
            a[1]  = a[1]  * x;
            a[2]  = a[2]  * x;
            a[3]  = a[3]  * x;
            a[4]  = a[4]  * y;
            a[5]  = a[5]  * y;
            a[6]  = a[6]  * y;
            a[7]  = a[7]  * y;
            a[8]  = a[8]  * z;
            a[9]  = a[9]  * z;
            a[10] = a[10] * z;
            a[11] = a[11] * z;
        }
        self
    }

    /// Rotates the origin around given vector.
    pub fn rotate_axis<Vector: VecType<T>>(&mut self, rad: T, axis: Vector) -> &mut Self {

        let Vec3::<T>(mut x, mut y, mut z) = axis.as_vec3(T::zero());

        let mut len: T = (x * x + y * y + z * z).sqrt();

        if len.is_normal() == false {
            return self;
        }

        len = T::one() / len;
        x = x * len;
        y = y * len;
        z = z * len;

        let s: T = rad.sin();
        let c: T = rad.cos();
        let t: T = T::one() - c;

        let a00 = self.data[0];
        let a01 = self.data[1];
        let a02 = self.data[2];
        let a03 = self.data[3];
        let a10 = self.data[4];
        let a11 = self.data[5];
        let a12 = self.data[6];
        let a13 = self.data[7];
        let a20 = self.data[8];
        let a21 = self.data[9];
        let a22 = self.data[10];
        let a23 = self.data[11];

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
        self.data[0] = a00 * b00 + a10 * b01 + a20 * b02;
        self.data[1] = a01 * b00 + a11 * b01 + a21 * b02;
        self.data[2] = a02 * b00 + a12 * b01 + a22 * b02;
        self.data[3] = a03 * b00 + a13 * b01 + a23 * b02;
        self.data[4] = a00 * b10 + a10 * b11 + a20 * b12;
        self.data[5] = a01 * b10 + a11 * b11 + a21 * b12;
        self.data[6] = a02 * b10 + a12 * b11 + a22 * b12;
        self.data[7] = a03 * b10 + a13 * b11 + a23 * b12;
        self.data[8] = a00 * b20 + a10 * b21 + a20 * b22;
        self.data[9] = a01 * b20 + a11 * b21 + a21 * b22;
        self.data[10] = a02 * b20 + a12 * b21 + a22 * b22;
        self.data[11] = a03 * b20 + a13 * b21 + a23 * b22;

        self
    }

    /// Rotates the origin around z.
    pub fn rotate(&mut self, rad: T) -> &mut Self {

        let s = rad.sin();
        let c = rad.cos();
        let a00 = self.data[0];
        let a01 = self.data[1];
        let a02 = self.data[2];
        let a03 = self.data[3];
        let a10 = self.data[4];
        let a11 = self.data[5];
        let a12 = self.data[6];
        let a13 = self.data[7];

        // Perform axis-specific matrix multiplication
        self.data[0] = a00 * c + a10 * s;
        self.data[1] = a01 * c + a11 * s;
        self.data[2] = a02 * c + a12 * s;
        self.data[3] = a03 * c + a13 * s;
        self.data[4] = a10 * c - a00 * s;
        self.data[5] = a11 * c - a01 * s;
        self.data[6] = a12 * c - a02 * s;
        self.data[7] = a13 * c - a03 * s;

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
        for (k, v) in other.data.iter().enumerate() {
            self.data[k] = *v;
        }
    }

    /// Returns a pure rotation matrix for given matrix
    pub fn get_rotation(self: &Self) -> Mat4<T> {
        let result;
        {
            let Vec3(x, y, z) = self.get_scale();
            let a = &self.data;
            result = Mat4 {
                data: [
                    a[E00] / x,
                    a[E01] / x,
                    a[E02] / x,
                    T::zero(),
                    a[E10] / y,
                    a[E11] / y,
                    a[E12] / y,
                    T::zero(),
                    a[E20] / z,
                    a[E21] / z,
                    a[E22] / z,
                    T::zero(),

                    T::zero(),
                    T::zero(),
                    T::zero(),
                    T::one(),
                ]
            }
        }
        result
    }

    /// Returns the matrix's translation vector.
    pub fn get_translation(self: &Self) -> Vec3<T> {
        let a = &self.data;
        Vec3(a[E30], a[E31], a[E32])
    }

    /// Returns the matrix's scaling vector.
    pub fn get_scale(self: &Self) -> Vec3<T> {
        let a = &self.data;
        let x = Vec3(a[E00], a[E01], a[E02]);
        let y = Vec3(a[E10], a[E11], a[E12]);
        let z = Vec3(a[E20], a[E21], a[E22]);
        Vec3(x.len(), y.len(), z.len())
    }

    /// Get rotation matrix euler angles
    pub fn get_euler(self: &Self) -> Vec3<T> {
        use std;
        let a = &self.data;
        let y: T;
        let z: T;
        let x: T;

        let half_pi = T::from_f64(std::f64::consts::PI).unwrap() / (T::one()+T::one());

    	if a[E01] > T::from_f32(0.998).unwrap() { // singularity at north pole
    		y = a[E20].atan2(a[E22]);
    		z = half_pi;
    		x = T::zero();
    	} else if a[E01] < T::from_f32(-0.998).unwrap() { // singularity at south pole
    		y = a[E20].atan2(a[E22]);
    		z = -half_pi;
    		x = T::zero();
    	} else {
        	y = (-a[E02]).atan2(a[E00]);
            x = (-a[E21]).atan2(a[E11]);
        	z = a[E01].asin();
        }

        Vec3(x, y, z)
    }
}

impl<T: Copy + fmt::Debug + Float + FromPrimitive> Mul<T> for Mat4<T> {
    type Output = Mat4<T>;
    fn mul(self, other: T) -> Mat4<T> {
        let a = &self.data;
        let b = other;
        Mat4 {
            data: [
                a[E00] * b,
                a[E01] * b,
                a[E02] * b,
                a[E03] * b,
                a[E10] * b,
                a[E11] * b,
                a[E12] * b,
                a[E13] * b,
                a[E20] * b,
                a[E21] * b,
                a[E22] * b,
                a[E23] * b,
                a[E30] * b,
                a[E31] * b,
                a[E32] * b,
                a[E33] * b,
            ]
        }
    }
}

impl<T: Copy + fmt::Debug + Float + FromPrimitive> Mul<Mat4<T>> for Mat4<T> {
    type Output = Mat4<T>;
    fn mul(self, other: Mat4<T>) -> Mat4<T> {
        let a = &self.data;
        let b = &other.data;
        Mat4 {
            data: [
                a[E00]*b[E00] + a[E10]*b[E01] + a[E20]*b[E02] + a[E30]*b[E03],
                a[E01]*b[E00] + a[E11]*b[E01] + a[E21]*b[E02] + a[E31]*b[E03],
                a[E02]*b[E00] + a[E12]*b[E01] + a[E22]*b[E02] + a[E32]*b[E03],
                a[E03]*b[E00] + a[E13]*b[E01] + a[E23]*b[E02] + a[E33]*b[E03],

                a[E00]*b[E10] + a[E10]*b[E11] + a[E20]*b[E12] + a[E30]*b[E13],
                a[E01]*b[E10] + a[E11]*b[E11] + a[E21]*b[E12] + a[E31]*b[E13],
                a[E02]*b[E10] + a[E12]*b[E11] + a[E22]*b[E12] + a[E32]*b[E13],
                a[E03]*b[E10] + a[E13]*b[E11] + a[E23]*b[E12] + a[E33]*b[E13],

                a[E00]*b[E20] + a[E10]*b[E21] + a[E20]*b[E22] + a[E30]*b[E23],
                a[E01]*b[E20] + a[E11]*b[E21] + a[E21]*b[E22] + a[E31]*b[E23],
                a[E02]*b[E20] + a[E12]*b[E21] + a[E22]*b[E22] + a[E32]*b[E23],
                a[E03]*b[E20] + a[E13]*b[E21] + a[E23]*b[E22] + a[E33]*b[E23],

                a[E00]*b[E30] + a[E10]*b[E31] + a[E20]*b[E32] + a[E30]*b[E33],
                a[E01]*b[E30] + a[E11]*b[E31] + a[E21]*b[E32] + a[E31]*b[E33],
                a[E02]*b[E30] + a[E12]*b[E31] + a[E22]*b[E32] + a[E32]*b[E33],
                a[E03]*b[E30] + a[E13]*b[E31] + a[E23]*b[E32] + a[E33]*b[E33],
            ]
        }
    }
}

impl<T: Copy + fmt::Debug + Float + FromPrimitive> fmt::Debug for Mat4<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = self.data;
        write!(f, "Mat4(
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
  {:?}, {:?}, {:?}, {:?}
)",
a[E00], a[E01], a[E02], a[E03],
a[E10], a[E11], a[E12], a[E13],
a[E20], a[E21], a[E22], a[E23],
a[E30], a[E31], a[E32], a[E33])
    }
}

#[doc(hidden)]
impl AsUniformValue for Mat4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        let a = &self.data;
        UniformValue::Mat4([
            [ a[0],  a[1],  a[2],  a[3] ],
            [ a[4],  a[5],  a[6],  a[7] ],
            [ a[8],  a[9],  a[10], a[11] ],
            [ a[12], a[13], a[14], a[15] ],
        ])
    }
}

#[doc(hidden)]
impl AsUniformValue for Mat4<f64> {
    fn as_uniform_value(&self) -> UniformValue {
        let a = &self.data;
        UniformValue::DoubleMat4([
            [ a[0],  a[1],  a[2],  a[3] ],
            [ a[4],  a[5],  a[6],  a[7] ],
            [ a[8],  a[9],  a[10], a[11] ],
            [ a[12], a[13], a[14], a[15] ],
        ])
    }
}
