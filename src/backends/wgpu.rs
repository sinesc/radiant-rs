use crate::prelude::*;
use crate::core::{Vertex, Mat4Trait};
use wgpu;
use winit;
use std::sync::OnceLock;

// Newtype that asserts Send+Sync for the event loop, which is safe because we always
// build it with with_any_thread(true) on Linux (the only non-macOS platform we target).
#[cfg(not(target_os = "macos"))]
struct SendableEventLoop(winit::event_loop::EventLoop<()>);
#[cfg(not(target_os = "macos"))]
unsafe impl Send for SendableEventLoop {}
#[cfg(not(target_os = "macos"))]
unsafe impl Sync for SendableEventLoop {}

// One event loop per process; created on first Display::new, reused by subsequent ones.
#[cfg(not(target_os = "macos"))]
static GLOBAL_EVENT_LOOP: OnceLock<Mutex<SendableEventLoop>> = OnceLock::new();

// Per-window event queues. Filled during pump_app_events; drained by each Display::poll_events.
#[cfg(not(target_os = "macos"))]
static WINDOW_EVENTS: OnceLock<Mutex<HashMap<winit::window::WindowId, Vec<crate::core::Event>>>> = OnceLock::new();

/// GPU resources that can be shared across displays that use the same context.
pub(crate) struct SharedGpu {
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

const MAX_BUFFERS: usize = 10;
const VERTEX_STRIDE: u64 = std::mem::size_of::<Vertex>() as u64;

// Byte offset of texture_uv within a repr(C) Vertex struct.
// Computed from field sizes: position(8) + offset(8) + rotation(4) + color(16) + bucket_id(4) + texture_id(4) = 44
const TEXTURE_UV_OFFSET: u64 = 44;

/// Maximum number of user-provided `Texture` uniforms bound to a texture program.
const MAX_USER_TEXTURES: usize = 8;
/// Group-0 binding of the first user-provided `Texture` uniform.
const USER_TEXTURE_BASE_BINDING: u32 = 3;

// --------------
// Public interface
// --------------

pub mod public {}

// --------------
// Error
// --------------

#[derive(Debug)]
pub enum Error {
    OsError(String),
    Incompatible(String),
    NotAvailable(String),
    WindowCreation(String),
    WgpuError(String),
    Unknown,
}

impl From<wgpu::CreateSurfaceError> for crate::core::Error {
    fn from(error: wgpu::CreateSurfaceError) -> crate::core::Error {
        crate::core::Error::BackendError(Error::WindowCreation(format!("Surface creation failed: {:?}", error)))
    }
}

// --------------
// Blend mode pipeline cache key
// --------------

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct BlendStateKey(u8, u8, u8, u8, u8, u8);

// --------------
// Blit resources (texture → frame copy, REPLACE blend, cached in Display)
// --------------

struct BlitResources {
    pipeline: Arc<wgpu::RenderPipeline>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
    nearest_sampler: Arc<wgpu::Sampler>,
    linear_sampler: Arc<wgpu::Sampler>,
    /// Fills unused user texture pool bindings.
    placeholder_view: Arc<wgpu::TextureView>,
}

impl BlitResources {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Arc<Self> {
        let vert = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Vert"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/texture.wgsl").into()),
        });
        let frag = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Frag"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/texture_frag.wgsl").into()),
        });
        let bind_group_layout = Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit BGL"),
            entries: &Context::texture_bgl_entries(),
        }));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Layout"),
            bind_group_layouts: &[Some(&*bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = Arc::new(Context::build_texture_pipeline(
            device, &vert, &frag, &layout, format, wgpu::BlendState::REPLACE,
        ));
        let nearest_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Nearest"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        }));
        let linear_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        }));
        let placeholder_view = Context::create_placeholder_view(device, queue);
        Arc::new(BlitResources { pipeline, bind_group_layout, nearest_sampler, linear_sampler, placeholder_view })
    }
}

impl From<&wgpu::BlendState> for BlendStateKey {
    fn from(bs: &wgpu::BlendState) -> Self {
        BlendStateKey(
            bs.color.src_factor as u8,
            bs.color.dst_factor as u8,
            bs.color.operation as u8,
            bs.alpha.src_factor as u8,
            bs.alpha.dst_factor as u8,
            bs.alpha.operation as u8,
        )
    }
}

// --------------
// Event handling (pump_app_events, not available on macOS)
// --------------

#[cfg(not(target_os = "macos"))]
// Collects events for all windows during a single pump_app_events call.
struct EventCollector {
    window_events: Vec<(winit::window::WindowId, crate::core::Event)>,
    // MouseDelta is a device event with no window — delivered to all registered windows.
    mouse_delta: Option<(i32, i32)>,
}

#[cfg(not(target_os = "macos"))]
impl winit::application::ApplicationHandler for EventCollector {
    fn resumed(&mut self, _: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::{WindowEvent, ElementState};
        use winit::keyboard::PhysicalKey;
        use crate::core::Event;
        let ev = match event {
            WindowEvent::CloseRequested => Some(Event::Close),
            WindowEvent::Focused(focused) => Some(if focused { Event::Focus } else { Event::Blur }),
            WindowEvent::CursorMoved { position, .. } => {
                Some(Event::MousePosition(position.x as i32, position.y as i32))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                mouse_button_to_id(button)
                    .map(|id| Event::MouseInput(id, state == ElementState::Pressed))
            }
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    physical_key: PhysicalKey::Code(code), state, ..
                }, ..
            } => {
                keycode_to_id(code)
                    .map(|id| Event::KeyboardInput(id, state == ElementState::Pressed))
            }
            _ => None,
        };
        if let Some(ev) = ev {
            self.window_events.push((window_id, ev));
        }
    }

    fn device_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta = Some((delta.0 as i32, delta.1 as i32));
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn mouse_button_to_id(button: winit::event::MouseButton) -> Option<usize> {
    let id = match button {
        winit::event::MouseButton::Left    => 0,
        winit::event::MouseButton::Right   => 1,
        winit::event::MouseButton::Middle  => 2,
        winit::event::MouseButton::Back    => 3,
        winit::event::MouseButton::Forward => 4,
        winit::event::MouseButton::Other(n) => {
            let id = n as usize + 5;
            if id < crate::core::NUM_BUTTONS { id } else { return None; }
        }
    };
    if id < crate::core::NUM_BUTTONS { Some(id) } else { None }
}

