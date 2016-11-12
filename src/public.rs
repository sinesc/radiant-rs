pub use core::{BlendMode, blendmodes, Display, DisplayInfo, Monitor, Layer, Renderer, RenderContext, Sprite, Font, FontInfo, Input, ButtonState, Color};
pub use maths::{Mat4, Vec2, Vec3, VecType};

pub mod utils {
    //! Optional utility features.
    pub use misc::{renderloop, mainloop, Rng};
}

pub mod scene {
    //! Optional scene abstraction.
    pub use core::{OpId, LayerId, SpriteId, FontId, Op, Scene};
}
