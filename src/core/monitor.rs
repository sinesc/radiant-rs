use crate::prelude::*;
use crate::backends::backend;

/// A monitor display mode (resolution, bit depth, refresh rate).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VideoMode {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// Bit depth: how many bits are available per color. Generally 24 or 32
    /// on modern systems, depending on whether the alpha channel is counted or not.
    pub depth: u16,
    /// Refresh rate in millihertz, e.g. `60_000` for 60 Hz.
    pub refresh_rate: u32,
}

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
        let (width, _) = self.inner.get_dimensions();
        width
    }

    /// Returns the current height in pixels.
    pub fn height(self: &Self) -> u32 {
        let (_, height) = self.inner.get_dimensions();
        height
    }

    /// Returns the current width and height in pixels.
    pub fn dimensions(self: &Self) -> (u32, u32) {
        self.inner.get_dimensions()
    }

    /// Returns the display modes supported by this monitor.
    pub fn supported_modes(self: &Self) -> Vec<VideoMode> {
        self.inner.supported_modes()
    }
}
