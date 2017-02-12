use prelude::*;
use core::{self, renderer, layer, Layer, rendercontext, RenderContext, RenderContextTexture};
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
    /// Defines the sprite origin. Defaults to (0.5, 0.5), meaning that the center of the
    /// sprite would be drawn at the coordinates given to [`Sprite::draw()`](#method.draw). Likewise, (0.0, 0.0)
    /// would mean that the sprite's top left corner would be drawn at the given coordinates.
    pub anchor      : Point2,
    width           : f32,
    height          : f32,
    num_frames      : u32,
    num_channels    : u32,
    bucket_id       : u32,
    texture_id      : u32,
    u_max           : f32,
    v_max           : f32,
    context         : RenderContext,
}

/// Sprite parameter layout type. Sprites are arranged either horizontally or
/// vertically on the the sprite sheet..
#[derive(Copy, Clone, PartialEq)]
pub enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

/// Sprite parameters as extracted from file name.
pub struct SpriteParameters {
    dimensions  : (u32, u32),
    num_frames  : u32,
    num_channels: u32,
    layout      : SpriteLayout
}

/// Sprite details after processing.
struct SpriteDescriptor {
    bucket_id       : u32,
    texture_size    : u32,
    frame_width     : u32,
    frame_height    : u32,
    num_channels    : u32,
    raw_frames      : Vec<RenderContextTexture>,
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
    pub fn draw<T>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color) -> &Self where Vec2<f32>: From<T> {

        let layer_channel_id = layer::channel_id(&layer);

        if layer_channel_id < self.num_channels {
            let bucket_id = self.bucket_id;
            let texture_id = self.texture_id(frame_id) + self.num_frames * layer_channel_id;
            let uv = Rect::new(0.0, 0.0, self.u_max, self.v_max);
            let dim = Point2(self.width, self.height);
            let scale = Point2(1.0, 1.0);
            layer::add_rect(layer, bucket_id, texture_id, uv, Vec2::from(position), self.anchor, dim, color, 0.0, scale);
        }

        self
    }

    /// Draws a sprite onto the given layer and applies given color, rotation and scaling.
    pub fn draw_transformed<T, U>(self: &Self, layer: &Layer, frame_id: u32, position: T, color: Color, rotation: f32, scale: U) -> &Self where Vec2<f32>: From<T> + From<U> {

        let layer_channel_id = layer::channel_id(&layer);

        if layer_channel_id < self.num_channels {
            let bucket_id = self.bucket_id;
            let texture_id = self.texture_id(frame_id) + self.num_frames * layer_channel_id;
            let uv = Rect::new(0.0, 0.0, self.u_max, self.v_max);
            let anchor = Point2(self.anchor.0, self.anchor.1);
            let dim = Point2(self.width, self.height);
            layer::add_rect(layer, bucket_id, texture_id, uv, Vec2::from(position), anchor, dim, color, rotation, Vec2::from(scale));
        }

        self
    }

    /// Returns the width of the sprite.
    pub fn width(self: &Self) -> f32 {
        self.width
    }

    /// Returns the height of the sprite.
    pub fn height(self: &Self) -> f32 {
        self.height
    }

    /// Returns the number of frames of the sprite.
    pub fn num_frames(self: &Self) -> u32 {
        self.num_frames
    }

    /// Returns the texture id for given frame
    fn texture_id(self: &Self, frame_id: u32) -> u32 {
        self.texture_id + (frame_id % self.num_frames)
    }
}

/// Creates a sprite from given descriptor.
fn sprite_from_descriptor(context: &RenderContext, descriptor: SpriteDescriptor) -> Sprite {

    let SpriteDescriptor { bucket_id, texture_size, frame_width, frame_height, num_channels, raw_frames } = descriptor;
    let num_frames = (raw_frames.len() as u32 / num_channels) as u32;
    let texture_id = rendercontext::lock(context).store_frames(bucket_id, raw_frames);

    Sprite {
        width       : frame_width as f32,
        height      : frame_height as f32,
        num_frames  : num_frames,
        num_channels: num_channels,
        anchor      : Point2(0.5, 0.5),
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        u_max       : (frame_width as f32 / texture_size as f32),
        v_max       : (frame_height as f32 / texture_size as f32),
        context     : context.clone()
    }
}

