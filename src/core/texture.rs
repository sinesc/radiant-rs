use prelude::*;
use core::{display, rendercontext, RenderContext, RenderTarget, RenderTargetType, Color, Uniform, AsUniform};
use glium;
use glium::Surface;

/// Texture minify- or magnify filtering function.
#[derive(Copy, Clone, PartialEq)]
pub enum TextureFilter {
    /// All nearby texels will be loaded and their values will be merged.
    Linear,
    /// The nearest texel will be loaded.
    Nearest,
}

/// Texture wrapping function.
#[derive(Copy, Clone, PartialEq)]
pub enum TextureWrap {
    /// Samples at coord x + 1 map to coord x.
    Repeat,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror,
    /// Samples at coord x + 1 map to coord 1.
    Clamp,
    /// Same as Mirror, but only for one repetition.
    MirrorClamp,
}

/// A texture.
///
/// Textures serve as drawing targets for userdefined [`Postprocessors`](trait.Postprocessor.html)
/// or custom [`Programs`](struct.Program.html). A texture can also be drawn with
/// [`Renderer::draw_rect()`](struct.Renderer.html#method.draw_rect).
#[derive(Clone)]
pub struct Texture {
    handle  : Rc<glium::texture::Texture2d>,
    minify  : TextureFilter,
    magnify : TextureFilter,
    wrap    : TextureWrap,
}

impl Texture {
    /// Creates a new texture with given dimensions.
    pub fn new(context: &RenderContext, width: u32, height: u32) -> Self {
        Self::wrapped_and_filtered(context, width, height, TextureFilter::Linear, TextureFilter::Linear, TextureWrap::Clamp)
    }
    /// Creates a new texture with given dimensions and wrapping function.
    pub fn wrapped(context: &RenderContext, width: u32, height: u32, wrap: TextureWrap) -> Self {
        Self::wrapped_and_filtered(context, width, height, TextureFilter::Linear, TextureFilter::Linear, wrap)
    }
    /// Creates a new texture with given dimensions and filters.
    pub fn filtered(context: &RenderContext, width: u32, height: u32, minify: TextureFilter, magnify: TextureFilter) -> Self {
        Self::wrapped_and_filtered(context, width, height, minify, magnify, TextureWrap::Clamp)
    }
    /// Creates a new texture with given dimensions and filters and wrapping function.
    pub fn wrapped_and_filtered(context: &RenderContext, width: u32, height: u32, minify: TextureFilter, magnify: TextureFilter, wrap: TextureWrap) -> Self {
        let context = rendercontext::lock(context);

        let texture = glium::texture::Texture2d::empty_with_format(
            display::handle(&context.display),
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height
        ).unwrap();

        Texture {
            handle  : Rc::new(texture),
            minify  : minify,
            magnify : magnify,
            wrap    : wrap,
        }
    }
    /// Clones texture with new filters and wrapping function. Both source and clone reference the same texture data.
    pub fn clone_with_options(self: &Self, minify: TextureFilter, magnify: TextureFilter, wrap: TextureWrap) -> Self {
        Texture {
            handle  : self.handle.clone(),
            minify  : minify,
            magnify : magnify,
            wrap    : wrap,
        }
    }
    /// Clears the texture with given color.
    pub fn clear(self: &Self, color: Color) {
        let Color(r, g, b, a) = color;
        self.handle.as_surface().clear_color(r, g, b, a);
    }
}

/// Returns the texture handle.
pub fn handle(texture: &Texture) -> &glium::texture::Texture2d {
    texture.handle.deref()
}

pub fn rc_handle(texture: &Texture) -> Rc<glium::texture::Texture2d> {
    texture.handle.clone()
}

/// Returns texture filters.
pub fn filters(texture: &Texture) -> (TextureFilter, TextureFilter, TextureWrap) {
    (texture.minify, texture.magnify, texture.wrap)
}

impl RenderTarget for Texture {
    fn get_target(self: &Self) -> RenderTargetType {
        RenderTargetType::Texture(self.handle.clone())
    }
}

impl AsUniform for Texture {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Texture(self.clone())
    }
}
