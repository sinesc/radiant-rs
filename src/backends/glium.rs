extern crate glium;
use prelude::*;
use core;
use self::glium::uniforms::Uniforms;
use self::glium::{glutin, Surface};

const MAX_BUFFERS: usize = 10;

// glutin's global events loop handling. Can only have one of these.

static mut EVENTS_LOOP: Option<glutin::EventsLoop> = None;

fn init_events_loop<T>(else_fn: T) -> &'static mut glutin::EventsLoop where T: FnOnce() -> glutin::EventsLoop {
    unsafe {
        if EVENTS_LOOP.is_none() {
            EVENTS_LOOP = Some(else_fn());
        }
        EVENTS_LOOP.as_mut().unwrap()
    }
}

// --------------
// Public interface provided to Radiant-API-user in radiant_rs::backend
// --------------

pub mod public {
    use super::glium;
    use super::core;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Arc, RwLock};
    use std::mem;

    /// Creates a new radiant_rs::Display from given glium::Display and glutin::EventsLoop.
    ///
    /// As an alternative to [`backend::create_renderer()`](fn.create_renderer.html), this allows for glium rendering but keeps radiant display handling.
    pub fn create_display(display: &glium::Display, events_loop: glium::glutin::EventsLoop) -> core::Display {

        let mut accepted = false;

        super::init_events_loop(|| { accepted = true; events_loop });

        if !accepted {
            panic!("Failed to use given events loop. Another events loop was already created.");
        }

        let display = super::Display(display.clone());
        let context = core::Context::new();
        context.set_primary_display(&display);

        core::Display {
            handle      : display,
            context     : context,
            frame       : Rc::new(RefCell::new(None)),
            input_data  : Arc::new(RwLock::new(core::InputData::new())),
            fullscreen  : Rc::new(RefCell::new(None)), // TODO: fullscreen state unknown, doesn't appear to be possible to retrieve from winit
        }
    }

    /// Creates a new radiant_rs::Renderer from given glium::Display.
    ///
    /// As an alternative to [`backend::create_display()`](fn.create_display.html), this allows for glium rendering and display handling while radiant only handles 2d rendering.
    /// Note that this renderer does not have a default target. Use `Renderer::render_to()` to specify one or `backend::target_frame()` to target
    /// a glium::Frame.
    // TODO: add helpful panic message if user tries to use this renderer without targeting a texture or glium::Frame first
    pub fn create_renderer(display: &glium::Display) -> core::Result<core::Renderer> {

        let display = super::Display(display.clone());

        let context = core::Context::new();
        context.set_primary_display(&display);

        let identity_texture = core::Texture::builder(&context).format(core::TextureFormat::F16F16F16F16).dimensions((1, 1)).build().unwrap();
        identity_texture.clear(core::Color::WHITE);

        Ok(core::Renderer {
            empty_texture   : identity_texture,
            program         : Rc::new(core::Program::new(&context, core::DEFAULT_FS)?),
            context         : context,
            target          : Rc::new(RefCell::new(Vec::new())),
        })
    }

    /// Causes given renderer to render to the given glium::Frame within the closure.
    ///
    /// Renderers default to rendering to the current frame, unless they were created by `backend::create_renderer()`, in which case control of the
    /// frame/display remains outside of Radiant. This method allows such Renderers to output to Glium frames.
    pub fn target_frame<F>(renderer: &core::Renderer, frame: &mut glium::Frame, draw_func: F) where F: FnMut() {

        let wrapper = {
            // we need to own the frame to wrap it
            let owned_frame = mem::replace(frame, unsafe { mem::uninitialized() });
            // wrap glium frame to make it rendertarget compatible
            let backend_frame = super::Frame(owned_frame);
            Rc::new(RefCell::new(Some(backend_frame)))
        };

        // render to that target
        renderer.render_to(&core::RenderTarget::frame(&wrapper), draw_func);

        // undo the wrapping
        let backend_frame = mem::replace(&mut *wrapper.borrow_mut(), None).unwrap();
        let owned_frame = backend_frame.into_inner();
        // move the frame back, forget owned_frame
        let uninitialized = mem::replace(frame, owned_frame);
        mem::forget(uninitialized);
    }

    /// Passes a mutable reference to the current glium::Frame used by Radiant to the given callback.
    pub fn take_frame<F>(display: &core::Display, mut callback: F) where F: FnMut(&mut glium::Frame) {
        display.frame(|ref mut frame| callback(&mut frame.0))
    }

    /// Returns the underlying glium Texture2d for given Texture
    pub fn get_texture(texture: &core::Texture) -> Rc<glium::Texture2d> {
        unsafe { mem::transmute(texture.handle.clone()) }
    }
}

// --------------
// Error
// --------------

#[derive(Debug)]
pub enum Error {
    OsError(String),
    Incompatible(String),
    NotAvailable(String),
    WindowCreation(String),
    Unknown,
}

impl From<glium::backend::glutin::DisplayCreationError> for core::Error {
    /// Converts image error to radiant error
    fn from(error: glium::backend::glutin::DisplayCreationError) -> core::Error {
        use self::glium::backend::glutin::DisplayCreationError as DCE;
        use self::glium::glutin::CreationError as CE;
        use self::glium::glutin::WindowCreationError as WCE;
        #[allow(unreachable_patterns)]
        let backend_error = match error {
            DCE::GlutinCreationError(creation_error) => match creation_error {
                CE::OsError(message) => Error::OsError(message),
                CE::NotSupported(message) => Error::Incompatible(format!("Not supported: {}", message)),
                CE::NoBackendAvailable(message) => Error::NotAvailable(message.to_string()),
                CE::RobustnessNotSupported => Error::Incompatible("Robustness not supported".to_string()),
                CE::OpenGlVersionNotSupported => Error::Incompatible("OpenGL version not supported".to_string()),
                CE::NoAvailablePixelFormat => Error::Incompatible("No available pixel format".to_string()),
                CE::PlatformSpecific(message) => Error::Incompatible(format!("Platform error: {}", message)),
                CE::Window(win_error) => match win_error {
                    WCE::OsError(message) => Error::WindowCreation(message),
                    WCE::NotSupported => Error::WindowCreation("Not supported".to_string()),
                }
            },
            DCE::IncompatibleOpenGl(message) => { Error::Incompatible(message.0) }
            _ => { Error::Unknown }
        };
        core::Error::BackendError(backend_error)
    }
}

