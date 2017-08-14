use backends::glium as backend;

/// An individual monitor, returned from [`Display::monitors()`](struct.Display.html#method.monitors).
#[derive(Clone)]
pub struct Monitor {
    id: backend::Monitor,
}

impl Monitor {
    pub(crate) fn new(monitor: backend::Monitor) -> Self {
        Self {
            id: monitor
        }
    }

    /// Returns the name of the device.
    pub fn name(self: &Self) -> String {
        self.id.get_name().unwrap_or("".to_string())
    }

    /// Returns the current width in pixels.
    pub fn width(self: &Self) -> u32 {
        let (width, _) = self.id.get_dimensions();
        width
    }

    /// Returns the current height in pixels.
    pub fn height(self: &Self) -> u32 {
        let (_, height) = self.id.get_dimensions();
        height
    }

    /// Returns the current width and height in pixels.
    pub fn dimensions(self: &Self) -> (u32, u32) {
        self.id.get_dimensions()
    }

    pub(crate) fn inner(self: &Self) -> &backend::Monitor {
        &self.id
    }
}
