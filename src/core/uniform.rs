use prelude::*;
use core::texture::Texture;

/// A uniform value.
///
/// Uniforms are values that can be passed to [`Programs`](struct.Program.html).
/// Various types also implement the [`AsUniform`](trait.AsUniform.html) trait
/// and can be directly used with [`Program::set_uniform()`](struct.Program.html#method.set_uniform).
#[derive(Clone, Debug)]
pub enum Uniform {
    Bool(bool),
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    Mat4([[f32; 4]; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Double(f64),
    DoubleMat4([[f64; 4]; 4]),
    DoubleVec2([f64; 2]),
    DoubleVec3([f64; 3]),
    DoubleVec4([f64; 4]),
    Texture(Texture),
}

/// A value usable as a uniform.
pub trait AsUniform {
    fn as_uniform(self: &Self) -> Uniform;
}

/// Multiple uniforms held by a program.
#[derive(Clone, Debug)]
pub struct UniformList (pub (crate) HashMap<String, Uniform>);

impl UniformList {
    /// Creates a new uniform list.
    pub fn new() -> Self {
        UniformList(HashMap::new())
    }
    /// Inserts a uniform into the list.
    pub fn insert(self: &mut Self, name: &str, uniform: Uniform) {
        self.0.insert(name.to_string(), uniform);
    }
    /// Removes a uniform from the list and returns whether it existed.
    pub fn remove(self: &mut Self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }
}

impl AsUniform for bool {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Bool(*self)
    }
}

impl AsUniform for i32 {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::SignedInt(*self)
    }
}

impl AsUniform for u32 {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::UnsignedInt(*self)
    }
}

impl AsUniform for f32 {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Float(*self)
    }
}

impl AsUniform for f64 {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Double(*self)
    }
}