#[cfg(not(target_os = "macos"))]
fn keycode_to_id(code: winit::keyboard::KeyCode) -> Option<usize> {
    use winit::keyboard::KeyCode;
    use crate::core::InputId;
    let id = match code {
        // Digits row
        KeyCode::Digit1        => InputId::Key1 as usize,
        KeyCode::Digit2        => InputId::Key2 as usize,
        KeyCode::Digit3        => InputId::Key3 as usize,
        KeyCode::Digit4        => InputId::Key4 as usize,
        KeyCode::Digit5        => InputId::Key5 as usize,
        KeyCode::Digit6        => InputId::Key6 as usize,
        KeyCode::Digit7        => InputId::Key7 as usize,
        KeyCode::Digit8        => InputId::Key8 as usize,
        KeyCode::Digit9        => InputId::Key9 as usize,
        KeyCode::Digit0        => InputId::Key0 as usize,
        // Letters
        KeyCode::KeyA          => InputId::A as usize,
        KeyCode::KeyB          => InputId::B as usize,
        KeyCode::KeyC          => InputId::C as usize,
        KeyCode::KeyD          => InputId::D as usize,
        KeyCode::KeyE          => InputId::E as usize,
        KeyCode::KeyF          => InputId::F as usize,
        KeyCode::KeyG          => InputId::G as usize,
        KeyCode::KeyH          => InputId::H as usize,
        KeyCode::KeyI          => InputId::I as usize,
        KeyCode::KeyJ          => InputId::J as usize,
        KeyCode::KeyK          => InputId::K as usize,
        KeyCode::KeyL          => InputId::L as usize,
        KeyCode::KeyM          => InputId::M as usize,
        KeyCode::KeyN          => InputId::N as usize,
        KeyCode::KeyO          => InputId::O as usize,
        KeyCode::KeyP          => InputId::P as usize,
        KeyCode::KeyQ          => InputId::Q as usize,
        KeyCode::KeyR          => InputId::R as usize,
        KeyCode::KeyS          => InputId::S as usize,
        KeyCode::KeyT          => InputId::T as usize,
        KeyCode::KeyU          => InputId::U as usize,
        KeyCode::KeyV          => InputId::V as usize,
        KeyCode::KeyW          => InputId::W as usize,
        KeyCode::KeyX          => InputId::X as usize,
        KeyCode::KeyY          => InputId::Y as usize,
        KeyCode::KeyZ          => InputId::Z as usize,
        // Control keys
        KeyCode::Escape        => InputId::Escape as usize,
        KeyCode::F1            => InputId::F1 as usize,
        KeyCode::F2            => InputId::F2 as usize,
        KeyCode::F3            => InputId::F3 as usize,
        KeyCode::F4            => InputId::F4 as usize,
        KeyCode::F5            => InputId::F5 as usize,
        KeyCode::F6            => InputId::F6 as usize,
        KeyCode::F7            => InputId::F7 as usize,
        KeyCode::F8            => InputId::F8 as usize,
        KeyCode::F9            => InputId::F9 as usize,
        KeyCode::F10           => InputId::F10 as usize,
        KeyCode::F11           => InputId::F11 as usize,
        KeyCode::F12           => InputId::F12 as usize,
        KeyCode::F13           => InputId::F13 as usize,
        KeyCode::F14           => InputId::F14 as usize,
        KeyCode::F15           => InputId::F15 as usize,
        KeyCode::PrintScreen   => InputId::Snapshot as usize,
        KeyCode::ScrollLock    => InputId::Scroll as usize,
        KeyCode::Pause         => InputId::Pause as usize,
        KeyCode::Insert        => InputId::Insert as usize,
        KeyCode::Home          => InputId::Home as usize,
        KeyCode::Delete        => InputId::Delete as usize,
        KeyCode::End           => InputId::End as usize,
        KeyCode::PageDown      => InputId::PageDown as usize,
        KeyCode::PageUp        => InputId::PageUp as usize,
        KeyCode::ArrowLeft     => InputId::CursorLeft as usize,
        KeyCode::ArrowUp       => InputId::CursorUp as usize,
        KeyCode::ArrowRight    => InputId::CursorRight as usize,
        KeyCode::ArrowDown     => InputId::CursorDown as usize,
        KeyCode::Backspace     => InputId::Backspace as usize,
        KeyCode::Enter         => InputId::Return as usize,
        KeyCode::Space         => InputId::Space as usize,
        // Numpad
        KeyCode::NumLock       => InputId::Numlock as usize,
        KeyCode::Numpad0       => InputId::Numpad0 as usize,
        KeyCode::Numpad1       => InputId::Numpad1 as usize,
        KeyCode::Numpad2       => InputId::Numpad2 as usize,
        KeyCode::Numpad3       => InputId::Numpad3 as usize,
        KeyCode::Numpad4       => InputId::Numpad4 as usize,
        KeyCode::Numpad5       => InputId::Numpad5 as usize,
        KeyCode::Numpad6       => InputId::Numpad6 as usize,
        KeyCode::Numpad7       => InputId::Numpad7 as usize,
        KeyCode::Numpad8       => InputId::Numpad8 as usize,
        KeyCode::Numpad9       => InputId::Numpad9 as usize,
        KeyCode::NumpadAdd     => InputId::Add as usize,
        KeyCode::NumpadDecimal => InputId::Decimal as usize,
        KeyCode::NumpadDivide  => InputId::Divide as usize,
        KeyCode::NumpadMultiply=> InputId::Multiply as usize,
        KeyCode::NumpadSubtract=> InputId::Subtract as usize,
        KeyCode::NumpadEnter   => InputId::NumpadEnter as usize,
        KeyCode::NumpadEqual   => InputId::NumpadEquals as usize,
        KeyCode::NumpadComma   => InputId::NumpadComma as usize,
        // Punctuation & symbols
        KeyCode::Quote         => InputId::Apostrophe as usize,
        KeyCode::Backslash     => InputId::Backslash as usize,
        KeyCode::CapsLock      => InputId::Capital as usize,
        KeyCode::Comma         => InputId::Comma as usize,
        KeyCode::Convert       => InputId::Convert as usize,
        KeyCode::Equal         => InputId::Equals as usize,
        KeyCode::Backquote     => InputId::Grave as usize,
        KeyCode::KanaMode      => InputId::Kana as usize,
        KeyCode::AltLeft       => InputId::LAlt as usize,
        KeyCode::BracketLeft   => InputId::LBracket as usize,
        KeyCode::ControlLeft   => InputId::LControl as usize,
        KeyCode::ShiftLeft     => InputId::LShift as usize,
        KeyCode::SuperLeft     => InputId::LWin as usize,
        KeyCode::LaunchMail    => InputId::Mail as usize,
        KeyCode::MediaSelect   => InputId::MediaSelect as usize,
        KeyCode::MediaStop     => InputId::MediaStop as usize,
        KeyCode::Minus         => InputId::Minus as usize,
        KeyCode::AudioVolumeMute    => InputId::Mute as usize,
        KeyCode::MediaTrackNext     => InputId::NextTrack as usize,
        KeyCode::NonConvert         => InputId::NoConvert as usize,
        KeyCode::IntlBackslash      => InputId::OEM102 as usize,
        KeyCode::Period             => InputId::Period as usize,
        KeyCode::MediaPlayPause     => InputId::PlayPause as usize,
        KeyCode::Power              => InputId::Power as usize,
        KeyCode::MediaTrackPrevious => InputId::PrevTrack as usize,
        KeyCode::AltRight           => InputId::RAlt as usize,
        KeyCode::BracketRight       => InputId::RBracket as usize,
        KeyCode::ControlRight       => InputId::RControl as usize,
        KeyCode::ShiftRight         => InputId::RShift as usize,
        KeyCode::SuperRight         => InputId::RWin as usize,
        KeyCode::Semicolon          => InputId::Semicolon as usize,
        KeyCode::Slash              => InputId::Slash as usize,
        KeyCode::Sleep              => InputId::Sleep as usize,
        KeyCode::Tab                => InputId::Tab as usize,
        KeyCode::AudioVolumeDown    => InputId::VolumeDown as usize,
        KeyCode::AudioVolumeUp      => InputId::VolumeUp as usize,
        KeyCode::WakeUp             => InputId::Wake as usize,
        KeyCode::BrowserBack        => InputId::WebBack as usize,
        KeyCode::BrowserFavorites   => InputId::WebFavorites as usize,
        KeyCode::BrowserForward     => InputId::WebForward as usize,
        KeyCode::BrowserHome        => InputId::WebHome as usize,
        KeyCode::BrowserRefresh     => InputId::WebRefresh as usize,
        KeyCode::BrowserSearch      => InputId::WebSearch as usize,
        KeyCode::BrowserStop        => InputId::WebStop as usize,
        KeyCode::IntlYen            => InputId::Yen as usize,
        KeyCode::ContextMenu        => InputId::Apps as usize,
        _ => return None,
    };
    Some(id)
}

// --------------
// Display
// --------------

#[derive(Clone)]
pub struct Display {
    inner: Arc<DisplayInner>,
}

struct DisplayInner {
    window: Arc<winit::window::Window>,
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: Mutex<wgpu::SurfaceConfiguration>,
    blit_resources: Arc<BlitResources>,
}

impl Display {
    pub fn new(descriptor: crate::core::DisplayBuilder, shared: Option<SharedGpu>) -> crate::core::Result<Display> {
        pollster::block_on(Self::create_async(descriptor, shared))
    }

    async fn create_async(descriptor: crate::core::DisplayBuilder, shared: Option<SharedGpu>) -> crate::core::Result<Display> {

        // Ensure the global event loop exists; create it on the first call.
        #[cfg(not(target_os = "macos"))]
        if GLOBAL_EVENT_LOOP.get().is_none() {
            let el_result = {
                #[cfg(target_os = "linux")]
                {
                    use winit::platform::x11::EventLoopBuilderExtX11;
                    winit::event_loop::EventLoop::builder()
                        .with_any_thread(true)
                        .build()
                }
                #[cfg(not(target_os = "linux"))]
                {
                    winit::event_loop::EventLoop::builder().build()
                }
            };
            match el_result {
                Ok(el) => { let _ = GLOBAL_EVENT_LOOP.set(Mutex::new(SendableEventLoop(el))); }
                Err(_) if GLOBAL_EVENT_LOOP.get().is_some() => {} // concurrent init won
                Err(e) => return Err(crate::core::Error::BackendError(Error::OsError(
                    format!("Failed to create event loop: {:?}", e)
                ))),
            }
        }

        let window_attributes = winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::PhysicalSize::new(descriptor.width, descriptor.height))
            .with_title(&descriptor.title)
            .with_transparent(descriptor.transparent)
            .with_decorations(descriptor.decorations)
            .with_visible(descriptor.visible)
            .with_resizable(true);

        #[allow(deprecated)]
        let window = {
            #[cfg(not(target_os = "macos"))]
            {
                GLOBAL_EVENT_LOOP.get()
                    .ok_or_else(|| crate::core::Error::BackendError(Error::OsError("Event loop unavailable".to_string())))?
                    .lock().unwrap().0
                    .create_window(window_attributes)
                    .map_err(|e| crate::core::Error::BackendError(Error::WindowCreation(format!("Failed to create window: {:?}", e))))?
            }
            #[cfg(target_os = "macos")]
            {
                return Err(crate::core::Error::BackendError(Error::NotAvailable(
                    "Multi-window / non-main-thread display creation not supported on macOS".to_string()
                )));
            }
        };
        let window = Arc::new(window);