/// Builds a sprite descriptor containing sprite dimensions and raw frames.
fn build_raw_frames(image: &mut image::DynamicImage, sprite_parameters: &SpriteParameters) -> SpriteDescriptor {

    let SpriteParameters { dimensions: (frame_width, frame_height), num_frames, num_channels, .. } = *sprite_parameters;
    let (bucket_id, texture_size) = renderer::bucket_info(frame_width, frame_height);
    let dimensions = image.dimensions();
    let mut raw_frames = Vec::new();

    for channel_id in 0..num_channels {
        for frame_id in 0..num_frames {
            raw_frames.push(build_raw_frame(image, dimensions, sprite_parameters, frame_id, channel_id, texture_size));
        }
    }

    // !todo #37340
    SpriteDescriptor { bucket_id: bucket_id, texture_size: texture_size, frame_width: frame_width, frame_height: frame_height, num_channels: num_channels, raw_frames: raw_frames }
}

/// Constructs a single RawFrame for a frame of a spritesheet
/// If neccessary, pads the image up to the next power of two
fn build_raw_frame(image: &mut image::DynamicImage, image_dimensions: (u32, u32), sprite_parameters: &SpriteParameters, frame_id: u32, channel_id: u32, pad_size: u32) -> RenderContextTexture {

    let SpriteParameters { dimensions: (frame_width, frame_height), .. } = *sprite_parameters;
    let (x, y) = get_frame_coordinates(image_dimensions, sprite_parameters, frame_id, channel_id);
    let subimage = image.crop(x, y, frame_width, frame_height);

    if frame_width != pad_size || frame_height != pad_size {

        // pad image if it doesn't match an available texture array size
        let mut dest = image::DynamicImage::new_rgba8(pad_size, pad_size);
        dest.copy_from(&subimage, 0, 0);
        RenderContextTexture {
            data: premultiply_alpha(dest.to_rgba()).into_raw(),
            width: pad_size,
            height: pad_size,
        }

    } else {

        // perfect fit
        RenderContextTexture {
            data: premultiply_alpha(subimage.to_rgba()).into_raw(),
            width: frame_width,
            height: frame_height,
        }
    }
}

/// Computes top/left frame coordinates for the given frame_id in a sprite-sheet
fn get_frame_coordinates(image_dimensions: (u32, u32), sprite_parameters: &SpriteParameters, frame_id: u32, channel_id: u32) -> (u32, u32) {

    let (img_width, img_height) = image_dimensions;
    let SpriteParameters { dimensions: (frame_width, frame_height), num_frames, num_channels, layout } = *sprite_parameters;

    assert!(frame_id < num_frames);
    assert!(channel_id < num_channels);

    if layout == SpriteLayout::HORIZONTAL {
        let per_line = img_width / frame_width;
        ((frame_id % per_line) * frame_width, (frame_id / per_line) * frame_height + channel_id  * frame_height)
    } else {
        let per_row = img_height / frame_height;
        ((frame_id / per_row) * frame_width + channel_id * frame_width, (frame_id % per_row) * frame_height)
    }
}

/// Multiplies image color channels with alpha channel
fn premultiply_alpha(mut image: image::RgbaImage) -> image::RgbaImage {
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        pixel[0] = (pixel[3] as u32 * pixel[0] as u32 / 255) as u8;
        pixel[1] = (pixel[3] as u32 * pixel[1] as u32 / 255) as u8;
        pixel[2] = (pixel[3] as u32 * pixel[2] as u32 / 255) as u8;
    }
    image
}

/// Parses sprite-sheet filename for dimensions and frame count
fn parse_parameters(dimensions: (u32, u32), path: &Path) -> SpriteParameters {

    lazy_static! { static ref MATCHER: Regex = Regex::new(r"_(\d+)x(\d+)x(\d+)(?:x(\d+))?\.").unwrap(); }

    let filename = path.file_name().unwrap().to_str().unwrap();
    let captures = MATCHER.captures(filename);

    match captures {
        Some(captures) => {
            let frame_width = captures.at(1).unwrap().parse::<u32>().unwrap();
            let frame_height = captures.at(2).unwrap().parse::<u32>().unwrap();
            let frame_count = captures.at(3).unwrap().parse::<u32>().unwrap();
            let frame_channels = captures.at(4).unwrap_or("1").parse::<u32>().unwrap();
            let frame_layout = if frame_height == dimensions.1 { SpriteLayout::HORIZONTAL } else { SpriteLayout::VERTICAL };
            assert!(frame_layout == SpriteLayout::VERTICAL || frame_width * frame_count == dimensions.0);
            assert!(frame_layout == SpriteLayout::HORIZONTAL || frame_height * frame_count == dimensions.1);
            SpriteParameters {
                dimensions  : (frame_width, frame_height),
                num_frames  : frame_count,
                num_channels: frame_channels,
                layout      : frame_layout
            }
        }
        None => SpriteParameters {
            dimensions  : dimensions,
            num_frames  : 1,
            num_channels: 1,
            layout      : SpriteLayout::HORIZONTAL
        }
    }
}
