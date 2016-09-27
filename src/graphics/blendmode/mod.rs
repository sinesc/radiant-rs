
pub mod blendmodes;

use glium;

#[derive(Copy, Clone)]
pub struct BlendMode (glium::draw_parameters::Blend);

impl BlendMode {
    pub fn set(&mut self, other: BlendMode) {
        self.0 = other.0;
    }
}

pub fn access_blendmode(blendmode: &BlendMode) -> glium::draw_parameters::Blend {
    blendmode.0
}
