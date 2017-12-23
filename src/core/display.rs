use prelude::*;
use core::input::{InputData, InputState, NUM_KEYS, NUM_BUTTONS};
use core::monitor;
use core::{AsRenderTarget, RenderTarget, Color};
use maths::Point2;
use core::builder::*;
use core::Event;
use backends::backend;

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    pub(crate) handle: backend::Display,
    pub(crate) frame: Rc<RefCell<Option<backend::Frame>>>,
    pub(crate) input_data: Arc<RwLock<InputData>>,
}

impl Display {

    /// Returns a [display builder](support/struct.DisplayBuilder.html) for display construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// let display = Display::builder().dimensions((640, 480)).vsync().title("Window!").build();
    /// ```
    pub fn builder() -> DisplayBuilder {
        DisplayBuilder::new()
    }

    /// Sets the window title.
    pub fn set_title(self: &Self, title: &str) {
        self.handle.set_title(title);
    }

    /// Makes the previously hidden window visible.
    pub fn show(self: &Self) {
        self.handle.show();
    }

    /// Hides the window.
    pub fn hide(self: &Self) {
        self.handle.hide();
    }

    /// Prepares a frame for rendering.
    pub fn prepare_frame(self: &Self) {
        if self.frame.borrow().is_some() {
            panic!("Current frame needs to be swapped before a new frame can be prepared.");
        }
        *self.frame.borrow_mut() = Some(self.handle.draw());
    }

    /// Prepares a frame for rendering and clears it.
    pub fn clear_frame(self: &Self, color: Color) {
        self.prepare_frame();
        if let Some(ref mut frame) = self.frame.borrow_mut().as_mut() {
            frame.clear(color);
        } else {
            panic!("Failed to prepare a frame for clear.");
        }
    }

    /// Swaps current drawing frame with visible frame.
    pub fn swap_frame(self: &Self) {
        let frame = mem::replace(&mut *self.frame.borrow_mut(), None);
        if let Some(frame) = frame {
            frame.swap();
        } else {
            panic!("No frame currently prepared, nothing to swap.");
        }
    }

    /// Enables cursor grab mode. While in this mode, the mouse cursor will be hidden and
    /// constrained to the window. Additionally, [`Input`](struct.Input.html) will be able to
    /// provide mouse movement deltas and allow mouse coordinates to exceed the window-bounds.
    ///
    /// Grab mode will be temporarily released when the window loses focus and automatically
    /// restored once it regains focus.
    pub fn grab_cursor(self: &Self) {
        self.handle.set_cursor_state(CursorState::Grab);
        self.input_data.write().unwrap().cursor_grabbed = true;
        self.handle.set_cursor_position(Point2(100, 100));
    }

