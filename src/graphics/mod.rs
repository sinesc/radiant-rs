mod blendmode;
mod display;
mod input;
mod layer;
mod renderer;
mod sprite;
mod font;

pub use self::blendmode::{blendmodes, BlendMode};
pub use self::input::{Input, ButtonState};
pub use self::display::{DisplayInfo, Monitor};
pub use self::sprite::Sprite;
pub use self::renderer::Renderer;
pub use self::font::{Font, FontInfo, FontCache};
pub use self::layer::Layer;

use prelude::*;
use glium;
use graphics::input::InputState;

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
    input_state: Arc<RwLock<InputState>>,
}

pub struct RenderContextTextureArray<'a> {
    dirty   : bool,
    data    : glium::texture::SrgbTexture2dArray,
    raw     : Vec<glium::texture::RawImage2d<'a, u8>>,
}

impl<'a> RenderContextTextureArray<'a> {
    pub fn new(display: &Display) -> Self {
        RenderContextTextureArray {
            dirty   : false,
            data    : glium::texture::SrgbTexture2dArray::empty(&display.handle, 2, 2, 1).unwrap(),
            raw     : Vec::new(),
        }
    }
}

pub struct RenderContextData<'a> {
    index_buffer    : glium::IndexBuffer<u32>,
    program         : glium::Program,
    tex_array       : Vec<RenderContextTextureArray<'a>>,
    target          : Option<glium::Frame>,
    display         : Display,
    font_cache      : font::FontCache,
    font_texture    : glium::texture::Texture2d,
}

impl<'a> RenderContextData<'a> {
    /// update texture arrays from registered textures
    fn tex_array_update(self: &mut Self) {
        for bucket_id in 0..self.tex_array.len() {
            if self.tex_array[bucket_id].dirty {
                self.tex_array[bucket_id].dirty = false;
                if self.tex_array[bucket_id].raw.len() > 0 {
                    let mut raw_images = Vec::new();
                    for ref frame in self.tex_array[bucket_id].raw.iter() {
                        raw_images.push(glium::texture::RawImage2d {
                            data: frame.data.clone(),
                            width: frame.width,
                            height: frame.height,
                            format: frame.format,
                        });
                    }
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::new(&self.display.handle, raw_images).unwrap();
                } else {
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::empty(&self.display.handle, 2, 2, 1).unwrap();
                }
            }
        }
    }
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
    fn store(self: &Self, bucket_id: u32, raw_frames: Vec<glium::texture::RawImage2d<'a, u8>>) -> u32 {
        let mut lock = self.lock();
        let texture_id = lock.tex_array[bucket_id as usize].raw.len() as u32;
        for frame in raw_frames {
            lock.tex_array[bucket_id as usize].raw.push(frame);
        }
        lock.tex_array[bucket_id as usize].dirty = true;
        texture_id
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
