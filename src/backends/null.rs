#![allow(unused_variables, dead_code, unused_mut)]
/*!
Null-Renderer

This backend is a no-op implementation of the backend interface, useful as a
reference for the minimum required implementation of a backend.
*/

use crate::core;
use crate::core::Mat4;

// --------------
// Public interface provided to Radiant-API-user in radiant_rs::backend
// --------------

pub mod public {}

// --------------
// Error
// --------------

#[derive(Debug)]
pub enum Error {
    Failed,
}

// --------------
// Shared GPU
// --------------

/// GPU resources that can be shared across displays that use the same context.
/// The null backend renders nothing, so there is nothing to share.
pub(crate) struct SharedGpu;

// --------------
// Display
// --------------

#[derive(Clone)]
pub struct Display();

impl Display {
    pub fn new(descriptor: core::DisplayBuilder, shared: Option<SharedGpu>) -> core::Result<Display> {
        Ok(Display())
    }
    pub fn draw(self: &Self) -> Frame {
        Frame()
    }
    pub fn framebuffer_dimensions(self: &Self) -> core::Point2<u32> {
        (0, 0)
    }
    pub fn window_dimensions(self: &Self) -> core::Point2<u32> {
        (0, 0)
    }
    pub fn set_fullscreen(self: &Self, monitor: Option<core::Monitor>) -> bool {
        false
    }
    pub fn set_dimensions(self: &Self, dimensions: core::Point2<u32>) {
    }
    pub fn set_maximized(self: &Self, maximized: bool) {
    }
    pub fn is_maximized(self: &Self) -> bool {
        false
    }
    pub fn set_cursor_position(self: &Self, position: core::Point2<i32>) {
    }
    pub fn set_cursor_state(self: &Self, state: core::CursorState) {
    }
    pub fn poll_events<F>(self: &Self, mut callback: F) where F: FnMut(core::Event) {
    }
    pub fn show(self: &Self) {
    }
    pub fn hide(self: &Self) {
    }
    pub fn set_title(self: &Self, title: &str) {
    }
}

// --------------
// Frame
// --------------

pub struct Frame();

impl Frame {
    pub fn clear(self: &mut Self, color: core::Color) {
    }
    pub fn finish(self: Self) {
    }
    pub fn dimensions(self: &Self) -> core::Point2<u32> {
        (0, 0)
    }
    pub fn copy_from_texture(self: &mut Self, source: &core::Texture, filter: core::TextureFilter) {
    }
    pub fn copy_rect(self: &mut Self, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
    }
    pub fn copy_rect_from_texture(self: &mut Self, source: &core::Texture, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
    }
}

// --------------
// Program
// --------------

pub struct Program {
    /// True for the built-in default texture program (no custom effect).
    is_builtin: bool,
}

impl Program {
    /// Creates a program for a custom sprite fragment shader (used by sprite layers).
    pub fn new_sprite(context: &Context, fragment_shader: &str) -> core::Result<Program> {
        Ok(Program { is_builtin: false })
    }

    /// Creates a program for a custom texture fragment shader (used by postprocessors and fills).
    pub fn new_texture(context: &Context, fragment_shader: &str) -> core::Result<Program> {
        Ok(Program { is_builtin: false })
    }

    /// Creates the built-in default texture program (no custom effect).
    pub fn new_default_texture(context: &Context) -> core::Result<Program> {
        Ok(Program { is_builtin: true })
    }
}

// --------------
// Monitor
// --------------

#[derive(Clone)]
pub struct Monitor();

impl Monitor {
    pub fn get_dimensions(self: &Self) -> core::Point2<u32> {
        (0, 0)
    }
    pub fn get_name(self: &Self) -> Option<String> {
        Some("Headless".to_string())
    }
}

pub struct MonitorIterator();

impl MonitorIterator {
    pub fn new() -> Self {
        MonitorIterator()
    }
}

impl Iterator for MonitorIterator {
    type Item = Monitor;
    fn next(&mut self) -> Option<Monitor> {
        None
    }
}

// --------------
// Texture2d
// --------------

pub struct Texture2d();

impl Texture2d {
    pub fn new(context: &Context, width: u32, height: u32, format: core::TextureFormat, data: Option<core::RawFrame>) -> Self {
        Texture2d()
    }
    pub fn clear(self: &Self, color: core::Color) {
    }
    pub fn write(self: &Self, rect: &core::Rect<u32>, data: &Vec<u8>) {
    }
    pub fn copy_from(self: &Self, src_texture: &core::Texture, filter: core::TextureFilter) {
    }
    pub fn copy_rect_from(self: &Self, src_texture: &core::Texture, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
    }
    pub fn copy_from_frame(self: &Self, src_frame: &Frame, filter: core::TextureFilter) {
    }
    pub fn copy_rect_from_frame(self: &Self, src_frame: &Frame, source_rect: core::Rect<i32>, target_rect: core::Rect<i32>, filter: core::TextureFilter) {
    }
}

// --------------
// Texture2dArray
// --------------

pub struct Texture2dArray();

impl Texture2dArray {
    pub fn new(context: &Context, raw: &Vec<core::RawFrame>) -> Self {
        Texture2dArray()
    }
}

// --------------
// Context
// --------------

pub struct Context();

impl Context {
    pub fn shared_gpu(&self) -> SharedGpu {
        SharedGpu
    }
    pub fn new(display: &Display, initial_capacity: usize) -> Self {
        Context()
    }
}

// --------------
// Drawing
// --------------

pub fn draw_layer(target: &core::RenderTarget, program: &core::Program, context: &mut core::ContextData, layer: &core::Layer, component: u32) {
}

pub fn draw_rect<T>(
    target: &core::RenderTarget,
    program: &core::Program,
    context: &mut core::ContextData,
    blend: core::BlendMode,
    info: core::DrawBuilder<T>,
    view_matrix: Mat4,
    model_matrix: Mat4,
    color: core::Color,
    texture: Option<&core::Texture>,
) {
}
