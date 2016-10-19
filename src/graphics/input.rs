use glium::glutin::{Event, ElementState, MouseButton/*, VirtualKeyCode*/};
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

    pub fn escape(self: &Self) -> bool {
        self.get().escape
    }

    pub fn should_close(self: &Self) -> bool {
        self.get().should_close
    }

    pub fn mouse(self: &Self) -> (u32, u32) {
        self.get().mouse
    }

    pub fn mouse_x(self: &Self) -> u32 {
        self.get().mouse.0
    }

    pub fn mouse_y(self: &Self) -> u32 {
        self.get().mouse.1
    }

    fn get(self: &Self) -> RwLockReadGuard<InputState> {
        self.input_state.read().unwrap()
    }
}

pub fn poll_events(display: &Display) {
    let mut input_state = display.input_state.write().unwrap();

    for event in display.handle.poll_events() {
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
                match scan_code {
                    56 => {
                        input_state.alt_left = new_state;
                    },
                    42 => {
                        input_state.shift_left = new_state;
                    },
                    54 => {
                        input_state.shift_right = new_state;
                    },
                    29 => {
                        input_state.ctrl_left = new_state;
                    },
                    1 => {
                        input_state.escape = new_state;
                    },
                    _ => ()
                }
                //println!("key: {}", scan_code);
            },
            Event::MouseMoved(x, y) => {
                input_state.mouse = (x as u32, y as u32);
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
            Event::Closed => {
                input_state.should_close = true;
            }
            _ => ()
        }
    }
}
