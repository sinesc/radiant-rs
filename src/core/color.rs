use prelude::*;
use core::{Uniform, AsUniform};

/// A color value consisting of four floating point values for the color channels red, green, blue
/// and alpha.
///
/// Various drawing methods accept color instances to be used as multiplicators in the drawing
/// process, e.g. [`Sprite::draw()`](struct.Sprite.html#method.draw) allows multiplying the sprite-
/// texture's color channels by given color.
#[derive(Copy, Clone, Default, PartialEq)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

impl Color {

    //const RED: Color = Color(1.0, 0, 0, 1.0);

    /// Creates a new instance with a channels set to zero.
    pub fn transparent() -> Color {
        Color(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new instance with color channels set to one and the alpha channel set to given value.
    pub fn alpha(alpha: f32) -> Color {
        Color(1.0, 1.0, 1.0, alpha)
    }

    /// Creates a new instance with color channels set to zero and the alpha channel set to given value.
    pub fn alpha_mask(alpha: f32) -> Color {
        Color(0.0, 0.0, 0.0, alpha)
    }

    /// Creates a new instance with all channels set to given value.
    pub fn alpha_pm(alpha: f32) -> Color {
        Color(alpha, alpha, alpha, alpha)
    }

    /// Creates a new instance with color channels set to given value and the alpha channel set to one.
    pub fn lightness(value: f32) -> Color {
        Color(value, value, value, 1.0)
    }

    /// Creates a new instance from given HSL (range 0.0 - 1.0)
    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
        if saturation == 0.0 {
            Color(lightness, lightness, lightness, alpha)
        } else {
            let q = if lightness < 0.5 {
                lightness * (1.0 + saturation)
            } else {
                lightness + saturation - lightness * saturation
            };
            let p = 2.0 * lightness - q;
            Color(
                Self::hue_to_rgb(p, q, hue + 1.0 / 3.0),
                Self::hue_to_rgb(p, q, hue),
                Self::hue_to_rgb(p, q, hue - 1.0 / 3.0),
                alpha
            )
        }
    }

    /// Creates a new instance from given color-temperature (~1000 to ~40000).
    ///
    /// Based on http://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/
    pub fn from_temperature(temperature: f32, alpha: f32) -> Color {

        let value = (temperature / 100.0).floor();
        let red;
        let green;
        let blue;

        if value <= 66.0 {
            red = 255;
            green = (99.4708025861 * value.ln() - 161.1195681661) as i32;
        } else {
            red = (329.698727466 * (value - 60.0).powf(-0.1332047592)) as i32;
            green = (288.1221695283 * (value - 60.0).powf(-0.0755148492)) as i32;
        }

        if value >= 66.0 {
            blue = 255;
        } else if value <= 19.0 {
            blue = 0;
        } else {
            blue = (138.5177312231 * (value - 10.0).ln() - 305.0447927307) as i32;
        }

        Color(
            cmp::max(0, cmp::min(255, red)) as f32 / 255.0,
            cmp::max(0, cmp::min(255, green)) as f32 / 255.0,
            cmp::max(0, cmp::min(255, blue)) as f32 / 255.0,
            alpha
        )
    }

    /// Returns value of the instance's red channel.
    pub fn r(self: &Self) -> f32 {
        self.0
    }

    /// Returns value of the instance's green channel.
    pub fn g(self: &Self) -> f32 {
        self.1
    }

    /// Returns value of the instance's blue channel.
    pub fn b(self: &Self) -> f32 {
        self.2
    }

    /// Returns value of the instance's alpha channel.
    pub fn a(self: &Self) -> f32 {
        self.3
    }

    /// Sets the instance's channels from another color object.
    pub fn set(self: &mut Self, value: Color) {
        self.0 = value.0;
        self.1 = value.1;
        self.2 = value.2;
        self.3 = value.3;
    }

    /// Sets a value for the instance's red channel
    pub fn set_r(self: &mut Self, value: f32) -> &mut Color {
        self.0 = value;
        self
    }

    /// Sets a value for the instance's green channel.
    pub fn set_g(self: &mut Self, value: f32) -> &mut Color {
        self.1 = value;
        self
    }

    /// Sets a value for the instance's blue channel.
    pub fn set_b(self: &mut Self, value: f32) -> &mut Color {
        self.2 = value;
        self
    }

    /// Sets a value for the instance's alpha channel.
    pub fn set_a(self: &mut Self, value: f32) -> &mut Color {
        self.3 = value;
        self
    }

    /// Multiplies the instance's color channels by given scaling factor. Does not modify alpha.
    pub fn scale(self: &mut Self, scaling: f32) -> &mut Color {
        self.0 *= scaling;
        self.1 *= scaling;
        self.2 *= scaling;
        self
    }

    // Returns new instance with alpha applied to all color channels.
    pub fn to_pm(self: &Self) -> Color {
        Color(self.0 * self.3, self.1 * self.3, self.2 * self.3, self.3)
    }

    /// Returns opaque white color.
    pub fn white() -> Color {
        Color(1.0, 1.0, 1.0, 1.0)
    }

    /// Returns opaque black color.
    pub fn black() -> Color {
        Color(0.0, 0.0, 0.0, 1.0)
    }

    /// Returns opaque red color.
    pub fn red() -> Color {
        Color(1.0, 0.0, 0.0, 1.0)
    }

    /// Returns opaque green color.
    pub fn green() -> Color {
        Color(0.0, 1.0, 0.0, 1.0)
    }

    /// Returns opaque blue color.
    pub fn blue() -> Color {
        Color(0.0, 0.0, 1.0, 1.0)
    }

    /// Returns opaque yellow color.
    pub fn yellow() -> Color {
        Color(1.0, 1.0, 0.0, 1.0)
    }

    /// Returns opaque cyan color.
    pub fn cyan() -> Color {
        Color(0.0, 1.0, 1.0, 1.0)
    }

    /// Returns opaque magenta color.
    pub fn magenta() -> Color {
        Color(1.0, 0.0, 1.0, 1.0)
    }

    /// Hue to rgb helper function uses by hsl.
    fn hue_to_rgb(p: f32, q: f32, mut hue: f32) -> f32 {
        if hue < 0.0 {
            hue += 1.0;
        }
        if hue > 1.0 {
            hue -= 1.0;
        }
        if hue < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * hue;
        }
        if hue < 1.0 / 2.0 {
            return q;
        }
        if hue < 2.0 / 3.0 {
            return p + (q - p) * (2.0/3.0 - hue) * 6.0;
        }
        return p;
    }
}


