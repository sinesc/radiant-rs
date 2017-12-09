pub mod blendmodes;
use core::{Color};

/// A blendmode for use with [`Layer::set_blendmode()`](struct.Layer.html#method.set_blendmode).
/// See [blendmodes](blendmodes/index.html) for a list of predefined modes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlendMode {
    /// The blending function for color channels.
    pub color: BlendingFunction,
    /// The blending function for alpha channels.
    pub alpha: BlendingFunction,
    /// A constant color that can be used in the blending functions.
    pub constant_value: Color,
}

impl Default for BlendMode {
    fn default() -> BlendMode {
        BlendMode {
            color: BlendingFunction::AlwaysReplace,
            alpha: BlendingFunction::AlwaysReplace,
            constant_value: Color::WHITE,
        }
    }
}

/// Function that the GPU will use for blending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendingFunction {
    /// Simply overwrite the destination pixel with the source pixel.
    ///
    /// The alpha channels are simply ignored. This is the default mode.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.5, 0.9, 0.4, 0.2)`.
    AlwaysReplace,

    /// For each individual component (red, green, blue, and alpha), the minimum value is chosen
    /// between the source and the destination.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.5, 0.1, 0.4, 0.2)`.
    Min,

    /// For each individual component (red, green, blue, and alpha), the maximum value is chosen
    /// between the source and the destination.
    ///
    /// For example writing `(0.5, 0.9, 0.4, 0.2)` over `(0.9, 0.1, 0.4, 0.3)` will
    /// result in `(0.9, 0.9, 0.4, 0.3)`.
    Max,

    /// For each individual component (red, green, blue, and alpha), a weighted addition
    /// between the source and the destination.
    ///
    /// The result is equal to `source_component * source_factor + dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    Addition {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },

    /// For each individual component (red, green, blue, and alpha), a weighted substraction
    /// of the source by the destination.
    ///
    /// The result is equal to `source_component * source_factor - dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    Subtraction {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },

    /// For each individual component (red, green, blue, and alpha), a weighted substraction
    /// of the destination by the source.
    ///
    /// The result is equal to `-source_component * source_factor + dest_component * dest_factor`,
    /// where `source_factor` and `dest_factor` are the values of `source` and `destination` of
    /// this enum.
    ReverseSubtraction {
        /// The factor to apply to the source pixel.
        source: LinearBlendingFactor,

        /// The factor to apply to the destination pixel.
        destination: LinearBlendingFactor,
    },
}

/// Indicates which value to multiply each component with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearBlendingFactor {
    /// Multiply the source or destination component by zero, which always
    /// gives `0.0`.
    Zero,

    /// Multiply the source or destination component by one, which always
    /// gives you the original value.
    One,

    /// Multiply the source or destination component by its corresponding value
    /// in the source.
    ///
    /// If you apply this to the source components, you get the values squared.
    SourceColor,

    /// Equivalent to `1 - SourceColor`.
    OneMinusSourceColor,

    /// Multiply the source or destination component by its corresponding value
    /// in the destination.
    ///
    /// If you apply this to the destination components, you get the values squared.
    DestinationColor,

    /// Equivalent to `1 - DestinationColor`.
    OneMinusDestinationColor,

    /// Multiply the source or destination component by the alpha value of the source.
    SourceAlpha,

    /// Multiply the source or destination component by the smallest value of
    /// `SourceAlpha` and `1 - DestinationAlpha`.
    SourceAlphaSaturate,

    /// Multiply the source or destination component by `1.0` minus the alpha value of the source.
    OneMinusSourceAlpha,

    /// Multiply the source or destination component by the alpha value of the destination.
    DestinationAlpha,

    /// Multiply the source or destination component by `1.0` minus the alpha value of the
    /// destination.
    OneMinusDestinationAlpha,

    /// Multiply the source or destination component by the corresponding value
    /// in `Blend::const_value`.
    ConstantColor,

    /// Multiply the source or destination compoent by `1.0` minus the corresponding
    /// value in `Blend::const_value`.
    OneMinusConstantColor,

    /// Multiply the source or destination component by the alpha value of `Blend::const_value`.
    ConstantAlpha,

    /// Multiply the source or destination componet by `1.0` minus the alpha value of
    /// `Blend::const_value`.
    OneMinusConstantAlpha,
}
