use prelude::*;
use std::ops::Not;
use core::Display;

/// The current state of a key or mousebutton.
#[derive(PartialEq, Copy, Clone)]
pub enum ButtonState {
    /// The key is not currently pressed.
    Up,
    /// The key was just pressed. This state is reported only once per key-press.
    Pressed,
    /// The key has been pressed and is still being held down.
    Down,
    /// The key has just been released. This state is reported only once per key-release.
    Released,
}

impl Not for ButtonState {
    type Output = bool;
    fn not(self) -> bool {
        self == ButtonState::Up || self == ButtonState::Released
    }
}

pub struct InputState {
    pub mouse           : (i32, i32),
    pub mouse_delta     : (i32, i32),
    pub button          : [ ButtonState; 256 ],
    pub key             : [ ButtonState; 256 ],
    pub should_close    : bool,
    pub cursor_grabbed  : bool,
    pub dimensions      : (u32, u32),
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mouse           : (0, 0),
            mouse_delta     : (0, 0),
            button          : [ ButtonState::Up; 256 ],
            key             : [ ButtonState::Up; 256 ],
            should_close    : false,
            cursor_grabbed  : false,
            dimensions      : (0, 0),
        }
    }
}

/// Basic keyboard and mouse support.
#[derive(Clone)]
pub struct Input {
    input_state: Arc<RwLock<InputState>>,
}

impl Input {

    /// Creates a new instance.
    pub fn new(display: &Display) -> Self {
        Input {
            input_state: display.input_state.clone(),
        }
    }

    /// Returns current mouse coordinates relative to the window.
    pub fn mouse(self: &Self) -> (i32, i32) {
        self.get().mouse
    }

    /// Returns mouse delta coordinates since last [`Display::poll_events()`](struct.Display.html#method.poll_events).
    pub fn mouse_delta(self: &Self) -> (i32, i32) {
        self.get().mouse_delta
    }

    /// Returns current mouse cursor x-axis position.
    pub fn mouse_x(self: &Self) -> i32 {
        self.get().mouse.0
    }

    /// Returns current mouse cursor y-axis position.
    pub fn mouse_y(self: &Self) -> i32 {
        self.get().mouse.1
    }

    /// Returns current mouse cursor x-axis delta.
    pub fn mouse_dx(self: &Self) -> i32 {
        self.get().mouse_delta.0
    }

    /// Returns current mouse cursor y-axis delta.
    pub fn mouse_dy(self: &Self) -> i32 {
        self.get().mouse_delta.1
    }

    /// Returns the state of the escape key.
    pub fn escape(self: &Self) -> ButtonState {
        self.get().key[1]
    }

    /// Returns the state of the left alt key.
    pub fn alt_left(self: &Self) -> ButtonState {
        self.get().key[56]
    }

    /// Returns the state of the left ctrl key.
    pub fn ctrl_left(self: &Self) -> ButtonState {
        self.get().key[29]
    }

    /// Returns the state of the left shift key.
    pub fn shift_left(self: &Self) -> ButtonState {
        self.get().key[42]
    }

    /// Returns the state of the right shift key.
    pub fn shift_right(self: &Self) -> ButtonState {
        self.get().key[54]
    }

    /// Returns the state of the cursor up key.
    pub fn cursor_up(self: &Self) -> ButtonState {
        self.get().key[72]
    }

    /// Returns the state of the cursor down key.
    pub fn cursor_down(self: &Self) -> ButtonState {
        self.get().key[80]
    }

    /// Returns the state of the cursor left key.
    pub fn cursor_left(self: &Self) -> ButtonState {
        self.get().key[75]
    }

    /// Returns the state of the cursor right key.
    pub fn cursor_right(self: &Self) -> ButtonState {
        self.get().key[77]
    }

    /// Returns the state of the enter/return key.
    pub fn enter(self: &Self) -> ButtonState {
        self.get().key[28]
    }

    /// Returns the state of the backspace key.
    pub fn backspace(self: &Self) -> ButtonState {
        self.get().key[14]
    }

    /// Returns the state of the tabulator key.
    pub fn tab(self: &Self) -> ButtonState {
        self.get().key[15]
    }

    /// Returns the state of the worthless key.
    pub fn capslock(self: &Self) -> ButtonState {
        self.get().key[58]
    }

    /// Returns the state of the given function key.
    pub fn f(self: &Self, index: u32) -> ButtonState {
        if index < 1 || index > 12 {
            ButtonState::Up
        } else {
            self.get().key[if index <= 10 { 58 + index as usize } else { 76 + index as usize }]
        }
    }

    fn get(self: &Self) -> RwLockReadGuard<InputState> {
        self.input_state.read().unwrap()
    }
}
