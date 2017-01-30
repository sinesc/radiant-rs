use glium;
use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{Surface, Program, DrawParameters, DrawError};
use prelude::*;
use core::{display, rendercontext, RenderContext, Display, Color};

// !todo mutex frame/texure, arc structure
pub enum TargetInfo {
    Display(glium::Display, Option<glium::Frame>),
    Framebuffer(glium::texture::Texture2d),
}

/// A target to render to.
#[derive(Clone)]
#[doc(hidden)]
pub struct Target (Arc<Mutex<TargetInfo>>);

impl Target {
    /// Returns a target for rendering to the display.
    pub fn display(context: &RenderContext) -> Target {
        let context = rendercontext::lock(&context);
        from_display(&context.display)
    }
    /// Returns a target for rendering to a texture.
    pub fn texture(context: &RenderContext, width: u32, height: u32) -> Target {

        let context = rendercontext::lock(context);

        let texture = glium::texture::Texture2d::empty_with_format(
            display::handle(&context.display),
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height
        ).unwrap();

        Target(Arc::new(Mutex::new(
            TargetInfo::Framebuffer(texture)
        )))
    }
    /// Returns true if this target is a texture.
    pub fn is_texture(self: &Self) -> bool {
        match *lock(self) {
            TargetInfo::Framebuffer(_) => true,
            _ => false,
        }
    }
    /// Returns true if this target is a display.
    pub fn is_display(self: &Self) -> bool {
        match *lock(self) {
            TargetInfo::Display(_, _) => true,
            _ => false,
        }
    }
}

/// Locks the target for rendering.
pub fn lock(target: &Target) -> MutexGuard<TargetInfo> {
    if let Ok(mutex) = target.0.try_lock() {
        mutex
    } else {
        panic!("Attempted to draw the current draw target.");
    }
}

/// Prepares the target for drawing.
pub fn prepare(target: &mut Target) {
    match *lock(target) {
        TargetInfo::Display(ref display, ref mut frame) => {
            if frame.is_none() {
                *frame = Some(display.draw());
            }
        }
        _ => { }
    }
}

/// Clears the target with given color.
pub fn clear(target: &mut Target, color: &Color) {
    let (r, g, b, a) = color.as_tuple();
    match *lock(target) {
        TargetInfo::Display(_, ref mut maybe_frame) => {
            if let Some(ref mut frame) = *maybe_frame {
                frame.clear_color(r, g, b, a);
            }
        }
        TargetInfo::Framebuffer(ref mut texture) => {
            texture.as_surface().clear_color(r, g, b, a);
        }
    }
}

/// Removes frame from current target, if present
pub fn take(target: &mut Target) -> Option<glium::Frame> {
    use std::mem::replace;
    match *lock(target) {
        TargetInfo::Display(_, ref mut maybe_frame) => {
            replace(maybe_frame, None)
        }
        _ => { None }
    }
}

/// Draws onto given target.
pub fn draw<'b, 'v, V, I, U>(target: &mut Target, vb: V, ib: I, program: &Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<(), DrawError>
    where I: Into<IndicesSource<'b>>, U: Uniforms, V: MultiVerticesSource<'v> {

    match *lock(target) {
        TargetInfo::Framebuffer(ref texture) => {
            texture.as_surface().draw(vb, ib, program, uniforms, draw_parameters)
        }
        TargetInfo::Display(_, ref mut frame) => {
            if frame.is_none() {
                panic!("Current display-target is not ready. Use Renderer::prepare_target on this target first.");
            }
            frame.as_mut().unwrap().draw(vb, ib, program, uniforms, draw_parameters)
        }
    }
}

/// Creates a target from given display.
pub fn from_display(display: &Display) -> Target {
    let handle = display::handle(display);
    Target(Arc::new(Mutex::new(
        TargetInfo::Display(handle.clone(), Some(handle.draw()))
    )))
}
