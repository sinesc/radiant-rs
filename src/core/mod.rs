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
mod rendertarget;
pub mod math;

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
pub use self::rendertarget::*;
pub use self::math::*;
use image;
use prelude::*;
use backends::backend;

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
    use palette::Srgb;
    //use palette::pixel::Srgb;
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        let rgb = Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0
        ).into_linear();
        pixel[0] = (alpha * rgb.red * 255.0) as u8;
        pixel[1] = (alpha * rgb.green * 255.0) as u8;
        pixel[2] = (alpha * rgb.blue * 255.0) as u8;
    }
    image
}