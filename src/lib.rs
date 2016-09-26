#[macro_use] extern crate glium;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;

mod display;
mod input;
mod renderer;
mod maths;
mod color;
mod scene;
mod avec;

pub use input::Input;
pub use color::Color;
pub use renderer::Renderer;
pub use renderer::Sprite;
pub use renderer::Layer;
pub use renderer::blendmodes;
pub use maths::{Mat4, Vec2, Vec3, Dir1};
pub use display::Descriptor;
pub use display::Monitor;
pub use scene::Scene;

// this is here so the struct members don't have to be public - could need a better solution
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
}

#[derive(Copy, Clone)]
pub struct BlendMode (glium::draw_parameters::Blend);
