use super::{Texture, Renderer};

/// A custom postprocessor.
///
/// Postprocessing happens in three steps:
///
/// - first, `target()` is invoked and expected to return an input texture target (to
/// which the user will draw the input data to be postprocessed).
/// - `process()` is invoked. Any drawing operations performed within will target the
/// input texture.
/// - `draw()` is invoked. Any drawing operations performed within will target the
/// destination defined by the user.
pub trait Postprocessor {
    /// Custom type for the args parameter supplied to `process()` and `draw()`.
    type T;
    /// Expected to return a texture for user drawing operations to target.
    fn target(self: &Self) -> &Texture;
    /// Optionally expected to processes input data. Draws issued within this function will
    /// target the texure returned by `target()` unless overridden via `Renderer::render_to()`.
    #[allow(unused_variables)]
    fn process(self: &Self, renderer: &Renderer, args: &Self::T) { }
    /// Expected to draw the final result. Draws issued within this function will
    /// target the rendertarget that the postprocessor is supposed to render to.
    fn draw(self: &Self, renderer: &Renderer, args: &Self::T);
}

mod basic;

pub mod postprocessors {
    //! A set of predefined postprocessors for use with `Renderer::postprocess()`.
    pub use super::basic::Basic;
}
