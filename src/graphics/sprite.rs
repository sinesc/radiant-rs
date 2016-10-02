use prelude::*;
use graphics::{RawFrame, Vertex, renderer};
use Color;
use image;
use image::GenericImage;
use regex::Regex;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub anchor_x    : f32,
    pub anchor_y    : f32,
    width           : u32,
    height          : u32,
    frames          : u32,
    bucket_id       : u32,
    bucket_pos      : u32,
    u_max           : f32,
    v_max           : f32,
}

#[derive(Copy, Clone, PartialEq)]
enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

struct FrameParameters (u32, u32, u32, SpriteLayout);

impl Sprite {
    pub fn width(self: &Self) -> u32 {
        self.width
    }
    pub fn height(self: &Self) -> u32 {
        self.height
    }
    pub fn frames(self: &Self) -> u32 {
        self.frames
    }
    pub fn bucket_id(self: &Self) -> u32 {
        self.bucket_id
    }
    pub fn texture_id(self: &Self, frame_id: u32) -> u32 {
        self.bucket_pos + (frame_id % self.frames)
    }
    pub fn u_max(self: &Self) -> f32 {
        self.u_max
    }
    pub fn v_max(self: &Self) -> f32 {
        self.v_max
    }
}

pub fn draw_sprite(sprite: &Sprite, vertex: &mut [Vertex], frame_id: u32, x: f32, y: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) {

    let texture_id = sprite.texture_id(frame_id);
    let bucket_id = sprite.bucket_id;

    // corner positions relative to x/y

    let anchor_x = sprite.anchor_x * sprite.width as f32;
    let anchor_y = sprite.anchor_y * sprite.height as f32;

    let offset_x0 = -anchor_x * scale_x;
    let offset_x1 = (sprite.width as f32 - anchor_x) * scale_x;
    let offset_y0 = -anchor_y * scale_y;
    let offset_y1 = (sprite.height as f32 - anchor_y) * scale_y;

    // fill vertex array

    vertex[0].position[0] = x;
    vertex[0].position[1] = y;
    vertex[0].offset[0] = offset_x0;
    vertex[0].offset[1] = offset_y0;
    vertex[0].rotation = rotation;
    vertex[0].bucket_id = bucket_id;
    vertex[0].texture_id = texture_id;
    vertex[0].color = color;
    vertex[0].texture_uv[0] = 0.0;
    vertex[0].texture_uv[1] = 0.0;

    vertex[1].position[0] = x;
    vertex[1].position[1] = y;
    vertex[1].offset[0] = offset_x1;
    vertex[1].offset[1] = offset_y0;
    vertex[1].rotation = rotation;
    vertex[1].bucket_id = bucket_id;
    vertex[1].texture_id = texture_id;
    vertex[1].color = color;
    vertex[1].texture_uv[0] = sprite.u_max;
    vertex[1].texture_uv[1] = 0.0;

    vertex[2].position[0] = x;
    vertex[2].position[1] = y;
    vertex[2].offset[0] = offset_x0;
    vertex[2].offset[1] = offset_y1;
    vertex[2].rotation = rotation;
    vertex[2].bucket_id = bucket_id;
    vertex[2].texture_id = texture_id;
    vertex[2].color = color;
    vertex[2].texture_uv[0] = 0.0;
    vertex[2].texture_uv[1] = sprite.v_max;

    vertex[3].position[0] = x;
    vertex[3].position[1] = y;
    vertex[3].offset[0] = offset_x1;
    vertex[3].offset[1] = offset_y1;
    vertex[3].rotation = rotation;
    vertex[3].bucket_id = bucket_id;
    vertex[3].texture_id = texture_id;
    vertex[3].color = color;
    vertex[3].texture_uv[0] = sprite.u_max;
    vertex[3].texture_uv[1] = sprite.v_max;
}

/// creates a new sprite instance. a sprite instance contains only meta information about a
/// sprite, the actual texture is kept by the renderer. use renderer::create_sprite() to create a sprite
pub fn create_sprite(width: u32, height: u32, frames: u32, bucket_pos: u32) -> Sprite {

    let (bucket_id, tex_size) = renderer::bucket_info(width, height);

    Sprite {
        width       : width,
        height      : height,
        frames      : frames,
        anchor_x    : 0.5,
        anchor_y    : 0.5,
        bucket_id   : bucket_id,
        bucket_pos  : bucket_pos,
        u_max       : (width as f32 / tex_size as f32),
        v_max       : (height as f32 / tex_size as f32),
    }
}

/// loads a spritesheet and returns a vector of frames
pub fn load_spritesheet(file: &str) -> (u32, u32, Vec<RawFrame>) {

    // load image file

    let path = Path::new(file);
    let mut image = image::open(&path).unwrap();
    let image_dimensions = image.to_rgba().dimensions(); // todo how much does this cost?

    // compute frame parameters

    let frame_parameters = parse_parameters(image_dimensions, path);
    let FrameParameters(frame_width, frame_height, frame_count, _) = frame_parameters;
    let (_, pad_size) = renderer::bucket_info(frame_width, frame_height);

    let mut frames = Vec::<RawFrame>::new();

    for frame_id in 0..frame_count {
        frames.push(build_frame_texture(&mut image, image_dimensions, &frame_parameters, frame_id, pad_size));
    }

    (frame_width, frame_height, frames)
}

/// parses sprite-sheet filename for dimensions and frame count
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

/// constructs a RawFrame for a single frame of a spritesheet
///
/// if neccessary, pads the image up to the next power of two
fn build_frame_texture(image: &mut image::DynamicImage, image_dimensions: (u32, u32), frame_parameters: &FrameParameters, frame_id: u32, pad_size: u32) -> RawFrame {

    let FrameParameters(frame_width, frame_height, _, _) = *frame_parameters;
    let (x, y) = get_frame_coordinates(image_dimensions, frame_parameters, frame_id);
    let subimage = image.crop(x, y, frame_width, frame_height);

    if frame_width != pad_size || frame_height != pad_size {

        // pad image if it doesn't match an available texture array size
        let mut dest = image::DynamicImage::new_rgba8(pad_size, pad_size);
        dest.copy_from(&subimage, 0, 0);
        create_raw_frame(dest.to_rgba())

    } else {

        // perfect fit
        create_raw_frame(subimage.to_rgba())
    }
}

/// computes top/left frame coordinates for the given frame_id in a sprite-sheet
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

/// creates a vector of vectors from given RgbaImage !todo lots of extra work for nothing, is this really required?
fn create_raw_frame(from: image::RgbaImage) -> RawFrame {

    let height = from.height();
    let width = from.width();
    let raw = from.into_raw();
    let mut data: RawFrame = vec!();
    let mut pos: u32 = 0;

    for _ in 0..height {
        let mut line = Vec::<(u8, u8, u8, u8)>::new();
        for _ in 0..width {
            line.push((raw[pos as usize], raw[pos as usize + 1], raw[pos as usize + 2], raw[pos as usize + 3]));
            pos += 4;
        }
        data.push(line);
    }

    data
}
