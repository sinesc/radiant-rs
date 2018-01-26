mod blendmode;
mod display;
mod input;
mod layer;
mod renderer;
mod sprite;
mod font;
mod rendercontext;
mod color;
mod monitor;
mod texture;
mod program;
mod uniform;
mod postprocessor;
mod builder;

pub use self::blendmode::*;
pub use self::input::*;
pub use self::display::*;
pub use self::sprite::*;
pub use self::renderer::*;
pub use self::font::*;
pub use self::layer::*;
pub use self::rendercontext::*;
pub use self::color::*;
pub use self::monitor::*;
pub use self::texture::*;
pub use self::program::*;
pub use self::uniform::*;
pub use self::postprocessor::*;
pub use self::builder::*;
use image;
use prelude::*;
use maths::{Rect, Point2};

/// A vertex.
#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position    : [f32; 2],
    pub offset      : [f32; 2],
    pub rotation    : f32,
    pub color       : (f32, f32, f32, f32),
    pub bucket_id   : u32,
    pub texture_id  : u32,
    pub texture_uv  : [f32; 2],
    pub components  : u32,
}

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
    /// Clears the target.
    fn clear(self: &Self, color: Color) {
        match *self {
            RenderTarget::Display(ref display) => {
                display.clear(color);
            },
            RenderTarget::Texture(ref texture) => {
                texture.handle.clear(color);
            }
            RenderTarget::None => { }
        }
    }
    /// Returns the dimensions of the target.
    fn dimensions(self: &Self) -> Point2<u32> {
        match *self {
            RenderTarget::Display(ref display) => {
                display.dimensions()
            },
            RenderTarget::Texture(ref texture) => {
                texture.dimensions()
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
                        target_display.frame(|ref mut frame| frame.copy_rect(source_rect, target_rect, filter));
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        target_display.frame(|ref mut frame| frame.copy_rect_from_texture(src_texture, source_rect, target_rect, filter));
                    }
                    RenderTarget::None => { }
                }
            },
            RenderTarget::Texture(ref target_texture) => {
                match *source {
                    RenderTarget::Display(ref src_display) => {
                        src_display.frame(|ref mut frame| target_texture.handle.copy_rect_from_frame(frame, source_rect, target_rect, filter));
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        target_texture.handle.copy_rect_from(src_texture, source_rect, target_rect, filter);
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
                        target_display.frame(|ref mut frame| frame.copy_from_texture(src_texture, filter));
                    }
                    RenderTarget::None => { }
                }
            },
            RenderTarget::Texture(ref target_texture) => {
                match *source {
                    RenderTarget::Display(ref src_display) => {
                        src_display.frame(|ref mut frame| target_texture.handle.copy_from_frame(frame, filter));
                    },
                    RenderTarget::Texture(ref src_texture) => {
                        target_texture.handle.copy_from(src_texture, filter);
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
    FullscreenError(String),
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

/// Converts Srgb to rgb and multiplies image color channels with alpha channel
pub fn convert_color(mut image: image::RgbaImage) -> image::RgbaImage {
    use palette::Rgb;
    use palette::pixel::Srgb;
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        let rgb = Rgb::from(Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0
        ));
        pixel[0] = (alpha * rgb.red * 255.0) as u8;
        pixel[1] = (alpha * rgb.green * 255.0) as u8;
        pixel[2] = (alpha * rgb.blue * 255.0) as u8;
    }
    image
}