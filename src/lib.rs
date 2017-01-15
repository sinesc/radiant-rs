#[macro_use] extern crate glium;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;
extern crate rusttype;
extern crate unicode_normalization;
extern crate font_loader;
extern crate avec;
#[macro_use] extern crate enum_primitive;

mod prelude;
mod core;
mod maths;
mod misc;

mod public;
pub use public::*;