        // Reuse GPU resources from shared context, or create them for the first display.
        let (instance, adapter, device, queue) = if let Some(s) = shared {
            (s.instance, s.adapter, s.device, s.queue)
        } else {
            let instance = Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            }));

            // Create a temporary surface just for adapter selection.
            let tmp_surface = instance.create_surface(window.clone()).map_err(crate::core::Error::from)?;

            let adapter = Arc::new(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: if descriptor.vsync {
                    wgpu::PowerPreference::HighPerformance
                } else {
                    wgpu::PowerPreference::LowPower
                },
                force_fallback_adapter: false,
                compatible_surface: Some(&tmp_surface),
            }).await.map_err(|_| {
                crate::core::Error::BackendError(Error::NotAvailable("No suitable GPU adapter found".to_string()))
            })?);

            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Radiant Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            }).await.map_err(|e| {
                crate::core::Error::BackendError(Error::WgpuError(format!("Failed to request device: {:?}", e)))
            })?;

            (instance, adapter, Arc::new(device), Arc::new(queue))
        };

        let surface = instance.create_surface(window.clone()).map_err(crate::core::Error::from)?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&*adapter);

        let present_mode = if descriptor.vsync {
            if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            }
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::FifoRelaxed
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&*device, &config);

        let blit_resources = BlitResources::new(&device, &queue, config.format);

        let inner = Arc::new(DisplayInner {
            window,
            instance,
            adapter,
            surface,
            device,
            queue,
            config: Mutex::new(config),
            blit_resources,
        });

        Ok(Display { inner })
    }

    pub fn draw(self: &Self) -> Frame {
        let frame = loop {
            match self.inner.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => break frame,
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    let size = self.inner.window.inner_size();
                    let mut config = self.inner.config.lock().unwrap();
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    self.inner.surface.configure(&self.inner.device, &config);
                }
                e => panic!("Failed to get swapchain frame: {:?}", e),
            }
        };
        let view = Arc::new(frame.texture.create_view(&wgpu::TextureViewDescriptor::default()));

        let command_encoder = self.inner.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Radiant Command Encoder") }
        );

        Frame {
            view,
            command_encoder,
            surface_texture: frame,
            clear_color: None,
            first_pass: true,
            device: self.inner.device.clone(),
            queue: self.inner.queue.clone(),
            blit_resources: self.inner.blit_resources.clone(),
        }
    }

    pub fn framebuffer_dimensions(self: &Self) -> crate::core::Point2<u32> {
        let size = self.inner.window.inner_size();
        (size.width, size.height)
    }

    pub fn window_dimensions(self: &Self) -> crate::core::Point2<u32> {
        let size = self.inner.window.inner_size();
        (size.width, size.height)
    }

    pub fn set_cursor_position(self: &Self, position: crate::core::Point2<i32>) {
        let _ = self.inner.window.set_cursor_position(winit::dpi::PhysicalPosition::new(position.0 as f64, position.1 as f64));
    }

    pub fn set_cursor_state(self: &Self, state: crate::core::CursorState) {
        use crate::core::CursorState as CS;
        use winit::window::CursorGrabMode;
        match state {
            CS::Normal => {
                self.inner.window.set_cursor_grab(CursorGrabMode::None).ok();
                self.inner.window.set_cursor_visible(true);
            }
            CS::Hide => {
                self.inner.window.set_cursor_visible(false);
            }
            CS::Grab => {
                // Locked keeps the cursor pinned so deltas are unbounded; Confined is the fallback
                // for compositors that don't support locking (cursor stops at window edge).
                if self.inner.window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
                    self.inner.window.set_cursor_grab(CursorGrabMode::Confined).ok();
                }
                self.inner.window.set_cursor_visible(false);
            }
        }
    }

    pub fn set_fullscreen(self: &Self, monitor: Option<crate::core::Monitor>) -> bool {
        if let Some(monitor) = monitor {
            self.inner.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor.inner.inner.winit_monitor))));
        } else {
            self.inner.window.set_fullscreen(None);
        }
        true
    }

    pub fn poll_events<F>(self: &Self, mut callback: F) where F: FnMut(crate::core::Event) {
        #[cfg(not(target_os = "macos"))]
        {
            use winit::platform::pump_events::EventLoopExtPumpEvents;

            // Pump the event loop and collect events for every window. Releasing the
            // event-loop lock before touching WINDOW_EVENTS avoids any lock-order issues.
            let collected = GLOBAL_EVENT_LOOP.get().and_then(|el_mutex| {
                el_mutex.lock().ok().map(|mut el| {
                    let mut collector = EventCollector {
                        window_events: Vec::new(),
                        mouse_delta: None,
                    };
                    el.0.pump_app_events(Some(std::time::Duration::ZERO), &mut collector);
                    collector
                })
            });

            // Distribute the just-collected events into the per-window queues.
            let queues = WINDOW_EVENTS.get_or_init(|| Mutex::new(HashMap::new()));
            if let Some(collector) = collected {
                if let Ok(mut map) = queues.lock() {
                    for (win_id, event) in collector.window_events {
                        map.entry(win_id).or_default().push(event);
                    }
                    if let Some((dx, dy)) = collector.mouse_delta {
                        for events in map.values_mut() {
                            events.push(crate::core::Event::MouseDelta(dx, dy));
                        }
                    }
                }
            }

            // Drain this window's events and forward them to the caller.
            if let Ok(mut map) = queues.lock() {
                if let Some(events) = map.remove(&self.inner.window.id()) {
                    for event in events {
                        callback(event);
                    }
                }
            }
        }
        // macOS: pump_app_events unavailable; poll_events remains a no-op
        #[cfg(target_os = "macos")]
        let _ = callback;
    }

    pub fn show(self: &Self) {
        self.inner.window.set_visible(true);
    }

    pub fn hide(self: &Self) {
        self.inner.window.set_visible(false);
    }

    pub fn set_title(self: &Self, title: &str) {
        self.inner.window.set_title(title);
    }
}

// --------------
// Frame
// --------------

pub struct Frame {
    view: Arc<wgpu::TextureView>,
    command_encoder: wgpu::CommandEncoder,
    surface_texture: wgpu::SurfaceTexture,
    clear_color: Option<[f64; 4]>,
    first_pass: bool,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    blit_resources: Arc<BlitResources>,
}

impl Frame {
    pub fn clear(self: &mut Self, color: crate::core::Color) {
        let crate::core::Color(r, g, b, a) = color;
        self.clear_color = Some([r as f64, g as f64, b as f64, a as f64]);
    }

    pub fn finish(mut self: Self) {
        // If no render pass was issued (e.g. empty frame), apply the pending clear now.
        if self.first_pass {
            if let Some(c) = self.clear_color {
                let encoder = &mut self.command_encoder;
                let view = &*self.view;
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: c[0], g: c[1], b: c[2], a: c[3] }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
        }
        self.queue.submit(std::iter::once(self.command_encoder.finish()));
        self.surface_texture.present();
        // device.poll(Wait) is omitted: it is unnecessary for rendering and panics on
        // device loss (e.g. when the window surface is torn down on close).
    }

    pub fn dimensions(self: &Self) -> crate::core::Point2<u32> {
        let size = self.surface_texture.texture.size();
        (size.width, size.height)
    }

    pub fn copy_from_texture(self: &mut Self, source: &crate::core::Texture, filter: crate::core::TextureFilter) {
        use crate::core::Mat4Trait;
        let uniforms = TextureUniforms {
            u_view: <[[f32; 4]; 4] as Mat4Trait<f32>>::viewport(1.0, 1.0),
            u_model: <[[f32; 4]; 4] as Mat4Trait<f32>>::identity(),
            _rd_color: [1.0, 1.0, 1.0, 1.0],
            _rd_offset: [0.0, 0.0],
            _rd_dimensions: [1.0, 1.0],
            _rd_flags: [0.0, 0.0, 0.0, 0.0],
        };
        let load_op = self.next_load_op();
        let view = self.view.clone();
        let blit = self.blit_resources.clone();
        let sampler = Context::sampler_for_filter(&blit.nearest_sampler, &blit.linear_sampler, filter);
        render_texture_quad(
            &mut self.command_encoder,
            &view,
            &self.device,
            &self.queue,
            &uniforms,
            &BLIT_QUAD,
            &*source.handle.view,
            sampler,
            &[],
            &blit.placeholder_view,
            &blit.pipeline,
            &blit.bind_group_layout,
            load_op,
        );
    }

    pub fn copy_rect(self: &mut Self, _source_rect: crate::core::Rect<i32>, _target_rect: crate::core::Rect<i32>, _filter: crate::core::TextureFilter) { /*TODO*/ }

    pub fn copy_rect_from_texture(self: &mut Self, source: &crate::core::Texture, source_rect: crate::core::Rect<i32>, target_rect: crate::core::Rect<i32>, filter: crate::core::TextureFilter) {
        use crate::core::Mat4Trait;

        let tw = source.handle.width as f32;
        let th = source.handle.height as f32;
        let frame_size = self.surface_texture.texture.size();
        let fw = frame_size.width as f32;
        let fh = frame_size.height as f32;

        // Source UV rect
        let sx = (source_rect.0).0 as f32;
        let sy = (source_rect.0).1 as f32;
        let sw = (source_rect.1).0 as f32;
        let sh = (source_rect.1).1 as f32;
        let u0 = sx / tw;
        let u1 = (sx + sw) / tw;
        let v0 = sy / th;
        let v1 = (sy + sh) / th;

        // Destination rect in [0,1] normalized frame space
        let dx = (target_rect.0).0 as f32;
        let dy = (target_rect.0).1 as f32;
        let dw = (target_rect.1).0 as f32;
        let dh = (target_rect.1).1 as f32;

        let quad = [
            Vertex { position: [0.0, 0.0], texture_uv: [u0, v0], ..Vertex::default() },
            Vertex { position: [1.0, 0.0], texture_uv: [u1, v0], ..Vertex::default() },
            Vertex { position: [0.0, 1.0], texture_uv: [u0, v1], ..Vertex::default() },
            Vertex { position: [1.0, 1.0], texture_uv: [u1, v1], ..Vertex::default() },
        ];
        let uniforms = TextureUniforms {
            u_view: <[[f32; 4]; 4] as Mat4Trait<f32>>::viewport(1.0, 1.0),
            u_model: <[[f32; 4]; 4] as Mat4Trait<f32>>::identity(),
            _rd_color: [1.0, 1.0, 1.0, 1.0],
            _rd_offset: [dx / fw, dy / fh],
            _rd_dimensions: [dw / fw, dh / fh],
            _rd_flags: [0.0, 0.0, 0.0, 0.0],
        };
        let load_op = self.next_load_op();
        let view = self.view.clone();
        let blit = self.blit_resources.clone();
        let sampler = Context::sampler_for_filter(&blit.nearest_sampler, &blit.linear_sampler, filter);
        render_texture_quad(
            &mut self.command_encoder,
            &view,
            &self.device,
            &self.queue,
            &uniforms,
            &quad,
            &*source.handle.view,
            sampler,
            &[],
            &blit.placeholder_view,
            &blit.pipeline,
            &blit.bind_group_layout,
            load_op,
        );
    }

