use prelude::*;
use core::{display, Display};

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
    /// The key is still being held down. When used as text-input, a letter repeat is expected.
    Repeat,
}

pub struct InputData {
    pub mouse           : (i32, i32),
    pub mouse_delta     : (i32, i32),
    pub button          : [ InputState; NUM_BUTTONS ],
    pub key             : [ InputState; NUM_KEYS ],
    pub should_close    : bool,
    pub cursor_grabbed  : bool,
    pub dimensions      : (u32, u32),
}

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
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq)]
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

    /// Returns an iterator over all keys and buttons.
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

    /// Returns true if given key is down/pressed.
    pub fn down(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Pressed) || (data.key[id] == InputState::Down) || (data.key[id] == InputState::Repeat)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Pressed) || (data.button[id - NUM_KEYS] == InputState::Down)
        }
    }

    /// Returns true if given key was just pressed or repeated due to still being held down (if report_repeats is true).
    pub fn pressed(self: &Self, key: InputId, report_repeats: bool) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Pressed) || (report_repeats && data.key[id] == InputState::Repeat)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Pressed)
        }
    }

    /// Returns true if given key is up/released.
    pub fn up(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Released) || (data.key[id] == InputState::Up)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Released) || (data.button[id - NUM_KEYS] == InputState::Up)
        }
    }

    /// Returns true if given key was just released.
    pub fn released(self: &Self, key: InputId) -> bool {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            (data.key[id] == InputState::Released)
        } else {
            (data.button[id - NUM_KEYS] == InputState::Released)
        }
    }

    /// Returns InputState for given key.
    pub fn state(self: &Self, key: InputId) -> InputState {
        let id = key as usize;
        let data = self.get();
        if id < NUM_KEYS {
            data.key[id]
        } else {
            data.button[id - NUM_KEYS]
        }
    }

    /// Returns input data.
    fn get(self: &Self) -> RwLockReadGuard<InputData> {
        self.input_data.read().unwrap()
    }
}

/// An iterator over all keys and buttons.
pub struct InputIterator<'a> {
    input_data: RwLockReadGuard<'a, InputData>,
    position: usize,
}

impl<'a> InputIterator<'a> {
    /// Returns an iterator over all keys currently pressed.
    pub fn down(self: Self) -> InputDownIterator<'a> {
        InputDownIterator(self)
    }
    /// Returns an iterator over all keys not currently pressed.
    pub fn up(self: Self) -> InputUpIterator<'a> {
        InputUpIterator(self)
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
            if (button_state == InputState::Down) || (button_state == InputState::Pressed) {
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
            if (button_state == InputState::Up) || (button_state == InputState::Released) {
                return Some(input_id);
            }
        }
        None
    }
}
