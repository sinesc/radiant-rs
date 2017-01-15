mod blendmode;
mod display;
mod input;
mod layer;
mod renderer;
mod sprite;
mod font;
mod rendercontext;
mod scene;
mod color;
mod monitor;

pub use self::blendmode::{blendmodes, BlendMode};
pub use self::input::{Input, InputId, InputState, InputIterator, InputUpIterator, InputDownIterator};
pub use self::display::{Display, DisplayInfo};
pub use self::sprite::Sprite;
pub use self::renderer::Renderer;
pub use self::font::{Font, FontInfo, FontCache};
pub use self::layer::Layer;
pub use self::rendercontext::{RenderContext, RenderContextData, RenderContextTexture, RenderContextTextureArray};
pub use self::color::Color;
pub use self::scene::*;
pub use self::monitor::Monitor;

use prelude::*;

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
