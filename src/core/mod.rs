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
mod texture;
mod program;

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
pub use self::texture::Texture;
pub use self::program::Program;

use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{self, Surface, DrawParameters, DrawError};
use std::rc::Rc;

/// An enum of render target type instances.
#[derive(Clone)]
pub enum RenderTargetType {
    Display(Display),
    Texture(Rc<glium::texture::Texture2d>),
}

impl RenderTargetType {
    /// Draws to the target.
    fn draw<'b, 'v, V, I, U>(self: &Self, vb: V, ib: I, program: &glium::Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<(), DrawError>
        where I: Into<IndicesSource<'b>>, U: Uniforms, V: MultiVerticesSource<'v> {

        match *self {
            RenderTargetType::Display(ref display) => {
                display::draw(display, vb, ib, program, uniforms, draw_parameters)
            },
            RenderTargetType::Texture(ref texture) => {
                texture.as_surface().draw(vb, ib, program, uniforms, draw_parameters)
            }
        }
    }
    /// Clears the target.
    fn clear(self: &Self, color: &Color) {
        match *self {
            RenderTargetType::Display(ref display) => {
                display::clear(display, color);
            },
            RenderTargetType::Texture(ref texture) => {
                let (r, g, b, a) = color.as_tuple();
                texture.as_surface().clear_color(r, g, b, a);
            }
        }
    }
}

/// A target for rendering.
pub trait RenderTarget {
    fn get_target(self: &Self) -> RenderTargetType;
}

/// A postprocessing filter.
pub trait Filter {
    /// Returns the filter's fragment shader.
    fn program() -> String;

    /// Sets the filter's uniforms.
    fn set_uniforms();

    /// Sets a uniform value.
    fn set_uniform(_: &str, _: f32) {

    }
}
