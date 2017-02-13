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
mod uniform;

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
pub use self::texture::{Texture, TextureFilter, TextureWrap};
pub use self::program::Program;
pub use self::uniform::{Uniform, AsUniform, UniformList, GliumUniform};

use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{self, Surface, DrawParameters, DrawError};
use image;
use prelude::*;

/// An enum of render target type instances.
#[derive(Clone)]
pub enum RenderTargetType {
    Display(Display),
    Texture(Rc<glium::texture::Texture2d>),
}

impl RenderTargetType {
    /// Draws to the target.
    fn draw<'b, 'v, V, I, U>(self: &Self, vb: V, ib: I, program: &glium::Program, uniforms: &U, draw_parameters: &DrawParameters) -> result::Result<(), DrawError>
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
                let Color(r, g, b, a) = *color;
                texture.as_surface().clear_color(r, g, b, a);
            }
        }
    }
}

/// A target for rendering.
pub trait RenderTarget {
    fn get_target(self: &Self) -> RenderTargetType;
}

/// Radiant errors.
#[derive(Debug)]
pub enum Error {
    ImageError(String),
    ShaderError(String),
    IoError(io::Error),
    Failed,
}

impl From<io::Error> for Error {
    /// Converts io error to radiant error
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<image::ImageError> for Error {
    /// Converts image error to radiant error
    fn from(error: image::ImageError) -> Error {
        use image::ImageError;
        match error {
            ImageError::IoError(error)          => { Error::IoError(error) }
            ImageError::FormatError(error)      => { Error::ImageError(format!("Image format error: {}", error)) }
            ImageError::UnsupportedError(error) => { Error::ImageError(format!("Image unsupported: {}", error)) }
            ImageError::UnsupportedColor(_)     => { Error::ImageError("Unsupported colorformat".to_string()) }
            _                                   => { Error::ImageError("Unknown image error".to_string()) }
        }
    }
}

/// Input/output result.
pub type Result<T> = result::Result<T, Error>;
