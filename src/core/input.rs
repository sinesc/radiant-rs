use prelude::*;
//use std::ops::Not;
use core::{display, Display};
use glium::glutin;

pub const NUM_KEYS: usize = 256;
pub const NUM_BUTTONS: usize = 16;

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

/*impl Not for InputState {
    type Output = bool;
    fn not(self) -> bool {
        self == InputState::Up || self == InputState::Released
    }
}*/

pub struct InputData {
    pub mouse           : (i32, i32),
    pub mouse_delta     : (i32, i32),
    pub button          : [ InputState; NUM_BUTTONS ],
    pub key             : [ InputState; NUM_KEYS ],
    pub should_close    : bool,
    pub cursor_grabbed  : bool,
    pub dimensions      : (u32, u32),
}

/*impl Clone for InputData {
    fn clone(self: &Self) -> InputData {
        InputData {
            mouse           : self.mouse,
            mouse_delta     : self.mouse_delta,
            button          : self.button,
            key             : self.key,
            should_close    : self.should_close,
            cursor_grabbed  : self.cursor_grabbed,
            dimensions      : self.dimensions,
        }
    }
}*/

impl InputData {
    pub fn new() -> InputData {
        InputData {
            mouse           : (0, 0),
            mouse_delta     : (0, 0),
            button          : [ InputState::Up; NUM_BUTTONS ],
            key             : [ InputState::Up; NUM_KEYS ],
            should_close    : false,
            cursor_grabbed  : false,
            dimensions      : (0, 0),
        }
    }
/*
    /// Returns current mouse coordinates relative to the window.
    pub fn mouse(self: &Self) -> (i32, i32) {
        self.mouse
    }

    /// Returns mouse delta coordinates since last [`Display::poll_events()`](struct.Display.html#method.poll_events).
    pub fn mouse_delta(self: &Self) -> (i32, i32) {
        self.mouse_delta
    }

    /// Returns true if given key is down/pressed
    pub fn down(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        if id < NUM_KEYS {
            (self.key[id] == InputState::Pressed) | (self.key[id] == InputState::Down)
        } else {
            (self.button[id - NUM_KEYS] == InputState::Pressed) | (self.key[id - NUM_KEYS] == InputState::Down)
        }
    }

    /// Returns true if given key is up/released
    pub fn up(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        if id < NUM_KEYS {
            (self.key[id] == InputState::Released) | (self.key[id] == InputState::Up)
        } else {
            (self.button[id - NUM_KEYS] == InputState::Released) | (self.key[id - NUM_KEYS] == InputState::Up)
        }
    }
*/
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    /// Input key and mousebutton ids
    pub enum InputId {
        Key1,
        Key2,
        Key3,
        Key4,
        Key5,
        Key6,
        Key7,
        Key8,
        Key9,
        Key0,

        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,

        Escape,

        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,

        Snapshot,
        Scroll,
        Pause,

        Insert,
        Home,
        Delete,
        End,
        PageDown,
        PageUp,

        CursorLeft,
        CursorUp,
        CursorRight,
        CursorDown,

        Backspace,
        Return,
        Space,

        Numlock,
        Numpad0,
        Numpad1,
        Numpad2,
        Numpad3,
        Numpad4,
        Numpad5,
        Numpad6,
        Numpad7,
        Numpad8,
        Numpad9,

        AbntC1,
        AbntC2,
        Add,
        Apostrophe,
        Apps,
        At,
        Ax,
        Backslash,
        Calculator,
        Capital,
        Colon,
        Comma,
        Convert,
        Decimal,
        Divide,
        Equals,
        Grave,
        Kana,
        Kanji,
        LAlt,
        LBracket,
        LControl,
        LMenu,
        LShift,
        LWin,
        Mail,
        MediaSelect,
        MediaStop,
        Minus,
        Multiply,
        Mute,
        MyComputer,
        NextTrack,
        NoConvert,
        NumpadComma,
        NumpadEnter,
        NumpadEquals,
        OEM102,
        Period,
        PlayPause,
        Power,
        PrevTrack,
        RAlt,
        RBracket,
        RControl,
        RMenu,
        RShift,
        RWin,
        Semicolon,
        Slash,
        Sleep,
        Stop,
        Subtract,
        Sysrq,
        Tab,
        Underline,
        Unlabeled,
        VolumeDown,
        VolumeUp,
        Wake,
        WebBack,
        WebFavorites,
        WebForward,
        WebHome,
        WebRefresh,
        WebSearch,
        WebStop,
        Yen,
        Compose,
        NavigateForward,
        NavigateBackward,

