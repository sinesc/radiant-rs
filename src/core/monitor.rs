use glium;

/// An individual monitor, returned from [`Display::monitors()`](struct.Display.html#method.monitors).
pub struct Monitor {
    id: glium::glutin::MonitorId,
}

impl Monitor {
    /// Returns the name of the device.
    pub fn name(&self) -> String {
        self.id.get_name().unwrap_or("".to_string())
    }

    /// Returns the current width in pixels.
    pub fn width(&self) -> u32 {
        let (width, _) = self.id.get_dimensions();
        width
    }

    /// Returns the current height in pixels.
    pub fn height(&self) -> u32 {
        let (_, height) = self.id.get_dimensions();
        height
    }

    /// Returns the current width and height in pixels.
    pub fn dimensions(&self) -> (u32, u32) {
        self.id.get_dimensions()
    }
}

pub fn get_id(monitor: Monitor) -> glium::glutin::MonitorId {
    monitor.id
}

pub fn from_id(id: glium::glutin::MonitorId) -> Monitor {
    Monitor {
        id: id
    }
}
