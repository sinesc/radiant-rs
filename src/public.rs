pub use core::{
    blendmodes, BlendMode,
    Display, DisplayInfo, Monitor,
    Renderer, RenderContext,
    Layer, Sprite, Font, FontInfo, Texture, Program,
    Color,
    Input, InputId, InputState, InputIterator, InputUpIterator, InputDownIterator,
};
pub use maths::{Mat4, Vec2, Vec3, Angle, Point2, Point3, Rect, VecType};

pub mod utils {
    //! Optional utility features.
    pub use misc::{renderloop, mainloop, approach, min, max, Rng, Periodic};
}

pub mod scene {
    //! Optional scene abstraction.
    pub use core::{OpId, LayerId, SpriteId, FontId, Op, Scene};
}
