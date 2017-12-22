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
pub use maths::{Point2, Rect};

// doc has trouble with another maths module
pub mod math {
    //! Mostly optional math structs.
    pub use maths::{Mat4, Mat4Stack, Vec2, Vec3, Angle, VecType};
}

pub mod utils {
    //! Optional utility features. These may eventually be moved into the example code or a separate library.
    pub use misc::{renderloop, mainloop, LoopState, lerp, approach, min, max, Rng, Periodic};
}

pub mod support {
    //! Support structures returned by various methods. Usually not required to be created manually.
    pub use core::{InputIterator, InputUpIterator, InputDownIterator};
    pub use core::{DrawBuilder, DisplayBuilder, FontBuilder, FontQueryBuilder, TextureBuilder};
}

pub mod backend {
    //! Backend specific integration methods. Backends can be switched via [cargo features](http://doc.crates.io/manifest.html#the-features-section).
    //! The documentation shown here depends on the features it was generated with.
    pub use backends::backend::public::*;
}