    // Returns the LoadOp for the next render pass and clears the first_pass flag.
    fn next_load_op(&mut self) -> wgpu::LoadOp<wgpu::Color> {
        if self.first_pass {
            self.first_pass = false;
            if let Some(c) = self.clear_color {
                return wgpu::LoadOp::Clear(wgpu::Color { r: c[0], g: c[1], b: c[2], a: c[3] });
            }
        }
        wgpu::LoadOp::Load
    }

    /// Draw sprites to this frame's surface.
    fn draw_sprites(
        &mut self,
        vertices: &[Vertex],
        dirty: bool,
        layer_hint: usize,
        component: u32,
        blend: wgpu::BlendState,
        font_texture_view: &wgpu::TextureView,
        tex_arrays: &[crate::core::RawFrameArray],
        sprite_uniforms: &SpriteUniforms,
        context: &mut Context,
        custom_program: Option<&Program>,
    ) {
        let num_vertices = vertices.len();
        let num_sprites = num_vertices / 4;
        if num_vertices == 0 {
            return;
        }

        let (vb_index, vb_dirty) = context.select_vertex_buffer(layer_hint, num_vertices);

        if dirty || vb_dirty {
            let vertex_bytes = unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    num_vertices * std::mem::size_of::<Vertex>(),
                )
            };
            self.queue.write_buffer(&context.vertex_buffers[vb_index].buffer, 0, vertex_bytes);
        }

        context.update_index_buffer(num_sprites);

        let uniform_bytes = unsafe {
            std::slice::from_raw_parts(
                sprite_uniforms as *const SpriteUniforms as *const u8,
                std::mem::size_of::<SpriteUniforms>(),
            )
        };
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Uniform Buffer"),
            size: uniform_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&uniform_buffer, 0, uniform_bytes);

        // _rd_comp: u32 padded to 16 bytes for uniform buffer alignment
        let comp_data = [component, 0u32, 0u32, 0u32];
        let comp_bytes = unsafe {
            std::slice::from_raw_parts(comp_data.as_ptr() as *const u8, 16)
        };
        let comp_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Component Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&comp_buffer, 0, comp_bytes);

        let sprite_bgl = custom_program.map_or(&context.sprite_bind_group_layout, |p| &*p.bind_group_layout);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Bind Group"),
            layout: sprite_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &uniform_buffer, offset: 0, size: None }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(font_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&context.nearest_sampler),
                },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&tex_arrays[1].data.view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&tex_arrays[2].data.view) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&tex_arrays[3].data.view) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&tex_arrays[4].data.view) },
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&tex_arrays[5].data.view) },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &comp_buffer, offset: 0, size: None }),
                },
            ],
        });

        let pipeline = match custom_program {
            Some(prog) => prog.get_or_create_pipeline(blend, context.format),
            None => context.get_or_create_sprite_pipeline(blend, context.format),
        };
        let load_op = self.next_load_op();
        let view = self.view.clone();

        let mut render_pass = self.command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Radiant Sprite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: load_op, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_blend_constant(wgpu::Color::TRANSPARENT);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, context.vertex_buffers[vb_index].buffer.slice(..));
        render_pass.set_index_buffer(context.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(num_sprites as u32 * 6), 0, 0..1);
    }

    /// Draw a textured quad to this frame.
    fn draw_quad(
        &mut self,
        uniforms: &TextureUniforms,
        tex_view: &wgpu::TextureView,
        filter: crate::core::TextureFilter,
        blend: wgpu::BlendState,
        context: &mut Context,
    ) {
        let pipeline = context.get_or_create_texture_pipeline(blend, context.format);
        let sampler = Context::sampler_for_filter(&context.nearest_sampler, &context.linear_sampler, filter);
        let load_op = self.next_load_op();
        let view = self.view.clone();
        render_texture_quad(
            &mut self.command_encoder,
            &view,
            &self.device,
            &self.queue,
            uniforms,
            &BLIT_QUAD,
            tex_view,
            sampler,
            &[],
            &context.placeholder_view,
            &pipeline,
            &context.texture_bind_group_layout,
            load_op,
        );
    }
}

// --------------
// Program — custom WGSL pipelines for sprite and texture (quad) shaders
// --------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ProgramKind {
    Sprite,
    Texture,
}

pub struct Program {
    kind: ProgramKind,
    /// True for the built-in default texture program (no custom effect).
    is_builtin: bool,
    device: Arc<wgpu::Device>,
    vert_module: Arc<wgpu::ShaderModule>,
    frag_module: Arc<wgpu::ShaderModule>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pipeline_layout: Arc<wgpu::PipelineLayout>,
    pipelines: std::sync::Mutex<HashMap<(BlendStateKey, wgpu::TextureFormat), Arc<wgpu::RenderPipeline>>>,
    sampler: Arc<wgpu::Sampler>,
}

impl Program {
    /// Creates a program for a custom sprite fragment shader (used by sprite layers).
    pub fn new_sprite(context: &Context, fragment_shader: &str) -> crate::core::Result<Program> {
        let device = &context.device;

        let vert_module = Arc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Sprite Vert"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/sprite.wgsl").into()),
        }));

        let frag_module = Arc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Sprite Frag"),
            source: wgpu::ShaderSource::Wgsl(fragment_shader.into()),
        }));

        let bind_group_layout = Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Custom Sprite BGL"),
            entries: &[
                Context::bgl_uniform(0, wgpu::ShaderStages::VERTEX),
                Context::bgl_texture_2d(1, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_sampler(2, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_texture_2d_array(3, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_texture_2d_array(4, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_texture_2d_array(5, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_texture_2d_array(6, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_texture_2d_array(7, wgpu::ShaderStages::FRAGMENT),
                Context::bgl_uniform(8, wgpu::ShaderStages::FRAGMENT),
            ],
        }));

        Self::finish(context, ProgramKind::Sprite, false, vert_module, frag_module, bind_group_layout)
    }

    /// Creates a program for a custom texture fragment shader (used by postprocessors and fills).
    ///
    /// The fragment shader either uses the engine preamble (providing `texture_uniforms`,
    /// `_rd_tex` at binding 1, `_rd_sampler` at binding 2, and `TextureFragmentInput`) or
    /// declares its own bindings following the same convention (0 = `texture_uniforms`,
    /// 1 = main texture, 2 = sampler).
    ///
    /// Additional inputs are routed from the program's uniforms:
    /// - `f32`/`bool` uniforms are packed into `texture_uniforms._rd_flags` (`.x`–`.w`
    ///   in first-set order, at most 4).
    /// - `Texture` uniforms are bound to the user texture pool: consecutive
    ///   `texture_2d<f32>` bindings starting at [`USER_TEXTURE_BASE_BINDING`] in first-set
    ///   order (at most [`MAX_USER_TEXTURES`]). The shader declares them itself and
    ///   samples them with the sampler at binding 2.
    pub fn new_texture(context: &Context, fragment_shader: &str) -> crate::core::Result<Program> {
        Self::new_texture_inner(context, fragment_shader, false)
    }

    /// Creates the built-in default texture program (no custom effect).
    pub fn new_default_texture(context: &Context) -> crate::core::Result<Program> {
        Self::new_texture_inner(context, include_str!("../shader/texture_frag.wgsl"), true)
    }

    fn new_texture_inner(context: &Context, fragment_shader: &str, is_builtin: bool) -> crate::core::Result<Program> {
        let device = &context.device;

        let vert_module = Arc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Texture Vert"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/texture.wgsl").into()),
        }));

        let frag_module = Arc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Texture Frag"),
            source: wgpu::ShaderSource::Wgsl(fragment_shader.into()),
        }));

        // Same layout as the context's built-in texture pipeline, so the draw paths can
        // bind identically regardless of which texture program is in use.
        let bind_group_layout = Arc::new(context.texture_bind_group_layout.clone());

        Self::finish(context, ProgramKind::Texture, is_builtin, vert_module, frag_module, bind_group_layout)
    }

    fn finish(
        context: &Context,
        kind: ProgramKind,
        is_builtin: bool,
        vert_module: Arc<wgpu::ShaderModule>,
        frag_module: Arc<wgpu::ShaderModule>,
        bind_group_layout: Arc<wgpu::BindGroupLayout>,
    ) -> crate::core::Result<Program> {
        let device = &context.device;

        let pipeline_layout = Arc::new(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Custom Pipeline Layout"),
            bind_group_layouts: &[Some(&*bind_group_layout)],
            immediate_size: 0,
        }));

        let sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Custom Program Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        }));

        Ok(Program {
            kind,
            is_builtin,
            device: context.device.clone(),
            vert_module,
            frag_module,
            bind_group_layout,
            pipeline_layout,
            pipelines: std::sync::Mutex::new(HashMap::new()),
            sampler,
        })
    }

    pub fn get_or_create_pipeline(&self, blend: wgpu::BlendState, format: wgpu::TextureFormat) -> Arc<wgpu::RenderPipeline> {
        let key = (BlendStateKey::from(&blend), format);
        let mut pipelines = self.pipelines.lock().unwrap();
        if !pipelines.contains_key(&key) {
            let pipeline = Arc::new(match self.kind {
                ProgramKind::Sprite => Context::build_sprite_pipeline(
                    &self.device, &self.vert_module, &self.frag_module,
                    &self.pipeline_layout, format, blend,
                ),
                ProgramKind::Texture => Context::build_texture_pipeline(
                    &self.device, &self.vert_module, &self.frag_module,
                    &self.pipeline_layout, format, blend,
                ),
            });
            pipelines.insert(key, pipeline);
        }
        pipelines[&key].clone()
    }
}

