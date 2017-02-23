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
mod postprocessor;

pub use self::blendmode::*;
pub use self::input::*;
pub use self::display::*;
pub use self::sprite::*;
pub use self::renderer::*;
pub use self::font::*;
pub use self::layer::*;
pub use self::rendercontext::*;
pub use self::color::*;
pub use self::scene::*;
pub use self::monitor::*;
pub use self::texture::*;
pub use self::program::*;
pub use self::uniform::*;
pub use self::postprocessor::*;
use backend::glium as backend;

use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{self, Surface, DrawParameters};
use image;
use prelude::*;
use maths::{Rect, Point2};

/// A target for rendering.
pub trait AsRenderTarget {
    /// Returns RenderTarget enum containing a texture or a frame.
    fn as_render_target(self: &Self) -> RenderTarget;
}

/// An enum of render target type instances.
#[derive(Clone)]
pub enum RenderTarget {
    None,
    Display(Display),
    Texture(Texture),
}

impl RenderTarget {
    /// Draws to the target.
    fn draw<'b, 'v, V, I, U>(self: &Self, vb: V, ib: I, program: &glium::Program, uniforms: &U, draw_parameters: &DrawParameters)
        where I: Into<IndicesSource<'b>>, U: Uniforms, V: MultiVerticesSource<'v> {

        match *self {
            RenderTarget::Display(ref display) => {
                display::draw(display, vb, ib, program, uniforms, draw_parameters).unwrap()
            }
            RenderTarget::Texture(ref texture) => {
                texture::handle(texture).as_surface().draw(vb, ib, program, uniforms, draw_parameters).unwrap()
            }
            RenderTarget::None => { }
        }
    }
    /// Clears the target.
    fn clear(self: &Self, color: Color) {
        match *self {
            RenderTarget::Display(ref display) => {
                display::clear(display, color);
            },
            RenderTarget::Texture(ref texture) => {
                let Color(r, g, b, a) = color;
                texture::handle(texture).as_surface().clear_color(r, g, b, a);
            }
            RenderTarget::None => { }
        }
    }
    /// Returns the dimensions of the target.
    fn dimensions(self: &Self) -> Point2<u32> {
        match *self {
            RenderTarget::Display(ref display) => {
                display.dimensions().into()
            },
            RenderTarget::Texture(ref texture) => {
                texture::handle(texture).as_surface().get_dimensions().into()
            }
            RenderTarget::None => {
                Point2(0, 0)
            }
        }
    }
    /// Blits a source rect to a rect on the target.
    fn blit_rect(self: &Self, source: &RenderTarget, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: TextureFilter) {
        match *self {
            RenderTarget::Display(ref target_display) => {
                match *source {
                    RenderTarget::Display(_) => {
                        display::copy_rect(&target_display, source_rect, target_rect, filter);
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        display::copy_rect_from_texture(&target_display, src_texture, source_rect, target_rect, filter);
                    }
                    RenderTarget::None => { }
                }
            },
            RenderTarget::Texture(ref target_texture) => {
                match *source {
                    RenderTarget::Display(ref src_display) => {
                        display::copy_rect_to_texture(&src_display, target_texture, source_rect, target_rect, filter);
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        let target_height = texture::handle(target_texture).as_surface().get_dimensions().1;
                        let source_height = texture::handle(src_texture).as_surface().get_dimensions().1;
                        let (glium_src_rect, glium_target_rect) = backend::blit_coords(source_rect, source_height, target_rect, target_height);
                        texture::handle(src_texture).as_surface().blit_color(&glium_src_rect, &texture::handle(target_texture).as_surface(), &glium_target_rect, backend::magnify_filter(filter));
                    }
                    RenderTarget::None => { }
                }
            }
            RenderTarget::None => { }
        }
    }
    /// Blits to the target.
    fn blit(self: &Self, source: &RenderTarget, filter: TextureFilter) {
        match *self {
            RenderTarget::Display(ref target_display) => {
                match *source {
                    RenderTarget::Display(_) => { /* blitting entire frame to entire frame makes no sense */ },
                    RenderTarget::Texture(ref src_texture) => {
                        display::copy_from_texture(target_display, src_texture, filter);
                    }
                    RenderTarget::None => { }
                }
            },
            RenderTarget::Texture(ref target_texture) => {
                match *source {
                    RenderTarget::Display(ref src_display) => {
                        display::copy_to_texture(src_display, target_texture, filter);
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        texture::handle(src_texture).as_surface().fill(&texture::handle(target_texture).as_surface(), backend::magnify_filter(filter))
                    }
                    RenderTarget::None => { }
                }
            }
            RenderTarget::None => { }
        }
    }
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

/// Radiant result.
pub type Result<T> = result::Result<T, Error>;
