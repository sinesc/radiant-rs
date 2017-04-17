
//! A set of predefined blendmodes for use with `Layer::set_blendmode()`.
//!
//! [WIP] These are mostly placeholders for now.

use glium::draw_parameters::*;
use super::BlendMode;
use core::{Color};

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

/// Adds source and destination.
pub const ADD: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Subtracts source from destination.
pub const SUBTRACT: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Subtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::Subtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Subtracts destination from source.
pub const REVERSE_SUBTRACT: BlendMode = BlendMode(Blend {
    color: BlendingFunction::ReverseSubtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::ReverseSubtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Writes the maximum of source and destination into destination.
pub const LIGHTEN: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Max,
    alpha: BlendingFunction::Max,
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Writes the minimum of source and destination into destination.
pub const DARKEN: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Min,
    alpha: BlendingFunction::Min,
    constant_value: (0.0, 0.0, 0.0, 0.0),
});

/// Alpha-blends source into destination.
pub const SQUARED: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceColor,
        destination: LinearBlendingFactor::DestinationColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Overlays source and destination.
pub const SCREEN: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Overlays source and destination, masking the destination where source alpha is low.
pub const SCREEN_MASK: BlendMode = BlendMode(Blend {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
});

/// Lika ALPHA but multiplies source with given color.
pub fn colorize(color: Color) -> BlendMode {
   BlendMode(Blend {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantAlpha,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantColor,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        constant_value: color.into()
    })
}

/// Fades between source at 1.0 and destination at 0.0
pub fn fade(value: f32) -> BlendMode {
   BlendMode(Blend {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantAlpha,
            destination: LinearBlendingFactor::ConstantAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::OneMinusConstantAlpha,
            destination: LinearBlendingFactor::OneMinusConstantAlpha,
        },
        constant_value: (1., 1., 1., value)
    })
}