// --------------
// Display
// --------------

#[derive(Clone)]
pub struct Display(glium::Display);

impl Display {
    pub fn new(descriptor: core::DisplayBuilder) -> core::Result<Display> {
        use self::glium::glutin::dpi::LogicalSize;

        let events_loop = Self::events_loop();

        let monitor = if let Some(ref monitor) = descriptor.monitor {
            monitor.inner.0.clone()
        } else {
            Self::events_loop().get_primary_monitor()
        };

        let display = {
            let parent_display;
            let gl_window;

            let window = glium::glutin::WindowBuilder::new()
                .with_dimensions(LogicalSize::from_physical((descriptor.width, descriptor.height), monitor.get_hidpi_factor()))
                .with_title(descriptor.title.clone())
                .with_transparency(descriptor.transparent)
                .with_decorations(descriptor.decorations)
                .with_visibility(descriptor.visible)
                .with_fullscreen(if let Some(ref monitor) = descriptor.monitor { Some(monitor.inner.0.clone()) } else { None });

            let mut context = glium::glutin::ContextBuilder::new()
                .with_vsync(descriptor.vsync);

            if let Some(ref parent_context) = descriptor.context {
                if parent_context.has_primary_display() {
                    parent_display = parent_context.lock().backend_context.as_ref().unwrap().display.clone();
                    gl_window = parent_display.gl_window();
                    context = context.with_shared_lists(gl_window.context());
                }
            }

            glium::Display::new(window, context, &events_loop)?
        };
        Ok(Display(display))
    }
    pub fn draw(self: &Self) -> Frame {
        Frame(self.0.draw())
    }
    pub fn framebuffer_dimensions(self: &Self) -> core::Point2<u32> {
        self.0.get_framebuffer_dimensions().into()
    }
    pub fn window_dimensions(self: &Self) -> core::Point2<u32> {
        self.0.gl_window().get_inner_size().map_or((0, 0), |l|  l.into())
    }
    pub fn set_cursor_position(self: &Self, position: core::Point2<i32>) {
        self.0.gl_window().set_cursor_position((position.0, position.1).into()).unwrap();
    }
    pub fn set_cursor_state(self: &Self, state: core::CursorState) {
        use core::CursorState as CS;
        match state {
            CS::Normal => self.0.gl_window().grab_cursor(false).unwrap(), // todo: handle grab vs hide now that glium doesn't anymore
            CS::Hide => self.0.gl_window().hide_cursor(true),
            CS::Grab => self.0.gl_window().grab_cursor(true).unwrap(),
        };
    }
    pub fn set_fullscreen(self: &Self, monitor: Option<core::Monitor>) -> bool {
        if let Some(monitor) = monitor {
            self.0.gl_window().set_fullscreen(Some(monitor.inner.0.clone()));
        } else {
            self.0.gl_window().set_fullscreen(None);
        }
        true
    }
    pub fn poll_events<F>(self: &Self, mut callback: F) where F: FnMut(core::Event) -> () {
        Self::events_loop().poll_events(|glutin_event| {
            if let Some(event) = Self::map_event(glutin_event) {
                callback(event);
            }
        });
    }
    pub fn show(self: &Self) {
        self.0.gl_window().show();
    }
    pub fn hide(self: &Self) {
        self.0.gl_window().hide()
    }
    pub fn set_title(self: &Self, title: &str) {
        self.0.gl_window().set_title(title);
    }
    fn events_loop() -> &'static mut glutin::EventsLoop {
        init_events_loop(|| glutin::EventsLoop::new())
    }
    fn map_event(event: glium::glutin::Event) -> Option<core::Event> {
        use self::glutin::ElementState;
        use self::glutin::Event as GlutinEvent;
        use self::glutin::DeviceEvent;
        use self::glutin::WindowEvent;
        use self::glutin::KeyboardInput;
        use self::glutin::MouseButton;

        match event {
            GlutinEvent::WindowEvent { event: window_event, .. } => {
                match window_event {
                    WindowEvent::Focused(true) => {
                        Some(core::Event::Focus)
                    }
                    WindowEvent::Focused(false) => {
                        Some(core::Event::Blur)
                    }
                    WindowEvent::CloseRequested => {
                        Some(core::Event::Close)
                    }
                    WindowEvent::CursorMoved { position: glutin::dpi::LogicalPosition { x, y }, .. } => {
                        Some(core::Event::MousePosition(x as i32, y as i32))
                    }
                    WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: Some(virtual_code), .. }, .. } => {
                        let key_id = Self::map_key_code(virtual_code) as usize;
                        if key_id < core::NUM_KEYS {
                            Some(core::Event::KeyboardInput(key_id, state == ElementState::Pressed))
                        } else {
                            None
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        match button {
                            MouseButton::Left => Some(core::Event::MouseInput(0, state == ElementState::Pressed)),
                            MouseButton::Middle => Some(core::Event::MouseInput(1, state == ElementState::Pressed)),
                            MouseButton::Right => Some(core::Event::MouseInput(2, state == ElementState::Pressed)),
                            MouseButton::Other(index) => Some(core::Event::MouseInput(3 + index as usize, state == ElementState::Pressed)),
                        }
                    }
                    _ => None
                }
            },
            GlutinEvent::DeviceEvent { event: device_event, .. } => {
                match device_event {
                    DeviceEvent::MouseMotion { delta: (x, y) } => {
                        Some(core::Event::MouseDelta(x as i32, y as i32))
                    }
                    _ => None
                }
            }
            _ => None
        }
    }
    fn map_key_code(key: glium::glutin::VirtualKeyCode) -> core::InputId {
        use self::glutin::VirtualKeyCode as VK;
        use core::InputId as IID;
        #[allow(unreachable_patterns)]
        match key {
            VK::Key1          => IID::Key1,
            VK::Key2          => IID::Key2,
            VK::Key3          => IID::Key3,
            VK::Key4          => IID::Key4,
            VK::Key5          => IID::Key5,
            VK::Key6          => IID::Key6,
            VK::Key7          => IID::Key7,
            VK::Key8          => IID::Key8,
            VK::Key9          => IID::Key9,
            VK::Key0          => IID::Key0,
            VK::A             => IID::A,
            VK::B             => IID::B,
            VK::C             => IID::C,
            VK::D             => IID::D,
            VK::E             => IID::E,
            VK::F             => IID::F,
            VK::G             => IID::G,
            VK::H             => IID::H,
            VK::I             => IID::I,
            VK::J             => IID::J,
            VK::K             => IID::K,
            VK::L             => IID::L,
            VK::M             => IID::M,
            VK::N             => IID::N,
            VK::O             => IID::O,
            VK::P             => IID::P,
            VK::Q             => IID::Q,
            VK::R             => IID::R,
            VK::S             => IID::S,
            VK::T             => IID::T,
            VK::U             => IID::U,
            VK::V             => IID::V,
            VK::W             => IID::W,
            VK::X             => IID::X,
            VK::Y             => IID::Y,
            VK::Z             => IID::Z,
            VK::Escape        => IID::Escape,
            VK::F1            => IID::F1,
            VK::F2            => IID::F2,
            VK::F3            => IID::F3,
            VK::F4            => IID::F4,
            VK::F5            => IID::F5,
            VK::F6            => IID::F6,
            VK::F7            => IID::F7,
            VK::F8            => IID::F8,
            VK::F9            => IID::F9,
            VK::F10           => IID::F10,
            VK::F11           => IID::F11,
            VK::F12           => IID::F12,
            VK::F13           => IID::F13,
            VK::F14           => IID::F14,
            VK::F15           => IID::F15,
            VK::Snapshot      => IID::Snapshot,
            VK::Scroll        => IID::Scroll,
            VK::Pause         => IID::Pause,
            VK::Insert        => IID::Insert,
            VK::Home          => IID::Home,
            VK::Delete        => IID::Delete,
            VK::End           => IID::End,
            VK::PageDown      => IID::PageDown,
            VK::PageUp        => IID::PageUp,
            VK::Left          => IID::CursorLeft,
            VK::Up            => IID::CursorUp,
            VK::Right         => IID::CursorRight,
            VK::Down          => IID::CursorDown,
            VK::Back          => IID::Backspace,
            VK::Return        => IID::Return,
            VK::Space         => IID::Space,
            //VK::Caret         => IID::Caret,
            VK::Numlock       => IID::Numlock,
            VK::Numpad0       => IID::Numpad0,
            VK::Numpad1       => IID::Numpad1,
            VK::Numpad2       => IID::Numpad2,
            VK::Numpad3       => IID::Numpad3,
            VK::Numpad4       => IID::Numpad4,
            VK::Numpad5       => IID::Numpad5,
            VK::Numpad6       => IID::Numpad6,
            VK::Numpad7       => IID::Numpad7,
            VK::Numpad8       => IID::Numpad8,
            VK::Numpad9       => IID::Numpad9,
            VK::AbntC1        => IID::AbntC1,
            VK::AbntC2        => IID::AbntC2,
            VK::Add           => IID::Add,
            VK::Apostrophe    => IID::Apostrophe,
            VK::Apps          => IID::Apps,
            VK::At            => IID::At,
            VK::Ax            => IID::Ax,
            VK::Backslash     => IID::Backslash,
            VK::Calculator    => IID::Calculator,
            VK::Capital       => IID::Capital,
            VK::Colon         => IID::Colon,
            VK::Comma         => IID::Comma,
            VK::Convert       => IID::Convert,
            VK::Decimal       => IID::Decimal,
            VK::Divide        => IID::Divide,
            VK::Equals        => IID::Equals,
            VK::Grave         => IID::Grave,
            VK::Kana          => IID::Kana,
            VK::Kanji         => IID::Kanji,
            VK::LAlt          => IID::LAlt,
            VK::LBracket      => IID::LBracket,
            VK::LControl      => IID::LControl,
            VK::LShift        => IID::LShift,
            VK::LWin          => IID::LWin,
            VK::Mail          => IID::Mail,
            VK::MediaSelect   => IID::MediaSelect,
            VK::MediaStop     => IID::MediaStop,
            VK::Minus         => IID::Minus,
            VK::Multiply      => IID::Multiply,
            VK::Mute          => IID::Mute,
            VK::MyComputer    => IID::MyComputer,
            VK::NextTrack     => IID::NextTrack,
            VK::NoConvert     => IID::NoConvert,
            VK::NumpadComma   => IID::NumpadComma,
            VK::NumpadEnter   => IID::NumpadEnter,
            VK::NumpadEquals  => IID::NumpadEquals,
            VK::OEM102        => IID::OEM102,
            VK::Period        => IID::Period,
            VK::PlayPause     => IID::PlayPause,
            VK::Power         => IID::Power,
            VK::PrevTrack     => IID::PrevTrack,
            VK::RAlt          => IID::RAlt,
            VK::RBracket      => IID::RBracket,
            VK::RControl      => IID::RControl,
            VK::RShift        => IID::RShift,
            VK::RWin          => IID::RWin,
            VK::Semicolon     => IID::Semicolon,
            VK::Slash         => IID::Slash,
            VK::Sleep         => IID::Sleep,
            VK::Stop          => IID::Stop,
            VK::Subtract      => IID::Subtract,
            VK::Sysrq         => IID::Sysrq,
            VK::Tab           => IID::Tab,
            VK::Underline     => IID::Underline,
            VK::Unlabeled     => IID::Unlabeled,
            VK::VolumeDown    => IID::VolumeDown,
            VK::VolumeUp      => IID::VolumeUp,
            VK::Wake          => IID::Wake,
            VK::WebBack       => IID::WebBack,
            VK::WebFavorites  => IID::WebFavorites,
            VK::WebForward    => IID::WebForward,
            VK::WebHome       => IID::WebHome,
            VK::WebRefresh    => IID::WebRefresh,
            VK::WebSearch     => IID::WebSearch,
            VK::WebStop       => IID::WebStop,
            VK::Yen           => IID::Yen,
            VK::Compose       => IID::Compose,
            VK::NavigateForward => IID::NavigateForward,
            VK::NavigateBackward => IID::NavigateBackward,
            _                 => IID::Unsupported
        }
    }
}

