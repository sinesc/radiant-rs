use prelude::*;
//use core::input::{InputData, InputState, NUM_KEYS, NUM_BUTTONS};
//use core::monitor::Monitor;
//use core::{AsRenderTarget, RenderTarget, Color};
use core::*;
use maths::Point2;
use core::builder::*;
use backends::backend;

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    pub(crate) handle: Rc<backend::Display>,
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

    /// Switches to fullscreen mode on the primary monitor.
    pub fn set_fullscreen(self: &Self) -> Result<()> {
        if let Some(backend_monitor) = backend::MonitorIterator::new(self).next() {
            let monitor = Monitor::new(backend_monitor);
            self.set_fullscreen_on(monitor)
        } else {
            Err(Error::FullscreenError("Failed select monitor device.".to_string()))
        }
    }

    /// Switches to fullscreen mode on the given monitor.
    pub fn set_fullscreen_on(self: &Self, monitor: Monitor) -> Result<()> {
        if !self.handle.set_fullscreen(Some(monitor)) {
            self.handle.set_fullscreen(None);
            Err(Error::FullscreenError("Failed to switch to fullscreen.".to_string()))
        } else {
            Ok(())
        }
    }

    /// Switches to windowed mode.
    pub fn set_windowed(self: &Self) {
        self.handle.set_fullscreen(None);
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
            frame.finish();
        } else {
            panic!("No frame currently prepared, nothing to swap.");
        }
    }

    /// Enables cursor grab mode. While in this mode, the mouse cursor will be hidden and
    /// constrained to the window.
    ///
    /// Grab mode will be temporarily released when the window loses focus and automatically
    /// restored once it regains focus.
    pub fn grab_cursor(self: &Self) {
        let mut input_data = self.input_data.write().unwrap();
        if input_data.has_focus {
            self.handle.set_cursor_state(CursorState::Grab);
        }
        input_data.cursor_grabbed = true;
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

    /// Sets the mouse cursor position.
    pub fn set_cursor_position(self: &Self, position: Point2<i32>) {
        self.handle.set_cursor_position(position);
    }

    /// Returns the window dimensions.
    pub fn dimensions(self: &Self) -> Point2<u32> {
        self.handle.framebuffer_dimensions()
    }

    /// Returns a vector of available monitors.
    pub fn monitors(self: &Self) -> Vec<Monitor> {
        let iter = backend::MonitorIterator::new(self);
        let mut result = Vec::<Monitor>::new();
        for monitor in iter {
            result.push(Monitor::new(monitor));
        }
        result
    }

    /// Polls for events like keyboard or mouse input and changes to the window. See
    /// [`Input`](struct.Input.html) for basic keyboard and mouse support.
    pub fn poll_events(self: &Self) -> &Self {
        let mut input_data = self.input_data.write().unwrap();

        // todo: poll_id, check if released/pressed(poll_id) == poll_id
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

        input_data.mouse_delta = (0, 0);

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
                Event::MouseDelta(x, y) => {
                    input_data.mouse_delta = (x, y);
                },
                Event::MousePosition(x, y) => {
                    input_data.mouse = (x, y);
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
                Event::Focus => {
                    input_data.has_focus = true;
                    // restore grab after focus loss
                    if input_data.cursor_grabbed {
                        self.handle.set_cursor_state(CursorState::Grab);
                    }
                }
                Event::Blur => {
                    input_data.has_focus = false;
                    self.handle.set_cursor_state(CursorState::Normal);
                }
                Event::Close => {
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
            handle: Rc::new(backend::Display::new(descriptor)),
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

impl AsRenderTarget for Display {
    fn as_render_target(self: &Self) -> RenderTarget {
        RenderTarget::Display(self.clone())
    }
}

/// The current state of the mouse cursor.
#[derive(Debug)]
pub enum CursorState {
    Normal,
    Hide,
    Grab,
}

// An input event.
#[derive(Debug, PartialEq)]
pub enum Event {
    KeyboardInput(usize, bool),
    MouseInput(usize, bool),
    MouseDelta(i32, i32),
    MousePosition(i32, i32),
    Focus,
    Blur,
    Close,
}

/// A struct describing a [`Display`](struct.Display.html) to be created.
#[derive(Clone, Debug)]
pub struct DisplayInfo {
    pub width       : u32,
    pub height      : u32,
    pub title       : String,
    pub transparent : bool,
    pub decorations : bool,
    pub monitor     : Option<Monitor>,
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
