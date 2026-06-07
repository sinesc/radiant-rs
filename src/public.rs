pub use crate::core::{
    BlendMode, BlendingFunction, LinearBlendingFactor, blendmodes,
    Display, Monitor,
    Renderer, RenderTarget, Context, AsRenderTarget,
    Layer, Sprite, Font, Color,
    Texture, TextureFormat, TextureFilter, TextureWrap,
    Program, Uniform, AsUniform,
    Postprocessor, postprocessors,
    Input, InputId, InputState,
    Result, Error
};

pub mod support {
    //! Support structures returned by various methods. Usually not required to be created manually.
    pub use crate::core::{InputIterator, InputUpIterator, InputDownIterator};
    pub use crate::core::{DrawBuilder, DisplayBuilder, FontBuilder, FontQueryBuilder, TextureBuilder};
    pub use crate::core::{SpriteParameters, SpriteLayout};
    pub use crate::core::Mat4Stack;
}

pub mod backend {
    //! Backend specific integration methods. Backends can be switched via [cargo features](http://doc.crates.io/manifest.html#the-features-section).
    //! The documentation shown here depends on the features it was generated with.
    #[allow(unused_imports)]
    pub use crate::backends::backend::public::*;
}