// --------------
// Frame
// --------------

pub struct Frame(glium::Frame);

impl Frame {

    /// Consumes the Frame, returning a glium::Frame.
    pub(crate) fn into_inner(self: Self) -> glium::Frame {
        self.0
    }

    /// Clears the frame with the given color.
    pub fn clear(self: &mut Self, color: core::Color) {
        let core::Color(r, g, b, a) = color;
        self.0.clear_color(r, g, b, a);
    }

    /// Finishes the frame. Called on swap.
    pub fn finish(self: Self) {
        self.0.finish().unwrap();
    }

    /// Copies given texture to given display.
    pub fn copy_from_texture(self: &Self, source: &core::Texture, filter: core::TextureFilter) {
        source.handle.0.as_surface().fill(&self.0, magnify_filter(filter));
    }

    /// Copies the source rectangle to the target rectangle on the given display.
    pub fn copy_rect(self: &Self, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
        let height = self.0.get_dimensions().1;
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, height, target_rect, height);
        self.0.blit_color(&glium_src_rect, &self.0, &glium_target_rect, magnify_filter(filter));
    }

    /// Copies the source rectangle from the given texture to the target rectangle on the given display.
    pub fn copy_rect_from_texture(self: &Self, source: &core::Texture, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
        let target_height = self.0.get_dimensions().1;
        let source_height = source.handle.0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        source.handle.0.as_surface().blit_color(&glium_src_rect, &self.0, &glium_target_rect, magnify_filter(filter));
    }

    /// Returns the dimensions of the frame.
    pub fn dimensions(self: &Self) -> core::Point2<u32> {
        self.0.get_dimensions().into()
    }
}

