use glium;
use core::{self, display, Display, font};
use prelude::*;
use std::borrow::Cow;

/// Number of texture buckets. Also requires change to renderer.rs at "let uniforms = uniform! { ... }"
pub const NUM_BUCKETS: usize = 6;

/// Initial sprite capacity. Automatically increases.
pub const INITIAL_CAPACITY: usize = 512;

/// A thread-safe render-context.
///
/// Required to load fonts or sprites and aquired from [`Renderer::context()`](struct.Renderer.html#method.context).
#[derive(Clone)]
pub struct RenderContext (Arc<Mutex<RenderContextData>>);
unsafe impl<'a> Send for RenderContext { }
unsafe impl<'a> Sync for RenderContext { }

impl RenderContext {
    /// Retrieves the display associated with this rendercontext.
    pub fn display(self: &Self) -> Display {
        lock(self).display.clone()
    }
}

pub fn new(data: RenderContextData) -> RenderContext {
    RenderContext(Arc::new(Mutex::new(data)))
}
pub fn lock<'b>(context: &'b RenderContext) -> MutexGuard<'b, RenderContextData> {
    context.0.lock().unwrap()
}

/// Individual Texture
#[derive(Clone)]
pub struct RenderContextTexture {
    pub data    : Vec<u8>,
    pub width   : u32,
    pub height  : u32,
}

impl<'a> glium::texture::Texture2dDataSource<'a> for RenderContextTexture {
    type Data = u8;
    fn into_raw(self: Self) -> glium::texture::RawImage2d<'a, Self::Data> {
        glium::texture::RawImage2d {
            data: Cow::Owned(self.data),
            width: self.width,
            height: self.height,
            format: glium::texture::ClientFormat::U8U8U8U8,
        }
    }
}

/// Texture data for a single texture array
pub struct RenderContextTextureArray {
    pub dirty   : bool,
    pub data    : Rc<glium::texture::Texture2dArray>,
    pub raw     : Vec<RenderContextTexture>,
}

impl RenderContextTextureArray {
    pub fn new(display: &Display) -> Self {
        RenderContextTextureArray {
            dirty   : false,
            data    : Rc::new(texture_array(display, Vec::new())),
            raw     : Vec::new(),
        }
    }
}

/// Internal data of a RenderContext
pub struct RenderContextData {
    pub index_buffer        : glium::IndexBuffer<u32>,
    pub tex_array           : Vec<RenderContextTextureArray>,
    pub display             : Display,
    pub font_cache          : font::FontCache,
    pub font_texture        : Rc<glium::texture::Texture2d>,
    pub vertex_buffer_single: glium::VertexBuffer<Vertex>,
}

impl RenderContextData {

    /// Create a new instance
    pub fn new(display: &Display, initial_capacity: usize) -> core::Result<Self> {

        let mut tex_array = Vec::new();
        let glium_handle = &display::handle(&display);

        for _ in 0..NUM_BUCKETS {
            tex_array.push(RenderContextTextureArray::new(display));
        }

        Ok(RenderContextData {
            index_buffer        : Self::create_index_buffer(glium_handle, initial_capacity),
            tex_array           : tex_array,
            display             : display.clone(),
            font_cache          : font::FontCache::new(512, 512, 0.01, 0.01),
            font_texture        : Rc::new(font::create_cache_texture(glium_handle, 512, 512)),
            vertex_buffer_single: Self::create_vertex_buffer_single(glium_handle),
        })
    }

    /// Update font-texture from cache
    pub fn update_font_cache(self: &Self) {
        self.font_cache.update(&self.font_texture);
    }

    /// Update texture arrays from registered textures
    pub fn update_tex_array(self: &mut Self) {
        for ref mut array in self.tex_array.iter_mut() {
            if array.dirty {
                array.dirty = false;
                array.data = Rc::new(texture_array(&self.display, array.raw.clone()));
            }
        }
    }

    /// Update index buffer to given size
    pub fn update_index_buffer(self: &mut Self, max_sprites: usize) {
        if max_sprites * 6 > self.index_buffer.len() {
            self.index_buffer = Self::create_index_buffer(&display::handle(&self.display), max_sprites);
        }
    }

    /// Store given frames to texture arrays
    pub fn store_frames<'a>(self: &mut Self, bucket_id: u32, raw_frames: Vec<RenderContextTexture>) -> u32 {
        let texture_id = self.tex_array[bucket_id as usize].raw.len() as u32;
        for frame in raw_frames {
            self.tex_array[bucket_id as usize].raw.push(frame);
        }
        self.tex_array[bucket_id as usize].dirty = true;
        texture_id
    }

    /// creates vertex pool for given number of sprites
    fn create_index_buffer(display: &glium::Display, max_sprites: usize) -> glium::index::IndexBuffer<u32> {

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

    /// creates a single rectangle vertex buffer
    fn create_vertex_buffer_single(display: &glium::Display) -> glium::VertexBuffer<Vertex> {
        glium::VertexBuffer::new(display,
            &[
                Vertex { position: [-0.5, -0.5], texture_uv: [ 0.0, 0.0 ] },
                Vertex { position: [-0.5,  0.5], texture_uv: [ 0.0, 1.0 ] },
                Vertex { position: [ 0.5, -0.5], texture_uv: [ 1.0, 0.0 ] },
                Vertex { position: [ 0.5,  0.5], texture_uv: [ 1.0, 1.0 ] },
            ]
        ).unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    texture_uv: [f32; 2],
}

implement_vertex!(Vertex, position, texture_uv);

/// Generates glium texture array from given vector of textures
fn texture_array(display: &Display, raw: Vec<RenderContextTexture>) -> glium::texture::Texture2dArray {
    use glium::texture;
    if raw.len() > 0 {
        texture::Texture2dArray::with_format(display::handle(display), raw.clone(), texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap).unwrap()
    } else {
        texture::Texture2dArray::empty_with_format(display::handle(display), texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap, 2, 2, 1).unwrap()
    }
}
