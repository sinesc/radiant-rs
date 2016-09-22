mod layer;
mod sprite;
pub use self::layer::Layer;
pub use self::sprite::Sprite;

use std::mem;
use std::path::Path;
use std::cmp;
use std::sync::{Arc, Mutex};
use glium;
use glium::Surface;
use image;
use image::GenericImage;
use regex::Regex;
use color::Color;
use display::Display;

#[derive(Copy, Clone, Default)]
struct Vertex {
    position    : [f32; 2],
    offset      : [f32; 2],
    rotation    : f32,
    color       : Color,
    bucket_id   : u32,
    texture_id  : u32,
    texture_uv  : [f32; 2],
}
implement_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv);

#[derive(Copy, Clone, PartialEq)]
enum SpriteLayout {
    VERTICAL,
    HORIZONTAL,
}

struct FrameParameters (u32, u32, u32, SpriteLayout);
type RawFrame = Vec<Vec<(u8, u8, u8, u8)>>;

struct GliumState {
    index_buffer    : glium::IndexBuffer<u32>,
    program         : glium::Program,
    tex_array       : Vec<Option<glium::texture::Texture2dArray>>,
    raw_tex_data    : Vec<Vec<RawFrame>>,
    target          : Option<glium::Frame>,
    display         : Display,
}

#[derive(Clone)]
pub struct Renderer {
    max_sprites     : u32,
    glium           : Arc<Mutex<GliumState>>,
}

unsafe impl Sync for Renderer { }
unsafe impl Send for Renderer { }

impl Renderer {

    /// returns a new sprite renderer instance
    ///
    /// display is a glium Display obtained by i.e. glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap()
    pub fn new(display: &Display, max_sprites: u32) -> Self {
        let glium = GliumState {
            index_buffer    : Self::create_index_buffer(&display.handle, max_sprites),
            program         : Self::create_program(&display.handle),
            tex_array       : Vec::new(),
            raw_tex_data    : Vec::new(),
            target          : Option::None,
            display         : display.clone(),
        };
        Renderer {
            max_sprites     : max_sprites,
            glium           : Arc::new(Mutex::new(glium)),
        }
    }

    /// returns a layer to draw on
    ///
    /// layers have a matrix and a blendmode that applies to all sprites on the layer
    pub fn layer(&self) -> Layer {
        Layer::new(self.clone())
    }

    /// registers a sprite texture for drawing
    ///
    /// must be done before first draw, calling  this function after draw will reset existing
    /// sprites and register new ones
    /// filename is epected to end on _<width>x<height>x<frames>.<extension>, i.e. asteroid_64x64x24.png
    pub fn texture(&self, file: &str) -> Sprite {

        let mut glium = self.glium.lock().unwrap();
        let path = Path::new(file);
        let mut image = image::open(&path).unwrap();
        let image_dimensions = image.to_rgba().dimensions(); // todo how much does this cost?

        let frame_parameters = Self::parse_parameters(image_dimensions, path);
        let FrameParameters(frame_width, frame_height, frame_count, _) = frame_parameters;

        // identify bucket_id (which texture array) and texture index in the array

        let (bucket_id, pad_size) = Self::bucket_info(frame_width, frame_height);

        while glium.raw_tex_data.len() <= bucket_id as usize {
            glium.raw_tex_data.push(Vec::new());
        }

        let bucket_pos = glium.raw_tex_data[bucket_id as usize].len() as u32;

        // append frames to the array

        for frame_id in 0..frame_count {
            let frame = Self::build_frame_texture(&mut image, image_dimensions, &frame_parameters, frame_id, pad_size);
            glium.raw_tex_data[bucket_id as usize].push(frame);
        }

        Sprite::new(frame_width, frame_height, frame_count, bucket_pos)
    }

    /// prepares a new target for drawing
    pub fn prepare_target(&self) {
        let mut glium = self.glium.lock().unwrap();
        glium.target = Some(glium.display.handle.draw());
    }

    /// clears the prepared target with given color
    pub fn clear_target(&self, color: &Color) {
        let mut glium = self.glium.lock().unwrap();
        let (r, g, b, a) = color.as_f32_tuple();
        glium.target.as_mut().unwrap().clear_color(r, g, b, a);
    }

    /// prepares a new target and clears it with given color
    pub fn prepare_and_clear_target(&self, color: &Color) {
        let mut glium = self.glium.lock().unwrap();
        let (r, g, b, a) = color.as_f32_tuple();
        let mut target = glium.display.handle.draw();
        target.clear_color(r, g, b, a);
        glium.target = Some(target);
    }

    /// finishes drawing and swaps the drawing target to front
    pub fn swap_target(&self) {
        let mut glium = self.glium.lock().unwrap();
        glium.target.take().unwrap().finish().unwrap();
    }

    /// takes the target frame from radiant-rs
    pub fn take_target(&self) -> glium::Frame {
        let mut glium = self.glium.lock().unwrap();
        glium.target.take().unwrap()
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
        let (x, y) = Self::get_frame_coordinates(image_dimensions, frame_parameters, frame_id);
        let subimage = image.crop(x, y, frame_width, frame_height);

        if frame_width != pad_size || frame_height != pad_size {

            // pad image if it doesn't match an available texture array size
            let mut dest = image::DynamicImage::new_rgba8(pad_size, pad_size);
            dest.copy_from(&subimage, 0, 0);
            Self::create_raw_frame(dest.to_rgba())

        } else {

            // perfect fit
            Self::create_raw_frame(subimage.to_rgba())
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

    /// returns the appropriate bucket_id for the given texture size
    fn bucket_info(width: u32, height: u32) -> (u32, u32) {
        let ln2 = (cmp::max(width, height) as f32).log2().ceil() as u32;
        let size = 2u32.pow(ln2);
        // skip sizes 1x1 to 16x16
        let bucket_id = cmp::max(0, ln2 - 4);
        (bucket_id, size)
    }

    // creates a vector of vectors from given RgbaImage !todo lots of extra work for nothing, is this really required?
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

    /// creates texture arrays from registered textures
    fn create_texture_arrays(&self) {

        let mut glium = self.glium.lock().unwrap();

        if glium.tex_array.len() == 0 {

            for bucket_id in 0..glium.raw_tex_data.len() {
                if glium.raw_tex_data[bucket_id].len() > 0 {
                    let raw_data = mem::replace(&mut glium.raw_tex_data[bucket_id as usize], Vec::new());
                    let array = glium::texture::Texture2dArray::new(&glium.display.handle, raw_data).unwrap();
                    glium.tex_array.push(Some(array));
                } else {
                    glium.tex_array.push(None);
                }
            }
        }
    }

    /// creates vertex pool for given number of sprites
    fn create_index_buffer(display: &glium::Display, max_sprites: u32) -> glium::index::IndexBuffer<u32> {

        let mut ib_data = Vec::with_capacity(max_sprites as usize * 6);

        for i in 0..max_sprites {
            let num = i as u32;
            ib_data.push(num * 4);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 2);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 3);
            ib_data.push(num * 4 + 2);
        }

        glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap()
    }

    /// creates the shader program
    fn create_program(display: &glium::Display) -> glium::Program {
        program!(display,
            140 => {
                vertex: include_str!("../shader/default.vs"),
                fragment: include_str!("../shader/default.fs")
            }
        ).unwrap()
    }

}