// --------------
// Program
// --------------

pub struct Program(glium::Program);

impl Program {
    /// Creates a shader program from given vertex- and fragment-shader sources.
    pub fn new(context: &Context, vertex_shader: &str, fragment_shader: &str) -> core::Result<Program> {
        use self::glium::program::ProgramCreationError;
        use core::Error;
        match glium::Program::from_source(&context.display, vertex_shader, fragment_shader, None) {
            Err(ProgramCreationError::CompilationError(message)) => { Err(Error::ShaderError(format!("Shader compilation failed with: {}", message))) }
            Err(ProgramCreationError::LinkingError(message))     => { Err(Error::ShaderError(format!("Shader linking failed with: {}", message))) }
            Err(_)                                               => { Err(Error::ShaderError("No shader support found".to_string())) }
            Ok(program)                                          => { Ok(Program(program)) }
        }
    }
}

// --------------
// Monitor
// --------------

#[derive(Clone)]
pub struct Monitor(glium::glutin::MonitorId);

impl Monitor {

    /// Returns the device dimensions
    pub fn get_dimensions(self: &Self) -> core::Point2<u32> {
        self.0.get_dimensions().into()
    }

    /// Returns the device name.
    pub fn get_name(self: &Self) -> Option<String> {
        self.0.get_name()
    }
}

pub struct MonitorIterator(glium::glutin::AvailableMonitorsIter);

impl MonitorIterator {

    /// Returns an inter
    pub fn new() -> Self {
        MonitorIterator(Display::events_loop().get_available_monitors())
    }
}

impl Iterator for MonitorIterator {
    type Item = Monitor;
    fn next(&mut self) -> Option<Monitor> {
        let current = self.0.next();
        match current {
            Some(monitor) => Some(Monitor(monitor)),
            None => None,
        }
    }
}

// --------------
// Texture2d
// --------------

pub struct Texture2d(glium::texture::Texture2d);

