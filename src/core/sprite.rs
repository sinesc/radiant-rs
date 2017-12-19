use prelude::*;
use core::{self, renderer, Layer, rendercontext, RenderContext, RawFrame};
use maths::{Point2, Vec2, Rect};
use Color;
use image::{self, GenericImage};
use regex::Regex;

/// A sprite used for drawing on a [`Layer`](struct.Layer.html).
///
/// Sprites are created from spritesheets containing one or more frames. To determine frame
/// dimensions, [`Sprite::from_file()`](#method.from_file) expects sprite sheet file names to
/// follow a specific pattern. (Future versions will add more configurable means to load sprites.)
#[derive(Clone)]
pub struct Sprite {
    anchor  : Point2<f32>,
    data    : Arc<SpriteData>,
}

pub struct SpriteData {
    width           : u16,
    height          : u16,
    pub num_frames  : u16,
    pub components  : u8,
    bucket_id       : u8,
    pub texture_id  : AtomicUsize,
    pub generation  : AtomicUsize,
    uv_max          : Point2<f32>,
}

/// Sprite parameter layout type. Sprites are arranged either horizontally or
/// vertically on the the sprite sheet..
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

/// Sprite parameters as extracted from file name.
#[derive(Debug)]
pub struct SpriteParameters {
    dimensions  : (u32, u32),
    num_frames  : (u32, u32),
    components  : u32,
    inner_margin: u32,
    layout      : SpriteLayout
}

impl<'a> Sprite {

    /// Creates a new sprite texture. Given filename is expected to end
    /// on _<width>x<height>x<frames>.<extension>, e.g. asteroid_64x64x24.png.
    pub fn from_file(context: &RenderContext, file: &str) -> core::Result<Sprite> {
        let path = Path::new(file);
        let mut image = image::open(&path)?;
        let parameters = parse_parameters(image.dimensions(), path);
        let descriptor = build_raw_frames(&mut image, &parameters);
        Result::Ok(sprite_from_descriptor(context, descriptor))
    }

    /// Creates a new sprite texture.
    pub fn from_data(context: &RenderContext, data: &[u8], parameters: &SpriteParameters) -> core::Result<Sprite> {
        let mut image = image::load_from_memory(data)?;
        let descriptor = build_raw_frames(&mut image, parameters);
        Result::Ok(sprite_from_descriptor(context, descriptor))
    }

    /// Draws a sprite onto the given layer.
    pub fn draw<T>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color) -> &Self where Point2: From<T> {
        let bucket_id = self.data.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = Rect::new(0.0, 0.0, self.data.uv_max.0, self.data.uv_max.1);
        let dim = Point2(self.data.width as f32, self.data.height as f32);
        let scale = Vec2(1.0, 1.0);
        let generation = self.data.generation.load(Ordering::Relaxed);
        layer.add_rect(Some(generation), bucket_id, texture_id, self.data.components, uv, Point2::from(position), self.anchor, dim, color, 0.0, scale);
        self
    }

    /// Draws a sprite onto the given layer and applies given color, rotation and scaling.
    pub fn draw_transformed<T, U>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color, rotation: f32, scale: U) -> &Self where Point2: From<T>, Vec2: From<U> {
        let bucket_id = self.data.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = Rect::new(0.0, 0.0, self.data.uv_max.0, self.data.uv_max.1);
        let dim = Point2(self.data.width as f32, self.data.height as f32);
        let generation = self.data.generation.load(Ordering::Relaxed);
        layer.add_rect(Some(generation), bucket_id, texture_id, self.data.components, uv, Point2::from(position), self.anchor, dim, color, rotation, Vec2::from(scale));
        self
    }

    /// Defines the sprite origin. Defaults to (0.5, 0.5), meaning that the center of the
    /// sprite would be drawn at the coordinates given to [`Sprite::draw()`](#method.draw). Likewise, (0.0, 0.0)
    /// would mean that the sprite's top left corner would be drawn at the given coordinates.
    pub fn set_anchor(self: &mut Self, anchor: Point2) -> &Self {
        self.anchor = Point2(anchor.0 * self.data.width as f32, anchor.1 * self.data.height as f32);
        self
    }

    /// Returns the width of the sprite.
    pub fn width(self: &Self) -> u32 {
        self.data.width as u32
    }

    /// Returns the height of the sprite.
    pub fn height(self: &Self) -> u32 {
        self.data.height as u32
    }

    /// Returns the number of frames of the sprite.
    pub fn num_frames(self: &Self) -> u32 {
        self.data.num_frames as u32
    }

    /// Returns the sprite wrapped in an std::Arc
    pub fn arc(self: Self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Returns the texture id for given frame
    fn texture_id(self: &Self, frame_id: u32) -> u32 {
        self.data.texture_id.load(Ordering::Relaxed) as u32 + (frame_id % self.data.num_frames as u32) * (self.data.components as u32)
    }
}

