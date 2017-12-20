use core::{FontInfo, Font};

/// A FontQueryBuilder builder, returned from Font::query().
#[must_use]
#[derive(Clone)]
pub struct FontQueryBuilder {
    info: FontInfo,
}

impl FontQueryBuilder {
    /// Sets a family for the Fonts.
    pub fn family(mut self: Self, family: &str) -> Self {
        self.info.family = family.to_string();
        self
    }
    /// Flags the Fonts to be italic.
    pub fn italic(mut self: Self) -> Self {
        self.info.italic = true;
        self
    }
    /// Flags the Fonts to be oblique.
    pub fn oblique(mut self: Self) -> Self {
        self.info.oblique = true;
        self
    }
    /// Flags the Fonts to be monospace.
    pub fn monospace(mut self: Self) -> Self {
        self.info.monospace = true;
        self
    }
    /// Returns a vector of matching font families.
    pub fn fetch(self: Self) -> Vec<String> {
        /*if let Some(file) = self.file {
            Font::from_file(self.context, file)
        } else {*/
            Font::query_specific(self.info)
        //}
    }
    /// Creates a new FontQueryBuilder instance.
    pub(crate) fn new() -> FontQueryBuilder {
        FontQueryBuilder {
            info: FontInfo { ..FontInfo::default() },
        }
    }
}
