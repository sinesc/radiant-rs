use glium;
use glium::{DisplayBuild, Surface};
use glium::glutin::WindowBuilder;
use prelude::*;
use core::input::{InputData, InputState, NUM_KEYS, NUM_BUTTONS};
use core::monitor;
use core::{AsRenderTarget, RenderTarget, Texture, Rect, Color, texture, TextureFilter};
use maths::Point2;
use core::builder::*;
use backend::glium as backend;
use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{Program, DrawParameters, DrawError};

/// A target to render to, e.g. a window or full screen.
#[derive(Clone)]
pub struct Display {
    handle: glium::Display,
    frame: Rc<RefCell<Option<glium::Frame>>>,
    input_data: Arc<RwLock<InputData>>,
}

#[allow(deprecated)]
impl Display {

    /// Returns a [display builder](support/struct.DisplayBuilder.html) for display construction.
    ///
    /// ```
    /// let display = Display::builder().dimensions((640, 480)).vsync().title("Window!").build();
    /// ```
    pub fn builder() -> DisplayBuilder {
        create_displaybuilder()
    }

    /// Sets the window title.
    pub fn set_title(self: &Self, title: &str) {
        self.window().set_title(title);
    }

    /// Makes the previously hidden window visible.
    pub fn show(self: &Self) {
        self.window().show();
    }

