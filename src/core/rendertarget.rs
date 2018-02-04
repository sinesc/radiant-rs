use prelude::*;
use core::{Texture, Color, TextureFilter, Rect, Point2};
use backends::backend;

pub const NO_FRAME_PREPARED: &'static str = "Failed to get frame: None prepared.";

/// A target for rendering.
pub trait AsRenderTarget {
    /// Returns a RenderTarget representing a texture or a frame.
    fn as_render_target(self: &Self) -> RenderTarget;
}

/// An opaque type representing rendering targets like Display or Texture.
#[derive(Clone)]
pub struct RenderTarget(pub(crate) RenderTargetInner);

impl RenderTarget {
    /// Creates a new frame rendertarget.
    pub(crate) fn frame(frame: &Rc<RefCell<Option<backend::Frame>>>) -> RenderTarget {
        RenderTarget(RenderTargetInner::Frame(frame.clone()))
    }
    /// Creates a new texture rendertarget.
    pub(crate) fn texture(texture: &Texture) -> RenderTarget{
        RenderTarget(RenderTargetInner::Texture(texture.clone()))
    }
    /// Creates a null rendertarget.
    pub fn none() -> RenderTarget{
        RenderTarget(RenderTargetInner::None)
    }
}

impl AsRenderTarget for RenderTarget {
    fn as_render_target(self: &Self) -> RenderTarget {
        self.clone()
    }
}

/// An enum of render target type instances.
#[derive(Clone)]
pub enum RenderTargetInner {
    None,
    Frame(Rc<RefCell<Option<backend::Frame>>>),  
    Texture(Texture),                            
}

impl RenderTargetInner {
    /// Clears the target.
    pub fn clear(self: &Self, color: Color) {
        match *self {
            RenderTargetInner::Frame(ref display) => {
                let mut frame = display.borrow_mut();
                frame.as_mut().expect(NO_FRAME_PREPARED).clear(color);
            },
            RenderTargetInner::Texture(ref texture) => {
                texture.clear(color);
            }
            RenderTargetInner::None => { }
        }
    }
    /// Returns the dimensions of the target.
    pub fn dimensions(self: &Self) -> Point2<u32> {
        match *self {
            RenderTargetInner::Frame(ref display) => {
                let mut frame = display.borrow_mut();
                frame.as_mut().expect(NO_FRAME_PREPARED).dimensions()
            },
            RenderTargetInner::Texture(ref texture) => {
                texture.dimensions()
            }
            RenderTargetInner::None => {
                (0, 0)
            }
        }
    }
    /// Blits a source rect to a rect on the target.
    pub fn blit_rect(self: &Self, source: &RenderTarget, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: TextureFilter) {
        match *self {
            RenderTargetInner::Frame(ref target_display) => {
                match source.0 {
                    RenderTargetInner::Frame(_) => {
                        let mut frame = target_display.borrow_mut();
                        frame.as_mut().expect(NO_FRAME_PREPARED).copy_rect(source_rect, target_rect, filter);
                    },
                    RenderTargetInner::Texture(ref src_texture) => {
                        let mut frame = target_display.borrow_mut();
                        frame.as_mut().expect(NO_FRAME_PREPARED).copy_rect_from_texture(src_texture, source_rect, target_rect, filter);
                    }
                    RenderTargetInner::None => { }
                }
            },
            RenderTargetInner::Texture(ref target_texture) => {
                match source.0 {
                    RenderTargetInner::Frame(ref src_display) => {
                        let mut frame = src_display.borrow_mut();
                        target_texture.handle.copy_rect_from_frame(frame.as_mut().expect(NO_FRAME_PREPARED), source_rect, target_rect, filter);
                    },
                    RenderTargetInner::Texture(ref src_texture) => {
                        target_texture.handle.copy_rect_from(src_texture, source_rect, target_rect, filter);
                    }
                    RenderTargetInner::None => { }
                }
            }
            RenderTargetInner::None => { }
        }
    }
    /// Blits to the target.
    pub fn blit(self: &Self, source: &RenderTarget, filter: TextureFilter) {
        match *self {
            RenderTargetInner::Frame(ref target_display) => {
                match source.0 {
                    RenderTargetInner::Frame(_) => { /* blitting entire frame to entire frame makes no sense */ },
                    RenderTargetInner::Texture(ref src_texture) => {
                        let mut frame = target_display.borrow_mut();
                        frame.as_mut().expect(NO_FRAME_PREPARED).copy_from_texture(src_texture, filter);
                    }
                    RenderTargetInner::None => { }
                }
            },
            RenderTargetInner::Texture(ref target_texture) => {
                match source.0 {
                    RenderTargetInner::Frame(ref src_display) => {
                        let mut frame = src_display.borrow_mut();
                        target_texture.handle.copy_from_frame(frame.as_mut().expect(NO_FRAME_PREPARED), filter);
                    },
                    RenderTargetInner::Texture(ref src_texture) => {
                        target_texture.handle.copy_from(src_texture, filter);
                    }
                    RenderTargetInner::None => { }
                }
            }
            RenderTargetInner::None => { }
        }
    }
}