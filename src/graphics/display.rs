use glium;
use glium::DisplayBuild;
use glium::glutin::{WindowBuilder, Event, ElementState, MouseButton/*, VirtualKeyCode*/};
use prelude::*;
use graphics::{Display, InputState};

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

    pub fn set_title(self: &Self, title: &str) {
        self.window().set_title(title);
    }

    pub fn show(self: &Self) {
        self.window().show();
    }

    pub fn hide(self: &Self) {
        self.window().hide();
    }

    pub fn grab_cursor(self: &Self) {
        let window = self.window();
        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
        self.input_state.write().unwrap().cursor_grabbed = true;
        window.set_cursor_position(100, 100).unwrap();
    }

    pub fn hide_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Hide).unwrap();
        self.input_state.write().unwrap().cursor_grabbed = false;
    }

    pub fn free_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
        self.input_state.write().unwrap().cursor_grabbed = false;
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

    pub fn poll_events(self: &Self) -> &Self {
        let mut input_state = self.input_state.write().unwrap();
        let window = self.window();

        for event in self.handle.poll_events() {
            match event {
                // !todo vkeys seem broken
                /*Event::KeyboardInput(element_state, scan_code, Some(virtual_code)) => {
                    let new_state = if element_state == ElementState::Pressed { true } else { false };
                    match virtual_code {
                        VirtualKeyCode::LAlt => {
                            self.alt_left = new_state;
                        },
                        VirtualKeyCode::RAlt => {
                            self.alt_right = new_state;
                        },
                        VirtualKeyCode::LShift => {
                            self.shift_left = new_state;
                        },
                        VirtualKeyCode::RShift => {
                            self.shift_right = new_state;
                        },
                        VirtualKeyCode::LControl => {
                            self.ctrl_left = new_state;
                        },
                        VirtualKeyCode::RControl => {
                            self.ctrl_right = new_state;
                        },
                        VirtualKeyCode::Escape => {
                            self.escape = new_state;
                        },
                        _ => {
                            println!("no idea");
                        }
                    }
                },*/
                Event::KeyboardInput(element_state, scan_code, _) => {
                    let new_state = if element_state == ElementState::Pressed { true } else { false };
                    input_state.key[scan_code as usize] = new_state;
                    //println!("key: {}", scan_code);
                },
                Event::MouseMoved(x, y) => {
                    if input_state.cursor_grabbed {
                        let center = ((input_state.dimensions.0 / 2) as i32, (input_state.dimensions.1 / 2) as i32);
                        let old_mouse = input_state.mouse;
                        let delta = (x - center.0, y - center.1);
                        input_state.mouse = (old_mouse.0 + delta.0, old_mouse.1 + delta.1);
                        input_state.mouse_delta = delta;
                        window.set_cursor_position(center.0, center.1).unwrap();
                    } else {
                        input_state.mouse = (x, y);
                    }
                },
                Event::MouseInput(element_state, button) => {
                    let new_state = if element_state == ElementState::Pressed { true } else { false };
                    if button == MouseButton::Left {
                        input_state.button.0 = new_state;
                    } else if button == MouseButton::Middle {
                        input_state.button.1 = new_state;
                    } else if button == MouseButton::Right {
                        input_state.button.2 = new_state;
                    }
                },
                Event::Focused(true) => {
                    // restore grab after focus loss
                    if input_state.cursor_grabbed {
                        window.set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
                        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
                    }
                }
                Event::Closed => {
                    input_state.should_close = true;
                }
                _ => ()
            }
        }

        input_state.dimensions = window.get_inner_size_pixels().unwrap_or((0, 0));

        self
    }

    fn window(self: &Self) -> glium::backend::glutin_backend::WinRef {
        self.handle.get_window().unwrap()
    }
}
