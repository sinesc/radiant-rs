use std::fmt;
use glium::uniforms::{AsUniformValue, UniformValue};
use glium::vertex::{Attribute, AttributeType};

#[derive(Copy, Clone, Default)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

// !todo use f32's so that both attribute and uniform use result in the same gl representation
impl Color {

    //const RED: Color = Color(1.0, 0, 0, 1.0);

    pub fn new() -> Color {
        Color(0.0, 0.0, 0.0, 0.0)
    }

    pub fn transparent() -> Color {
        Color(0.0, 0.0, 0.0, 0.0)
    }

    pub fn alpha(alpha: f32) -> Color {
        Color(1.0, 1.0, 1.0, alpha)
    }

    pub fn alpha_pm(alpha: f32) -> Color {
        Color(alpha, alpha, alpha, alpha)
    }

    pub fn lightness(value: f32) -> Color {
        Color(value, value, value, 1.0)
    }

    pub fn white() -> Color {
        Color(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Color {
        Color(0.0, 0.0, 0.0, 1.0)
    }

    pub fn red() -> Color {
        Color(1.0, 0.0, 0.0, 1.0)
    }

    pub fn green() -> Color {
        Color(0.0, 1.0, 0.0, 1.0)
    }

    pub fn blue() -> Color {
        Color(0.0, 0.0, 1.0, 1.0)
    }

    pub fn yellow() -> Color {
        Color(1.0, 1.0, 0.0, 1.0)
    }

    pub fn cyan() -> Color {
        Color(0.0, 1.0, 1.0, 1.0)
    }

    pub fn purple() -> Color {
        Color(1.0, 0.0, 1.0, 1.0)
    }

    pub fn r(&self) -> f32 {
        self.0
    }

    pub fn g(&self) -> f32 {
        self.1
    }

    pub fn b(&self) -> f32 {
        self.2
    }

    pub fn a(&self) -> f32 {
        self.3
    }

    pub fn set_r(&mut self, value: f32) -> &mut Color {
        self.0 = value;
        self
    }

    pub fn set_g(&mut self, value: f32) -> &mut Color {
        self.1 = value;
        self
    }

    pub fn set_b(&mut self, value: f32) -> &mut Color {
        self.2 = value;
        self
    }

    pub fn set_a(&mut self, value: f32) -> &mut Color {
        self.3 = value;
        self
    }

    pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
    }

    /*
     * creates new color instance from given color temperature value
     * based on http://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/
     *
     */
/*    pub fn set_temperature(temperature: f32, alpha: f32) {

        let value = (temperature / 100) | 0;
        let red = 0, green = 0, blue = 0;

        if (value <= 66) {
            red = 255;
            green = (99.4708025861 * Math.log(value) - 161.1195681661) | 0;
        } else {
            red = (329.698727466 * Math.pow(value - 60, -0.1332047592)) | 0;
            green = (288.1221695283 * Math.pow(value - 60, -0.0755148492)) | 0;
        }

        if (value >= 66) {
            blue = 255;
        } else if (value <= 19) {
            blue = 0;
        } else {
            blue = (138.5177312231 * Math.log(value - 10) - 305.0447927307) | 0;
        }

        return new Color(Math.max(0, Math.min(255, red)),
                         Math.max(0, Math.min(255, green)),
                         Math.max(0, Math.min(255, blue)),
                         alpha);

    }
*/
}

unsafe impl Attribute for Color {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

impl AsUniformValue for Color {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec4([ self.0, self.1, self.2, self.3 ])
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}
