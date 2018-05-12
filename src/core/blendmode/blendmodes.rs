
//! A set of predefined blendmodes for use with `Layer::set_blendmode()`.
//!
//! See [`BlendMode`](../struct.BlendMode.html) to define your own blendmodes.

use super::*;
use core::{Color};

/// Replaces source into destination, overwriting color and alpha values.
pub const COPY: BlendMode = BlendMode {
    color: BlendingFunction::AlwaysReplace,
    alpha: BlendingFunction::AlwaysReplace,
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Alpha-blends source into destination.
pub const ALPHA: BlendMode = BlendMode {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Adds source and destination.
pub const ADD: BlendMode = BlendMode {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Subtracts source from destination.
pub const SUBTRACT: BlendMode = BlendMode {
    color: BlendingFunction::Subtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::Subtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Subtracts destination from source.
pub const REVERSE_SUBTRACT: BlendMode = BlendMode {
    color: BlendingFunction::ReverseSubtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    alpha: BlendingFunction::ReverseSubtraction {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::One,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Writes the maximum of source and destination into destination.
pub const LIGHTEN: BlendMode = BlendMode {
    color: BlendingFunction::Max,
    alpha: BlendingFunction::Max,
    constant_value: Color(0.0, 0.0, 0.0, 0.0),
};

/// Writes the minimum of source and destination into destination.
pub const DARKEN: BlendMode = BlendMode {
    color: BlendingFunction::Min,
    alpha: BlendingFunction::Min,
    constant_value: Color(0.0, 0.0, 0.0, 0.0),
};

/// Alpha-blends source into destination.
pub const SQUARED: BlendMode = BlendMode {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceColor,
        destination: LinearBlendingFactor::DestinationColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Overlays source and destination.
pub const SCREEN: BlendMode = BlendMode {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Overlays source and destination, masking the destination where source alpha is low.
pub const SCREEN_MASK: BlendMode = BlendMode {
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceColor,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::One,
        destination: LinearBlendingFactor::OneMinusSourceAlpha,
    },
    constant_value: Color(0.0, 0.0, 0.0, 0.0)
};

/// Lika ALPHA but multiplies source with given color.
pub fn colorize(color: Color) -> BlendMode {
   BlendMode {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantAlpha,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantColor,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        constant_value: color
    }
}

/// Fades between source at 1.0 and destination at 0.0
pub fn fade(value: f32) -> BlendMode {
   BlendMode {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::ConstantAlpha,
            destination: LinearBlendingFactor::ConstantAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::OneMinusConstantAlpha,
            destination: LinearBlendingFactor::OneMinusConstantAlpha,
        },
        constant_value: Color(1., 1., 1., value)
    }
}