// --------------
// Uniform buffer structs (must match WGSL layout exactly)
// --------------

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct SpriteUniforms {
    u_view: [[f32; 4]; 4],
    u_model: [[f32; 4]; 4],
    _rd_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct TextureUniforms {
    u_view: [[f32; 4]; 4],       // 64 bytes
    u_model: [[f32; 4]; 4],      // 64 bytes
    _rd_color: [f32; 4],          // 16 bytes
    _rd_offset: [f32; 2],         // 8 bytes
    _rd_dimensions: [f32; 2],     // 8 bytes
    _rd_flags: [f32; 4],          // 16 bytes (custom shader params: [horizontal, brightness, 0, 0])
    // total: 176 bytes — matches WGSL struct
}

impl TextureUniforms {
    /// Identity transform — renders the quad at (0,0)–(1,1) with no color tint.
    fn identity() -> Self {
        TextureUniforms {
            u_view: crate::core::Mat4::viewport(1.0, 1.0).into(),
            u_model: crate::core::Mat4::<f32>::identity().into(),
            _rd_color: [1.0, 1.0, 1.0, 1.0],
            _rd_offset: [0.0, 0.0],
            _rd_dimensions: [1.0, 1.0],
            _rd_flags: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

// --------------
// Monitor
// --------------

#[derive(Clone)]
pub struct Monitor {
    inner: MonitorInner,
}

#[derive(Clone)]
struct MonitorInner {
    winit_monitor: winit::monitor::MonitorHandle,
}

impl Monitor {
    pub fn get_dimensions(self: &Self) -> crate::core::Point2<u32> {
        let size = self.inner.winit_monitor.size();
        (size.width, size.height)
    }

    pub fn get_name(self: &Self) -> Option<String> {
        self.inner.winit_monitor.name()
    }
}

pub struct MonitorIterator {
    monitors: std::vec::IntoIter<winit::monitor::MonitorHandle>,
}

impl MonitorIterator {
    pub fn new() -> Self {
        MonitorIterator { monitors: Vec::new().into_iter() }
    }
}

impl Iterator for MonitorIterator {
    type Item = Monitor;
    fn next(&mut self) -> Option<Monitor> {
        self.monitors.next().map(|m| Monitor { inner: MonitorInner { winit_monitor: m } })
    }
}

// --------------
// Texture2d
// --------------

pub struct Texture2d {
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
    blit_res: Arc<BlitResources>,
}

impl Texture2d {
    pub fn new(context: &Context, width: u32, height: u32, format: crate::core::TextureFormat, data: Option<crate::core::RawFrame>) -> Self {
        let wgpu_format = Self::convert_format(format);
        let bpp = Self::bytes_per_pixel_for(format);

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Radiant Texture2d"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));

        if let Some(raw) = data {
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &raw.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(raw.width * raw.channels as u32),
                    rows_per_image: Some(raw.height),
                },
                wgpu::Extent3d { width: raw.width, height: raw.height, depth_or_array_layers: 1 },
            );
        }

        let blit_res = BlitResources::new(&context.device, &context.queue, wgpu_format);

        Texture2d {
            texture: Arc::new(texture),
            view,
            device: context.device.clone(),
            queue: context.queue.clone(),
            width,
            height,
            bytes_per_pixel: bpp,
            blit_res,
        }
    }

    pub fn clear(self: &Self, color: crate::core::Color) {
        let crate::core::Color(r, g, b, a) = color;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Texture Clear") });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Texture Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: r as f64, g: g as f64, b: b as f64, a: a as f64 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn write(self: &Self, rect: &crate::core::Rect<u32>, data: &Vec<u8>) {
        let x = (rect.0).0;
        let y = (rect.0).1;
        let w = (rect.1).0 - x;
        let h = (rect.1).1 - y;
        if w == 0 || h == 0 {
            return;
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * self.bytes_per_pixel),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    }

    pub fn copy_from(self: &Self, src_texture: &crate::core::Texture, filter: crate::core::TextureFilter) {
        let uniforms = TextureUniforms::identity();
        let sampler = Context::sampler_for_filter(&self.blit_res.nearest_sampler, &self.blit_res.linear_sampler, filter);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tex CopyFrom") });
        render_texture_quad(
            &mut encoder, &self.view,
            &self.device, &self.queue,
            &uniforms, &BLIT_QUAD,
            &*src_texture.handle.view,
            sampler,
            &[],
            &self.blit_res.placeholder_view,
            &self.blit_res.pipeline,
            &self.blit_res.bind_group_layout,
            wgpu::LoadOp::Load,
        );
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn copy_rect_from(self: &Self, src_texture: &crate::core::Texture, source_rect: crate::core::Rect<i32>, target_rect: crate::core::Rect<i32>, filter: crate::core::TextureFilter) {
        let tw = src_texture.handle.width as f32;
        let th = src_texture.handle.height as f32;
        let dw_full = self.width as f32;
        let dh_full = self.height as f32;

        let sx = (source_rect.0).0 as f32; let sy = (source_rect.0).1 as f32;
        let sw = (source_rect.1).0 as f32; let sh = (source_rect.1).1 as f32;
        let u0 = sx / tw; let u1 = (sx + sw) / tw;
        let v0 = sy / th; let v1 = (sy + sh) / th;

        let dx = (target_rect.0).0 as f32; let dy = (target_rect.0).1 as f32;
        let dw = (target_rect.1).0 as f32; let dh = (target_rect.1).1 as f32;

        let quad = [
            Vertex { position: [0.0, 0.0], texture_uv: [u0, v0], ..Vertex::default() },
            Vertex { position: [1.0, 0.0], texture_uv: [u1, v0], ..Vertex::default() },
            Vertex { position: [0.0, 1.0], texture_uv: [u0, v1], ..Vertex::default() },
            Vertex { position: [1.0, 1.0], texture_uv: [u1, v1], ..Vertex::default() },
        ];
        let uniforms = TextureUniforms {
            u_view: crate::core::Mat4::viewport(1.0, 1.0).into(),
            u_model: crate::core::Mat4::<f32>::identity().into(),
            _rd_color: [1.0, 1.0, 1.0, 1.0],
            _rd_offset: [dx / dw_full, dy / dh_full],
            _rd_dimensions: [dw / dw_full, dh / dh_full],
            _rd_flags: [0.0, 0.0, 0.0, 0.0],
        };
        let sampler = Context::sampler_for_filter(&self.blit_res.nearest_sampler, &self.blit_res.linear_sampler, filter);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tex CopyRectFrom") });
        render_texture_quad(
            &mut encoder, &self.view,
            &self.device, &self.queue,
            &uniforms, &quad,
            &*src_texture.handle.view,
            sampler,
            &[],
            &self.blit_res.placeholder_view,
            &self.blit_res.pipeline,
            &self.blit_res.bind_group_layout,
            wgpu::LoadOp::Load,
        );
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn copy_from_frame(self: &Self, _src_frame: &Frame, _filter: crate::core::TextureFilter) { /*TODO*/ }

    pub fn copy_rect_from_frame(self: &Self, _src_frame: &Frame, _source_rect: crate::core::Rect<i32>, _target_rect: crate::core::Rect<i32>, _filter: crate::core::TextureFilter) { /*TODO*/ }

    fn convert_format(format: crate::core::TextureFormat) -> wgpu::TextureFormat {
        use crate::core::TextureFormat as RF;
        match format {
            RF::U8              => wgpu::TextureFormat::R8Unorm,
            RF::U16             => wgpu::TextureFormat::R16Uint,
            RF::U8U8            => wgpu::TextureFormat::Rg8Unorm,
            RF::U16U16          => wgpu::TextureFormat::Rg16Uint,
            RF::U10U10U10       => wgpu::TextureFormat::Rgb10a2Unorm,
            RF::U12U12U12       => wgpu::TextureFormat::Rgba8Unorm,     // approximation
            RF::U16U16U16       => wgpu::TextureFormat::Rgba16Uint,     // approximation
            RF::U2U2U2U2        => wgpu::TextureFormat::Rgba8Unorm,     // approximation
            RF::U4U4U4U4        => wgpu::TextureFormat::Rgba8Unorm,     // approximation
            RF::U5U5U5U1        => wgpu::TextureFormat::Rgba8Unorm,     // approximation
            RF::U8U8U8U8        => wgpu::TextureFormat::Rgba8Unorm,
            RF::U10U10U10U2     => wgpu::TextureFormat::Rgb10a2Unorm,
            RF::U12U12U12U12    => wgpu::TextureFormat::Rgba16Uint,     // approximation
            RF::U16U16U16U16    => wgpu::TextureFormat::Rgba16Uint,
            RF::I16I16I16I16    => wgpu::TextureFormat::Rgba16Sint,
            RF::F16             => wgpu::TextureFormat::R16Float,
            RF::F16F16          => wgpu::TextureFormat::Rg16Float,
            RF::F16F16F16F16    => wgpu::TextureFormat::Rgba16Float,
            RF::F32             => wgpu::TextureFormat::R32Float,
            RF::F32F32          => wgpu::TextureFormat::Rg32Float,
            RF::F32F32F32F32    => wgpu::TextureFormat::Rgba32Float,
            RF::F11F11F10       => wgpu::TextureFormat::Rg11b10Ufloat,
        }
    }

    fn bytes_per_pixel_for(format: crate::core::TextureFormat) -> u32 {
        use crate::core::TextureFormat as RF;
        match format {
            RF::U8 | RF::F16 => 1,
            RF::U8U8 | RF::U16 | RF::F16F16 | RF::F32 => 2,
            RF::U8U8U8U8 | RF::U10U10U10U2 | RF::U10U10U10 | RF::U12U12U12
            | RF::U2U2U2U2 | RF::U4U4U4U4 | RF::U5U5U5U1 => 4,
            RF::U16U16 | RF::F32F32 => 4,
            RF::U16U16U16 | RF::U12U12U12U12 | RF::U16U16U16U16
            | RF::I16I16I16I16 | RF::F16F16F16F16 => 8,
            RF::F11F11F10 => 4,
            RF::F32F32F32F32 => 16,
        }
    }
}

// --------------
// Texture2dArray
// --------------

pub struct Texture2dArray {
    view: wgpu::TextureView,
}

impl Texture2dArray {
    pub fn new(context: &Context, raw: &Vec<crate::core::RawFrame>) -> Self {
        let layer_count = if raw.is_empty() { 1 } else { raw.len() as u32 };
        let width = raw.first().map_or(1, |f| f.width);
        let height = raw.first().map_or(1, |f| f.height);

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Radiant Texture2dArray"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: layer_count },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        for (i, frame) in raw.iter().enumerate() {
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                &frame.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(frame.width * frame.channels as u32),
                    rows_per_image: Some(frame.height),
                },
                wgpu::Extent3d { width: frame.width, height: frame.height, depth_or_array_layers: 1 },
            );
        }

        Texture2dArray { view }
    }
}

// --------------
// Context
// --------------

struct VertexBufferCacheItem {
    hint: usize,
    age: usize,
    buffer: wgpu::Buffer,
    capacity: usize,
}

pub struct Context {
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    format: wgpu::TextureFormat,
    index_buffer: wgpu::Buffer,
    index_count: usize,
    vertex_buffers: Vec<VertexBufferCacheItem>,
    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    sprite_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    // Shader modules kept alive for lazily creating pipelines by blend mode
    sprite_vert_module: wgpu::ShaderModule,
    sprite_frag_module: wgpu::ShaderModule,
    texture_vert_module: wgpu::ShaderModule,
    texture_frag_module: wgpu::ShaderModule,
    sprite_pipeline_layout: wgpu::PipelineLayout,
    texture_pipeline_layout: wgpu::PipelineLayout,
    sprite_pipelines: HashMap<(BlendStateKey, wgpu::TextureFormat), Arc<wgpu::RenderPipeline>>,
    texture_pipelines: HashMap<(BlendStateKey, wgpu::TextureFormat), Arc<wgpu::RenderPipeline>>,
    // 1×1 white Rgba8Unorm texture used as placeholder when draw_rect has no source texture
    placeholder_view: Arc<wgpu::TextureView>,
}

impl Context {
    pub fn shared_gpu(&self) -> SharedGpu {
        SharedGpu {
            instance: self.instance.clone(),
            adapter: self.adapter.clone(),
            device: self.device.clone(),
            queue: self.queue.clone(),
        }
    }

    pub fn new(display: &Display, initial_capacity: usize) -> Self {
        let format = display.inner.config.lock().unwrap().format;
        let instance = display.inner.instance.clone();
        let adapter = display.inner.adapter.clone();
        let device = display.inner.device.clone();
        let queue = display.inner.queue.clone();

        let index_buffer = Self::create_index_buffer(&device, &queue, initial_capacity);

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        // Sprite bind group layout: uniforms, font tex, sampler, 5 tex arrays, comp uniform
        let sprite_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Bind Group Layout"),
            entries: &[
                Self::bgl_uniform(0, wgpu::ShaderStages::VERTEX),
                Self::bgl_texture_2d(1, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_sampler(2, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_texture_2d_array(3, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_texture_2d_array(4, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_texture_2d_array(5, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_texture_2d_array(6, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_texture_2d_array(7, wgpu::ShaderStages::FRAGMENT),
                Self::bgl_uniform(8, wgpu::ShaderStages::FRAGMENT),
            ],
        });

        // Texture bind group layout: uniforms, source tex, sampler, user texture pool
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &Self::texture_bgl_entries(),
        });

        // Shader modules
        let sprite_vert_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/sprite.wgsl").into()),
        });
        let sprite_frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/sprite_frag.wgsl").into()),
        });
        let texture_vert_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/texture.wgsl").into()),
        });
        let texture_frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/texture_frag.wgsl").into()),
        });

        let sprite_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[Some(&sprite_bind_group_layout)],
            immediate_size: 0,
        });
        let texture_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texture Pipeline Layout"),
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        // 1×1 white placeholder texture
        let placeholder_view = Self::create_placeholder_view(&device, &queue);

        // Pre-create the default alpha-blending pipelines
        let mut sprite_pipelines = HashMap::new();
        let mut texture_pipelines = HashMap::new();

        let default_blend = wgpu::BlendState::ALPHA_BLENDING;
        let blend_key = BlendStateKey::from(&default_blend);
        let composite_key = (blend_key, format);
        sprite_pipelines.insert(composite_key, Arc::new(Self::build_sprite_pipeline(
            &device, &sprite_vert_module, &sprite_frag_module, &sprite_pipeline_layout, format, default_blend,
        )));
        texture_pipelines.insert(composite_key, Arc::new(Self::build_texture_pipeline(
            &device, &texture_vert_module, &texture_frag_module, &texture_pipeline_layout, format, default_blend,
        )));

        Context {
            instance,
            adapter,
            device,
            queue,
            format,
            index_buffer,
            index_count: initial_capacity * 6,
            vertex_buffers: Vec::new(),
            nearest_sampler,
            linear_sampler,
            sprite_bind_group_layout,
            texture_bind_group_layout,
            sprite_vert_module,
            sprite_frag_module,
            texture_vert_module,
            texture_frag_module,
            sprite_pipeline_layout,
            texture_pipeline_layout,
            sprite_pipelines,
            texture_pipelines,
            placeholder_view,
        }
    }

    // Bind group layout entry helpers
    fn bgl_uniform(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    fn bgl_texture_2d(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }
    fn bgl_texture_2d_array(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        }
    }
    fn bgl_sampler(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    /// Group-0 bind group layout entries for texture programs.
    ///
    /// Binding 0 = `texture_uniforms` (visible to vertex and fragment; custom fragment
    /// shaders may read e.g. `_rd_flags`), binding 1 = main texture, binding 2 = sampler,
    /// bindings 3..3+MAX_USER_TEXTURES = user texture pool (filled from `Texture`
    /// uniforms, see [`Program::new_texture`]).
    fn texture_bgl_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![
            Self::bgl_uniform(0, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT),
            Self::bgl_texture_2d(1, wgpu::ShaderStages::FRAGMENT),
            Self::bgl_sampler(2, wgpu::ShaderStages::FRAGMENT),
        ];
        for i in 0..MAX_USER_TEXTURES {
            entries.push(Self::bgl_texture_2d(USER_TEXTURE_BASE_BINDING + i as u32, wgpu::ShaderStages::FRAGMENT));
        }
        entries
    }

    /// Creates a 1×1 white placeholder texture view.
    fn create_placeholder_view(device: &wgpu::Device, queue: &wgpu::Queue) -> Arc<wgpu::TextureView> {
        let placeholder_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Placeholder Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &placeholder_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[255u8, 255, 255, 255],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        Arc::new(placeholder_tex.create_view(&wgpu::TextureViewDescriptor::default()))
    }

    fn get_or_create_sprite_pipeline(&mut self, blend: wgpu::BlendState, target_format: wgpu::TextureFormat) -> Arc<wgpu::RenderPipeline> {
        let key = (BlendStateKey::from(&blend), target_format);
        if !self.sprite_pipelines.contains_key(&key) {
            let pipeline = Arc::new(Self::build_sprite_pipeline(
                &self.device, &self.sprite_vert_module, &self.sprite_frag_module,
                &self.sprite_pipeline_layout, target_format, blend,
            ));
            self.sprite_pipelines.insert(key, pipeline);
        }
        self.sprite_pipelines[&key].clone()
    }

    fn get_or_create_texture_pipeline(&mut self, blend: wgpu::BlendState, target_format: wgpu::TextureFormat) -> Arc<wgpu::RenderPipeline> {
        let key = (BlendStateKey::from(&blend), target_format);
        if !self.texture_pipelines.contains_key(&key) {
            let pipeline = Arc::new(Self::build_texture_pipeline(
                &self.device, &self.texture_vert_module, &self.texture_frag_module,
                &self.texture_pipeline_layout, target_format, blend,
            ));
            self.texture_pipelines.insert(key, pipeline);
        }
        self.texture_pipelines[&key].clone()
    }

    fn build_sprite_pipeline(
        device: &wgpu::Device,
        vert: &wgpu::ShaderModule,
        frag: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        blend: wgpu::BlendState,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: vert,
                entry_point: Some("main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,  // position
                        1 => Float32x2,  // offset
                        2 => Float32,    // rotation
                        3 => Float32x4,  // color
                        4 => Uint32,     // bucket_id
                        5 => Uint32,     // texture_id
                        6 => Float32x2,  // texture_uv
                        7 => Uint32,     // components
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: frag,
                entry_point: Some("main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }

    fn build_texture_pipeline(
        device: &wgpu::Device,
        vert: &wgpu::ShaderModule,
        frag: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        blend: wgpu::BlendState,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: vert,
                entry_point: Some("main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    // Only position (@location 0) and texture_uv (@location 6) are read by the shader.
                    // Extra attributes in the buffer are silently ignored by wgpu.
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: TEXTURE_UV_OFFSET,
                            shader_location: 6,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: frag,
                entry_point: Some("main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }

    fn create_index_buffer(device: &Arc<wgpu::Device>, queue: &Arc<wgpu::Queue>, max_sprites: usize) -> wgpu::Buffer {
        let mut data: Vec<u32> = Vec::with_capacity(max_sprites * 6);
        for i in 0..max_sprites {
            let base = i as u32 * 4;
            data.extend_from_slice(&[base, base+1, base+2, base+1, base+3, base+2]);
        }
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (data.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_ne_bytes()).collect();
        queue.write_buffer(&buffer, 0, &bytes);
        buffer
    }

    fn select_vertex_buffer(&mut self, buffer_hint: usize, num_vertices: usize) -> (usize, bool) {
        for buffer in self.vertex_buffers.iter_mut() {
            buffer.age += 1;
        }
        if let Some(id) = self.vertex_buffers.iter().position(|item| item.hint == buffer_hint && item.capacity >= num_vertices) {
            self.vertex_buffers[id].age = 0;
            (id, false)
        } else if self.vertex_buffers.len() < MAX_BUFFERS {
            let buffer = Self::create_vertex_buffer(&self.device, num_vertices);
            self.vertex_buffers.push(VertexBufferCacheItem { hint: buffer_hint, age: 0, buffer, capacity: num_vertices });
            (self.vertex_buffers.len() - 1, true)
        } else {
            let (id, _) = self.vertex_buffers.iter().enumerate().max_by(|&(_, a), &(_, b)| a.age.cmp(&b.age)).unwrap();
            let buffer = Self::create_vertex_buffer(&self.device, num_vertices);
            self.vertex_buffers[id] = VertexBufferCacheItem { hint: buffer_hint, age: 0, buffer, capacity: num_vertices };
            (id, true)
        }
    }

    fn create_vertex_buffer(device: &Arc<wgpu::Device>, num_vertices: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (num_vertices * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn update_index_buffer(&mut self, max_sprites: usize) {
        if max_sprites * 6 > self.index_count {
            self.index_buffer = Self::create_index_buffer(&self.device, &self.queue, max_sprites);
            self.index_count = max_sprites * 6;
        }
    }

    fn blendmode_to_wgpu(blendmode: &crate::core::BlendMode) -> wgpu::BlendState {
        use crate::core::{BlendingFunction, LinearBlendingFactor};

        fn convert(factor: LinearBlendingFactor) -> wgpu::BlendFactor {
            match factor {
                LinearBlendingFactor::Zero                  => wgpu::BlendFactor::Zero,
                LinearBlendingFactor::One                   => wgpu::BlendFactor::One,
                LinearBlendingFactor::SourceColor           => wgpu::BlendFactor::Src,
                LinearBlendingFactor::OneMinusSourceColor   => wgpu::BlendFactor::OneMinusSrc,
                LinearBlendingFactor::DestinationColor      => wgpu::BlendFactor::Dst,
                LinearBlendingFactor::OneMinusDestinationColor => wgpu::BlendFactor::OneMinusDst,
                LinearBlendingFactor::SourceAlpha           => wgpu::BlendFactor::SrcAlpha,
                LinearBlendingFactor::SourceAlphaSaturate   => wgpu::BlendFactor::SrcAlphaSaturated,
                LinearBlendingFactor::OneMinusSourceAlpha   => wgpu::BlendFactor::OneMinusSrcAlpha,
                LinearBlendingFactor::DestinationAlpha      => wgpu::BlendFactor::DstAlpha,
                LinearBlendingFactor::OneMinusDestinationAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
                LinearBlendingFactor::ConstantColor         => wgpu::BlendFactor::Constant,
                LinearBlendingFactor::OneMinusConstantColor => wgpu::BlendFactor::OneMinusConstant,
                LinearBlendingFactor::ConstantAlpha         => wgpu::BlendFactor::Constant,
                LinearBlendingFactor::OneMinusConstantAlpha => wgpu::BlendFactor::OneMinusConstant,
            }
        }

        fn blend_component(func: BlendingFunction) -> wgpu::BlendComponent {
            match func {
                BlendingFunction::AlwaysReplace => wgpu::BlendComponent::REPLACE,
                BlendingFunction::Min => wgpu::BlendComponent { operation: wgpu::BlendOperation::Min, src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One },
                BlendingFunction::Max => wgpu::BlendComponent { operation: wgpu::BlendOperation::Max, src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One },
                BlendingFunction::Addition { source, destination } => wgpu::BlendComponent { operation: wgpu::BlendOperation::Add, src_factor: convert(source), dst_factor: convert(destination) },
                BlendingFunction::Subtraction { source, destination } => wgpu::BlendComponent { operation: wgpu::BlendOperation::Subtract, src_factor: convert(source), dst_factor: convert(destination) },
                BlendingFunction::ReverseSubtraction { source, destination } => wgpu::BlendComponent { operation: wgpu::BlendOperation::ReverseSubtract, src_factor: convert(source), dst_factor: convert(destination) },
            }
        }

        wgpu::BlendState {
            color: blend_component(blendmode.color),
            alpha: blend_component(blendmode.alpha),
        }
    }

    fn sampler_for_filter<'a>(nearest: &'a wgpu::Sampler, linear: &'a wgpu::Sampler, filter: crate::core::TextureFilter) -> &'a wgpu::Sampler {
        if filter == crate::core::TextureFilter::Linear { linear } else { nearest }
    }

}

// --------------
// Shared render helper
// --------------

/// Index buffer for a two-triangle quad (vertices 0-3 forming a rectangle).
const QUAD_INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];

/// Blit quad: covers (0,0)–(1,1) with V non-flipped (v=0 at image top = screen top).
/// Used for all texture quad rendering in wgpu (which stores textures with v=0 at top).
const BLIT_QUAD: [Vertex; 4] = [
    Vertex { position: [0.0, 0.0], offset: [0.0, 0.0], rotation: 0.0, color: [1.0, 1.0, 1.0, 1.0], bucket_id: 0, texture_id: 0, texture_uv: [0.0, 0.0], components: 0 },
    Vertex { position: [1.0, 0.0], offset: [0.0, 0.0], rotation: 0.0, color: [1.0, 1.0, 1.0, 1.0], bucket_id: 0, texture_id: 0, texture_uv: [1.0, 0.0], components: 0 },
    Vertex { position: [0.0, 1.0], offset: [0.0, 0.0], rotation: 0.0, color: [1.0, 1.0, 1.0, 1.0], bucket_id: 0, texture_id: 0, texture_uv: [0.0, 1.0], components: 0 },
    Vertex { position: [1.0, 1.0], offset: [0.0, 0.0], rotation: 0.0, color: [1.0, 1.0, 1.0, 1.0], bucket_id: 0, texture_id: 0, texture_uv: [1.0, 1.0], components: 0 },
];

/// Issue a textured quad draw into `target_view` using the texture pipeline.
fn render_texture_quad(
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    uniforms: &TextureUniforms,
    quad: &[Vertex; 4],
    tex_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    user_textures: &[&wgpu::TextureView],
    placeholder: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    layout: &wgpu::BindGroupLayout,
    load_op: wgpu::LoadOp<wgpu::Color>,
) {
    let indices = QUAD_INDICES;

    let vb = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Quad VB"),
        size: (4 * std::mem::size_of::<Vertex>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ib = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Quad IB"),
        size: (6 * 4) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ub = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Quad UB"),
        size: std::mem::size_of::<TextureUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(quad.as_ptr() as *const u8, 4 * std::mem::size_of::<Vertex>())
    };
    let index_bytes = unsafe {
        std::slice::from_raw_parts(indices.as_ptr() as *const u8, 6 * 4)
    };
    let uniform_bytes = unsafe {
        std::slice::from_raw_parts(uniforms as *const TextureUniforms as *const u8, std::mem::size_of::<TextureUniforms>())
    };

    queue.write_buffer(&vb, 0, vertex_bytes);
    queue.write_buffer(&ib, 0, index_bytes);
    queue.write_buffer(&ub, 0, uniform_bytes);

    // Bind the user texture pool (see `Context::texture_bgl_entries`); slots without a
    // corresponding `Texture` uniform receive the placeholder.
    let mut entries = vec![
        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &ub, offset: 0, size: None }) },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(tex_view) },
        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
    ];
    for i in 0..MAX_USER_TEXTURES {
        let view = user_textures.get(i).copied().unwrap_or(placeholder);
        entries.push(wgpu::BindGroupEntry {
            binding: USER_TEXTURE_BASE_BINDING + i as u32,
            resource: wgpu::BindingResource::TextureView(view),
        });
    }

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Quad Bind Group"),
        layout,
        entries: &entries,
    });

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Quad Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations { load: load_op, store: wgpu::StoreOp::Store },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.set_vertex_buffer(0, vb.slice(..));
    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
    pass.draw_indexed(0..6, 0, 0..1);
}

