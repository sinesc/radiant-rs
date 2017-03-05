
// temporary. ultimately, the public interface of this module needs to be backend agnostic

pub mod glium {

    use core::*;
    use maths::*;
    use glium;
    use glium::glutin;

    pub fn blit_coords(source_rect: Rect<i32>, source_height: u32, target_rect: Rect<i32>, target_height: u32) -> (glium::Rect, glium::BlitTarget) {
        (glium::Rect {
            left: (source_rect.0).0 as u32,
            bottom: (source_height as i32 - (source_rect.1).1 as i32 - (source_rect.0).1 as i32) as u32,
            width: (source_rect.1).0 as u32,
            height: (source_rect.1).1 as u32,
        },
        glium::BlitTarget {
            left: (target_rect.0).0 as u32,
            bottom: (target_height as i32 - (target_rect.1).1 as i32 - (target_rect.0).1 as i32) as u32,
            width: (target_rect.1).0 as i32,
            height: (target_rect.1).1 as i32,
        })
    }

    pub fn magnify_filter(filter: TextureFilter) -> glium::uniforms::MagnifySamplerFilter {
        if filter == TextureFilter::Linear {
            glium::uniforms::MagnifySamplerFilter::Linear
        } else {
            glium::uniforms::MagnifySamplerFilter::Nearest
        }
    }

    pub enum Event {
        KeyboardInput(usize, bool),
        MouseInput(usize, bool),
        MouseMoved(i32, i32),
        Focused,
        Closed,
    }

    pub struct EventIterator<'a> {
        it: glium::backend::glutin_backend::PollEventsIter<'a>,
    }

    impl<'a> Iterator for EventIterator<'a> {
        type Item = Event;

        fn next(self: &mut Self) -> Option<Event> {
            use glium::glutin::{ElementState, MouseButton};
            use glium::glutin::Event as GlutinEvent;

            let event = self.it.next();

            if let Some(event) = event {
                match event {
                    GlutinEvent::KeyboardInput(element_state, _, Some(virtual_code)) => {
                        let key_id = input_id(virtual_code) as usize;
                        if key_id < NUM_KEYS {
                            Some(Event::KeyboardInput(key_id, element_state == ElementState::Pressed))
                        } else {
                            None
                        }
                    },
                    GlutinEvent::MouseMoved(x, y) => {
                        Some(Event::MouseMoved(x, y))
                    },
                    GlutinEvent::MouseInput(element_state, button) => {
                        let button_id = match button {
                            MouseButton::Left => 0,
                            MouseButton::Middle => 1,
                            MouseButton::Right => 2,
                            MouseButton::Other(x) => (x - 1) as usize,
                        };
                        if button_id < NUM_BUTTONS {
                            Some(Event::MouseInput(button_id, element_state == ElementState::Pressed))
                        } else {
                            None
                        }
                    },
                    GlutinEvent::Focused(true) => {
                        Some(Event::Focused)
                    }
                    GlutinEvent::Closed => {
                        Some(Event::Closed)
                    }
                    _ => None
                }
            } else {
                None
            }
        }
    }

    pub fn poll_events(handle: &glium::Display) -> EventIterator {
        EventIterator {
            it: handle.poll_events(),
        }
    }

    pub fn input_id(key: glutin::VirtualKeyCode) -> InputId {
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
}
