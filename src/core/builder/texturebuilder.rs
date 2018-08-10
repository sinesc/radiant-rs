use core::*;

/// A Texture builder.
#[must_use]
#[derive(Clone)]
pub struct TextureBuilder<'a> {
    pub(crate) minify  : TextureFilter,
    pub(crate) magnify : TextureFilter,
    pub(crate) wrap    : TextureWrap,
    pub(crate) format  : TextureFormat,
    pub(crate) width   : u32,
    pub(crate) height  : u32,
    pub(crate) file    : Option<&'a str>,
    pub(crate) data    : Option<RawFrame>,
    pub(crate) context : &'a Context,
}

impl<'a> TextureBuilder<'a> {
    /// Sets a width for the texture.
    pub fn width(mut self: Self, width: u32) -> Self {
        self.width = width;
        self
    }
    /// Sets a height for the texture.
    pub fn height(mut self: Self, height: u32) -> Self {
        self.height = height;
        self
    }
    /// Sets dimensions for the texture.
    pub fn dimensions<T>(mut self: Self, dimensions: T) -> Self where Point2<u32>: From<T> {
        let dimensions = Point2::<u32>::from(dimensions);
        self.width = dimensions.0;
        self.height = dimensions.1;
        self
    }
    /// Sets a minification filter for the texture.
    pub fn minify(mut self: Self, minify: TextureFilter) -> Self {
        self.minify = minify;
        self
    }
    /// Sets a magnification filter for the texture.
    pub fn magnify(mut self: Self, magnify: TextureFilter) -> Self {
        self.magnify = magnify;
        self
    }
    /// Sets a wrapping type for the texture.
    pub fn wrap(mut self: Self, wrap: TextureWrap) -> Self {
        self.wrap = wrap;
        self
    }
    /// Sets an internal format for the texture.
    pub fn format(mut self: Self, format: TextureFormat) -> Self {
        self.format = format;
        self
    }
    pub fn file(mut self: Self, file: &'a str) -> Self {
        self.file = Some(file);
        self
    }
    /// Returns the constructed Texture instance.
    pub fn build(self: Self) -> Result<Texture> {
        Texture::from_builder(self)
    }
    pub(crate) fn new<'b>(context: &'b Context) -> TextureBuilder {
        TextureBuilder {
            context : context,
            minify  : TextureFilter::Linear,
            magnify : TextureFilter::Linear,
            wrap    : TextureWrap::Clamp,
            format  : TextureFormat::F16F16F16F16,
            width   : 1,
            height  : 1,
            file    : None,
            data    : None,
        }
    }
}