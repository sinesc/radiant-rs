use prelude::*;
use core::{self, Layer, rendercontext, RenderContext, Color};
use maths::{Point2, Vec2, Rect};
use core::builder::*;
use rusttype;
use backends::backend;
use font_loader::system_fonts;

static FONT_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

/// A font used for writing on a [`Layer`](struct.Layer.html).
///
/// Use [`Font::builder()`](#method.builder) to create a new font from a registered system font or
/// a local file. The [`Font::from_file()`](#method.from_file) is a shortcut to achieve the latter.
///
/// In addition to the usual properties of a font, radiant also assigns a fixed size
/// to each font object. Instead of modifying this value, you can clone a new font
/// with a different size using [`Font::with_size()`](struct.Font.html#method.with_size).
#[derive(Clone)]
pub struct Font {
    data    : Vec<u8>,
    font_id : usize,
    size    : f32,
    context : RenderContext,
}

impl Font {

    /// Returns a [font builder](support/struct.FontBuilder.html) for font construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let rendercontext = renderer.context();
    /// let my_font = Font::builder(&rendercontext).family("Arial").size(16.0).build().unwrap();
    /// ```
    pub fn builder(context: &RenderContext) -> FontBuilder {
        create_fontbuilder(context)
    }

    /// Creates a font instance from a file.
    pub fn from_file(context: &RenderContext, file: &str) -> core::Result<Font> {
        use std::io::Read;
        let mut f = File::open(Path::new(file))?;
        let mut font_data = Vec::new();
        f.read_to_end(&mut font_data)?;
        Ok(Self::create(context, font_data, 12.0))
    }

    /// Returns the names of all available system fonts.
    pub fn query_all() -> Vec<String> {
        system_fonts::query_all()
    }

    /// Returns a query builder to retrieve the names of all available system fonts with the given properties (e.g. monospace).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// let monospace_fonts = Font::query().monospace().italic().fetch();
    /// ```
    pub fn query() -> FontQueryBuilder {
        create_fontquerybuilder()
    }

    /// Returns a new font instance with given size.
    pub fn with_size(self: &Self, size: f32) -> Font {
        let mut font = (*self).clone();
        font.size = size;
        font
    }

    /// Write to given layer.
    pub fn write<T>(self: &Self, layer: &Layer, text: &str, position: T, color: Color) -> &Font where Vec2<f32>: From<T> {
        let position = Vec2::from(position);
        self.write_paragraph(layer, text, position.0, position.1, 0.0, color, 0.0, 1.0, 1.0);
        self
    }

    /// Write to given layer. Breaks lines after max_width pixels.
    pub fn write_wrapped<T>(self: &Self, layer: &Layer, text: &str, position: T, color: Color, max_width: f32) -> &Font where Vec2<f32>: From<T> {
        let position = Vec2::from(position);
        self.write_paragraph(layer, text, position.0, position.1, max_width, color, 0.0, 1.0, 1.0);
        self
    }

    /// Write to given layer. Breaks lines after max_width pixels and applies given rotation and scaling.
    pub fn write_transformed<T, U>(self: &Self, layer: &Layer, text: &str, position: T, color: Color, max_width: f32, rotation: f32, scale: U) -> &Font where Vec2<f32>: From<T>+From<U> {
        let position = Vec2::from(position);
        let scale = Vec2::from(scale);
        self.write_paragraph(layer, text, position.0, position.1, max_width, color, rotation, scale.0, scale.1);
        self
    }

    /// Returns the font wrapped in an std::Arc.
    pub fn arc(self: Self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Returns the names of all available system fonts with the given properties (e.g. monospace).
    pub(crate) fn query_specific(info: FontInfo) -> Vec<String> {
        system_fonts::query_specific(&mut Self::build_property(&info))
    }

    /// Creates a new font instance from given FontInfo struct.
    pub(crate) fn from_info(context: &RenderContext, info: FontInfo) -> Font {
        let (font_data, _) = system_fonts::get(&Self::build_property(&info)).unwrap();
        Self::create(context, font_data, info.size)
    }

    /// Creates a new unique font
    fn create(context: &RenderContext, font_data: Vec<u8>, size: f32) -> Font {
        Font {
            data    : font_data,
            font_id : FONT_COUNTER.fetch_add(1, Ordering::Relaxed),
            size    : size,
            context : context.clone(),
        }
    }

    /// Write text to given layer using given font
    fn write_paragraph(self: &Self, layer: &Layer, text: &str, x: f32, y: f32, max_width: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) {

        // !todo probably expensive, but rusttype is completely opaque. would be nice to be able to store Font::info outside of a "may or may not own" container
        let rt_font = rusttype::FontCollection::from_bytes(&self.data[..]).into_font().unwrap();

        let bucket_id = 0;
        let glyphs = Self::layout_paragraph(&rt_font, rusttype::Scale::uniform(self.size), max_width, &text);
        let context = rendercontext::lock(&self.context);

        context.font_cache.queue(self.font_id, &glyphs);

        let anchor = Point2(0., 0.);
        let scale = Vec2(scale_x, scale_y);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        for glyph in &glyphs {
            if let Some((uv, pos, dim)) = context.font_cache.rect_for(self.font_id, glyph) {
                let dist_x = pos.0 * scale_x;
                let dist_y = pos.1 * scale_y;
                let offset_x = x + dist_x * cos_rot - dist_y * sin_rot;
                let offset_y = y + dist_x * sin_rot + dist_y * cos_rot;
                layer.add_rect(None, bucket_id, 0, 1, uv, Point2(offset_x, offset_y), anchor, dim, color, rotation, scale);
            }
        }
    }

    /// Layout a paragraph of glyphs
    fn layout_paragraph<'a>(font: &'a rusttype::Font, scale: rusttype::Scale, width: f32, text: &str) -> Vec<rusttype::PositionedGlyph<'a>> {

