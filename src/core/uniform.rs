use prelude::*;
use glium;
use glium::uniforms::{Uniforms, AsUniformValue};
use core::texture::{self, Texture, TextureFilter, TextureWrap};

/// A uniform value.
#[derive(Clone)]
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
#[derive(Clone)]
pub struct UniformList (HashMap<String, Uniform>);

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

/// Creates a GliumUniformList from the given uniform list.
pub fn to_glium_uniforms(list: &UniformList) -> GliumUniformList {
    GliumUniformList::from_uniform_list(list)
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

// -----------------------------------------------------------------------------
// Below is a whole buch of crap to work around some lifetimes I don't want the
// user to have to deal with.
// Ideally, I want GliumUniformList to be a vector of glium UniformValues but ran into
// lifetime issues with that.
// !todo revisit this later. It should really be possible.
// !todo noticed that texture::handle did a & on the rcs deref, removing that fixed some issues.
// -----------------------------------------------------------------------------

/// !todo get rid of
pub enum GliumUniform<'a> {
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
    Texture2d(&'a glium::texture::Texture2d),
    Texture2dArray(&'a glium::texture::SrgbTexture2dArray),
    Sampled2d(glium::uniforms::Sampler<'a, glium::texture::Texture2d>),
}

/// A structure to implement gliums Uniforms trait on. This requires a lifetime
/// which I don't want in the public interface.
pub struct GliumUniformList<'a>(Vec<(&'a str, GliumUniform<'a>)>);

impl<'a> GliumUniformList<'a> {
    pub fn from_uniform_list(list: &'a UniformList) -> Self {
        let mut result = GliumUniformList(Vec::new());
        for (name, uniform) in list.0.iter() {
            result.add_uniform(name, uniform);
        }
        result
    }
    pub fn add(self: &mut Self, name: &'a str, uniform: GliumUniform<'a>) -> &mut Self {
        self.0.push((name, uniform));
        self
    }
    fn add_uniform(self: &mut Self, name: &'a str, uniform: &'a Uniform) {
        use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, SamplerWrapFunction};
        self.0.push((name, match *uniform {
            Uniform::Bool(val) => { GliumUniform::Bool(val) },
            Uniform::SignedInt(val) => { GliumUniform::SignedInt(val) },
            Uniform::UnsignedInt(val) => { GliumUniform::UnsignedInt(val) },
            Uniform::Float(val) => { GliumUniform::Float(val) },
            Uniform::Mat4(val) => { GliumUniform::Mat4(val) },
            Uniform::Vec2(val) => { GliumUniform::Vec2(val) },
            Uniform::Vec3(val) => { GliumUniform::Vec3(val) },
            Uniform::Vec4(val) => { GliumUniform::Vec4(val) },
            Uniform::Double(val) => { GliumUniform::Double(val) },
            Uniform::DoubleMat4(val) => { GliumUniform::DoubleMat4(val) },
            Uniform::DoubleVec2(val) => { GliumUniform::DoubleVec2(val) },
            Uniform::DoubleVec3(val) => { GliumUniform::DoubleVec3(val) },
            Uniform::DoubleVec4(val) => { GliumUniform::DoubleVec4(val) },
            Uniform::Texture(ref val) => {
                let (minify, magnify, wrap) = texture::filters(val);
                let glium_minify = if minify == TextureFilter::Linear { MinifySamplerFilter::Linear } else { MinifySamplerFilter::Nearest };
                let glium_magnify = if magnify == TextureFilter::Linear { MagnifySamplerFilter::Linear } else { MagnifySamplerFilter::Nearest };
                let glium_wrap = match wrap {
                    TextureWrap::Repeat         => SamplerWrapFunction::Repeat,
                    TextureWrap::Mirror         => SamplerWrapFunction::Mirror,
                    TextureWrap::Clamp          => SamplerWrapFunction::Clamp,
                    TextureWrap::MirrorClamp    => SamplerWrapFunction::MirrorClamp,
                };
                GliumUniform::Sampled2d(
                    texture::handle(val)
                                .sampled()
                                .minify_filter(glium_minify)
                                .magnify_filter(glium_magnify)
                                .wrap_function(glium_wrap)
                )
            },
            //Uniform::Texture(ref val) => { GliumUniform::Texture2d(texture::handle(val)) },
        }));
    }
}

impl<'b> Uniforms for GliumUniformList<'b> {
    fn visit_values<'a, F>(self: &'a Self, mut output: F) where F: FnMut(&str, glium::uniforms::UniformValue<'a>) {
        use glium::uniforms::UniformValue;
        for &(name, ref uniform) in &self.0 {
            output(name, match *uniform {
                GliumUniform::Bool(val) => { UniformValue::Bool(val) },
                GliumUniform::SignedInt(val) => { UniformValue::SignedInt(val) },
                GliumUniform::UnsignedInt(val) => { UniformValue::UnsignedInt(val) },
                GliumUniform::Float(val) => { UniformValue::Float(val) },
                GliumUniform::Mat4(val) => { UniformValue::Mat4(val) },
                GliumUniform::Vec2(val) => { UniformValue::Vec2(val) },
                GliumUniform::Vec3(val) => { UniformValue::Vec3(val) },
                GliumUniform::Vec4(val) => { UniformValue::Vec4(val) },
                GliumUniform::Double(val) => { UniformValue::Double(val) },
                GliumUniform::DoubleMat4(val) => { UniformValue::DoubleMat4(val) },
                GliumUniform::DoubleVec2(val) => { UniformValue::DoubleVec2(val) },
                GliumUniform::DoubleVec3(val) => { UniformValue::DoubleVec3(val) },
                GliumUniform::DoubleVec4(val) => { UniformValue::DoubleVec4(val) },
                GliumUniform::Sampled2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2dArray(ref val) => {
                    val.as_uniform_value()
                }
            });
        }
    }
}
