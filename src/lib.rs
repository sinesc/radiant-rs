/*!
 * Radiant example code support library. 
 *
 * # This is not the library you are looking for
 * This library implements very basic, minimally tested math/support features for the radiant-rs examples.
 * It may be useful for early prototyping or super simple games, you likely want to look for something else though.
 */
extern crate radiant_rs;
extern crate num_traits;
mod math;
mod misc;
mod prelude;

pub mod loops {
    //! Time controlled loops.
    pub use super::misc::{renderloop, mainloop, LoopState};
}

pub mod maths {
    //! Basic math types, traits and methods.
    pub use super::math::{Angle, Vec2, Vec3, Mat4, Matrix, Vector};
    pub use super::misc::{min, max, lerp, approach};
}

pub mod util {
    //! Everything else.
    pub use super::misc::{Rng, Periodic, ARng};
}

pub use misc::renderloop;
pub use math::{Vec2, Mat4, Matrix};