fn draw_rect_custom<T>(
    target: &crate::core::RenderTarget,
    program: &crate::core::Program,
    backend_prog: &Program,
    backend_context: &mut Context,
    wgpu_blend: wgpu::BlendState,
    info: &crate::core::DrawBuilder<T>,
    view_matrix: crate::core::Mat4,
    model_matrix: crate::core::Mat4,
    color: crate::core::Color,
    texture: Option<&crate::core::Texture>,
) {
    use crate::core::Point2Trait;
    use crate::core::Uniform;

    // Pack user scalar uniforms (Float/Bool, in first-set order) into _rd_flags.
    let scalars = program.uniforms.iter()
        .filter_map(|(_, u)| match u {
            Uniform::Float(f) => Some(*f),
            Uniform::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        })
        .take(4)
        .collect::<Vec<_>>();
    let mut _rd_flags = [0.0f32; 4];
    for (slot, value) in _rd_flags.iter_mut().zip(scalars.iter()) {
        *slot = *value;
    }

    // Collect user texture uniforms (in first-set order) for the user texture pool.
    let user_textures: Vec<Arc<wgpu::TextureView>> = program.uniforms.iter()
        .filter_map(|(_, u)| match u {
            Uniform::Texture(t) => Some(t.handle.view.clone()),
            _ => None,
        })
        .take(MAX_USER_TEXTURES)
        .collect();
    let user_refs: Vec<&wgpu::TextureView> = user_textures.iter().map(|v| v.as_ref()).collect();

    let uniforms = TextureUniforms {
        u_view: view_matrix.into(),
        u_model: model_matrix.into(),
        _rd_color: color.into(),
        _rd_offset: info.rect.0.as_array(),
        _rd_dimensions: info.rect.1.as_array(),
        _rd_flags,
    };

    let tex_view: Arc<wgpu::TextureView> = if let Some(tex) = texture {
        tex.handle.view.clone()
    } else {
        backend_context.placeholder_view.clone()
    };

    let placeholder = &backend_context.placeholder_view;

    match &target.0 {
        crate::core::RenderTargetInner::Frame(frame_rc) => {
            let pipeline = backend_prog.get_or_create_pipeline(wgpu_blend, backend_context.format);
            let mut frame = frame_rc.borrow_mut();
            let frame = frame.as_mut().expect("No frame prepared");
            let load_op = frame.next_load_op();
            let view = frame.view.clone();
            render_texture_quad(
                &mut frame.command_encoder, &view,
                &frame.device, &frame.queue,
                &uniforms, &BLIT_QUAD, &*tex_view,
                &*backend_prog.sampler,
                &user_refs, &**placeholder,
                &pipeline, &backend_prog.bind_group_layout,
                load_op,
            );
        }
        crate::core::RenderTargetInner::Texture(dest_texture) => {
            let dest_view = dest_texture.handle.view.clone();
            let dest_format = dest_texture.handle.texture.format();
            let pipeline = backend_prog.get_or_create_pipeline(wgpu_blend, dest_format);
            let mut encoder = backend_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Custom RTT") });
            render_texture_quad(
                &mut encoder, &dest_view,
                &backend_context.device, &backend_context.queue,
                &uniforms, &BLIT_QUAD, &*tex_view,
                &*backend_prog.sampler,
                &user_refs, &**placeholder,
                &pipeline, &backend_prog.bind_group_layout,
                wgpu::LoadOp::Load,
            );
            backend_context.queue.submit(std::iter::once(encoder.finish()));
        }
        crate::core::RenderTargetInner::None => {}
    }
}