impl Texture2d {
    pub fn new(context: &Context, width: u32, height: u32, format: core::TextureFormat, data: Option<core::RawFrame>) -> Self {
        let texture = if let Some(rawdata) = data {
            glium::texture::Texture2d::with_format(
                &context.display,
                RawFrame(rawdata),
                Self::convert_format(format),
                glium::texture::MipmapsOption::NoMipmap
            ).unwrap()
        } else {
            let texture = glium::texture::Texture2d::empty_with_format(
                &context.display,
                Self::convert_format(format),
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            ).unwrap();
            texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
            texture
        };
        Texture2d(texture)
    }
    pub fn clear(self: &Self, color: core::Color) {
        let core::Color(r, g, b, a) = color;
        self.0.as_surface().clear_color(r, g, b, a);
    }
    pub fn write(self: &Self, rect: &core::Rect<u32>, data: &Vec<u8>) {
        use std::borrow::Cow;
        self.0.main_level().write(
            glium::Rect {
                left: (rect.0).0,
                bottom: (rect.0).1,
                width: (rect.1).0 - (rect.0).0, // !todo Rect is terrible
                height: (rect.1).1 - (rect.0).1,
            },
            glium::texture::RawImage2d {
                data: Cow::Borrowed(&data),
                width: (rect.1).0 - (rect.0).0,
                height: (rect.1).1 - (rect.0).1,
                format: glium::texture::ClientFormat::U8
            }
        );
    }
    pub fn copy_from(self: &Self, src_texture: &core::Texture, filter: core::TextureFilter) {
        src_texture.handle.0.as_surface().fill(&self.0.as_surface(), magnify_filter(filter))
    }
    pub fn copy_rect_from(self: &Self, src_texture: &core::Texture, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
        let target_height = self.0.height();
        let source_height = src_texture.handle.0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        src_texture.handle.0.as_surface().blit_color(&glium_src_rect, &self.0.as_surface(), &glium_target_rect, magnify_filter(filter));
    }
    pub fn copy_from_frame(self: &Self, src_frame: &Frame, filter: core::TextureFilter) {
        src_frame.0.fill(&self.0.as_surface(), magnify_filter(filter));
    }
    pub fn copy_rect_from_frame(self: &Self, src_frame: &Frame, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
        let source_height = src_frame.0.get_dimensions().1;
        let target_height = self.0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        src_frame.0.blit_color(&glium_src_rect, &self.0.as_surface(), &glium_target_rect, magnify_filter(filter));
    }

    /// Converts TextureFormat to the supported gliums texture formats
    fn convert_format(format: core::TextureFormat) -> glium::texture::UncompressedFloatFormat {
        use self::glium::texture::UncompressedFloatFormat as GF;
        use core::TextureFormat as RF;
        match format {
            RF::U8              => GF::U8,
            RF::U16             => GF::U16,
            RF::U8U8            => GF::U8U8,
            RF::U16U16          => GF::U16U16,
            RF::U10U10U10       => GF::U10U10U10,
            RF::U12U12U12       => GF::U12U12U12,
            RF::U16U16U16       => GF::U16U16U16,
            RF::U2U2U2U2        => GF::U2U2U2U2,
            RF::U4U4U4U4        => GF::U4U4U4U4,
            RF::U5U5U5U1        => GF::U5U5U5U1,
            RF::U8U8U8U8        => GF::U8U8U8U8,
            RF::U10U10U10U2     => GF::U10U10U10U2,
            RF::U12U12U12U12    => GF::U12U12U12U12,
            RF::U16U16U16U16    => GF::U16U16U16U16,
            RF::I16I16I16I16    => GF::I16I16I16I16,
            RF::F16             => GF::F16,
            RF::F16F16          => GF::F16F16,
            RF::F16F16F16F16    => GF::F16F16F16F16,
            RF::F32             => GF::F32,
            RF::F32F32          => GF::F32F32,
            RF::F32F32F32F32    => GF::F32F32F32F32,
            RF::F11F11F10       => GF::F11F11F10,
        }
    }
}

// --------------
// Texture2dArray
// --------------

#[derive(Clone)]
struct RawFrame(core::RawFrame);

impl<'a> glium::texture::Texture2dDataSource<'a> for RawFrame {
    type Data = u8;
    fn into_raw(self: Self) -> glium::texture::RawImage2d<'a, Self::Data> {
        use std::borrow::Cow;
        glium::texture::RawImage2d {
            data: Cow::Owned(self.0.data),
            width: self.0.width,
            height: self.0.height,
            format: match self.0.channels {
                1 => glium::texture::ClientFormat::U8,
                4 => glium::texture::ClientFormat::U8U8U8U8,
                _ => glium::texture::ClientFormat::U8,  // todo: ugly, need enum
            }
        }
    }
}

pub struct Texture2dArray(glium::texture::Texture2dArray);

impl Texture2dArray {
    /// Generates glium texture array from given vector of textures
    pub fn new(context: &Context, raw: &Vec<core::RawFrame>) -> Self {

        use self::glium::texture;
        use std::mem::transmute;

        let raw_wrapped: Vec<RawFrame> = unsafe { transmute(raw.clone()) };

        Texture2dArray(
            if raw_wrapped.len() > 0 {
                texture::Texture2dArray::with_format(&context.display, raw_wrapped, texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap).unwrap()
            } else {
                texture::Texture2dArray::empty_with_format(&context.display, texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap, 2, 2, 1).unwrap()
            }
        )
    }
}

// --------------
// Context
// --------------

struct VertexBufferCacheItem {
    hint    : usize,
    age     : usize,
    buffer  : glium::VertexBuffer<Vertex>,
}

impl VertexBufferCacheItem {
    pub fn new(display: &glium::Display, num_vertices: usize, buffer_hint: usize) -> VertexBufferCacheItem {
        VertexBufferCacheItem {
            hint: buffer_hint,
            age: 0,
            buffer: if buffer_hint == 0 {
                glium::VertexBuffer::empty(display, num_vertices).unwrap()
            } else {
                glium::VertexBuffer::empty_dynamic(display, num_vertices).unwrap()
            }
        }
    }
}

