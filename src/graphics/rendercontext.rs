use glium;
use graphics::{Display, FontCache};

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
            data    : glium::texture::SrgbTexture2dArray::empty(&display.handle, 2, 2, 1).unwrap(),
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
    pub font_cache      : FontCache,
    pub font_texture    : glium::texture::Texture2d,
}

impl<'a> RenderContextData<'a> {

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
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::new(&self.display.handle, raw_images).unwrap();
                } else {
                    self.tex_array[bucket_id].data = glium::texture::SrgbTexture2dArray::empty(&self.display.handle, 2, 2, 1).unwrap();
                }
            }
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
}