/// Draw sprites directly into a texture view (for render-to-texture).
fn draw_sprites_to_texture(
    vertices: &[Vertex],
    dirty: bool,
    layer_hint: usize,
    component: u32,
    blend: wgpu::BlendState,
    font_texture_view: &wgpu::TextureView,
    tex_arrays: &[crate::core::RawFrameArray],
    sprite_uniforms: &SpriteUniforms,
    context: &mut Context,
    target_view: &wgpu::TextureView,
    dest_format: wgpu::TextureFormat,
    custom_program: Option<&Program>,
) {
    let num_vertices = vertices.len();
    let num_sprites = num_vertices / 4;
    if num_vertices == 0 {
        return;
    }

    let (vb_index, vb_dirty) = context.select_vertex_buffer(layer_hint, num_vertices);

    let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("RTT Encoder") });

    if dirty || vb_dirty {
        let vertex_bytes = unsafe {
            std::slice::from_raw_parts(vertices.as_ptr() as *const u8, num_vertices * std::mem::size_of::<Vertex>())
        };
        context.queue.write_buffer(&context.vertex_buffers[vb_index].buffer, 0, vertex_bytes);
    }
    context.update_index_buffer(num_sprites);

    let uniform_bytes = unsafe {
        std::slice::from_raw_parts(sprite_uniforms as *const SpriteUniforms as *const u8, std::mem::size_of::<SpriteUniforms>())
    };
    let uniform_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: uniform_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    context.queue.write_buffer(&uniform_buffer, 0, uniform_bytes);

    let comp_data = [component, 0u32, 0u32, 0u32];
    let comp_bytes = unsafe { std::slice::from_raw_parts(comp_data.as_ptr() as *const u8, 16) };
    let comp_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
    });
    context.queue.write_buffer(&comp_buffer, 0, comp_bytes);

    let sprite_bgl = custom_program.map_or(&context.sprite_bind_group_layout, |p| &*p.bind_group_layout);
    let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: sprite_bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &uniform_buffer, offset: 0, size: None }) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(font_texture_view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&context.nearest_sampler) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&tex_arrays[1].data.view) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&tex_arrays[2].data.view) },
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&tex_arrays[3].data.view) },
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&tex_arrays[4].data.view) },
            wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&tex_arrays[5].data.view) },
            wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &comp_buffer, offset: 0, size: None }) },
        ],
    });

    let pipeline = match custom_program {
        Some(prog) => prog.get_or_create_pipeline(blend, dest_format),
        None => context.get_or_create_sprite_pipeline(blend, dest_format),
    };
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("RTT Sprite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_blend_constant(wgpu::Color::TRANSPARENT);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, context.vertex_buffers[vb_index].buffer.slice(..));
        pass.set_index_buffer(context.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..(num_sprites as u32 * 6), 0, 0..1);
    }
    context.queue.submit(std::iter::once(encoder.finish()));
}