pub struct Context {
    display         : glium::Display,
    index_buffer    : glium::IndexBuffer<u32>,
    vertex_buffers  : Vec<VertexBufferCacheItem>,
}

impl Context {

    /// Creates a new backend context.
    pub fn new(display: &Display, initial_capacity: usize) -> Self {
        Context {
            display         : display.0.clone(),
            index_buffer    : Self::create_index_buffer(&display.0, initial_capacity),
            vertex_buffers  : Vec::new(),
        }
    }

    /// Creates an index buffer for rectangles consisting of two triangles.
    fn create_index_buffer(display: &glium::Display, max_sprites: usize) -> glium::IndexBuffer<u32> {

        let mut data = Vec::with_capacity(max_sprites as usize * 6);

        for i in 0..max_sprites {
            let num = i as u32;
            data.push(num * 4);
            data.push(num * 4 + 1);
            data.push(num * 4 + 2);
            data.push(num * 4 + 1);
            data.push(num * 4 + 3);
            data.push(num * 4 + 2);
        }

        glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &data).unwrap()
    }

    /// Update index buffer to given size
    fn update_index_buffer(self: &mut Self, max_sprites: usize) {
        if max_sprites * 6 > self.index_buffer.len() {
            self.index_buffer = Self::create_index_buffer(&self.display, max_sprites);
        }
    }

    /// Select a vertex buffer for given number of vertices.
    fn select_vertex_buffer(self: &mut Self, buffer_hint: usize, num_vertices: usize) -> (usize, bool) {

        for buffer in self.vertex_buffers.iter_mut() {
            buffer.age += 1;
        }

        if let Some(id) = self.vertex_buffers.iter().position(|ref item| item.hint == buffer_hint && item.buffer.len() >= num_vertices) {
            self.vertex_buffers[id].age = 0;
            (id, false)
        } else if self.vertex_buffers.len() < MAX_BUFFERS {
            self.vertex_buffers.push(VertexBufferCacheItem::new(&self.display, num_vertices, buffer_hint));
            (self.vertex_buffers.len() - 1, true)
        } else {
            if let Some((id, _)) = self.vertex_buffers.iter().enumerate().max_by(|&(_, a), &(_, b)| a.age.cmp(&b.age)) {
                self.vertex_buffers[id] = VertexBufferCacheItem::new(&self.display, num_vertices, buffer_hint);
                (id, true)
            } else {
                (1, true)
            }
        }
    }

    /// Draw given vertices.
    fn draw(self: &mut Self, target: &core::RenderTarget, vertices: &[Vertex], dirty: bool, buffer_hint: usize, program: &Program, uniforms: &GliumUniformList, blendmode: &core::BlendMode) {

        let num_vertices = vertices.len();
        let num_sprites = num_vertices / 4;

        if num_vertices < 1 {
            return;
        }

        // set up vertex buffer

        let (vb_index, vb_dirty) = self.select_vertex_buffer(buffer_hint, num_vertices);
        {
            if dirty || vb_dirty {
                let vb_slice = self.vertex_buffers[vb_index].buffer.slice(0 .. num_vertices).unwrap();
                vb_slice.write(&vertices[0 .. num_vertices]);
            }
        }

        // set up index buffer

        self.update_index_buffer(num_sprites);
        let ib_slice = self.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : glium_blendmode(blendmode),
            .. Default::default()
        };

        // draw

        match target.0 {
            core::RenderTargetInner::Frame(ref display) => {
                let mut frame = display.borrow_mut();
                let frame = &mut frame.as_mut().expect(core::NO_FRAME_PREPARED).0;
                frame.draw(&self.vertex_buffers[vb_index].buffer, &ib_slice, &program.0, uniforms, &draw_parameters).unwrap();
            }
            core::RenderTargetInner::Texture(ref texture) => {
                texture.handle.0.as_surface().draw(&self.vertex_buffers[vb_index].buffer, &ib_slice, &program.0, uniforms, &draw_parameters).unwrap();
            }
            core::RenderTargetInner::None => { }
        }
    }
}

// --------------
// Uniforms
// --------------