        use unicode_normalization::UnicodeNormalization;
        let mut result = Vec::new();
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = rusttype::point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;

        for c in text.nfc() {
            if c.is_control() {
                match c {
                    '\n' => {
                        caret = rusttype::point(0.0, caret.y + advance_height);
                    },
                    _ => {}
                }
                continue;
            }

            let base_glyph = if let Some(glyph) = font.glyph(c) {
                glyph
            } else {
                continue;
            };

            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }

            last_glyph_id = Some(base_glyph.id());
            let mut glyph = base_glyph.scaled(scale).positioned(caret);

            if let Some(bb) = glyph.pixel_bounding_box() {
                if width > 0.0 && bb.max.x > width as i32 {
                    caret = rusttype::point(0.0, caret.y + advance_height);
                    glyph = glyph.into_unpositioned().positioned(caret);
                    last_glyph_id = None;
                }
            }

            caret.x += glyph.unpositioned().h_metrics().advance_width;
            result.push(glyph);
        }
        result
    }

    /// Builds a FontProperty for the underlying system_fonts library
    fn build_property(info: &FontInfo) -> system_fonts::FontProperty {
        let mut property = system_fonts::FontPropertyBuilder::new();
        if info.family != "" {
            property = property.family(&info.family);
        }
        if info.italic {
            property = property.italic();
        }
        if info.oblique {
            property = property.oblique();
        }
        if info.bold {
            property = property.bold();
        }
        if info.monospace {
            property = property.monospace();
        }
        property.build()
    } 
}

/// A wrapper around rusttype's font cache.
pub struct FontCache {
    cache   : Mutex<rusttype::gpu_cache::Cache>,
    queue   : Mutex<Vec<(Rect<u32>, Vec<u8>)>>,
    dirty   : AtomicBool,
}

impl FontCache {

    /// Creates a new fontcache instant.
    pub fn new(width: u32, height: u32, scale_tolerance: f32, position_tolerance: f32) -> FontCache {
        FontCache {
            cache: Mutex::new(rusttype::gpu_cache::Cache::new(width, height, scale_tolerance, position_tolerance)),
            queue: Mutex::new(Vec::new()),
            dirty: AtomicBool::new(false),
        }
    }

    /// Queues a glyph for caching.
    pub fn queue(self: &Self, font_id: usize, glyphs: &[rusttype::PositionedGlyph]) {

        let mut cache = self.cache.lock().unwrap();
        let mut queue = self.queue.lock().unwrap();
        let mut dirties = false;

        for glyph in glyphs {
            cache.queue_glyph(font_id, glyph.clone());
        }

        cache.cache_queued(|rect, data| {
            queue.push( (Rect(Point2(rect.min.x, rect.min.y), Point2(rect.max.x, rect.max.y)), data.to_vec()) );
            dirties = true;
        }).unwrap();

        if dirties {
            self.dirty.store(dirties, Ordering::Relaxed);
        }
    }

    /// Updates the font cache texture.
    pub fn update(self: &Self, texture: &backend::Texture2d) {
        if self.dirty.load(Ordering::Relaxed) {
            let mut queue = self.queue.lock().unwrap();
            for &(ref rect, ref data) in queue.deref() {
                texture.write(rect, data);
            }
            queue.clear();
            self.dirty.store(false, Ordering::Relaxed);
        }
    }

    /// Returns a rectangle of uv coordinates for the given glyph as well as its offset and dimensions.
    pub fn rect_for(self: &Self, font_id: usize, glyph: &rusttype::PositionedGlyph) -> Option<(Rect, Point2, Point2)> {
        let cache = self.cache.lock().unwrap();
        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(font_id, glyph) {
            let uv = Rect::new(uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y);
            let pos = Point2(screen_rect.min.x as f32, screen_rect.min.y as f32);
            let dim = Point2((screen_rect.max.x - screen_rect.min.x) as f32, (screen_rect.max.y - screen_rect.min.y) as f32);
            Some((uv, pos, dim))
        } else {
            None
        }
    }
}

/// A struct used to filter the result of [`Font::query_specific()`](struct.Font.html#method.query_specific)
/// or to describe a [`Font`](struct.Font.html) to be created from a system font
/// via [`Font::from_info()`](struct.Font.html#method.from_info).
#[derive(Clone)]
pub struct FontInfo {
    pub italic      : bool,
    pub oblique     : bool,
    pub bold        : bool,
    pub monospace   : bool,
    pub family      : String,
    pub size        : f32,
}

impl Default for FontInfo {
    fn default() -> FontInfo {
        FontInfo {
            italic      : false,
            oblique     : false,
            bold        : false,
            monospace   : false,
            family      : "".to_string(),
            size        : 10.0,
        }
   }
}
