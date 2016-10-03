#[macro_use] extern crate glium;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;
extern crate rusttype;
extern crate unicode_normalization;

mod prelude;
mod avec;
mod color;
mod graphics;
mod maths;
mod scene;

pub use graphics::{BlendMode, blendmodes, Display, Descriptor, Monitor, Layer, Renderer, Sprite, Font, Input};
pub use maths::{Mat4, Vec2, Vec3, Dir1};
pub use color::Color;
pub use scene::{Scene, Operation};

pub mod utils;
