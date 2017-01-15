use glium;
use glium::DisplayBuild;
use glium::glutin::{WindowBuilder, Event, ElementState, MouseButton/*, VirtualKeyCode*/};
use prelude::*;
use core::input::{InputData, InputState};
use core::monitor;

/// A struct describing a [`Display`](struct.Display.html) to be created.
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

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
    input_data: Arc<RwLock<InputData>>,
}

pub fn handle(display: &Display) -> &glium::Display {
    &display.handle
}

pub fn input_data(display: &Display) -> &Arc<RwLock<InputData>> {
    &display.input_data
}

impl Display {

    /// Creates a new instance from given [`DisplayInfo`](struct.DisplayInfo.html).
    pub fn new(descriptor: DisplayInfo) -> Display {

        let mut builder = WindowBuilder::new()
            .with_dimensions(descriptor.width, descriptor.height)
            .with_title(descriptor.title)
            .with_transparency(descriptor.transparent)
            .with_decorations(descriptor.decorations);

        if descriptor.monitor >= 0 {
            let monitor = Self::monitor(descriptor.monitor as u32);
            if monitor.is_some() {
                builder = builder.with_fullscreen(monitor::get_id(monitor.unwrap()));
            }
            // !todo error
        }
        if descriptor.vsync {
            builder = builder.with_vsync();
        }

        Display {
            handle: builder.build_glium().unwrap(),
            input_data: Arc::new(RwLock::new(InputData::new())),
        }
    }

    /// Sets the window title.
    pub fn set_title(self: &Self, title: &str) {
        self.window().set_title(title);
    }

    /// Makes the previously hidden window visible.
    pub fn show(self: &Self) {
        self.window().show();
    }

    /// Hides the window.
    pub fn hide(self: &Self) {
        self.window().hide();
    }

    /// Enables cursor grab mode. While in this mode, the mouse cursor will be hidden and
    /// constrained to the window. Additionally, [`Input`](struct.Input.html) will be able to
    /// provide mouse movement deltas and allow mouse coordinates to exceed the window-bounds.
    ///
    /// Grab mode will be temporarily released when the window loses focus and automatically
    /// restored once it regains focus.
    pub fn grab_cursor(self: &Self) {
        let window = self.window();
        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = true;
        window.set_cursor_position(100, 100).unwrap();
    }

    /// Hides the mouse cursor while it is inside the window.
    pub fn hide_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Hide).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Releases a previously grabbed or hidden cursor and makes it visible again.
    pub fn free_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Returns the window dimensions.
    pub fn dimensions(self: &Self) -> (u32, u32) {
        self.handle.get_framebuffer_dimensions()
    }

    /// Returns monitor details for given monitor id.
    pub fn monitor(index: u32) -> Option<monitor::Monitor> {
        let mut iter = glium::glutin::get_available_monitors();
        let result = iter.nth(index as usize);
        if result.is_some() {
            Some(monitor::from_id(result.unwrap()))
        } else {
            None
        }
    }

    /// Returns a vector of available monitors.
    pub fn monitors() -> Vec<monitor::Monitor> {
        let iter = glium::glutin::get_available_monitors();
        let mut result = Vec::<monitor::Monitor>::new();
        for monitor in iter {
            result.push(monitor::from_id(monitor));
        }
        result
    }
/*
    pub fn from_window_builder(builder: WindowBuilder<'static>) -> Display {
        Display {
            handle: builder.build_glium().unwrap(),
            input_data: Arc::new(RwLock::new(InputData::new())),
        }
    }
*/

    /// Polls for events like keyboard or mouse input and changes to the window. See
    /// [`Input`](struct.Input.html) for basic keyboard and mouse support.
    pub fn poll_events(self: &Self) -> &Self {
        let mut input_data = self.input_data.write().unwrap();
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
                    let new_state = if element_state == ElementState::Pressed { InputState::Down } else { InputState::Up };
                    let current_state = input_data.key[scan_code as usize];

                    input_data.key[scan_code as usize] = if current_state == InputState::Up && new_state == InputState::Down {
                        InputState::Pressed
                    } else if current_state == InputState::Down && new_state == InputState::Up {
                        InputState::Released
                    } else {
                        new_state
                    };

                    //println!("key: {}", scan_code);
                },
                Event::MouseMoved(x, y) => {
                    if input_data.cursor_grabbed {
                        let center = ((input_data.dimensions.0 / 2) as i32, (input_data.dimensions.1 / 2) as i32);
                        let old_mouse = input_data.mouse;
                        let delta = (x - center.0, y - center.1);
                        input_data.mouse = (old_mouse.0 + delta.0, old_mouse.1 + delta.1);
                        input_data.mouse_delta = delta;
                        window.set_cursor_position(center.0, center.1).unwrap();
                    } else {
                        input_data.mouse = (x, y);
                    }
                },
                Event::MouseInput(element_state, button) => {
                    let button_id = match button {
                        MouseButton::Left => 1,
                        MouseButton::Middle => 2,
                        MouseButton::Right => 3,
                        MouseButton::Other(x) => x,
                    };
                    let new_state = if element_state == ElementState::Pressed { InputState::Down } else { InputState::Up };
                    let current_state = input_data.button[button_id as usize];

                    input_data.button[button_id as usize] = if current_state == InputState::Up && new_state == InputState::Down {
                        InputState::Pressed
                    } else if current_state == InputState::Down && new_state == InputState::Up {
                        InputState::Released
                    } else {
                        new_state
                    };
                },
                Event::Focused(true) => {
                    // restore grab after focus loss
                    if input_data.cursor_grabbed {
                        window.set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
                        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
                    }
                }
                Event::Closed => {
                    input_data.should_close = true;
                }
                _ => ()
            }
        }

        input_data.dimensions = window.get_inner_size_pixels().unwrap_or((0, 0));

        self
    }

    /// Returns true once after the attached window was closed
    pub fn was_closed(self: &Self) -> bool {
        let mut input_data = self.input_data.write().unwrap();
        let result = input_data.should_close;
        input_data.should_close = false;
        result
    }

    /// returns a reference to the underlying glutin window
    fn window(self: &Self) -> glium::backend::glutin_backend::WinRef {
        self.handle.get_window().unwrap()
    }
}
