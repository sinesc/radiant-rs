use prelude::*;
use core::{display, rendercontext, RenderContext, Color, Uniform, AsUniform, RenderTarget, AsRenderTarget};
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

/// Internal texture format. Note that the shader will always see a floating
/// point representation. U[n]* will have their minimum value mapped to 0.0 and
/// their maximum to 1.0.
///
/// The following formats are recommended:
///
/// - `F16F16F16F16` for multipass, color gradiant heavy effects
/// - `F11F11F10` if no alpha channel is required
/// - `U8U8U8U8` for single pass drawing
#[derive(Copy, Clone, PartialEq)]
pub enum TextureFormat {
    U8,
    U16,
    U8U8,
    U16U16,
    U10U10U10,
    U12U12U12,
    U16U16U16,
    U2U2U2U2,
    U4U4U4U4,
    U5U5U5U1,
    U8U8U8U8,
    U10U10U10U2,
    U12U12U12U12,
    U16U16U16U16,
    I16I16I16I16,
    F16,
    F16F16,
    F16F16F16F16,
    F32,
    F32F32,
    F32F32F32F32,
    F11F11F10,
}

/// A struct used to describe a [`Texture`](struct.Texture.html) to be created via [`Texture::from_info()`](struct.Texture.html#method.from_info).
#[derive(Clone)]
pub struct TextureInfo {
    pub minify  : TextureFilter,
    pub magnify : TextureFilter,
    pub wrap    : TextureWrap,
    pub format  : TextureFormat,
    pub width   : u32,
    pub height  : u32,
}

impl Default for TextureInfo {
    fn default() -> TextureInfo {
        TextureInfo {
            minify  : TextureFilter::Linear,
            magnify : TextureFilter::Linear,
            wrap    : TextureWrap::Clamp,
            format  : TextureFormat::F16F16F16F16,
            width   : 1,
            height  : 1,
        }
   }
}

/// A texture to draw or draw to.
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
    /// Creates a new texture with given dimensions. The texture will use linear interpolation
    /// for magnification or minification and internally use the `F16F16F16F16` format.
    pub fn new(context: &RenderContext, width: u32, height: u32) -> Self {
        Self::from_info(context, TextureInfo {
            width: width,
            height: height,
            ..TextureInfo::default()
        })
    }
    /// Creates a new texture with given dimensions and filters. It will internally use the `F16F16F16F16` format.
    pub fn filtered(context: &RenderContext, width: u32, height: u32, minify: TextureFilter, magnify: TextureFilter) -> Self {
        Self::from_info(context, TextureInfo {
            width: width,
            height: height,
            minify: minify,
            magnify: magnify,
            ..TextureInfo::default()
        })
    }
    /// Creates a new texture from given TextureInfo struct.
    pub fn from_info(context: &RenderContext, info: TextureInfo) -> Self {
        let context = rendercontext::lock(context);

        let texture = glium::texture::Texture2d::empty_with_format(
            display::handle(&context.display),
            convert_format(info.format),
            glium::texture::MipmapsOption::NoMipmap,
            info.width,
            info.height,
        ).unwrap();

        texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

        Texture {
            handle  : Rc::new(texture),
            minify  : info.minify,
            magnify : info.magnify,
            wrap    : info.wrap,
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

/// Returns texture filters.
pub fn filters(texture: &Texture) -> (TextureFilter, TextureFilter, TextureWrap) {
    (texture.minify, texture.magnify, texture.wrap)
}

impl AsRenderTarget for Texture {
    fn as_render_target(self: &Self) -> RenderTarget {
        RenderTarget::Texture(self.clone())
    }
}

impl AsUniform for Texture {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Texture(self.clone())
    }
}

/// Converts TextureFormat to the supported gliums texture formats
fn convert_format(format: TextureFormat) -> glium::texture::UncompressedFloatFormat {
    use glium::texture::UncompressedFloatFormat as GF;
    use self::TextureFormat as RF;
    match format {
        RF::U8 => GF::U8,
        RF::U16 => GF::U16,
        RF::U8U8 => GF::U8U8,
        RF::U16U16 => GF::U16U16,
        RF::U10U10U10 => GF::U10U10U10,
        RF::U12U12U12 => GF::U12U12U12,
        RF::U16U16U16 => GF::U16U16U16,
        RF::U2U2U2U2 => GF::U2U2U2U2,
        RF::U4U4U4U4 => GF::U4U4U4U4,
        RF::U5U5U5U1 => GF::U5U5U5U1,
        RF::U8U8U8U8 => GF::U8U8U8U8,
        RF::U10U10U10U2 => GF::U10U10U10U2,
        RF::U12U12U12U12 => GF::U12U12U12U12,
        RF::U16U16U16U16 => GF::U16U16U16U16,
        RF::I16I16I16I16 => GF::I16I16I16I16,
        RF::F16 => GF::F16,
        RF::F16F16 => GF::F16F16,
        RF::F16F16F16F16 => GF::F16F16F16F16,
        RF::F32 => GF::F32,
        RF::F32F32 => GF::F32F32,
        RF::F32F32F32F32 => GF::F32F32F32F32,
        RF::F11F11F10 => GF::F11F11F10,
    }
}
