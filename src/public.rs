pub use core::{
    BlendMode, blendmodes,
    Display, DisplayInfo, Monitor,
    Renderer, RenderContext, RenderTarget, AsRenderTarget,
    Layer, Sprite, Font, FontInfo, Color,
    Texture, TextureInfo, TextureFormat, TextureFilter, TextureWrap,
    Program, Uniform, AsUniform,
    Postprocessor, postprocessors,
    Input, InputId, InputState, InputIterator, InputUpIterator, InputDownIterator,
    Result, Error
};
pub use maths::{Mat4, Vec2, Vec3, Angle, Point2, Rect, VecType};

pub mod utils {
    //! Optional utility features. These may eventually be moved into the example code or a separate library.
    pub use misc::{renderloop, mainloop, LoopState, lerp, approach, min, max, Rng, Periodic};
}

pub mod builders {
    //! Builder structures returned by various methods.
    pub use core::DrawRectBuilder;
    pub use core::DisplayBuilder;
    pub use core::FontBuilder;
    pub use core::TextureBuilder;
}

#[deprecated(note="Removed for being out of scope of this library")]
#[allow(deprecated)]
pub mod scene {
    //! Optional scene abstraction.
    //!
    //! Currently ~~**work in progress**~~ and not particularly useful yet.
    pub use core::{OpId, LayerId, SpriteId, FontId, Op, Scene};
}
