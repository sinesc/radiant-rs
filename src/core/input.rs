use prelude::*;
use std::ops::Not;
use core::{display, Display};

/// The current state of a key or mousebutton.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InputState {
    /// The key is not currently pressed.
    Up,
    /// The key was just pressed. This state is reported only once per key-press.
    Pressed,
    /// The key has been pressed and is still being held down.
    Down,
    /// The key has just been released. This state is reported only once per key-release.
    Released,
}

impl Not for InputState {
    type Output = bool;
    fn not(self) -> bool {
        self == InputState::Up || self == InputState::Released
    }
}

pub struct InputData {
    pub mouse           : (i32, i32),
    pub mouse_delta     : (i32, i32),
    pub button          : [ InputState; 256 ],
    pub key             : [ InputState; 256 ],
    pub should_close    : bool,
    pub cursor_grabbed  : bool,
    pub dimensions      : (u32, u32),
}

impl InputData {
    pub fn new() -> InputData {
        InputData {
            mouse           : (0, 0),
            mouse_delta     : (0, 0),
            button          : [ InputState::Up; 256 ],
            key             : [ InputState::Up; 256 ],
            should_close    : false,
            cursor_grabbed  : false,
            dimensions      : (0, 0),
        }
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    /// Input key and mousebutton ids
    pub enum InputId {
        Escape = 1,
        Backspace = 14,
        Tab = 15,
        Enter = 28,
        CtrlLeft = 29,
        ShiftLeft = 42,
        ShiftRight = 54,
        AltLeft = 56,
        CapsLock = 58,
        F1 = 59,
        F2 = 60,
        F3 = 61,
        F4 = 62,
        F5 = 63,
        F6 = 64,
        F7 = 65,
        F8 = 66,
        F9 = 67,
        F10 = 68,
        F11 = 88,
        F12 = 89,
        CursorUp = 72,
        CursorDown = 80,
        CursorLeft = 75,
        CursorRight = 77,
        Mouse1 = 257,
        Mouse2 = 258,
        Mouse3 = 259,
        Mouse4 = 260,
        Mouse5 = 261,
        Unsupported = 513,
    }
}

/// An iterator over all keys and buttons.
pub struct InputIterator<'a> {
    input_data: RwLockReadGuard<'a, InputData>,
    position: u32,
}

impl<'a> InputIterator<'a> {
    pub fn down(self: Self) -> InputDownIterator<'a> {
        InputDownIterator(self)
    }
}

impl<'a> Iterator for InputIterator<'a> {
    type Item = (InputId, InputState);

    fn next(self: &mut Self) -> Option<(InputId, InputState)> {

        let position = self.position;
        self.position += 1;

        if position < 256 {
            Some((InputId::from_u32(position).unwrap_or(InputId::Unsupported), self.input_data.key[position as usize]))
        } else if position < 512 {
            Some((InputId::from_u32(position).unwrap_or(InputId::Unsupported), self.input_data.button[position as usize - 256]))
        } else {
            None
        }
    }
}

/// An iterator over all keys all buttons currently pressed.
pub struct InputDownIterator<'a>(InputIterator<'a>);

impl<'a> Iterator for InputDownIterator<'a> {
    type Item = InputId;

    fn next(self: &mut Self) -> Option<InputId> {
        while let Some(current) = self.0.next() {
            let (input_id, button_state) = current;
            if (button_state == InputState::Down) | (button_state == InputState::Pressed) {
                println!("{:?}", input_id);
                return Some(input_id);
            }
        }
        None
    }
}

/// An iterator over all keys all buttons currently not pressed.
pub struct InputUpIterator<'a>(InputIterator<'a>);

impl<'a> Iterator for InputUpIterator<'a> {
    type Item = InputId;

    fn next(self: &mut Self) -> Option<InputId> {
        while let Some(current) = self.0.next() {
            let (input_id, button_state) = current;
            if (button_state == InputState::Up) | (button_state == InputState::Released) {
                return Some(input_id);
            }
        }
        None
    }
}

/// Basic keyboard and mouse support.
#[derive(Clone)]
pub struct Input {
    input_data: Arc<RwLock<InputData>>,
}

impl Input {

    /// Creates a new instance.
    pub fn new(display: &Display) -> Self {
        Input {
            input_data: display::input_data(display).clone(),
        }
    }

    /// Returns an iterator over all keys and buttons
    pub fn iter(self: &Self) -> InputIterator {
        InputIterator {
            input_data: self.input_data.read().unwrap(),
            position: 0,
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
    pub fn escape(self: &Self) -> InputState {
        self.get().key[1]
    }

    /// Returns the state of the left alt key.
    pub fn alt_left(self: &Self) -> InputState {
        self.get().key[56]
    }

    /// Returns the state of the left ctrl key.
    pub fn ctrl_left(self: &Self) -> InputState {
        self.get().key[29]
    }

    /// Returns the state of the left shift key.
    pub fn shift_left(self: &Self) -> InputState {
        self.get().key[42]
    }

    /// Returns the state of the right shift key.
    pub fn shift_right(self: &Self) -> InputState {
        self.get().key[54]
    }

    /// Returns the state of the cursor up key.
    pub fn cursor_up(self: &Self) -> InputState {
        self.get().key[72]
    }

    /// Returns the state of the cursor down key.
    pub fn cursor_down(self: &Self) -> InputState {
        self.get().key[80]
    }

    /// Returns the state of the cursor left key.
    pub fn cursor_left(self: &Self) -> InputState {
        self.get().key[75]
    }

    /// Returns the state of the cursor right key.
    pub fn cursor_right(self: &Self) -> InputState {
        self.get().key[77]
    }

    /// Returns the state of the enter/return key.
    pub fn enter(self: &Self) -> InputState {
        self.get().key[28]
    }

    /// Returns the state of the backspace key.
    pub fn backspace(self: &Self) -> InputState {
        self.get().key[14]
    }

    /// Returns the state of the tabulator key.
    pub fn tab(self: &Self) -> InputState {
        self.get().key[15]
    }

    /// Returns the state of the worthless key.
    pub fn capslock(self: &Self) -> InputState {
        self.get().key[58]
    }

    /// Returns the state of the given function key.
    pub fn f(self: &Self, index: u32) -> InputState {
        if index < 1 || index > 12 {
            InputState::Up
        } else {
            self.get().key[if index <= 10 { 58 + index as usize } else { 76 + index as usize }]
        }
    }

    fn get(self: &Self) -> RwLockReadGuard<InputData> {
        self.input_data.read().unwrap()
    }
}
