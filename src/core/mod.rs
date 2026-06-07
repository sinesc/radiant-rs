mod blendmode;
mod display;
mod input;
mod layer;
mod renderer;
mod sprite;
mod font;
mod context;
mod color;
mod monitor;
mod texture;
mod program;
mod uniform;
mod postprocessor;
mod builder;
mod rendertarget;
mod math;

pub use self::blendmode::*;
pub use self::input::*;
pub use self::display::*;
pub use self::sprite::*;
pub use self::renderer::*;
pub use self::font::*;
pub use self::layer::*;
pub use self::context::*;
pub use self::color::*;
pub use self::monitor::*;
pub use self::texture::*;
pub use self::program::*;
pub use self::uniform::*;
pub use self::postprocessor::*;
pub use self::builder::*;
pub use self::rendertarget::*;
pub use self::math::*;
use image;
use crate::prelude::*;
use crate::backends::backend;

/// A vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub position    : [f32; 2],
    pub offset      : [f32; 2],
    pub rotation    : f32,
    pub color       : [f32; 4],
    pub bucket_id   : u32,
    pub texture_id  : u32,
    pub texture_uv  : [f32; 2],
    pub components  : u32,
}

/// Radiant errors.
#[derive(Debug)]
pub enum Error {
    ImageError(String),
    ShaderError(String),
    IoError(io::Error),
    FullscreenError(String),
    FontError(String),
    BackendError(backend::Error),
    Failed,
}

impl From<io::Error> for Error {
    /// Converts io error to radiant error
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<backend::Error> for Error {
    fn from(error: backend::Error) -> Error {
        Error::BackendError(error)
    }
}

impl From<image::ImageError> for Error {
    /// Converts image error to radiant error
    fn from(error: image::ImageError) -> Error {
        use image::ImageError;
        match error {
            ImageError::IoError(error)          => { Error::IoError(error) }
            ImageError::Decoding(error)         => { Error::ImageError(format!("Image decoding error: {}", error)) }
            ImageError::Encoding(error)         => { Error::ImageError(format!("Image encoding error: {}", error)) }
            ImageError::Parameter(error)        => { Error::ImageError(format!("Image parameter error: {}", error)) }
            ImageError::Limits(error)           => { Error::ImageError(format!("Image limits error: {}", error)) }
            ImageError::Unsupported(error)      => { Error::ImageError(format!("Image unsupported: {}", error)) }
        }
    }
}

/// Radiant result.
pub type Result<T> = result::Result<T, Error>;

/// Converts Srgb to rgb and multiplies image color channels with alpha channel
pub fn convert_color(mut image: image::RgbaImage) -> image::RgbaImage {
    use palette::encoding::{Srgb, IntoLinear};
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        let r = Srgb::into_linear(pixel[0] as f32 / 255.0);
        let g = Srgb::into_linear(pixel[1] as f32 / 255.0);
        let b = Srgb::into_linear(pixel[2] as f32 / 255.0);
        pixel[0] = (alpha * r * 255.0) as u8;
        pixel[1] = (alpha * g * 255.0) as u8;
        pixel[2] = (alpha * b * 255.0) as u8;
    }
    image
}