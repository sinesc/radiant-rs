use prelude::*;
use graphics::{layer, Layer, Point, Rect};
use Color;
use rusttype;
use glium;
use font_loader::system_fonts;

use std::borrow::Cow;

#[derive(Clone)]
pub struct FontInfo {
    pub italic      : bool,
    pub oblique     : bool,
    pub bold        : bool,
    pub monospace   : bool,
    pub family      : String,
}

impl Default for FontInfo {
    fn default() -> FontInfo {
        FontInfo {
            italic      : false,
            oblique     : false,
            bold        : false,
            monospace   : false,
            family      : "".to_string(),
        }
   }
}

pub struct FontCache {
    cache   : rusttype::gpu_cache::Cache,
    queue   : Vec<(rusttype::Rect<u32>, Vec<u8>)>,
}

impl FontCache {
    pub fn new(width: u32, height: u32, scale_tolerance: f32, position_tolerance: f32) -> FontCache {
        FontCache {
            cache: rusttype::gpu_cache::Cache::new(width, height, scale_tolerance, position_tolerance),
            queue: Vec::new(),
        }
    }

    pub fn queue(self: &mut Self, font_id: usize, glyphs: &[rusttype::PositionedGlyph]) {
        for glyph in glyphs {
            self.cache.queue_glyph(font_id, glyph.clone());
        }
        let queue = &mut self.queue;
        self.cache.cache_queued(|rect, data| {
            queue.push((rect, data.to_vec()));
        }).unwrap();
    }

    pub fn update(self: &mut Self, texture: &mut glium::texture::Texture2d) {
        for &(ref rect, ref data) in &self.queue {
            texture.main_level().write(
                glium::Rect {
                    left: rect.min.x,
                    bottom: rect.min.y,
                    width: rect.width(),
                    height: rect.height()
                },
                glium::texture::RawImage2d {
                    data: Cow::Borrowed(&data),
                    width: rect.width(),
                    height: rect.height(),
                    format: glium::texture::ClientFormat::U8
                }
            );
        }
        self.queue.clear();
    }

    pub fn needs_update(self: &Self) -> bool {
        self.queue.len() > 0
    }

    pub fn rect_for(self: &Self, font_id: usize, glyph: &rusttype::PositionedGlyph) -> Option<(Rect, Point, Point)> {
        if let Ok(Some((uv_rect, screen_rect))) = self.cache.rect_for(font_id, glyph) {
            let uv = Rect::new(uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y);
            let pos = Point::new(screen_rect.min.x as f32, screen_rect.min.y as f32);
            let dim = Point::new((screen_rect.max.x - screen_rect.min.x) as f32, (screen_rect.max.y - screen_rect.min.y) as f32);
            Some((uv, pos, dim))
        } else {
            None
        }
    }
}

static FONT_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

pub struct Font<'a> {
    font    : rusttype::Font<'a>,
    font_id : usize,
}

impl<'a> Font<'a> {

    pub fn enumerate_systemfonts() -> Vec<String> {
        system_fonts::query_all()
    }

    pub fn write(self: &Self, layer: &Layer, text: &str, x: f32, y: f32, size: f32, max_width: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {

        let bucket_id = 0;
        let glyphs = layout_paragraph(&self.font, rusttype::Scale::uniform(size), max_width, &text);
        let mut font_cache = layer.font_cache.lock().unwrap();

        font_cache.queue(self.font_id, &glyphs);

        let anchor = Point::new(0.0, 0.0);
        let scale = Point::new(scale_x, scale_y);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        for glyph in &glyphs {
            if let Some((uv, pos, dim)) = font_cache.rect_for(self.font_id, glyph) {
                let dist_x = pos.x * scale_x;
                let dist_y = pos.y * scale_y;
                let offset_x = x + dist_x * cos_rot - dist_y * sin_rot;
                let offset_y = y + dist_x * sin_rot + dist_y * cos_rot;
                layer::add_rect(layer, bucket_id, 0, uv, Point::new(offset_x, offset_y), anchor, dim, color, rotation, scale);
            }
        }

        self
    }
}

pub fn create_cache_texture(display: &glium::Display, width: u32, height: u32) -> glium::texture::Texture2d {
    glium::texture::Texture2d::with_format(
        display,
        glium::texture::RawImage2d {
            data: Cow::Owned(vec![128u8; width as usize * height as usize]),
            width: width,
            height: height,
            format: glium::texture::ClientFormat::U8
        },
        glium::texture::UncompressedFloatFormat::U8,
        glium::texture::MipmapsOption::NoMipmap
    ).unwrap()
}

/// insert given glyphs into given cache texture
pub fn update_cache_texture<'a>(layer: &Layer, texture: &mut glium::texture::Texture2d) {
    layer.font_cache.lock().unwrap().update(texture);
}

fn create_font<'a>(font_data: Vec<u8>) -> Font<'a> {
    Font {
        font    : rusttype::FontCollection::from_bytes(font_data).into_font().unwrap(),
        font_id : FONT_COUNTER.fetch_add(1, Ordering::Relaxed),
    }
}

pub fn create_font_from_file<'a>(file: &str) -> Font<'a> {
    let mut f = File::open(Path::new(file)).unwrap();
    let mut font_data = Vec::new();
    f.read_to_end(&mut font_data);
    create_font(font_data)
}

pub fn create_font_from_info<'a>(info: FontInfo) -> Font<'a> {
    let mut property = system_fonts::FontPropertyBuilder::new().family(&info.family);
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
    let (font_data, _) = system_fonts::get(&property.build()).unwrap();
    create_font(font_data)
}

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
                '\r' => {
                    caret = rusttype::point(0.0, caret.y + advance_height);
                }
                '\n' => {},
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
            if bb.max.x > width as i32 {
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