/// Sprite details after processing.
struct SpriteDescriptor {
    bucket_id       : u32,
    texture_size    : u32,
    frame_width     : u32,
    frame_height    : u32,
    components      : u32,
    raw_frames      : Vec<RawFrame>,
}

/// Creates a sprite from given descriptor.
fn sprite_from_descriptor(context: &RenderContext, descriptor: SpriteDescriptor) -> Sprite {

    let SpriteDescriptor { bucket_id, texture_size, frame_width, frame_height, components, raw_frames } = descriptor;
    let num_frames = (raw_frames.len() as u32 / components) as u32;

    let mut context = rendercontext::lock(context);
    let texture_id = context.store_frames(bucket_id, raw_frames);

    let sprite_data = Arc::new(SpriteData {
        width       : frame_width as u16,
        height      : frame_height as u16,
        num_frames  : num_frames as u16,
        components  : components as u8,
        bucket_id   : bucket_id as u8,
        texture_id  : AtomicUsize::new(texture_id as usize),
        uv_max      : Point2(frame_width as f32 / texture_size as f32, frame_height as f32 / texture_size as f32),
        generation  : AtomicUsize::new(context.generation()),
    });

    context.store_sprite(bucket_id, Arc::downgrade(&sprite_data));

    Sprite {
        anchor: Point2(frame_width as f32 / 2.0, frame_height as f32 / 2.0),
        data: sprite_data,
    }
}

/// Builds a sprite descriptor containing sprite dimensions and raw frames.
fn build_raw_frames(image: &mut image::DynamicImage, sprite_parameters: &SpriteParameters) -> SpriteDescriptor {

    let SpriteParameters { dimensions: (frame_width, frame_height), num_frames, components, .. } = *sprite_parameters;
    let (bucket_id, texture_size) = renderer::bucket_info(frame_width, frame_height);
    let num_frames = num_frames.0 * num_frames.1;
    let mut raw_frames = Vec::new();

    for frame_id in 0..num_frames {
        for component in 0..components {
            raw_frames.push(build_raw_frame(image, sprite_parameters, frame_id, component, texture_size));
        }
    }

    // !todo #37340
    SpriteDescriptor { bucket_id: bucket_id, texture_size: texture_size, frame_width: frame_width, frame_height: frame_height, components: components, raw_frames: raw_frames }
}

/// Constructs a single RawFrame for a frame of a spritesheet
/// If neccessary, pads the image up to the next power of two
fn build_raw_frame(image: &mut image::DynamicImage, sprite_parameters: &SpriteParameters, frame_id: u32, component: u32, pad_size: u32) -> RawFrame {

    let SpriteParameters { dimensions: (frame_width, frame_height), .. } = *sprite_parameters;
    let (x, y) = get_frame_coordinates(sprite_parameters, frame_id, component);
    let subimage = image.crop(x, y, frame_width, frame_height);

    if frame_width != pad_size || frame_height != pad_size {

        // pad image if it doesn't match an available texture array size
        let mut dest = image::DynamicImage::new_rgba8(pad_size, pad_size);
        dest.copy_from(&subimage, 0, 0);
        RawFrame {
            data: convert_color(dest.to_rgba()).into_raw(),
            width: pad_size,
            height: pad_size,
        }

    } else {

        // perfect fit
        RawFrame {
            data: convert_color(subimage.to_rgba()).into_raw(),
            width: frame_width,
            height: frame_height,
        }
    }
}

