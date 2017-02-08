use prelude::*;
use core::{display, rendercontext, RenderContext, RenderTarget, RenderTargetType, Color};
use glium;
use glium::Surface;

/// A texture.
#[derive(Clone)]
pub struct Texture (Rc<glium::texture::Texture2d>);

pub fn handle(texture: &Texture) -> &glium::texture::Texture2d {
    texture.0.deref()
}

impl Texture {
    /// Creates a new texture with given dimensions.
    pub fn new(context: &RenderContext, width: u32, height: u32) -> Texture {

        let context = rendercontext::lock(context);

        let texture = glium::texture::Texture2d::empty_with_format(
            display::handle(&context.display),
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height
        ).unwrap();

        Texture(Rc::new(texture))
    }
    /// Clears the texture with given color.
    pub fn clear(self: &Self, color: &Color) {
        let (r, g, b, a) = color.as_tuple();
        self.0.as_surface().clear_color(r, g, b, a);
    }
}

impl RenderTarget for Texture {
    fn get_target(self: &Self) -> RenderTargetType {
        RenderTargetType::Texture(self.0.clone())
    }
}