    /// Hides the window.
    pub fn hide(self: &Self) {
        self.window().hide();
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
            let Color(r, g, b, a) = color;
            frame.clear_color(r, g, b, a);
        } else {
            panic!("Failed to prepare a frame for clear.");
        }
    }

    /// Swaps current drawing frame with visible frame.
    pub fn swap_frame(self: &Self) {
        let frame = mem::replace(&mut *self.frame.borrow_mut(), None);
        if let Some(frame) = frame {
            frame.finish().unwrap();
        } else {
            panic!("No frame currently prepared, nothing to swap.");
        }
    }

    /// Enables cursor grab mode. While in this mode, the mouse cursor will be hidden and
    /// constrained to the window. Additionally, [`Input`](struct.Input.html) will be able to
    /// provide mouse movement deltas and allow mouse coordinates to exceed the window-bounds.
    ///
    /// Grab mode will be temporarily released when the window loses focus and automatically
    /// restored once it regains focus.
    pub fn grab_cursor(self: &Self) {
        let window = self.window();
        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = true;
        window.set_cursor_position(100, 100).unwrap();
    }

    /// Hides the mouse cursor while it is inside the window.
    pub fn hide_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Hide).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Releases a previously grabbed or hidden cursor and makes it visible again.
    pub fn free_cursor(self: &Self) {
        self.window().set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
        self.input_data.write().unwrap().cursor_grabbed = false;
    }

    /// Returns the window dimensions.
    pub fn dimensions(self: &Self) -> Point2<u32> {
        self.handle.get_framebuffer_dimensions().into()
    }

    /// Returns monitor details for given monitor id.
    pub fn monitor(index: u32) -> Option<monitor::Monitor> {
        let mut iter = glium::glutin::get_available_monitors();
        let result = iter.nth(index as usize);
        if result.is_some() {
            Some(monitor::from_id(result.unwrap()))
        } else {
            None
        }
    }

    /// Returns a vector of available monitors.
    pub fn monitors() -> Vec<monitor::Monitor> {
        let iter = glium::glutin::get_available_monitors();
        let mut result = Vec::<monitor::Monitor>::new();
        for monitor in iter {
            result.push(monitor::from_id(monitor));
        }
        result
    }

    /// Takes a glium::Display and returns a radiant::Display.
    pub fn from_glium(display: glium::Display) -> Display {
        Display {
            handle: display,
            frame: Rc::new(RefCell::new(None)),
            input_data: Arc::new(RwLock::new(InputData::new())),
        }
    }

    /// Polls for events like keyboard or mouse input and changes to the window. See
    /// [`Input`](struct.Input.html) for basic keyboard and mouse support.
    pub fn poll_events(self: &Self) -> &Self {
        let mut input_data = self.input_data.write().unwrap();

        // !todo poll_id, check if released/pressed(poll_id) == poll_id
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

        for event in backend::poll_events(&self.handle) {
            match event {
                backend::Event::KeyboardInput(key_id, down) => {
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
                backend::Event::MouseMoved(x, y) => {
                    if input_data.cursor_grabbed {
                        let center = ((input_data.dimensions.0 / 2) as i32, (input_data.dimensions.1 / 2) as i32);
                        let old_mouse = input_data.mouse;
                        let delta = (x - center.0, y - center.1);
                        input_data.mouse = (old_mouse.0 + delta.0, old_mouse.1 + delta.1);
                        input_data.mouse_delta = delta;
                        let window = self.window();
                        window.set_cursor_position(center.0, center.1).unwrap();
                    } else {
                        input_data.mouse = (x, y);
                    }
                },
                backend::Event::MouseInput(button_id, down) => {
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
                backend::Event::Focused => {
                    // restore grab after focus loss
                    if input_data.cursor_grabbed {
                        let window = self.window();
                        window.set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
                        window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
                    }
                }
                backend::Event::Closed => {
                    input_data.should_close = true;
                }
            }
        }

        input_data.dimensions = self.window().get_inner_size_pixels().unwrap_or((0, 0));

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
    #[deprecated(since="0.5", note="Use Display::builder() instead.")]
    #[allow(deprecated)]
    pub fn new(descriptor: DisplayInfo) -> Display {

        let mut builder = WindowBuilder::new()
            .with_dimensions(descriptor.width, descriptor.height)
            .with_title(descriptor.title)
            .with_transparency(descriptor.transparent)
            .with_decorations(descriptor.decorations);

        if descriptor.monitor >= 0 {
            let monitor = Self::monitor(descriptor.monitor as u32);
            if monitor.is_some() {
                builder = builder.with_fullscreen(monitor::get_id(monitor.unwrap()));
            }
            // !todo error
        }
        if descriptor.vsync {
            builder = builder.with_vsync();
        }

        Display {
            handle: builder.build_glium().unwrap(),
            frame: Rc::new(RefCell::new(None)),
            input_data: Arc::new(RwLock::new(InputData::new())),
        }
    }

    /// returns a reference to the underlying glutin window
    fn window(self: &Self) -> glium::backend::glutin_backend::WinRef {
        self.handle.get_window().unwrap()
    }
}

/// Returns the glium display handle
pub fn handle(display: &Display) -> &glium::Display {
    &display.handle
}

/// Returns an RwLocked reference to the input data.
pub fn input_data(display: &Display) -> &Arc<RwLock<InputData>> {
    &display.input_data
}

/// Draws to the display.
pub fn draw<'b, 'v, V, I, U>(display: &Display, vb: V, ib: I, program: &Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<(), DrawError>
    where I: Into<IndicesSource<'b>>, U: Uniforms, V: MultiVerticesSource<'v>
{
    display.frame.borrow_mut().as_mut().unwrap().draw(vb, ib, program, uniforms, draw_parameters)
}

/// Copies given texture to given display.
pub fn copy_from_texture(display: &Display, source: &Texture, filter: TextureFilter) {
    texture::handle(source).as_surface().fill(display.frame.borrow().as_ref().unwrap(), backend::magnify_filter(filter));
}

/// Copies given display to given texture.
pub fn copy_to_texture(display: &Display, target: &Texture, filter: TextureFilter) {
    display.frame.borrow().as_ref().unwrap().fill(&texture::handle(target).as_surface(), backend::magnify_filter(filter));
}

/// Copies the source rectangle to the target rectangle on the given display.
pub fn copy_rect(display: &Display, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: TextureFilter) {
    let height = display.dimensions().1;
    let (glium_src_rect, glium_target_rect) = backend::blit_coords(source_rect, height, target_rect, height);
    display.frame.borrow().as_ref().unwrap().blit_color(&glium_src_rect, display.frame.borrow().as_ref().unwrap(), &glium_target_rect, backend::magnify_filter(filter));
}

/// Copies the source rectangle from the given texture to the target rectangle on the given display.
pub fn copy_rect_from_texture(display: &Display, source: &Texture, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: TextureFilter) {
    let target_height = display.dimensions().1;
    let source_height = texture::handle(source).as_surface().get_dimensions().1;
    let (glium_src_rect, glium_target_rect) = backend::blit_coords(source_rect, source_height, target_rect, target_height);
    texture::handle(source).as_surface().blit_color(&glium_src_rect, display.frame.borrow().as_ref().unwrap(), &glium_target_rect, backend::magnify_filter(filter));
}

/// Copies the source rectangle from the given display to the target rectangle on the given texture.
pub fn copy_rect_to_texture(display: &Display, target: &Texture, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: TextureFilter) {
    let source_height = display.dimensions().1;
    let target_height = texture::handle(target).as_surface().get_dimensions().1;
    let (glium_src_rect, glium_target_rect) = backend::blit_coords(source_rect, source_height, target_rect, target_height);
    display.frame.borrow().as_ref().unwrap().blit_color(&glium_src_rect, &texture::handle(target).as_surface(), &glium_target_rect, backend::magnify_filter(filter));
}

/// Clears the display with given color without swapping buffers.
pub fn clear(display: &Display, color: Color) {
    if let Some(ref mut frame) = display.frame.borrow_mut().as_mut() {
        let Color(r, g, b, a) = color;
        frame.clear_color(r, g, b, a);
    } else {
        panic!("Failed to clear frame: None prepared.");
    }
}

impl AsRenderTarget for Display {
    fn as_render_target(self: &Self) -> RenderTarget {
        RenderTarget::Display(self.clone())
    }
}

/// A struct describing a [`Display`](struct.Display.html) to be created.
#[derive(Clone)]
#[deprecated(since="0.5", note="See Display::builder() instead.")]
#[allow(deprecated)]
pub struct DisplayInfo {
    pub width       : u32,
    pub height      : u32,
    pub title       : String,
    pub transparent : bool,
    pub decorations : bool,
    pub monitor     : i32,
    pub vsync       : bool,
}

#[allow(deprecated)]
impl Default for DisplayInfo {
    fn default() -> DisplayInfo {
        DisplayInfo {
            width       : 640,
            height      : 480,
            title       : "".to_string(),
            transparent : false,
            decorations : true,
            monitor     : -1,
            vsync       : false,
        }
   }
}
