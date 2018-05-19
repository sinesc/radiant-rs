use prelude::*;
use core::{self, Renderer, Layer, RenderContext, RawFrame};
use core::math::*;
use Color;
use image::{self, GenericImage};
use regex::Regex;

/// A sprite used for drawing on a [`Layer`](struct.Layer.html).
///
/// Sprites are created from spritesheets containing one or more frames. To determine frame
/// dimensions, [`Sprite::from_file()`](#method.from_file) expects sprite sheet file names to
/// follow a specific pattern. (Future versions will add more configurable means to load sprites.)
#[derive(Clone, Debug)]
pub struct Sprite {
    anchor  : Point2<f32>,
    data    : Arc<SpriteData>,
}

impl<'a> Sprite {

    /// Creates a new sprite texture. Given filename is expected to end
    /// on _<width>x<height>x<frames>.<extension>, e.g. asteroid_64x64x24.png.
    pub fn from_file(context: &RenderContext, file: &str) -> core::Result<Self> {
        let path = Path::new(file);
        let mut image = image::open(&path)?;
        let parameters = Self::parse_parameters(image.dimensions(), path);
        let descriptor = Self::build_raw_frames(&mut image, &parameters);
        Result::Ok(Self::new(context, descriptor))
    }

    /// Creates a new sprite texture.
    pub fn from_data(context: &RenderContext, data: &[u8], parameters: &SpriteParameters) -> core::Result<Self> {
        let mut image = image::load_from_memory(data)?;
        let descriptor = Self::build_raw_frames(&mut image, parameters);
        Result::Ok(Self::new(context, descriptor))
    }

    /// Draws a sprite onto the given layer.
    pub fn draw<T>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color) -> &Self where Point2: From<T> {
        let bucket_id = self.data.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = ((0.0, 0.0), (self.data.uv_max.0, self.data.uv_max.1));
        let dim = (self.data.width as f32, self.data.height as f32);
        let scale = (1.0, 1.0);
        let generation = self.data.generation.load(Ordering::Relaxed);
        layer.add_rect(Some(generation), bucket_id, texture_id, self.data.components, uv, Point2::from(position), self.anchor, dim, color, 0.0, scale);
        self
    }

    /// Draws a sprite onto the given layer and applies given color, rotation and scaling.
    pub fn draw_transformed<T, U>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color, rotation: f32, scale: U) -> &Self where Point2: From<T>, Point2: From<U> {
        let bucket_id = self.data.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = ((0.0, 0.0), (self.data.uv_max.0, self.data.uv_max.1));
        let dim = (self.data.width as f32, self.data.height as f32);
        let generation = self.data.generation.load(Ordering::Relaxed);
        layer.add_rect(Some(generation), bucket_id, texture_id, self.data.components, uv, Point2::from(position), self.anchor, dim, color, rotation, Point2::from(scale));
        self
    }

    /// Defines the sprite origin. Defaults to (0.5, 0.5), meaning that the center of the
    /// sprite would be drawn at the coordinates given to [`Sprite::draw()`](#method.draw). Likewise, (0.0, 0.0)
    /// would mean that the sprite's top left corner would be drawn at the given coordinates.
    pub fn set_anchor(self: &mut Self, anchor: Point2) -> &Self {
        self.anchor = (anchor.0 * self.data.width as f32, anchor.1 * self.data.height as f32);
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

    /// Creates a sprite from given descriptor.
    fn new(context: &RenderContext, descriptor: SpriteRawInfo) -> Self {

        let SpriteRawInfo { bucket_id, texture_size, frame_width, frame_height, components, raw_frames } = descriptor;
        let num_frames = (raw_frames.len() as u32 / components) as u32;

        let mut context = context.lock();
        let texture_id = context.store_frames(bucket_id, raw_frames);

        let sprite_data = Arc::new(SpriteData {
            width       : frame_width as u16,
            height      : frame_height as u16,
            num_frames  : num_frames as u16,
            components  : components as u8,
            bucket_id   : bucket_id as u8,
            texture_id  : AtomicUsize::new(texture_id as usize),
            uv_max      : (frame_width as f32 / texture_size as f32, frame_height as f32 / texture_size as f32),
            generation  : AtomicUsize::new(context.generation()),
        });

        context.store_sprite(bucket_id, Arc::downgrade(&sprite_data));

        Sprite {
            anchor: (frame_width as f32 / 2.0, frame_height as f32 / 2.0),
            data: sprite_data,
        }
    }

    /// Builds a sprite descriptor containing sprite dimensions and raw frames.
    fn build_raw_frames(image: &mut image::DynamicImage, sprite_parameters: &SpriteParameters) -> SpriteRawInfo {

        let SpriteParameters { dimensions: (frame_width, frame_height), num_frames, components, .. } = *sprite_parameters;
        let (bucket_id, texture_size) = Renderer::bucket_info(frame_width, frame_height);
        let num_frames = num_frames.0 * num_frames.1;
        let mut raw_frames = Vec::new();

        for frame_id in 0..num_frames {
            for component in 0..components {
                raw_frames.push(Self::build_raw_frame(image, sprite_parameters, frame_id, component, texture_size));
            }
        }

        SpriteRawInfo { bucket_id, texture_size, frame_width, frame_height, components, raw_frames }
    }

    /// Constructs a single RawFrame for a frame of a spritesheet
    /// If neccessary, pads the image up to the next power of two
    fn build_raw_frame(image: &mut image::DynamicImage, sprite_parameters: &SpriteParameters, frame_id: u32, component: u32, pad_size: u32) -> RawFrame {

        let SpriteParameters { dimensions: (frame_width, frame_height), .. } = *sprite_parameters;
        let (x, y) = Self::get_frame_coordinates(sprite_parameters, frame_id, component);
        let subimage = image.crop(x, y, frame_width, frame_height);

        if frame_width != pad_size || frame_height != pad_size {

            // pad image if it doesn't match an available texture array size
            let mut dest = image::DynamicImage::new_rgba8(pad_size, pad_size);
            dest.copy_from(&subimage, 0, 0);
            RawFrame {
                data: core::convert_color(dest.to_rgba()).into_raw(),
                width: pad_size,
                height: pad_size,
                channels: 4,
            }

        } else {

            // perfect fit
            RawFrame {
                data: core::convert_color(subimage.to_rgba()).into_raw(),
                width: frame_width,
                height: frame_height,
                channels: 4,
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
                let frame_width = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                let frame_height = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                let frame_count = captures.get(3).map_or("0", |m| m.as_str()).parse::<u32>().unwrap();
                let frame_channels = captures.get(4).map_or("1", |m| m.as_str()).parse::<u32>().unwrap();
                let inner_margin = captures.get(5).map_or("0", |m| m.as_str()).parse::<u32>().unwrap();
                let frame_layout = if frame_height == dimensions.1 || frame_count == 0 { SpriteLayout::HORIZONTAL } else { SpriteLayout::VERTICAL };

                // calculate frame counts if not provided
                let num_frames = if frame_count == 0 {
                    let num_x = dimensions.0 as f32 / (frame_width + inner_margin) as f32 + (inner_margin as f32 / (frame_width + inner_margin) as f32);
                    let num_y = dimensions.1 as f32 / (frame_height + inner_margin) as f32 + (inner_margin as f32 / (frame_height + inner_margin) as f32);
                    assert!(num_x.fract() <= f32::EPSILON, "bad sprite geometry: slicing image horizontally into frames resulted in non-integer frame-count");
                    assert!(num_y.fract() <= f32::EPSILON, "bad sprite geometry: slicing image vertically into frames resulted in non-integer frame-count");
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

    /// Returns the texture id for given frame
    fn texture_id(self: &Self, frame_id: u32) -> u32 {
        self.data.texture_id.load(Ordering::Relaxed) as u32 + (frame_id % self.data.num_frames as u32) * (self.data.components as u32)
    }
}

/// Internal sprite data. (Multiple) sprites can hold a reference to this.
#[derive(Debug)]
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
/// vertically on the the sprite sheet.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

/// Sprite parameters describe how individual sprites are positioned on a sprite sheet.
#[derive(Debug, Copy, Clone)]
pub struct SpriteParameters {
    /// Dimensions of the sprite sheet.
    pub dimensions  : Point2<u32>,
    /// Number of frames per line and row.
    pub num_frames  : Point2<u32>,
    /// Number of sprite-components in the sheet.
    pub components  : u32,
    /// Margin around each frame.
    pub inner_margin: u32,
    /// Layout of sprites within the sheet.
    pub layout      : SpriteLayout
}

/// Sprite details after processing.
struct SpriteRawInfo {
    bucket_id       : u32,
    texture_size    : u32,
    frame_width     : u32,
    frame_height    : u32,
    components      : u32,
    raw_frames      : Vec<RawFrame>,
}
