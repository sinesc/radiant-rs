use glium;
use glium::DisplayBuild;
use glium::glutin::WindowBuilder;
use prelude::*;
use graphics::{input, Display, InputState};

pub struct Monitor {
    id: glium::glutin::MonitorId,
}

impl Monitor {
    pub fn name(&self) -> String {
        self.id.get_name().unwrap_or("".to_string())
    }
    pub fn width(&self) -> u32 {
        let (width, _) = self.id.get_dimensions();
        width
    }
    pub fn height(&self) -> u32 {
        let (_, height) = self.id.get_dimensions();
        height
    }
    pub fn dimensions(&self) -> (u32, u32) {
        self.id.get_dimensions()
    }
}

#[derive(Clone)]
pub struct DisplayInfo {
    pub width       : u32,
    pub height      : u32,
    pub title       : String,
    pub transparent : bool,
    pub decorations : bool,
    pub monitor     : i32,
    pub vsync       : bool,
}

impl Default for DisplayInfo {
    fn default() -> DisplayInfo {
        DisplayInfo {
            width       : 640,
            height      : 480,
            title       : "".to_string(),
            transparent : false,
            decorations : true,
            monitor     : -1,
            vsync       : false,
        }
   }
}

impl Display {
    pub fn new(descriptor: DisplayInfo) -> Display {

        let mut builder = WindowBuilder::new()
            .with_dimensions(descriptor.width, descriptor.height)
            .with_title(descriptor.title)
            .with_transparency(descriptor.transparent)
            .with_decorations(descriptor.decorations);

        if descriptor.monitor >= 0 {
            let monitor = Self::monitor(descriptor.monitor as u32);
            if monitor.is_some() {
                builder = builder.with_fullscreen(monitor.unwrap().id);
            }
            // !todo error
        }
        if descriptor.vsync {
            builder = builder.with_vsync();
        }

        Display {
            handle: builder.build_glium().unwrap(),
            input_state: Arc::new(RwLock::new(InputState::new())),
        }
    }

    pub fn dimensions(self: &Self) -> (u32, u32) {
        self.handle.get_framebuffer_dimensions()
    }

    pub fn monitor(index: u32) -> Option<Monitor> {
        let mut iter = glium::glutin::get_available_monitors();
        let result = iter.nth(index as usize);
        if result.is_some() {
            Some(Monitor {
                id: result.unwrap()
            })
        } else {
            None
        }
    }

    pub fn monitors() -> Vec<Monitor> {
        let iter = glium::glutin::get_available_monitors();
        let mut result = Vec::<Monitor>::new();
        for monitor in iter {
            result.push(Monitor {
                id: monitor,
            });
        }
        result
    }

    pub fn from_window_builder(builder: WindowBuilder<'static>) -> Display {
        Display {
            handle: builder.build_glium().unwrap(),
            input_state: Arc::new(RwLock::new(InputState::new())),
        }
    }

    pub fn poll_events(&self) -> &Self {
        input::poll_events(self);
        self
    }
}