enum GliumUniform<'a> {
    Bool(bool),
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    Mat4([[f32; 4]; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Double(f64),
    DoubleMat4([[f64; 4]; 4]),
    DoubleVec2([f64; 2]),
    DoubleVec3([f64; 3]),
    DoubleVec4([f64; 4]),
    Texture2d(&'a glium::texture::Texture2d),
    Texture2dArray(&'a glium::texture::Texture2dArray),
    Sampled2d(glium::uniforms::Sampler<'a, glium::texture::Texture2d>),
}

/// A structure to implement gliums Uniforms trait on.
struct GliumUniformList<'a>(Vec<(&'a str, GliumUniform<'a>)>);

impl<'a> GliumUniformList<'a> {
    pub fn from_uniform_list(list: &'a core::UniformList) -> Self {
        let mut result = GliumUniformList(Vec::new());
        for (name, uniform) in list.0.iter() {
            result.add_uniform(name, uniform);
        }
        result
    }
    pub fn add(self: &mut Self, name: &'a str, uniform: GliumUniform<'a>) -> &mut Self {
        self.0.push((name, uniform));
        self
    }
    fn add_uniform(self: &mut Self, name: &'a str, uniform: &'a core::Uniform) {
        use self::glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, SamplerWrapFunction};
        use core::Uniform as CU;
        use core::TextureWrap as TW;
        self.0.push((name, match *uniform {
            CU::Bool(val) => { GliumUniform::Bool(val) },
            CU::SignedInt(val) => { GliumUniform::SignedInt(val) },
            CU::UnsignedInt(val) => { GliumUniform::UnsignedInt(val) },
            CU::Float(val) => { GliumUniform::Float(val) },
            CU::Mat4(val) => { GliumUniform::Mat4(val) },
            CU::Vec2(val) => { GliumUniform::Vec2(val) },
            CU::Vec3(val) => { GliumUniform::Vec3(val) },
            CU::Vec4(val) => { GliumUniform::Vec4(val) },
            CU::Double(val) => { GliumUniform::Double(val) },
            CU::DoubleMat4(val) => { GliumUniform::DoubleMat4(val) },
            CU::DoubleVec2(val) => { GliumUniform::DoubleVec2(val) },
            CU::DoubleVec3(val) => { GliumUniform::DoubleVec3(val) },
            CU::DoubleVec4(val) => { GliumUniform::DoubleVec4(val) },
            CU::Texture(ref val) => {
                let glium_minify = if val.minify == core::TextureFilter::Linear { MinifySamplerFilter::Linear } else { MinifySamplerFilter::Nearest };
                let glium_magnify = if val.magnify == core::TextureFilter::Linear { MagnifySamplerFilter::Linear } else { MagnifySamplerFilter::Nearest };
                let glium_wrap = match val.wrap {
                    TW::Repeat         => SamplerWrapFunction::Repeat,
                    TW::Mirror         => SamplerWrapFunction::Mirror,
                    TW::Clamp          => SamplerWrapFunction::Clamp,
                    TW::MirrorClamp    => SamplerWrapFunction::MirrorClamp,
                };
                GliumUniform::Sampled2d(
                    val.handle.0
                        .sampled()
                        .minify_filter(glium_minify)
                        .magnify_filter(glium_magnify)
                        .wrap_function(glium_wrap)
                )
            },
        }));
    }
}

impl<'b> Uniforms for GliumUniformList<'b> {
    fn visit_values<'a, F>(self: &'a Self, mut output: F) where F: FnMut(&str, glium::uniforms::UniformValue<'a>) {
        use self::glium::uniforms::{UniformValue, AsUniformValue};
        for &(name, ref uniform) in &self.0 {
            output(name, match *uniform {
                GliumUniform::Bool(val) => { UniformValue::Bool(val) },
                GliumUniform::SignedInt(val) => { UniformValue::SignedInt(val) },
                GliumUniform::UnsignedInt(val) => { UniformValue::UnsignedInt(val) },
                GliumUniform::Float(val) => { UniformValue::Float(val) },
                GliumUniform::Mat4(val) => { UniformValue::Mat4(val) },
                GliumUniform::Vec2(val) => { UniformValue::Vec2(val) },
                GliumUniform::Vec3(val) => { UniformValue::Vec3(val) },
                GliumUniform::Vec4(val) => { UniformValue::Vec4(val) },
                GliumUniform::Double(val) => { UniformValue::Double(val) },
                GliumUniform::DoubleMat4(val) => { UniformValue::DoubleMat4(val) },
                GliumUniform::DoubleVec2(val) => { UniformValue::DoubleVec2(val) },
                GliumUniform::DoubleVec3(val) => { UniformValue::DoubleVec3(val) },
                GliumUniform::DoubleVec4(val) => { UniformValue::DoubleVec4(val) },
                GliumUniform::Sampled2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2dArray(ref val) => {
                    val.as_uniform_value()
                }
            });
        }
    }
}

// --------------
// Vertex
// --------------

#[derive(Copy, Clone)]
struct Vertex(core::Vertex);

macro_rules! implement_wrapped_vertex {
    ($struct_name:ident, $($field_name:ident),+) => (
        impl glium::vertex::Vertex for $struct_name {
            #[inline]
            fn build_bindings() -> glium::vertex::VertexFormat {
                use std::borrow::Cow;

                // TODO: use a &'static [] if possible

                Cow::Owned(vec![
                    $(
                        (
                            Cow::Borrowed(stringify!($field_name)),
                            {
                                // calculate the offset of the struct fields
                                let dummy: $struct_name = unsafe { ::std::mem::uninitialized() };
                                let offset: usize = {
                                    let dummy_ref = &(dummy.0);
                                    let field_ref = &(dummy.0).$field_name;
                                    (field_ref as *const _ as usize) - (dummy_ref as *const _ as usize)
                                };
                                // NOTE: `glium::vertex::Vertex` requires `$struct_name` to have `Copy` trait
                                // `Copy` excludes `Drop`, so we don't have to `std::mem::forget(dummy)`
                                offset
                            },
                            {
                                fn attr_type_of_val<T: glium::vertex::Attribute>(_: &T)
                                    -> glium::vertex::AttributeType
                                {
                                    <T as glium::vertex::Attribute>::get_type()
                                }
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0usize) };
                                attr_type_of_val(&(dummy.0).$field_name)
                            },
                            false
                        )
                    ),+
                ])
            }
        }
    );

    ($struct_name:ident, $($field_name:ident),+,) => (
        implement_wrapped_vertex!($struct_name, $($field_name),+);
    );
}

implement_wrapped_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv, components);

// --------------
// Drawing
// --------------

