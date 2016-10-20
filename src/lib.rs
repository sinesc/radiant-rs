#[macro_use] extern crate glium;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;
extern crate rusttype;
extern crate unicode_normalization;
extern crate font_loader;

mod prelude;
mod avec;
mod color;
mod graphics;
mod maths;
mod scene;

pub use graphics::{BlendMode, blendmodes, Display, DisplayInfo, Monitor, Layer, Renderer, RenderContext, Sprite, Font, FontInfo, Input, ButtonState};
pub use maths::{Mat4, Vec2, Vec3, Dir1, VecType};
pub use color::Color;
pub use scene::{Scene, Operation};

pub mod utils;
