use prelude::*;
use core::{self, Context, Color, Uniform, AsUniform, RenderTarget, AsRenderTarget, Point2};
use core::builder::*;
use image::{self, GenericImage};
use backends::backend;

/// A texture to draw or draw to.
///
/// Textures serve as drawing targets for userdefined [`Postprocessors`](trait.Postprocessor.html)
/// or custom [`Programs`](struct.Program.html). A texture can also be drawn with
/// [`Renderer::rect()`](struct.Renderer.html#method.rect).
#[derive(Clone)]
pub struct Texture {
    pub(crate) handle       : Rc<backend::Texture2d>,
    pub(crate) minify       : TextureFilter,
    pub(crate) magnify      : TextureFilter,
    pub(crate) wrap         : TextureWrap,
    pub(crate) dimensions   : Point2<u32>,
}

impl Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Texture")
            .field("minify", &self.minify)
            .field("magnify", &self.magnify)
            .field("wrap", &self.wrap)
            .field("dimensions", &self.dimensions)
            .finish()
    }
}

impl Texture {
    /// Returns a texture builder for texture construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build().unwrap();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let context = display.context();
    /// let tex = Texture::builder(&context)
    ///                     .dimensions((640, 480))
    ///                     .magnify(TextureFilter::Nearest)
    ///                     .minify(TextureFilter::Linear)
    ///                     .build()
    ///                     .unwrap();
    /// ```
    pub fn builder(context: &Context) -> TextureBuilder {
        TextureBuilder::new(context)
    }
    /// Creates a new texture with given dimensions. The texture will use linear interpolation
    /// for magnification or minification and internally use the `F16F16F16F16` format.
    pub fn new(context: &Context, width: u32, height: u32) -> Self {
        Self::builder(context).width(width).height(height).build().unwrap()
    }
    /// Creates a new texture from given file.
    pub fn from_file(context: &Context, file: &str) -> core::Result<Self> {
        Self::builder(context).file(file).build()
    }
    /// Creates a new texture with given dimensions and filters. It will internally use the `F16F16F16F16` format.
    pub fn filtered(context: &Context, width: u32, height: u32, minify: TextureFilter, magnify: TextureFilter) -> Self {
        Self::builder(context).width(width).height(height).minify(minify).magnify(magnify).build().unwrap()
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
    /// Creates a new texture from given TextureBuilder.
    pub(crate) fn from_builder(mut builder: TextureBuilder) -> core::Result<Self> {
        let mut context = builder.context.lock();
        let context = context.deref_mut();
        if let Some(filename) = builder.file {
            let image = image::open(filename)?;
            builder.width = image.dimensions().0;
            builder.height = image.dimensions().1;
            builder.data = Some(core::RawFrame {
                data: core::convert_color(image.to_rgba()).into_raw(),
                width: builder.width,
                height: builder.height,
                channels: 4,
            });
        }
        let texture = backend::Texture2d::new(context.backend_context.as_ref().unwrap(), builder.width, builder.height, builder.format, builder.data);
        Ok(Texture {
            handle      : Rc::new(texture),
            minify      : builder.minify,
            magnify     : builder.magnify,
            wrap        : builder.wrap,
            dimensions  : (builder.width, builder.height),
        })
    }
}

impl AsRenderTarget for Texture {
    fn as_render_target(self: &Self) -> RenderTarget {
        RenderTarget::texture(self)
    }
}

impl AsUniform for Texture {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Texture(self.clone())
    }
}

/// Texture minify- or magnify filtering function.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureFilter {
    /// All nearby texels will be loaded and their values will be merged.
    Linear,
    /// The nearest texel will be loaded.
    Nearest,
}

/// Texture wrapping function.
#[derive(Copy, Clone, Debug, PartialEq)]
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
#[derive(Copy, Clone, Debug, PartialEq)]
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