impl<T> From<(T, T, T, T)> for Color where T: Debug + Float, f32: From<T> {
    fn from(source: (T, T, T, T)) -> Self {
        Color(source.0.into(), source.1.into(), source.2.into(), source.3.into())
    }
}

impl<T> From<[ T; 4 ]> for Color where T: Debug + Float, f32: From<T> {
    fn from(source: [ T; 4 ]) -> Self {
        Color(source[0].into(), source[1].into(), source[2].into(), source[3].into())
    }
}

impl From<Color> for [ f32; 4 ] {
    fn from(source: Color) -> Self {
        [ source.0, source.1, source.2, source.3 ]
    }
}

impl<'a> From<&'a Color> for [ f32; 4 ] {
    fn from(source: &'a Color) -> Self {
        [ source.0, source.1, source.2, source.3 ]
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(source: Color) -> Self {
        (source.0, source.1, source.2, source.3)
    }
}

impl<'a> From<&'a Color> for (f32, f32, f32, f32) {
    fn from(source: &'a Color) -> Self {
        (source.0, source.1, source.2, source.3)
    }
}

impl AsUniform for Color {
    fn as_uniform(self: &Self) -> Uniform {
        Uniform::Vec4([ self.0, self.1, self.2, self.3 ])
    }
}

impl fmt::Debug for Color {
    fn fmt(self: &Self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}