pub fn draw_layer(target: &core::RenderTarget, program: &core::Program, context: &mut core::ContextData, layer: &core::Layer, component: u32) {

    use self::glium::uniforms::{MagnifySamplerFilter, SamplerWrapFunction};
    use std::mem::transmute;

    let mut glium_uniforms = GliumUniformList::from_uniform_list(&program.uniforms);
    glium_uniforms
        .add("u_view", GliumUniform::Mat4(*layer.view_matrix().deref().deref()))
        .add("u_model", GliumUniform::Mat4(*layer.model_matrix().deref().deref()))
        .add("_rd_color", GliumUniform::Vec4(layer.color().deref().into()))
        .add("_rd_tex", GliumUniform::Sampled2d(context.font_texture.as_ref().unwrap().0.sampled().magnify_filter(MagnifySamplerFilter::Nearest).wrap_function(SamplerWrapFunction::Clamp)))
        .add("_rd_comp", GliumUniform::UnsignedInt(component))
        .add("_rd_tex1", GliumUniform::Texture2dArray(&context.tex_arrays[1].data.0))
        .add("_rd_tex2", GliumUniform::Texture2dArray(&context.tex_arrays[2].data.0))
        .add("_rd_tex3", GliumUniform::Texture2dArray(&context.tex_arrays[3].data.0))
        .add("_rd_tex4", GliumUniform::Texture2dArray(&context.tex_arrays[4].data.0))
        .add("_rd_tex5", GliumUniform::Texture2dArray(&context.tex_arrays[5].data.0));

    let vertices = layer.vertices();
    let vertices = vertices.deref();

    context.backend_context.as_mut().unwrap().draw(target, unsafe { transmute(vertices) }, layer.undirty(), layer.id(), &program.sprite_program, &glium_uniforms, &layer.blendmode());
}

pub fn draw_rect<T>(target: &core::RenderTarget, program: &core::Program, context: &mut core::ContextData, blend: core::BlendMode, info: core::DrawBuilder<T>, view_matrix: core::Mat4, model_matrix: core::Mat4, color: core::Color, texture: &core::Texture) {

    use std::mem::transmute;
    use core::Point2Trait;

    let mut glium_uniforms = GliumUniformList::from_uniform_list(&program.uniforms);
    glium_uniforms
        .add("u_view", GliumUniform::Mat4(view_matrix.into()))
        .add("u_model", GliumUniform::Mat4(model_matrix.into()))
        .add("_rd_color", GliumUniform::Vec4(color.into()))
        .add("_rd_tex", GliumUniform::Texture2d(&texture.handle.0))
        .add("_rd_offset", GliumUniform::Vec2(info.rect.0.as_array()))
        .add("_rd_dimensions", GliumUniform::Vec2(info.rect.1.as_array()))
        .add("_rd_has_tex", GliumUniform::Bool(info.texture.is_some()));

    let vertices = &context.single_rect;
    let vertices = &vertices[..];

    context.backend_context.as_mut().unwrap().draw(target, unsafe { transmute(vertices) }, false, 0, &program.texture_program, &glium_uniforms, &blend);
}

// --------------
// Misc
// --------------

// Converts given blendmode to glium blendmode
fn glium_blendmode(blendmode: &core::BlendMode) -> glium::Blend {

    fn blendfunc(function: core::BlendingFunction) -> glium::BlendingFunction {

        use core::BlendingFunction as CF;
        use self::glium::BlendingFunction as GF;

        fn blendfactor(factor: core::LinearBlendingFactor) -> glium::LinearBlendingFactor {
            use core::LinearBlendingFactor as CB;
            use self::glium::LinearBlendingFactor as GB;
            match factor {
                CB::Zero                      => GB::Zero,
                CB::One                       => GB::One,
                CB::SourceColor               => GB::SourceColor,
                CB::OneMinusSourceColor       => GB::OneMinusSourceColor,
                CB::DestinationColor          => GB::DestinationColor,
                CB::OneMinusDestinationColor  => GB::OneMinusDestinationColor,
                CB::SourceAlpha               => GB::SourceAlpha,
                CB::OneMinusSourceAlpha       => GB::OneMinusSourceAlpha,
                CB::DestinationAlpha          => GB::DestinationAlpha,
                CB::OneMinusDestinationAlpha  => GB::OneMinusDestinationAlpha,
                CB::SourceAlphaSaturate       => GB::SourceAlphaSaturate,
                CB::ConstantColor             => GB::ConstantColor,
                CB::OneMinusConstantColor     => GB::OneMinusConstantColor,
                CB::ConstantAlpha             => GB::ConstantAlpha,
                CB::OneMinusConstantAlpha     => GB::OneMinusConstantAlpha,
            }
        }

        match function {
            CF::AlwaysReplace                               => GF::AlwaysReplace,
            CF::Min                                         => GF::Min,
            CF::Max                                         => GF::Max,
            CF::Addition { source, destination }            => GF::Addition { source: blendfactor(source), destination: blendfactor(destination) },
            CF::Subtraction { source, destination }         => GF::Subtraction { source: blendfactor(source), destination: blendfactor(destination) },
            CF::ReverseSubtraction { source, destination }  => GF::Subtraction { source: blendfactor(source), destination: blendfactor(destination) },
        }
    }

    glium::Blend {
        color: blendfunc(blendmode.color),
        alpha: blendfunc(blendmode.alpha),
        constant_value: blendmode.constant_value.into(),
    }
}

// Converts copy/blit operations coordinate rectangles to gliums rectangle format.
fn blit_coords(source_rect: core::Rect<i32>, source_height: u32, target_rect: core::Rect<i32>, target_height: u32) -> (glium::Rect, glium::BlitTarget) {
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

// Converts texture filter to glium magnify filter.
fn magnify_filter(filter: core::TextureFilter) -> glium::uniforms::MagnifySamplerFilter {
    if filter == core::TextureFilter::Linear {
        glium::uniforms::MagnifySamplerFilter::Linear
    } else {
        glium::uniforms::MagnifySamplerFilter::Nearest
    }
}
