use prelude::*;
use backends::backend;

/// An individual monitor, returned from [`Display::monitors()`](struct.Display.html#method.monitors).
#[derive(Clone)]
pub struct Monitor {
    pub(crate) inner: backend::Monitor,
}

impl fmt::Debug for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Monitor")
    }
}

impl Monitor {
    pub(crate) fn new(monitor: backend::Monitor) -> Self {
        Self {
            inner: monitor
        }
    }

    /// Returns the name of the device.
    pub fn name(self: &Self) -> String {
        self.inner.get_name().unwrap_or("".to_string())
    }

    /// Returns the current width in pixels.
    pub fn width(self: &Self) -> u32 {
        let (width, _) = self.inner.get_dimensions().into();
        width
    }

    /// Returns the current height in pixels.
    pub fn height(self: &Self) -> u32 {
        let (_, height) = self.inner.get_dimensions().into();
        height
    }

    /// Returns the current width and height in pixels.
    pub fn dimensions(self: &Self) -> (u32, u32) {
        self.inner.get_dimensions().into()
    }
}
