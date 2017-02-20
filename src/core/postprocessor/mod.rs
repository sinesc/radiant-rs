use super::{Texture, Renderer};

/// A custom postprocessor.
///
/// Postprocessing happens in three steps:
///
/// - first, `target()` is invoked and expected to return the input texture target (from
/// where the postprocessor intends to read input data).
/// - `process()` is invoked and expected to perform the neccessary processing
/// **excluding** the final draw operation.
/// - `draw()` is invoked. At this point the renderer has already restored the drawing
/// target so that this method is only required to draw the postprocessing result.
///
/// While you could combine processing and drawing within `draw()`, it is recommended
/// to separate these as future versions might expand on this functionality.
pub trait Postprocessor {
    /// Custom type for the args parameter supplied to `process()` and `draw()`.
    type T;
    /// Expected to return a texture to draw to.
    fn target(self: &Self) -> &Texture;
    /// Expected to processes input data. Draws issued within this function will
    /// target the texure returned by `target()` unless overridden via `Renderer::render_to()`.
    /// This function is provided as you could also do this in `draw()`.
    #[allow(unused_variables)]
    fn process(self: &Self, renderer: &Renderer, args: &Self::T) { }
    /// Expected to draw final result. Draws issued within this function will
    /// target the rendertarget that the postprocessor is supposed to render to.
    fn draw(self: &Self, renderer: &Renderer, args: &Self::T);
}

mod basic;

pub mod postprocessors {
    //! A set of predefined postprocessors for use with `Renderer::postprocess()`.
    pub use super::basic::Basic;
}
