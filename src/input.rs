use glium::glutin::{Event, ElementState, MouseButton/*, VirtualKeyCode*/};
use glium::Display;

pub struct MouseCoordinate {
    pub x: u32,
    pub y: u32,
}

pub struct Input {
    pub mouse       : MouseCoordinate,
    pub button      : (bool, bool, bool),
    pub key         : [ bool; 255 ],
    pub alt_left    : bool,
    pub alt_right   : bool,
    pub ctrl_left   : bool,
    pub ctrl_right  : bool,
    pub shift_left  : bool,
    pub shift_right : bool,
    pub escape      : bool,
    pub should_close: bool,
    display         : Display,
}

impl Input {

    pub fn new(display: Display) -> Self {
        Input {
            mouse       : MouseCoordinate { x: 0, y: 0 },
            button      : (false, false, false),
            key         : [ false; 255 ],
            alt_left    : false,
            alt_right   : false,
            ctrl_left   : false,
            ctrl_right  : false,
            shift_left  : false,
            shift_right : false,
            escape      : false,
            should_close: false,
            display     : display,
        }
    }

    pub fn poll(&mut self) {

        for event in self.display.poll_events() {
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
                    self.key[scan_code as usize] = new_state;
                    match scan_code {
                        56 => {
                            self.alt_left = new_state;
                        },
                        42 => {
                            self.shift_left = new_state;
                        },
                        54 => {
                            self.shift_right = new_state;
                        },
                        29 => {
                            self.ctrl_left = new_state;
                        },
                        1 => {
                            self.escape = new_state;
                        },
                        _ => ()
                    }
                    //println!("key: {}", scan_code);
                },
                Event::MouseMoved(x, y) => {
                    self.mouse.x = x as u32;
                    self.mouse.y = y as u32;
                },
                Event::MouseInput(element_state, button) => {
                    let new_state = if element_state == ElementState::Pressed { true } else { false };
                    if button == MouseButton::Left {
                        self.button.0 = new_state;
                    } else if button == MouseButton::Middle {
                        self.button.1 = new_state;
                    } else if button == MouseButton::Right {
                        self.button.2 = new_state;
                    }
                },
                Event::Closed => {
                    self.should_close = true;
                }
                _ => ()
            }
        }
    }
}
