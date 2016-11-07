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

pub use self::blendmode::{blendmodes, BlendMode};
pub use self::input::{Input, ButtonState};
pub use self::display::{DisplayInfo, Monitor};
pub use self::sprite::Sprite;
pub use self::renderer::Renderer;
pub use self::font::{Font, FontInfo, FontCache};
pub use self::layer::Layer;
pub use self::rendercontext::{RenderContextData, RenderContextTextureArray};
pub use self::color::Color;
pub use self::scene::*;

use prelude::*;
use glium;
use self::input::InputState;

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
    input_state: Arc<RwLock<InputState>>,
}

/// A thread-safe render-context.
///
/// Required to load fonts or sprites and aquired from [`Renderer::context()`](struct.Renderer.html#method.context).
pub struct RenderContext<'a> (Mutex<RenderContextData<'a>>);
unsafe impl<'a> Send for RenderContext<'a> { }
unsafe impl<'a> Sync for RenderContext<'a> { }

impl<'a> RenderContext<'a> {
    fn new(data: RenderContextData) -> RenderContext {
        RenderContext (Mutex::new(data))
    }
    fn lock(self: &Self) -> MutexGuard<RenderContextData<'a>> {
        self.0.lock().unwrap()
    }
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
