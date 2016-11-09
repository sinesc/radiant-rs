

pub use core::{BlendMode, blendmodes, Display, DisplayInfo, Monitor, Layer, Renderer, RenderContext, Sprite, Font, FontInfo, Input, ButtonState, Color};
pub use maths::{Mat4, Vec2, Vec3, VecType};

pub mod utils {
    //! Various optional utility features including a Scene and mainloop helpers.
    pub use misc::{renderloop, mainloop, Rng};
}

pub mod scene {
    //! Optional ultility wrapper to simplify scene handling.
    pub use core::{OpId, LayerId, SpriteId, FontId, Op, Scene};
}
