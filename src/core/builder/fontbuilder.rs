use core::{Result, Font, FontInfo, Context};

/// A font builder.
///
/// Obtained from [`Font::builder()`](../struct.Font.html#method.builder).
///
/// # Examples
///
/// ```rust
/// # use radiant_rs::*;
/// # let display = Display::builder().hidden().build().unwrap();
/// # let renderer = Renderer::new(&display).unwrap();
/// # let context = display.context();
/// let my_font = Font::builder(&context).family("Arial").size(16.0).build().unwrap();
/// ```
#[must_use]
#[derive(Clone)]
pub struct FontBuilder<'a> {
    info    : FontInfo,
    context : &'a Context,
    file    : Option<&'a str>,
}

impl<'a> FontBuilder<'a> {
    /// Sets a family for the font. The font will be retrieved from the operating system.
    /// Mutually exclusive with file().
    pub fn family(mut self: Self, family: &str) -> Self {
        self.info.family = family.to_string();
        self
    }
    /// Sets file for the Font to be loaded from.
    /// Mutually exclusive with family().
    pub fn file(mut self: Self, file: &'a str) -> Self {
        self.file = Some(file);
        self
    }
    /// Flags the font to be italic.
    pub fn italic(mut self: Self) -> Self {
        self.info.italic = true;
        self
    }
    /// Flags the font to be oblique.
    pub fn oblique(mut self: Self) -> Self {
        self.info.oblique = true;
        self
    }
    /// Flags the font to be monospace.
    pub fn monospace(mut self: Self) -> Self {
        self.info.monospace = true;
        self
    }
    /// Flags the font to be bold.
    pub fn bold(mut self: Self) -> Self {
        self.info.bold = true;
        self
    }
    /// Sets the fontsize.
    pub fn size(mut self: Self, size: f32) -> Self {
        self.info.size = size;
        self
    }
    /// Returns the constructed font instance.
    pub fn build(self: Self) -> Result<Font> {
        if let Some(file) = self.file {
            Font::from_file(self.context, file)
        } else {
            Font::from_info(self.context, self.info)
        }
    }
    // Creates a new FontBuilder instance.
    pub(crate) fn new<'b>(context: &'b Context) -> FontBuilder {
        FontBuilder {
            context : context,
            info    : FontInfo { ..FontInfo::default() },
            file    : None,
        }
    }
}