        Mouse1 = NUM_KEYS as isize +0,
        Mouse2 = NUM_KEYS as isize +1,
        Mouse3 = NUM_KEYS as isize +2,
        Mouse4 = NUM_KEYS as isize +3,
        Mouse5 = NUM_KEYS as isize +4,
        Mouse6 = NUM_KEYS as isize +5,
        Mouse7 = NUM_KEYS as isize +6,
        Mouse8 = NUM_KEYS as isize +7,
        Mouse9 = NUM_KEYS as isize +8,
        Mouse10 = NUM_KEYS as isize +9,
        Mouse11 = NUM_KEYS as isize +10,
        Mouse12 = NUM_KEYS as isize +11,
        Mouse13 = NUM_KEYS as isize +12,
        Mouse14 = NUM_KEYS as isize +13,
        Mouse15 = NUM_KEYS as isize +14,
        Mouse16 = NUM_KEYS as isize +15,

        Unsupported = (NUM_KEYS + NUM_BUTTONS) as isize,
    }
}

impl InputId {
    pub fn button(id: usize) -> InputId {
        let base = InputId::Mouse1 as isize;
        if (id >= 1) & (id <= 16) {
            InputId::from_isize(base + (id as isize - 1)).unwrap()
        } else {
            InputId::Unsupported
        }
    }
}

pub fn input_id_from_glutin(key: glutin::VirtualKeyCode) -> InputId {
    use glium::glutin::VirtualKeyCode as VK;
    match key {
        VK::Key1          => InputId::Key1,
        VK::Key2          => InputId::Key2,
        VK::Key3          => InputId::Key3,
        VK::Key4          => InputId::Key4,
        VK::Key5          => InputId::Key5,
        VK::Key6          => InputId::Key6,
        VK::Key7          => InputId::Key7,
        VK::Key8          => InputId::Key8,
        VK::Key9          => InputId::Key9,
        VK::Key0          => InputId::Key0,
        VK::A             => InputId::A,
        VK::B             => InputId::B,
        VK::C             => InputId::C,
        VK::D             => InputId::D,
        VK::E             => InputId::E,
        VK::F             => InputId::F,
        VK::G             => InputId::G,
        VK::H             => InputId::H,
        VK::I             => InputId::I,
        VK::J             => InputId::J,
        VK::K             => InputId::K,
        VK::L             => InputId::L,
        VK::M             => InputId::M,
        VK::N             => InputId::N,
        VK::O             => InputId::O,
        VK::P             => InputId::P,
        VK::Q             => InputId::Q,
        VK::R             => InputId::R,
        VK::S             => InputId::S,
        VK::T             => InputId::T,
        VK::U             => InputId::U,
        VK::V             => InputId::V,
        VK::W             => InputId::W,
        VK::X             => InputId::X,
        VK::Y             => InputId::Y,
        VK::Z             => InputId::Z,
        VK::Escape        => InputId::Escape,
        VK::F1            => InputId::F1,
        VK::F2            => InputId::F2,
        VK::F3            => InputId::F3,
        VK::F4            => InputId::F4,
        VK::F5            => InputId::F5,
        VK::F6            => InputId::F6,
        VK::F7            => InputId::F7,
        VK::F8            => InputId::F8,
        VK::F9            => InputId::F9,
        VK::F10           => InputId::F10,
        VK::F11           => InputId::F11,
        VK::F12           => InputId::F12,
        VK::F13           => InputId::F13,
        VK::F14           => InputId::F14,
        VK::F15           => InputId::F15,
        VK::Snapshot      => InputId::Snapshot,
        VK::Scroll        => InputId::Scroll,
        VK::Pause         => InputId::Pause,
        VK::Insert        => InputId::Insert,
        VK::Home          => InputId::Home,
        VK::Delete        => InputId::Delete,
        VK::End           => InputId::End,
        VK::PageDown      => InputId::PageDown,
        VK::PageUp        => InputId::PageUp,
        VK::Left          => InputId::CursorLeft,
        VK::Up            => InputId::CursorUp,
        VK::Right         => InputId::CursorRight,
        VK::Down          => InputId::CursorDown,
        VK::Back          => InputId::Backspace,
        VK::Return        => InputId::Return,
        VK::Space         => InputId::Space,
        VK::Numlock       => InputId::Numlock,
        VK::Numpad0       => InputId::Numpad0,
        VK::Numpad1       => InputId::Numpad1,
        VK::Numpad2       => InputId::Numpad2,
        VK::Numpad3       => InputId::Numpad3,
        VK::Numpad4       => InputId::Numpad4,
        VK::Numpad5       => InputId::Numpad5,
        VK::Numpad6       => InputId::Numpad6,
        VK::Numpad7       => InputId::Numpad7,
        VK::Numpad8       => InputId::Numpad8,
        VK::Numpad9       => InputId::Numpad9,
        VK::AbntC1        => InputId::AbntC1,
        VK::AbntC2        => InputId::AbntC2,
        VK::Add           => InputId::Add,
        VK::Apostrophe    => InputId::Apostrophe,
        VK::Apps          => InputId::Apps,
        VK::At            => InputId::At,
        VK::Ax            => InputId::Ax,
        VK::Backslash     => InputId::Backslash,
        VK::Calculator    => InputId::Calculator,
        VK::Capital       => InputId::Capital,
        VK::Colon         => InputId::Colon,
        VK::Comma         => InputId::Comma,
        VK::Convert       => InputId::Convert,
        VK::Decimal       => InputId::Decimal,
        VK::Divide        => InputId::Divide,
        VK::Equals        => InputId::Equals,
        VK::Grave         => InputId::Grave,
        VK::Kana          => InputId::Kana,
        VK::Kanji         => InputId::Kanji,
        VK::LAlt          => InputId::LAlt,
        VK::LBracket      => InputId::LBracket,
        VK::LControl      => InputId::LControl,
        VK::LMenu         => InputId::LMenu,
        VK::LShift        => InputId::LShift,
        VK::LWin          => InputId::LWin,
        VK::Mail          => InputId::Mail,
        VK::MediaSelect   => InputId::MediaSelect,
        VK::MediaStop     => InputId::MediaStop,
        VK::Minus         => InputId::Minus,
        VK::Multiply      => InputId::Multiply,
        VK::Mute          => InputId::Mute,
        VK::MyComputer    => InputId::MyComputer,
        VK::NextTrack     => InputId::NextTrack,
        VK::NoConvert     => InputId::NoConvert,
        VK::NumpadComma   => InputId::NumpadComma,
        VK::NumpadEnter   => InputId::NumpadEnter,
        VK::NumpadEquals  => InputId::NumpadEquals,
        VK::OEM102        => InputId::OEM102,
        VK::Period        => InputId::Period,
        VK::PlayPause     => InputId::PlayPause,
        VK::Power         => InputId::Power,
        VK::PrevTrack     => InputId::PrevTrack,
        VK::RAlt          => InputId::RAlt,
        VK::RBracket      => InputId::RBracket,
        VK::RControl      => InputId::RControl,
        VK::RMenu         => InputId::RMenu,
        VK::RShift        => InputId::RShift,
        VK::RWin          => InputId::RWin,
        VK::Semicolon     => InputId::Semicolon,
        VK::Slash         => InputId::Slash,
        VK::Sleep         => InputId::Sleep,
        VK::Stop          => InputId::Stop,
        VK::Subtract      => InputId::Subtract,
        VK::Sysrq         => InputId::Sysrq,
        VK::Tab           => InputId::Tab,
        VK::Underline     => InputId::Underline,
        VK::Unlabeled     => InputId::Unlabeled,
        VK::VolumeDown    => InputId::VolumeDown,
        VK::VolumeUp      => InputId::VolumeUp,
        VK::Wake          => InputId::Wake,
        VK::WebBack       => InputId::WebBack,
        VK::WebFavorites  => InputId::WebFavorites,
        VK::WebForward    => InputId::WebForward,
        VK::WebHome       => InputId::WebHome,
        VK::WebRefresh    => InputId::WebRefresh,
        VK::WebSearch     => InputId::WebSearch,
        VK::WebStop       => InputId::WebStop,
        VK::Yen           => InputId::Yen,
        VK::Compose       => InputId::Compose,
        VK::NavigateForward => InputId::NavigateForward,
        VK::NavigateBackward => InputId::NavigateBackward,
    }
}

