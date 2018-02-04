use prelude::*;
use super::{Vec3, Vector};

/// A 4x4 matrix.
pub type Mat4<T = f32> = [ [ T; 4 ]; 4 ];

/// Matrix-methods for 4x4 arrays.
pub trait Matrix<T: Debug + Float + NumCast> { 
    /// Sets the matrix value from another matrix.
    fn set<M>(self: &mut Self, other: M) -> &mut Self where Mat4<T>: From<M>;
    /// Translate matrix by given vector.
    fn translate<V>(self: &mut Self, translation_vector: V) -> &mut Self where V: Vector<T>;
    /// Scale matrix by given vector.
    fn scale<V>(self: &mut Self, scaling_vector: V) -> &mut Self where V: Vector<T>;
    /// Scales at given position.
    fn scale_at<P, V>(self: &mut Self, position: P, scaling_vector: V) -> &mut Self where P: Vector<T>, V: Vector<T>;
    /// Rotates the origin around z.
    fn rotate(self: &mut Self, radians: T) -> &mut Self;
    /// Rotates around z at given position.
    fn rotate_at<P>(self: &mut Self, position: P, radians: T) -> &mut Self where P: Vector<T>;
    /// Rotates around axis.
    fn rotate_axis<V>(self: &mut Self, radians: T, axis: V) -> &mut Self where Vec3<T>: From<V>;
    /// Rotates around axis at given position.
    fn rotate_axis_at<P, V>(self: &mut Self, position: P, radians: T, axis: V) -> &mut Self where P: Vector<T>, Vec3<T>: From<V>;
    /// Returns a pure rotation matrix for given matrix
    fn get_rotation(self: &Self) -> Self;
    /// Returns the matrix's translation vector.
    fn get_translation(self: &Self) -> Vec3<T>;
    /// Returns the matrix's scaling vector.
    fn get_scale(self: &Self) -> Vec3<T>;
    /// Get rotation matrix euler angles.
    fn get_euler(self: &Self) -> Vec3<T>;
}

