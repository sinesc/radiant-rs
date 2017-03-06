pub use core::{
    BlendMode, blendmodes,
    Display, Monitor,
    Renderer, RenderTarget, RenderContext, AsRenderTarget,
    Layer, Sprite, Font, Color,
    Texture, TextureFormat, TextureFilter, TextureWrap,
    Program, Uniform, AsUniform,
    Postprocessor, postprocessors,
    Input, InputId, InputState,
    Result, Error
};
pub use maths::{Mat4, Vec2, Vec3, Angle, Point2, Rect, VecType};

pub mod utils {
    //! Optional utility features. These may eventually be moved into the example code or a separate library.
    pub use misc::{renderloop, mainloop, LoopState, lerp, approach, min, max, Rng, Periodic};
}

pub mod support {
    //! Support structures returned by various methods. Usually not required to be created manually.

    pub use core::{InputIterator, InputUpIterator, InputDownIterator};
    pub use core::{DrawBuilder, DisplayBuilder, FontBuilder, FontQueryBuilder, TextureBuilder};
}
