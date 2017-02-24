#[allow(deprecated)]
use core::{Result, Font, FontInfo, RenderContext};

/// A Font builder.
#[must_use]
#[allow(deprecated)]
pub struct FontBuilder<'a> {
    info    : FontInfo,
    context : &'a RenderContext,
    file    : Option<&'a str>,
}

#[allow(deprecated)]
pub fn create_fontbuilder<'a>(context: &'a RenderContext) -> FontBuilder {
    FontBuilder {
        context : context,
        info    : FontInfo { ..FontInfo::default() },
        file    : None,
    }
}

#[allow(deprecated)]
impl<'a> FontBuilder<'a> {
    /// Sets a family for the Font. The font will be retrieved from the operating system.
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
    /// Flags the Font to be italic.
    pub fn italic(mut self: Self) -> Self {
        self.info.italic = true;
        self
    }
    /// Flags the Font to be oblique.
    pub fn oblique(mut self: Self) -> Self {
        self.info.oblique = true;
        self
    }
    /// Flags the Font to be monospace.
    pub fn monospace(mut self: Self) -> Self {
        self.info.monospace = true;
        self
    }
    /// Sets the fontsize.
    pub fn size(mut self: Self, size: f32) -> Self {
        self.info.size = size;
        self
    }
    /// Returns the constructed Font instance.
    #[allow(deprecated)]
    pub fn build(self: Self) -> Result<Font> {
        if let Some(file) = self.file {
            Font::from_file(self.context, file)
        } else {
            Ok(Font::from_info(self.context, self.info))
        }
    }
}
