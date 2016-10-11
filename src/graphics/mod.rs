mod blendmode;
mod display;
mod input;
mod layer;
mod renderer;
mod sprite;
mod font;

pub use self::blendmode::BlendMode;
pub use self::blendmode::blendmodes;
pub use self::input::Input;
pub use self::display::{Descriptor, Monitor};
pub use self::sprite::Sprite;
pub use self::renderer::Renderer;
pub use self::font::{Font, FontInfo};

use prelude::*;
use glium;
use color::Color;
use maths::Mat4;
use avec::AVec;

#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
}

pub struct Layer {
    view_matrix : Mutex<Mat4<f32>>,
    model_matrix: Mutex<Mat4<f32>>,
    blend       : Mutex<BlendMode>,
    color       : Mutex<Color>,
    gid         : usize,
    lid         : AtomicUsize,
	vertex_data : AVec<Vertex>,
    font_cache  : Mutex<font::FontCache>,
}

#[derive(Copy, Clone)]
pub struct Point {
    x: f32,
    y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x: x, y: y }
    }
}

#[derive(Copy, Clone)]
pub struct Rect (Point, Point);
impl Rect {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
        Rect(Point { x: x1, y: y1 }, Point { x: x2, y: y2 })
    }
}


pub type RawFrame = Vec<Vec<(u8, u8, u8, u8)>>;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    position    : [f32; 2],
    offset      : [f32; 2],
    rotation    : f32,
    color       : Color,
    bucket_id   : u32,
    texture_id  : u32,
    texture_uv  : [f32; 2],
}
implement_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv);
