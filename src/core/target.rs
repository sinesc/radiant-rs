use glium;
use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::{Surface, Program, DrawParameters, DrawError};
use core::{display, rendercontext, RenderContext, Display, Color};

enum TargetInfo {
    Display(glium::Display, Option<glium::Frame>),
    Framebuffer(glium::texture::Texture2d),
}

/// A target to render to.
pub struct Target (TargetInfo);

/// Prepares the target for drawing.
pub fn prepare(target: &mut Target) {
    match target.0 {
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
    match target.0 {
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

/// Finishes drawing on the target. Swaps display targets to frontbuffer.
pub fn finish(target: &mut Target) {
    use std::mem::replace;
    match target.0 {
        TargetInfo::Display(_, ref mut maybe_frame) => {
            let mut maybe_frame = replace(maybe_frame, None);
            if maybe_frame.is_some() {
                let error = maybe_frame.take().unwrap().finish();
            }
        }
        _ => { }
    }
}

/// Draws onto given target.
pub fn draw<'b, 'v, V, I, U>(target: &mut Target, vb: V, ib: I, program: &Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<(), DrawError>
    where I: Into<IndicesSource<'b>>, U: Uniforms, V: MultiVerticesSource<'v> {

    match target.0 {
        TargetInfo::Framebuffer(ref texture) => {
            texture.as_surface().draw(vb, ib, program, uniforms, draw_parameters)
        }
        TargetInfo::Display(_, ref mut frame) => {
            frame.as_mut().unwrap().draw(vb, ib, program, uniforms, draw_parameters)
        }
    }
}

/// Creates a target from given display.
pub fn from_display(display: &Display) -> Target {
    let handle = display::handle(display);
    Target(
        TargetInfo::Display(handle.clone(), Some(handle.draw()))
    )
}

/// Creates a new framebuffer target.
pub fn create_framebuffer(context: &RenderContext, width: u32, height: u32) -> Target {

    let context = rendercontext::lock(context);

    let texture = glium::texture::Texture2d::empty_with_format(
        display::handle(&context.display),
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
        glium::texture::MipmapsOption::NoMipmap,
        width,
        height
    ).unwrap();

    Target (
        TargetInfo::Framebuffer(texture)
    )
}
