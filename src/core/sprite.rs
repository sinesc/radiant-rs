use prelude::*;
use core::{renderer, layer, Layer, Point, Rect, rendercontext, RenderContext, RenderContextTexture};
use Color;
use image;
use image::GenericImage;
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
    pub anchor      : (f32, f32),
    width           : f32,
    height          : f32,
    num_frames      : u32,
    bucket_id       : u32,
    texture_id      : u32,
    u_max           : f32,
    v_max           : f32,
    context         : RenderContext,
}

#[derive(Copy, Clone, PartialEq)]
enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

struct FrameParameters (u32, u32, u32, SpriteLayout);

impl<'a> Sprite {

    /// Creates a new sprite texture
    ///
    /// The given filename is expected to end on _<width>x<height>x<frames>.<extension>, e.g. asteroid_64x64x24.png.
    pub fn from_file(context: &RenderContext, file: &str) -> Sprite {

        let (bucket_id, texture_size, frame_width, frame_height, raw_frames) = load_spritesheet(file);
        let num_frames = raw_frames.len() as u32;
        let texture_id = rendercontext::lock(context).store_frames(bucket_id, raw_frames);

        Sprite {
            width       : frame_width as f32,
            height      : frame_height as f32,
            num_frames  : num_frames,
            anchor      : (0.5, 0.5),
            bucket_id   : bucket_id,
            texture_id  : texture_id,
            u_max       : (frame_width as f32 / texture_size as f32),
            v_max       : (frame_height as f32 / texture_size as f32),
            context     : context.clone()
        }
    }
// !todo when sprite is dropped, texture would have to be removed (easy)
// alters the texture ids of all following sprites (bah)
//   a) create a lookup table in the context sprite_texture_id <-> texture_array_texture_id
//  or b) keep a list of sprite instances in the context and mutate those (sounds sucky)


    /// Draws a sprite onto the given layer.
    pub fn draw(self: &Self, layer: &Layer, frame_id: u32, x: f32, y: f32, color: Color) -> &Self {

        let bucket_id = self.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = Rect::new(0.0, 0.0, self.u_max, self.v_max);
        let anchor = Point::new(self.anchor.0, self.anchor.1);
        let pos = Point::new(x, y);
        let dim = Point::new(self.width, self.height);
        let scale = Point::new(1.0, 1.0);

        layer::add_rect(layer, bucket_id, texture_id, uv, pos, anchor, dim, color, 0.0, scale);
        self
    }

    /// Draws a sprite onto the given layer and applies given color, rotation and scaling.
    pub fn draw_transformed(self: &Self, layer: &Layer, frame_id: u32, x: f32, y: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {

        let bucket_id = self.bucket_id;
        let texture_id = self.texture_id(frame_id);
        let uv = Rect::new(0.0, 0.0, self.u_max, self.v_max);
        let anchor = Point::new(self.anchor.0, self.anchor.1);
        let pos = Point::new(x, y);
        let dim = Point::new(self.width, self.height);
        let scale = Point::new(scale_x, scale_y);

        layer::add_rect(layer, bucket_id, texture_id, uv, pos, anchor, dim, color, rotation, scale);
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

/// Loads a spritesheet and returns a vector of frames
pub fn load_spritesheet(file: &str) -> (u32, u32, u32, u32, Vec<RenderContextTexture>) {

    // load image file

    let path = Path::new(file);
    let mut image = image::open(&path).unwrap();    // !note extremely slow in debug compile
    let image_dimensions = image.dimensions();

    // compute frame parameters

    let frame_parameters = parse_parameters(image_dimensions, path);
    let FrameParameters(frame_width, frame_height, frame_count, _) = frame_parameters;
    let (bucket_id, pad_size) = renderer::bucket_info(frame_width, frame_height);

    let mut raw_frames = Vec::new();

    for frame_id in 0..frame_count {
        raw_frames.push(build_frame_texture(&mut image, image_dimensions, &frame_parameters, frame_id, pad_size));
    }

    (bucket_id, pad_size, frame_width, frame_height, raw_frames)
}

/// Parses sprite-sheet filename for dimensions and frame count
fn parse_parameters(dimensions: (u32, u32), path: &Path) -> FrameParameters {

    lazy_static! { static ref MATCHER: Regex = Regex::new(r"_(\d+)x(\d+)x(\d+)\.").unwrap(); }

    let filename = path.file_name().unwrap().to_str().unwrap();
    let captures = MATCHER.captures(filename);

    match captures {
        Some(captures) => {
            let frame_width = captures.at(1).unwrap().parse::<u32>().unwrap();
            let frame_height = captures.at(2).unwrap().parse::<u32>().unwrap();
            let frame_count = captures.at(3).unwrap().parse::<u32>().unwrap();
            let frame_layout = if frame_height == dimensions.1 { SpriteLayout::HORIZONTAL } else { SpriteLayout::VERTICAL };
            assert!(frame_layout == SpriteLayout::VERTICAL || frame_width * frame_count == dimensions.0);
            assert!(frame_layout == SpriteLayout::HORIZONTAL || frame_height * frame_count == dimensions.1);
            FrameParameters(frame_width, frame_height, frame_count, frame_layout)
        }
        None => FrameParameters(dimensions.0, dimensions.1, 1, SpriteLayout::HORIZONTAL)
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

/// Constructs a RawFrame for a single frame of a spritesheet
///
/// If neccessary, pads the image up to the next power of two
fn build_frame_texture(image: &mut image::DynamicImage, image_dimensions: (u32, u32), frame_parameters: &FrameParameters, frame_id: u32, pad_size: u32) -> RenderContextTexture {

    let FrameParameters(frame_width, frame_height, _, _) = *frame_parameters;
    let (x, y) = get_frame_coordinates(image_dimensions, frame_parameters, frame_id);
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
fn get_frame_coordinates(image_dimensions: (u32, u32), frame_parameters: &FrameParameters, frame_id: u32) -> (u32, u32) {

    let (img_width, img_height) = image_dimensions;
    let FrameParameters(frame_width, frame_height, frame_count, frame_layout) = *frame_parameters;

    assert!(frame_id < frame_count);

    if frame_layout == SpriteLayout::HORIZONTAL {
        let spl = img_width / frame_width;
        ((frame_id % spl) * frame_width, (frame_id / spl) * frame_height)
    } else {
        let spl = img_height / frame_height;
        ((frame_id / spl) * frame_width, (frame_id % spl) * frame_height)
    }
}
