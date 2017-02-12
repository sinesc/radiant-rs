pub mod blendmodes;

use glium;

/// A blendmode for use with `Layer::set_blendmode()`.
/// See [blendmodes](blendmodes/index.html) for a list of predefined modes.
#[derive(Copy, Clone)]
pub struct BlendMode (glium::draw_parameters::Blend);

impl BlendMode {
    pub fn set(&mut self, other: BlendMode) {
        self.0 = other.0;
    }
}

pub fn inner(blendmode: &BlendMode) -> glium::draw_parameters::Blend {
    blendmode.0
}
