#[macro_use] extern crate glium;
extern crate rand;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;

mod display;
mod input;
mod renderer;
mod maths;
mod color;

pub use input::Input;
pub use color::Color;
pub use renderer::Renderer;
pub use renderer::Sprite;
pub use renderer::Layer;
pub use maths::{Mat4, Vec2, Vec3, Dir1};
//pub use display::Display;
pub use display::Descriptor;
pub use display::Monitor;

// this is only here so handle doesn't have to be public. need better solution
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
}
