use num::traits::Float;
use std::fmt;
use maths::vec3::Vec3;
use maths::VecType;
use glium::uniforms::{AsUniformValue, UniformValue};

#[derive(Copy, Clone, Debug)]
pub struct Mat4<T: Copy + fmt::Display + Float> {
    d: [ T; 16 ],
}

impl<T: Copy + fmt::Display + Float> Mat4<T> {

    pub fn new() -> Mat4<T> {
        Mat4::<T> {
            d: [ T::zero(); 16 ]
        }
    }

    pub fn new_identity() -> Mat4<T> {
        Mat4::<T> {
            d: [
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

    pub fn identity(&mut self) -> &mut Self {

        self.d[0] = T::one();
        self.d[1] = T::zero();
        self.d[2] = T::zero();
        self.d[3] = T::zero();
        self.d[4] = T::zero();
        self.d[5] = T::one();
        self.d[6] = T::zero();
        self.d[7] = T::zero();
        self.d[8] = T::zero();
        self.d[9] = T::zero();
        self.d[10] = T::one();
        self.d[11] = T::zero();
        self.d[12] = T::zero();
        self.d[13] = T::zero();
        self.d[14] = T::zero();
        self.d[15] = T::one();

        self
    }

    pub fn translate<Vector: VecType<T>>(&mut self, v: Vector) -> &mut Self {

        let Vec3::<T>(x, y, z) = v.as_vec3();

        self.d[12] = self.d[0] * x + self.d[4] * y + self.d[8] * z + self.d[12];
        self.d[13] = self.d[1] * x + self.d[5] * y + self.d[9] * z + self.d[13];
        self.d[14] = self.d[2] * x + self.d[6] * y + self.d[10] * z + self.d[14];
        self.d[15] = self.d[3] * x + self.d[7] * y + self.d[11] * z + self.d[15];

        self
    }

    pub fn scale<Vector: VecType<T>>(&mut self, v: Vector) -> &mut Self {

        let Vec3::<T>(x, y, z) = v.as_vec3();

        self.d[0]  = self.d[0]  * x;
        self.d[1]  = self.d[1]  * x;
        self.d[2]  = self.d[2]  * x;
        self.d[3]  = self.d[3]  * x;
        self.d[4]  = self.d[4]  * y;
        self.d[5]  = self.d[5]  * y;
        self.d[6]  = self.d[6]  * y;
        self.d[7]  = self.d[7]  * y;
        self.d[8]  = self.d[8]  * z;
        self.d[9]  = self.d[9]  * z;
        self.d[10] = self.d[10] * z;
        self.d[11] = self.d[11] * z;

        self
    }

    pub fn rotate<Vector: VecType<T>>(&mut self, rad: T, axis: Vector) -> &mut Self {

        let Vec3::<T>(mut x, mut y, mut z) = axis.as_vec3();

        let mut len: T = (x * x + y * y + z * z).sqrt();

        if len.is_normal() {
            return self;
        }

        len = T::one() / len;
        x = x * len;
        y = y * len;
        z = z * len;

        let s: T = rad.sin();
        let c: T = rad.cos();
        let t: T = T::one() - c;

        let a00 = self.d[0];
        let a01 = self.d[1];
        let a02 = self.d[2];
        let a03 = self.d[3];
        let a10 = self.d[4];
        let a11 = self.d[5];
        let a12 = self.d[6];
        let a13 = self.d[7];
        let a20 = self.d[8];
        let a21 = self.d[9];
        let a22 = self.d[10];
        let a23 = self.d[11];

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
        self.d[0] = a00 * b00 + a10 * b01 + a20 * b02;
        self.d[1] = a01 * b00 + a11 * b01 + a21 * b02;
        self.d[2] = a02 * b00 + a12 * b01 + a22 * b02;
        self.d[3] = a03 * b00 + a13 * b01 + a23 * b02;
        self.d[4] = a00 * b10 + a10 * b11 + a20 * b12;
        self.d[5] = a01 * b10 + a11 * b11 + a21 * b12;
        self.d[6] = a02 * b10 + a12 * b11 + a22 * b12;
        self.d[7] = a03 * b10 + a13 * b11 + a23 * b12;
        self.d[8] = a00 * b20 + a10 * b21 + a20 * b22;
        self.d[9] = a01 * b20 + a11 * b21 + a21 * b22;
        self.d[10] = a02 * b20 + a12 * b21 + a22 * b22;
        self.d[11] = a03 * b20 + a13 * b21 + a23 * b22;

        self
    }

    pub fn rotate_z(&mut self, rad: T) -> &mut Self {

        let s = rad.sin();
        let c = rad.cos();
        let a00 = self.d[0];
        let a01 = self.d[1];
        let a02 = self.d[2];
        let a03 = self.d[3];
        let a10 = self.d[4];
        let a11 = self.d[5];
        let a12 = self.d[6];
        let a13 = self.d[7];

        // Perform axis-specific matrix multiplication
        self.d[0] = a00 * c + a10 * s;
        self.d[1] = a01 * c + a11 * s;
        self.d[2] = a02 * c + a12 * s;
        self.d[3] = a03 * c + a13 * s;
        self.d[4] = a10 * c - a00 * s;
        self.d[5] = a11 * c - a01 * s;
        self.d[6] = a12 * c - a02 * s;
        self.d[7] = a13 * c - a03 * s;

        self
    }

    pub fn rotate_z_at<Vector: VecType<T>>(&mut self, v: Vector, rad: T) -> &mut Self {
        let v3 = v.as_vec3();
        self.translate(v3)
            .rotate_z(rad)
            .translate(-v3);

        self
    }
}

impl AsUniformValue for Mat4<f32> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4([
            [ self.d[0],  self.d[1],  self.d[2],  self.d[3] ],
            [ self.d[4],  self.d[5],  self.d[6],  self.d[7] ],
            [ self.d[8],  self.d[9],  self.d[10], self.d[11] ],
            [ self.d[12], self.d[13], self.d[14], self.d[15] ],
        ])
    }
}

impl AsUniformValue for Mat4<f64> {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::DoubleMat4([
            [ self.d[0],  self.d[1],  self.d[2],  self.d[3] ],
            [ self.d[4],  self.d[5],  self.d[6],  self.d[7] ],
            [ self.d[8],  self.d[9],  self.d[10], self.d[11] ],
            [ self.d[12], self.d[13], self.d[14], self.d[15] ],
        ])
    }
}