impl<T> Matrix<T> for Mat4<T> where T: Debug + Float + NumCast {   
    fn set<M>(self: &mut Self, other: M) -> &mut Self where Mat4<T>: From<M> {
        *self = other.into();
        self
    }
    fn translate<V>(self: &mut Self, translation_vector: V) -> &mut Self where V: Vector<T> {
        let Vec3::<T>(x, y, z) = translation_vector.as_vec3(T::zero());
        self[3][0] = self[0][0]* x + self[1][0] * y + self[2][0] * z + self[3][0];
        self[3][1] = self[0][1]* x + self[1][1] * y + self[2][1] * z + self[3][1];
        self[3][2] = self[0][2]* x + self[1][2] * y + self[2][2] * z + self[3][2];
        self[3][3] = self[0][3]* x + self[1][3] * y + self[2][3] * z + self[3][3];
        self
    }
    fn scale<V>(self: &mut Self, scaling_vector: V) -> &mut Self where V: Vector<T> {
        let Vec3::<T>(x, y, z) = scaling_vector.as_vec3(T::one());
        self[0][0] = self[0][0] * x;
        self[0][1] = self[0][1] * x;
        self[0][2] = self[0][2] * x;
        self[0][3] = self[0][3] * x;
        self[1][0] = self[1][0] * y;
        self[1][1] = self[1][1] * y;
        self[1][2] = self[1][2] * y;
        self[1][3] = self[1][3] * y;
        self[2][0] = self[2][0] * z;
        self[2][1] = self[2][1] * z;
        self[2][2] = self[2][2] * z;
        self[2][3] = self[2][3] * z;
        self
    }
    fn scale_at<P, V>(self: &mut Self, position: P, scaling_vector: V) -> &mut Self where P: Vector<T>, V: Vector<T> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .scale(scaling_vector)
            .translate(-position)
    }
    fn rotate(self: &mut Self, radians: T) -> &mut Self {
        let s = radians.sin();
        let c = radians.cos();
        let a00 = self[0][0];
        let a01 = self[0][1];
        let a02 = self[0][2];
        let a03 = self[0][3];
        let a10 = self[1][0];
        let a11 = self[1][1];
        let a12 = self[1][2];
        let a13 = self[1][3];
        self[0][0] = a00 * c + a10 * s;
        self[0][1] = a01 * c + a11 * s;
        self[0][2] = a02 * c + a12 * s;
        self[0][3] = a03 * c + a13 * s;
        self[1][0] = a10 * c - a00 * s;
        self[1][1] = a11 * c - a01 * s;
        self[1][2] = a12 * c - a02 * s;
        self[1][3] = a13 * c - a03 * s;
        self
    }
    fn rotate_at<P>(self: &mut Self, position: P, radians: T) -> &mut Self where P: Vector<T> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .rotate(radians)
            .translate(-position)
    }
    fn rotate_axis<V>(self: &mut Self, radians: T, axis: V) -> &mut Self where Vec3<T>: From<V> {

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

        let a00 = self[0][0];
        let a01 = self[0][1];
        let a02 = self[0][2];
        let a03 = self[0][3];
        let a10 = self[1][0];
        let a11 = self[1][1];
        let a12 = self[1][2];
        let a13 = self[1][3];
        let a20 = self[2][0];
        let a21 = self[2][1];
        let a22 = self[2][2];
        let a23 = self[2][3];

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
        self[0][0] = a00 * b00 + a10 * b01 + a20 * b02;
        self[0][1] = a01 * b00 + a11 * b01 + a21 * b02;
        self[0][2] = a02 * b00 + a12 * b01 + a22 * b02;
        self[0][3] = a03 * b00 + a13 * b01 + a23 * b02;
        self[1][0] = a00 * b10 + a10 * b11 + a20 * b12;
        self[1][1] = a01 * b10 + a11 * b11 + a21 * b12;
        self[1][2] = a02 * b10 + a12 * b11 + a22 * b12;
        self[1][3] = a03 * b10 + a13 * b11 + a23 * b12;
        self[2][0] = a00 * b20 + a10 * b21 + a20 * b22;
        self[2][1] = a01 * b20 + a11 * b21 + a21 * b22;
        self[2][2] = a02 * b20 + a12 * b21 + a22 * b22;
        self[2][3] = a03 * b20 + a13 * b21 + a23 * b22;

        self
    }
    fn rotate_axis_at<P, V>(self: &mut Self, position: P, radians: T, axis: V) -> &mut Self where P: Vector<T>, Vec3<T>: From<V> {
        let position = position.as_vec3(T::zero());
        self.translate(position)
            .rotate_axis(radians, axis)
            .translate(-position)
    }
    fn get_rotation(self: &Self) -> Self {
        let Vec3(x, y, z) = self.get_scale();
        let a = self;
        [
            [ a[0][0] / x, a[0][1] / x, a[0][2] / x, T::zero()  ],
            [ a[1][0] / y, a[1][1] / y, a[1][2] / y, T::zero(), ],
            [ a[2][0] / z, a[2][1] / z, a[2][2] / z, T::zero(), ],
            [ T::zero(),   T::zero(),   T::zero(),   T::one(),  ],
        ]
    }
    fn get_translation(self: &Self) -> Vec3<T> {
        let a = self;
        Vec3(a[3][0], a[3][1], a[3][2])
    }
    fn get_scale(self: &Self) -> Vec3<T> {
        let a = self;
        let x = Vec3(a[0][0], a[0][1], a[0][2]);
        let y = Vec3(a[1][0], a[1][1], a[1][2]);
        let z = Vec3(a[2][0], a[2][1], a[2][2]);
        Vec3(x.len(), y.len(), z.len())
    }
    #[allow(non_snake_case)]
    fn get_euler(self: &Self) -> Vec3<T> {
        let a = self;
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