// --------------
// Public draw functions called from radiant_core
// --------------

pub fn draw_layer(target: &crate::core::RenderTarget, program: &crate::core::Program, context: &mut crate::core::ContextData, layer: &crate::core::Layer, component: u32) {
    let vertices = layer.vertices();
    let vertices = vertices.deref();
    let dirty = layer.undirty();
    let layer_id = layer.id();

    let view_matrix = *layer.view_matrix().deref().deref();
    let model_matrix = *layer.model_matrix().deref().deref();
    let color = layer.color().deref().clone();
    let blendmode = layer.blendmode().clone();
    let wgpu_blend = Context::blendmode_to_wgpu(&blendmode);

    let sprite_uniforms = SpriteUniforms {
        u_view: view_matrix,
        u_model: model_matrix,
        _rd_color: color.into(),
    };

    let font_view = context.font_texture.as_ref().map(|t| t.view.clone());
    // Invariant: sprite_program is only Some for sprite programs (see core::Program::new).
    let custom_sprite_prog = program.sprite_program.as_deref();

    let backend_context = context.backend_context.as_mut().unwrap();

    match &target.0 {
        crate::core::RenderTargetInner::Frame(frame_rc) => {
            let mut frame = frame_rc.borrow_mut();
            let frame = frame.as_mut().expect("No frame prepared");
            let font_view_ref = font_view.as_ref().map(|v| v.as_ref());
            if let Some(fv) = font_view_ref {
                frame.draw_sprites(
                    vertices, dirty, layer_id, component,
                    wgpu_blend, fv, &context.tex_arrays,
                    &sprite_uniforms, backend_context, custom_sprite_prog,
                );
            }
        }
        crate::core::RenderTargetInner::Texture(texture) => {
            // Render to texture using a temporary command encoder
            let dest_format = texture.handle.texture.format();
            let font_view_ref = font_view.as_ref().map(|v| v.as_ref());
            if let Some(fv) = font_view_ref {
                draw_sprites_to_texture(
                    vertices, dirty, layer_id, component,
                    wgpu_blend, fv, &context.tex_arrays,
                    &sprite_uniforms, backend_context, &*texture.handle.view, dest_format,
                    custom_sprite_prog,
                );
            }
        }
        crate::core::RenderTargetInner::None => {}
    }
}

pub fn draw_rect<T>(
    target: &crate::core::RenderTarget,
    program: &crate::core::Program,
    context: &mut crate::core::ContextData,
    blend: crate::core::BlendMode,
    info: crate::core::DrawBuilder<T>,
    view_matrix: crate::core::Mat4,
    model_matrix: crate::core::Mat4,
    color: crate::core::Color,
    texture: Option<&crate::core::Texture>,
) {
    use crate::core::Point2Trait;

    let wgpu_blend = Context::blendmode_to_wgpu(&blend);

    let backend_context = context.backend_context.as_mut().unwrap();

    // Custom programs use a dedicated draw path that packs uniforms and uses the program's pipeline.
    let backend_prog = program.texture_program.clone();
    if !backend_prog.is_builtin {
        draw_rect_custom(target, program, &backend_prog, backend_context, wgpu_blend, &info, view_matrix, model_matrix, color, texture);
        return;
    }

    let uniforms = TextureUniforms {
        u_view: view_matrix.into(),
        u_model: model_matrix.into(),
        _rd_color: color.into(),
        _rd_offset: info.rect.0.as_array(),
        _rd_dimensions: info.rect.1.as_array(),
        _rd_flags: [0.0, 0.0, 0.0, 0.0],
    };

    // Resolve texture view: use provided texture or 1×1 white placeholder
    let tex_view: Arc<wgpu::TextureView> = if let Some(tex) = texture {
        tex.handle.view.clone()
    } else {
        backend_context.placeholder_view.clone()
    };

    let filter = texture.map_or(crate::core::TextureFilter::Nearest, |t| t.magnify);

    match &target.0 {
        crate::core::RenderTargetInner::Frame(frame_rc) => {
            let mut frame = frame_rc.borrow_mut();
            let frame = frame.as_mut().expect("No frame prepared");
            frame.draw_quad(&uniforms, &*tex_view, filter, wgpu_blend, backend_context);
        }
        crate::core::RenderTargetInner::Texture(dest_texture) => {
            let dest_view = dest_texture.handle.view.clone();
            let dest_format = dest_texture.handle.texture.format();
            let pipeline = backend_context.get_or_create_texture_pipeline(wgpu_blend, dest_format);
            let sampler = Context::sampler_for_filter(&backend_context.nearest_sampler, &backend_context.linear_sampler, filter);
            let mut encoder = backend_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("DrawRect RTT") });
            render_texture_quad(
                &mut encoder,
                &dest_view,
                &backend_context.device,
                &backend_context.queue,
                &uniforms,
                &BLIT_QUAD,
                &*tex_view,
                sampler,
                &[],
                &*backend_context.placeholder_view,
                &pipeline,
                &backend_context.texture_bind_group_layout,
                wgpu::LoadOp::Load,
            );
            backend_context.queue.submit(std::iter::once(encoder.finish()));
        }
        crate::core::RenderTargetInner::None => {}
    }
}
