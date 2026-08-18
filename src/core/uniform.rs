use crate::prelude::*;
use crate::core::texture::Texture;

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
///
/// Uniforms are kept in insertion order; re-inserting an existing name replaces the
/// value without changing its position.
#[derive(Clone, Debug)]
pub struct UniformList (pub (crate) Vec<(String, Uniform)>);

impl UniformList {
    /// Creates a new uniform list.
    pub fn new() -> Self {
        UniformList(Vec::new())
    }
    /// Inserts a uniform into the list.
    pub fn insert(self: &mut Self, name: &str, uniform: Uniform) {
        for entry in self.0.iter_mut() {
            if entry.0 == name {
                entry.1 = uniform;
                return;
            }
        }
        self.0.push((name.to_string(), uniform));
    }
    /// Removes a uniform from the list and returns whether it existed.
    pub fn remove(self: &mut Self, name: &str) -> bool {
        let len = self.0.len();
        self.0.retain(|(n, _)| n != name);
        self.0.len() != len
    }
    /// Returns the uniform with the given name.
    pub fn get(self: &Self, name: &str) -> Option<&Uniform> {
        self.0.iter().find(|(n, _)| n == name).map(|(_, u)| u)
    }
    /// Iterates (name, uniform) pairs in insertion order.
    pub fn iter(self: &Self) -> impl Iterator<Item = (&str, &Uniform)> {
        self.0.iter().map(|(n, u)| (n.as_str(), u))
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