    /// Hides the mouse cursor while it is inside the window.
    pub fn hide_cursor(self: &Self) {
        self.handle.set_cursor_state(CursorState::Hide);
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Releases a previously grabbed or hidden cursor and makes it visible again.
    pub fn free_cursor(self: &Self) {
        self.handle.set_cursor_state(CursorState::Normal);
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Returns the window dimensions.
    pub fn dimensions(self: &Self) -> Point2<u32> {
        self.handle.framebuffer_dimensions()
    }

    /// Returns a vector of available monitors.
    pub fn monitors(self: &Self) -> Vec<monitor::Monitor> {
        let iter = backend::MonitorIterator::new(&self.handle);
        let mut result = Vec::<monitor::Monitor>::new();
        for monitor in iter {
            result.push(monitor::Monitor::new(monitor));
        }
        result
    }

    /// Polls for events like keyboard or mouse input and changes to the window. See
    /// [`Input`](struct.Input.html) for basic keyboard and mouse support.
    pub fn poll_events(self: &Self) -> &Self {
        let mut input_data = self.input_data.write().unwrap();

        // !todo poll_id, check if released/pressed(poll_id) == poll_id
        for key_id in 0..NUM_KEYS {
            match input_data.key[key_id] {
                InputState::Pressed | InputState::Repeat => {
                    input_data.key[key_id] = InputState::Down;
                }
                InputState::Released => {
                    input_data.key[key_id] = InputState::Up;
                }
                _ => { }
            }
        }

        for button_id in 0..NUM_BUTTONS {
            match input_data.button[button_id] {
                InputState::Pressed => {
                    input_data.button[button_id] = InputState::Down;
                }
                InputState::Released => {
                    input_data.button[button_id] = InputState::Up;
                }
                _ => { }
            }
        }

        self.handle.poll_events(|event| {
            match event {
                Event::KeyboardInput(key_id, down) => {
                    let currently_down = match input_data.key[key_id] {
                        InputState::Down | InputState::Pressed | InputState::Repeat => true,
                        _ => false
                    };
                    if !currently_down && down {
                        input_data.key[key_id] = InputState::Pressed;
                    } else if currently_down && !down {
                        input_data.key[key_id] = InputState::Released;
                    } else if currently_down && down {
                        input_data.key[key_id] = InputState::Repeat;
                    }
                },
                Event::MouseMoved(x, y) => {
                    if input_data.cursor_grabbed {
                        let center = ((input_data.dimensions.0 / 2) as i32, (input_data.dimensions.1 / 2) as i32);
                        let old_mouse = input_data.mouse;
                        let delta = (x - center.0, y - center.1);
                        input_data.mouse = (old_mouse.0 + delta.0, old_mouse.1 + delta.1);
                        input_data.mouse_delta = delta;
                        self.handle.set_cursor_position(Point2(center.0, center.1));
                    } else {
                        input_data.mouse = (x, y);
                    }
                },
                Event::MouseInput(button_id, down) => {
                    let currently_down = match input_data.button[button_id] {
                        InputState::Down | InputState::Pressed => true,
                        _ => false
                    };
                    if !currently_down && down {
                        input_data.button[button_id] = InputState::Pressed
                    } else if currently_down && !down {
                        input_data.button[button_id] = InputState::Released
                    }
                },
                Event::Focused => {
                    // restore grab after focus loss
                    if input_data.cursor_grabbed {
                        self.handle.set_cursor_state(CursorState::Normal);
                        self.handle.set_cursor_state(CursorState::Grab);
                    }
                }
                Event::Closed => {
                    input_data.should_close = true;
                }
            }
        });

        input_data.dimensions = self.handle.window_dimensions().into();

        self
    }

    /// Returns true once after the attached window was closed
    pub fn was_closed(self: &Self) -> bool {
        let mut input_data = self.input_data.write().unwrap();
        let result = input_data.should_close;
        input_data.should_close = false;
        result
    }

    /// Creates a new instance from given [`DisplayInfo`](support/struct.DisplayInfo.html).
    pub(crate) fn new(descriptor: DisplayInfo) -> Display {
        Display {
            handle: backend::Display::new(descriptor),
            frame: Rc::new(RefCell::new(None)),
            input_data: Arc::new(RwLock::new(InputData::new())),
        }
    }

    /// Clears the display with given color without swapping buffers.
    pub(crate) fn clear(self: &Self, color: Color) {
        self.frame.borrow_mut().as_mut().expect("Failed to clear frame: None prepared.").clear(color);
    }

    /// Provides a mutable reference to the backend frame to the given function.
    pub(crate) fn frame<T>(self: &Self, func: T) where T: FnOnce(&mut backend::Frame) {
        let mut frame = self.frame.borrow_mut();
        func(frame.as_mut().expect("Failed to get frame: None prepared."));
    }
}

pub enum CursorState {
    Normal,
    Hide,
    Grab,
}

impl AsRenderTarget for Display {
    fn as_render_target(self: &Self) -> RenderTarget {
        RenderTarget::Display(self.clone())
    }
}

/// A struct describing a [`Display`](struct.Display.html) to be created.
#[derive(Clone)]
pub struct DisplayInfo {
    pub width       : u32,
    pub height      : u32,
    pub title       : String,
    pub transparent : bool,
    pub decorations : bool,
    pub monitor     : Option<monitor::Monitor>,
    pub vsync       : bool,
    pub visible     : bool,
}

impl Default for DisplayInfo {
    fn default() -> DisplayInfo {
        DisplayInfo {
            width       : 640,
            height      : 480,
            title       : "".to_string(),
            transparent : false,
            decorations : true,
            monitor     : None,
            vsync       : false,
            visible     : true,
        }
   }
}