/// An iterator over all keys and buttons.
pub struct InputIterator<'a> {
    input_data: RwLockReadGuard<'a, InputData>,
    position: usize,
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

        if position < NUM_KEYS {
            Some((InputId::from_usize(position).unwrap_or(InputId::Unsupported), self.input_data.key[position]))
        } else if position < NUM_KEYS + NUM_BUTTONS {
            Some((InputId::from_usize(position).unwrap_or(InputId::Unsupported), self.input_data.button[position - NUM_KEYS]))
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

    /// Returns a copy of the current input data
    /*pub fn snapshot(self: &Self) -> InputData {
        (*self.get().deref()).clone()
    }*/

    /// Returns current mouse coordinates relative to the window.
    pub fn mouse(self: &Self) -> (i32, i32) {
        self.get().mouse
    }

    /// Returns mouse delta coordinates since last [`Display::poll_events()`](struct.Display.html#method.poll_events).
    pub fn mouse_delta(self: &Self) -> (i32, i32) {
        self.get().mouse_delta
    }

    /// Returns InputState for given key
    pub fn state(self: &Self, key: InputId) -> InputState {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            data.key[id]
        } else {
            data.button[id - NUM_KEYS]
        }
    }

    /// Returns true if given key is down/pressed
    pub fn down(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Pressed) | (data.key[id] == InputState::Down)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Pressed) | (data.key[id - NUM_KEYS] == InputState::Down)
        }
    }

    /// Returns true if given key is up/released
    pub fn up(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Released) | (data.key[id] == InputState::Up)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Released) | (data.key[id - NUM_KEYS] == InputState::Up)
        }
    }

    /// Returns input data
    fn get(self: &Self) -> RwLockReadGuard<InputData> {
        self.input_data.read().unwrap()
    }
}
