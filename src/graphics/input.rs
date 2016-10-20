use prelude::*;
use graphics::{Display, InputState};

#[derive(Clone)]
pub struct Input {
    input_state: Arc<RwLock<InputState>>,
}

impl Input {

    pub fn new(display: &Display) -> Self {
        Input {
            input_state: display.input_state.clone(),
        }
    }

    pub fn should_close(self: &Self) -> bool {
        self.get().should_close
    }

    pub fn mouse(self: &Self) -> (i32, i32) {
        self.get().mouse
    }

    pub fn mouse_delta(self: &Self) -> (i32, i32) {
        self.get().mouse_delta
    }

    pub fn mouse_x(self: &Self) -> i32 {
        self.get().mouse.0
    }

    pub fn mouse_y(self: &Self) -> i32 {
        self.get().mouse.1
    }

    pub fn mouse_dx(self: &Self) -> i32 {
        self.get().mouse_delta.0
    }

    pub fn mouse_dy(self: &Self) -> i32 {
        self.get().mouse_delta.1
    }

    pub fn escape(self: &Self) -> bool {
        self.get().key[1]
    }

    pub fn alt_left(self: &Self) -> bool {
        self.get().key[56]
    }

    pub fn ctrl_left(self: &Self) -> bool {
        self.get().key[29]
    }

    pub fn shift_left(self: &Self) -> bool {
        self.get().key[42]
    }

    pub fn shift_right(self: &Self) -> bool {
        self.get().key[54]
    }

    pub fn cursor_up(self: &Self) -> bool {
        self.get().key[72]
    }

    pub fn cursor_down(self: &Self) -> bool {
        self.get().key[80]
    }

    pub fn cursor_left(self: &Self) -> bool {
        self.get().key[75]
    }

    pub fn cursor_right(self: &Self) -> bool {
        self.get().key[77]
    }

    pub fn enter(self: &Self) -> bool {
        self.get().key[28]
    }

    pub fn backspace(self: &Self) -> bool {
        self.get().key[14]
    }

    pub fn tab(self: &Self) -> bool {
        self.get().key[15]
    }

    pub fn capslock(self: &Self) -> bool {
        self.get().key[58]
    }

    pub fn f(self: &Self, index: u32) -> bool {
        if index < 1 || index > 12 {
            false
        } else {
            self.get().key[if index <= 10 { 58 + index as usize } else { 76 + index as usize }]
        }
    }

    fn get(self: &Self) -> RwLockReadGuard<InputState> {
        self.input_state.read().unwrap()
    }
}
