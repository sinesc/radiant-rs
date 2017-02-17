use super::{Texture, Renderer, BlendMode};

/// A custom postprocessor.
///
/// Postprocessing happens in three steps:
///
/// - first, `target()` is invoked and expected to return the input texture target (from
/// where the postprocessor intends to read input data).
/// - `process()` is invoked and expected to perform the neccessary processing
/// **excluding** the final draw operation.
/// - `draw()` is invoked. At this point the renderer has already restored the drawing
/// target so that this method is only required to draw the postprocessing result
/// to the current target.
pub trait Postprocessor {
    /// Expected to return a texture to draw to.
    fn target(self: &mut Self) -> &Texture;
    /// Expected to processes input data. Simple postprocessors may not need to implement this.
    #[allow(unused_variables)]
    fn process(self: &mut Self, renderer: &Renderer) { }
    /// Expected to draw final result to current target using given blendmode.
    fn draw(self: &mut Self, renderer: &Renderer, blendmode: BlendMode);
}

mod basic;

pub mod postprocessors {
    pub use super::basic::Basic;
}
