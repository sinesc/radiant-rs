#![allow(unused_variables, dead_code, unused_mut)]
/*!
Null-Renderer

This sample show the minimum required implementation of a backend.
*/

use crate::radiant_core;
use crate::radiant_core::math::*;

// --------------
// Public interface provided to Radiant-API-user in radiant_rs::backend
// --------------

pub mod public {

}

// --------------
// Error
// --------------

#[derive(Debug)]
pub enum Error {
    Failed,
}

// --------------
// Display
// --------------

#[derive(Clone)]
pub struct Display();

impl Display {
    pub fn new(descriptor: crate::radiant_core::DisplayInfo) -> crate::radiant_core::Result<Display> {
        Ok(Display())
    }
    pub fn draw(self: &Self) -> Frame {
        Frame()
    }
    pub fn framebuffer_dimensions(self: &Self) -> Point2<u32> {
        (0, 0)
    }
    pub fn window_dimensions(self: &Self) -> Point2<u32> {
        (0, 0)
    }
    pub fn set_fullscreen(self: &Self, monitor: Option<crate::radiant_core::Monitor>) -> bool {
        false
    }
    pub fn set_cursor_position(self: &Self, position: Point2<i32>) {
    }
    pub fn set_cursor_state(self: &Self, state: crate::radiant_core::CursorState) {
    }
    pub fn poll_events<F>(self: &Self, mut callback: F) where F: FnMut(crate::radiant_core::Event) -> () {
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
    pub fn clear(self: &mut Self, color: crate::radiant_core::Color) {
    }
    pub fn finish(self: Self) {
    }
    pub fn copy_from_texture(self: &Self, source: &crate::radiant_core::Texture, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn copy_rect(self: &Self, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn copy_rect_from_texture(self: &Self, source: &crate::radiant_core::Texture, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn dimensions(self: &Self) -> Point2<u32> {
        (0, 0)
    }
}

// --------------
// Program
// --------------

pub struct Program();

impl Program {
    pub fn new(display: &Display, vertex_shader: &str, fragment_shader: &str) -> crate::radiant_core::Result<Program> {
        Ok(Program())
    }
}

// --------------
// Monitor
// --------------

#[derive(Clone)]
pub struct Monitor();

impl Monitor {
    pub fn get_dimensions(self: &Self) -> Point2<u32> {
        (0, 0)
    }
    pub fn get_name(self: &Self) -> Option<String> {
        Some("Headless".to_string())
    }
}

pub struct MonitorIterator();

impl MonitorIterator {
    pub fn new(display: &crate::radiant_core::Display) -> Self {
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
    pub fn new(display: &Display, width: u32, height: u32, format: crate::radiant_core::TextureFormat, data: Option<crate::radiant_core::RawFrame>) -> Texture2d {
        Texture2d()
    }
    pub fn clear(self: &Self, color: crate::radiant_core::Color) {
    }
    pub fn write(self: &Self, rect: &Rect<u32>, data: &Vec<u8>) {
    }
    pub fn copy_from(self: &Self, src_texture: &crate::radiant_core::Texture, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn copy_rect_from(self: &Self, src_texture: &crate::radiant_core::Texture, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn copy_from_frame(self: &Self, src_frame: &Frame, filter: crate::radiant_core::TextureFilter) {
    }
    pub fn copy_rect_from_frame(self: &Self, src_frame: &Frame, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: crate::radiant_core::TextureFilter) {
    }
}

// --------------
// Texture2dArray
// --------------

pub struct Texture2dArray();

impl Texture2dArray {
    pub fn new(display: &Display, raw: &Vec<crate::radiant_core::RawFrame>) -> Self {
        Texture2dArray()
    }
}

// --------------
// Context
// --------------

pub struct Context();

impl Context {
    pub fn new(display: &Display, initial_capacity: usize) -> Self {
        Context()
    }
}

// --------------
// Drawing
// --------------

pub fn draw_layer(target: &crate::radiant_core::RenderTarget, program: &crate::radiant_core::Program, context: &mut crate::radiant_core::ContextData, layer: &crate::radiant_core::Layer, component: u32) {
}

pub fn draw_rect(target: &crate::radiant_core::RenderTarget, program: &crate::radiant_core::Program, context: &mut crate::radiant_core::ContextData, blend: crate::radiant_core::BlendMode, info: crate::radiant_core::DrawRectInfo, view_matrix: Mat4, model_matrix: Mat4, color: crate::radiant_core::Color, texture: &crate::radiant_core::Texture) {
}
