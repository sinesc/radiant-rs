
//! A set of predefined blendmodes for use with `Layer::set_blendmode()`.
//!
//! [WIP] These are mostly placeholders for now.

use glium::draw_parameters::*;
use super::BlendMode;

/// Replaces source into destination, overwriting color and alpha values.
pub const COPY: BlendMode = BlendMode(Blend {
     color: BlendingFunction::AlwaysReplace,
     alpha: BlendingFunction::AlwaysReplace,
     constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Alpha-blends source into destination.
pub const ALPHA: BlendMode = BlendMode(Blend {
     color: BlendingFunction::Addition {
         source: LinearBlendingFactor::One,
         destination: LinearBlendingFactor::OneMinusSourceAlpha,
     },
     alpha: BlendingFunction::Addition {
         source: LinearBlendingFactor::One,
         destination: LinearBlendingFactor::OneMinusSourceAlpha,
     },
     constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Writes the maximum of source and destination into destination.
pub const MAX: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Max,
    alpha: BlendingFunction::Max,
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Writes the minimum of source and destination into destination.
pub const MIN: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Min,
    alpha: BlendingFunction::Min,
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Additively lightens destination based on source. !PLACEHOLDER
pub const LIGHTEN: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceAlpha,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Additively overlays destination with source. !PLACEHOLDER
pub const OVERLAY: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceAlpha,
        destination: LinearBlendingFactor::SourceAlpha,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Sets the blend function for the layer (like alpha, but adds brightness value)
pub fn alpha_const(brightness: f32) -> BlendMode {
   BlendMode(Blend {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantAlpha,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::One,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        constant_value: (0.0, 0.0, 0.0, brightness)
    })
}
