pub use core::{
    BlendMode, blendmodes,
    Display, DisplayInfo, Monitor,
    Renderer, RenderContext, RenderTarget,
    Layer, Sprite, Font, FontInfo, Texture, TextureFilter, TextureWrap, Color,
    Program, Uniform, AsUniform,
    Postprocessor, postprocessors,
    Input, InputId, InputState, InputIterator, InputUpIterator, InputDownIterator,
    Result, Error
};
pub use maths::{Mat4, Vec2, Vec3, Angle, Point2, Point3, Rect, VecType};

pub mod utils {
    //! Optional utility features. These may eventually be moved into the example code or a separate library.
    pub use misc::{renderloop, mainloop, LoopState, lerp, approach, min, max, Rng, Periodic};
}

#[deprecated(note="Removed for being out of scope of this library")]
#[allow(deprecated)]
pub mod scene {
    //! Optional scene abstraction.
    //!
    //! Currently **work in progress** and not particularly useful yet.
    pub use core::{OpId, LayerId, SpriteId, FontId, Op, Scene};
}