/// Computes top/left frame coordinates for the given frame_id/component in a sprite-sheet
fn get_frame_coordinates(sprite_parameters: &SpriteParameters, frame_id: u32, component: u32) -> (u32, u32) {

    let SpriteParameters { dimensions: (frame_width, frame_height), inner_margin, num_frames, components, layout } = *sprite_parameters;

    assert!(frame_id < num_frames.0 * num_frames.1);
    assert!(component < components);

    if layout == SpriteLayout::HORIZONTAL {
        let per_line = num_frames.0;
        let column = frame_id % per_line;
        let row = frame_id / per_line;
        (column * (frame_width + inner_margin), (row + component) * (frame_height + inner_margin))
    } else {
        let per_row = num_frames.1;
        let row = frame_id / per_row;
        let column = frame_id % per_row;
        ((row + component) * (frame_width + inner_margin), column * (frame_height + inner_margin))
    }
}

/// Parses sprite-sheet filename for dimensions and frame count
fn parse_parameters(dimensions: (u32, u32), path: &Path) -> SpriteParameters {

    // e.g. mysprite_16x16x30.png (16x16, 30 frames)
    // mysprite_16x16x30x2.png (16x16, 30 frames, 2 components)
    // mysprite_16x16x30+1.png (16x16, inner margin of 1 px, 30 frames)
    // mysprite_16x16+1.png (16x16, inner margin of 1 px, all frames horizontally ordered)
    lazy_static! { static ref MATCHER: Regex = Regex::new(r"_(\d+)x(\d+)(?:x(\d+)(?:x(\d+))?)?(?:\+(\d+))?\.").unwrap(); }

    let filename = path.file_name().unwrap().to_str().unwrap();
    let captures = MATCHER.captures(filename);

    match captures {
        Some(captures) => {
            let frame_width = captures.at(1).unwrap().parse::<u32>().unwrap();
            let frame_height = captures.at(2).unwrap().parse::<u32>().unwrap();
            let frame_count = captures.at(3).unwrap_or("0").parse::<u32>().unwrap();
            let frame_channels = captures.at(4).unwrap_or("1").parse::<u32>().unwrap();
            let inner_margin = captures.at(5).unwrap_or("0").parse::<u32>().unwrap();
            let frame_layout = if frame_height == dimensions.1 || frame_count == 0 { SpriteLayout::HORIZONTAL } else { SpriteLayout::VERTICAL };

            // calculate frame counts if not provided
            let num_frames = if frame_count == 0 {
                let num_x = dimensions.0 as f32 / (frame_width + inner_margin) as f32 + (inner_margin as f32 / (frame_width + inner_margin) as f32);
                let num_y = dimensions.1 as f32 / (frame_height + inner_margin) as f32 + (inner_margin as f32 / (frame_height + inner_margin) as f32);
                assert!(num_x.fract() <= f32::EPSILON);
                assert!(num_y.fract() <= f32::EPSILON);
                (num_x as u32, num_y as u32)
            } else if frame_layout == SpriteLayout::HORIZONTAL {
                (frame_count, 1)
            } else {
                (1, frame_count)
            };

            SpriteParameters {
                dimensions  : (frame_width, frame_height),
                inner_margin: inner_margin,
                num_frames  : num_frames,
                components  : frame_channels,
                layout      : frame_layout
            }
        }
        None => SpriteParameters {
            dimensions  : dimensions,
            inner_margin: 0,
            num_frames  : (1, 1),
            components  : 1,
            layout      : SpriteLayout::HORIZONTAL
        }
    }
}

/// Converts Srgb to rgb and multiplies image color channels with alpha channel
fn convert_color(mut image: image::RgbaImage) -> image::RgbaImage {
    use palette::Rgb;
    use palette::pixel::Srgb;
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        let rgb = Rgb::from(Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0
        ));
        pixel[0] = (alpha * rgb.red * 255.0) as u8;
        pixel[1] = (alpha * rgb.green * 255.0) as u8;
        pixel[2] = (alpha * rgb.blue * 255.0) as u8;
    }
    image
}
