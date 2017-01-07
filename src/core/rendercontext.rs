use glium;
use core::{display, Display, font};
use prelude::*;

// Number of texture buckets. Also requires change to renderer.rs at "let uniforms = uniform! { ... }"
pub const NUM_BUCKETS: usize = 6;

// Initial sprite capacity. Automatically increases.
pub const INITIAL_CAPACITY: usize = 512;

/// A thread-safe render-context.
///
/// Required to load fonts or sprites and aquired from [`Renderer::context()`](struct.Renderer.html#method.context).
#[derive(Clone)]
pub struct RenderContext<'a> (Arc<Mutex<RenderContextData<'a>>>);
unsafe impl<'a> Send for RenderContext<'a> { }
unsafe impl<'a> Sync for RenderContext<'a> { }

pub fn new(data: RenderContextData) -> RenderContext {
    RenderContext(Arc::new(Mutex::new(data)))
}
pub fn lock<'a, 'b>(context: &'b RenderContext<'a>) -> MutexGuard<'b, RenderContextData<'a>> {
    context.0.lock().unwrap()
}

/// Texture data for a single texture array
pub struct RenderContextTextureArray<'a> {
    pub dirty   : bool,
    pub data    : glium::texture::SrgbTexture2dArray,
    pub raw     : Vec<glium::texture::RawImage2d<'a, u8>>,
}

impl<'a> RenderContextTextureArray<'a> {
    pub fn new(display: &Display) -> Self {
        RenderContextTextureArray {
            dirty   : false,
            data    : glium::texture::SrgbTexture2dArray::empty(display::handle(&display), 2, 2, 1).unwrap(),
            raw     : Vec::new(),
        }
    }
}

/// Internal data of a RenderContext
pub struct RenderContextData<'a> {
    pub index_buffer    : glium::IndexBuffer<u32>,
    pub program         : glium::Program,
    pub tex_array       : Vec<RenderContextTextureArray<'a>>,
    pub target          : Option<glium::Frame>,
    pub display         : Display,
    pub font_cache      : font::FontCache,
    pub font_texture    : glium::texture::Texture2d,
}

impl<'a> RenderContextData<'a> {

    /// Create a new instance
    pub fn new(display: &Display, initial_capacity: usize) -> Self {

        let mut tex_array = Vec::new();

        for _ in 0..NUM_BUCKETS {
            tex_array.push(RenderContextTextureArray::new(display));
        }

        RenderContextData {
            index_buffer    : Self::create_index_buffer(&display::handle(&display), initial_capacity),
            program         : Self::create_program(&display::handle(&display)),
            tex_array       : tex_array,
            target          : Option::None,
            display         : display.clone(),
            font_cache      : font::FontCache::new(512, 512, 0.01, 0.01),
            font_texture    : font::create_cache_texture(&display::handle(&display), 512, 512),
        }
    }

    /// Update font-texture from cache
    pub fn update_font_cache(self: &mut Self) {
        self.font_cache.update(&mut self.font_texture);
    }

    /// Update texture arrays from registered textures
    pub fn update_tex_array(self: &mut Self) {
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
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::new(display::handle(&self.display), raw_images).unwrap();
                } else {
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::empty(display::handle(&self.display), 2, 2, 1).unwrap();
                }
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
    pub fn store_frames(self: &mut Self, bucket_id: u32, raw_frames: Vec<glium::texture::RawImage2d<'a, u8>>) -> u32 {
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
