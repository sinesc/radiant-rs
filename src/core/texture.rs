use prelude::*;
use core::{RenderContext, Color, Uniform, AsUniform, RenderTarget, AsRenderTarget};
use core::builder::*;
use maths::Point2;
use backends::backend;

/// A texture to draw or draw to.
///
/// Textures serve as drawing targets for userdefined [`Postprocessors`](trait.Postprocessor.html)
/// or custom [`Programs`](struct.Program.html). A texture can also be drawn with
/// [`Renderer::rect()`](struct.Renderer.html#method.rect).
#[derive(Clone)]
pub struct Texture {
    pub(crate) handle   : Rc<backend::Texture2d>,
    pub(crate) minify   : TextureFilter,
    pub(crate) magnify  : TextureFilter,
    pub(crate) wrap     : TextureWrap,
    dimensions          : Point2<u32>,
}

impl Texture {
    /// Returns a textu builder for texture construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let rendercontext = renderer.context();
    /// let tex = Texture::builder(&rendercontext)
    ///                     .dimensions((640, 480))
    ///                     .magnify(TextureFilter::Nearest)
    ///                     .minify(TextureFilter::Linear)
    ///                     .build()
    ///                     .unwrap();
    /// ```
    pub fn builder(context: &RenderContext) -> TextureBuilder {
        TextureBuilder::new(context)
    }
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
    /// Clones texture with new filters and wrapping function. Both source and clone reference the same texture data.
    pub fn clone_with_options(self: &Self, minify: TextureFilter, magnify: TextureFilter, wrap: TextureWrap) -> Self {
        Texture {
            handle      : self.handle.clone(),
            minify      : minify,
            magnify     : magnify,
            wrap        : wrap,
            dimensions  : self.dimensions,
        }
    }
    /// Clears the texture with given color.
    pub fn clear(self: &Self, color: Color) {
        self.handle.clear(color);
    }
    /// Returns the dimensions of the texture.
    pub fn dimensions(self: &Self) -> Point2<u32> {
        self.dimensions
    }
    /// Creates a new texture from given TextureInfo struct.
    pub(crate) fn from_info(context: &RenderContext, info: TextureInfo) -> Self {
        let mut context = context.lock();
        let context = context.deref_mut();
        let texture = backend::Texture2d::new(context, &info);
        Texture {
            handle      : Rc::new(texture),
            minify      : info.minify,
            magnify     : info.magnify,
            wrap        : info.wrap,
            dimensions  : Point2(info.width, info.height),
        }
    }
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
