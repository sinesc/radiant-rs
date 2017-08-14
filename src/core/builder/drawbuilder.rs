use std::marker::PhantomData;
use core::{Color, Renderer, DrawRectInfo, DrawRectInfoViewSource, Texture, BlendMode, Program};
use maths::{Rect, Mat4};

#[derive(Clone)]
pub struct DrawBuilderFill;

#[derive(Clone)]
pub struct DrawBuilderRect;

/// A rectangle builder.
///
/// Obtained from [`Renderer::rect()`](../struct.Renderer.html#method.rect)
/// or [`Renderer::fill()`](../struct.Renderer.html#method.fill)
#[must_use]
#[derive(Clone)]
pub struct DrawBuilder<'a, T: 'a> {
    renderer    : &'a Renderer,
    phantomdata : PhantomData<&'a T>,
    info        : DrawRectInfo<'a>,
}

pub fn create_drawbuilderrect(renderer: &Renderer, rect: Rect) -> DrawBuilder<DrawBuilderRect> {
    DrawBuilder {
        renderer: renderer,
        phantomdata: PhantomData,
        info: DrawRectInfo {
            rect: rect,
            color: None,
            texture: None,
            blendmode: None,
            view_matrix: DrawRectInfoViewSource::Target,
            model_matrix: None,
            program: None,
        }
    }
}

pub fn create_drawbuilderfill(renderer: &Renderer) -> DrawBuilder<DrawBuilderFill> {
    DrawBuilder {
        renderer: renderer,
        phantomdata: PhantomData,
        info: DrawRectInfo {
            rect: (0., 0., 1., 1.).into(),
            color: None,
            texture: None,
            blendmode: None,
            view_matrix: DrawRectInfoViewSource::One,
            model_matrix: None,
            program: None,
        }
    }
}

/// The following implementations are available when drawing with [`Renderer::rect()`](../struct.Renderer.html#method.rect)
/// or [`Renderer::fill()`](../struct.Renderer.html#method.fill).
impl<'a, T> DrawBuilder<'a, T> {
    /// Sets a color for drawing. Defaults to white. If a texture is supplied in
    /// addtion to the color, each fragment will be computed from texel color * color.
    pub fn color(mut self: Self, color: Color) -> Self {
        self.info.color = Some(color);
        self
    }
    /// Sets the texture for drawing. If a color is supplied in
    /// addtion to the texture, each fragment will be computed from texel color * color.
    pub fn texture(mut self: Self, texture: &'a Texture) -> Self {
        self.info.texture = Some(texture);
        self
    }
    /// Sets the blendmode used to blend the source with the target.
    pub fn blendmode(mut self: Self, blendmode: BlendMode) -> Self {
        self.info.blendmode = Some(blendmode);
        self
    }
    /// Sets the fragment shader program used to draw.
    pub fn program(mut self: Self, program: &'a Program) -> Self {
        self.info.program = Some(program);
        self
    }
    /// Sets a model matrix for drawing.
    pub fn model_matrix(mut self: Self, model_matrix: Mat4) -> Self {
        self.info.model_matrix = Some(model_matrix);
        self
    }
    /// Draws the rectangle.
    pub fn draw(self: Self) {
        self.renderer.draw_rect(self.info);
    }
}

/// The following implementations are only available when drawing with [`Renderer::rect()`](../struct.Renderer.html#method.rect)
impl<'a> DrawBuilder<'a, DrawBuilderRect> {
    /// Sets a view matrix for drawing.
    pub fn view_matrix(mut self: Self, view_matrix: Mat4) -> Self {
        self.info.view_matrix = DrawRectInfoViewSource::Matrix(view_matrix);
        self
    }
    /// Uses a view matrix that maps the dimensions of the target to the pixel-size of the target.
    ///
    /// This is the default setting. It means that a point, e.g. (12., 34.) is mapped to the pixel
    /// (12, 34) on the target.
    pub fn view_target(mut self: Self) -> Self {
        self.info.view_matrix = DrawRectInfoViewSource::Target;
        self
    }
    /// Uses a view matrix that maps the dimensions of the display to the pixel-size of the target.
    pub fn view_display(mut self: Self) -> Self {
        self.info.view_matrix = DrawRectInfoViewSource::Display;
        self
    }
    /// Uses a view matrix that maps the dimensions of the source to the pixel-size of the target.
    pub fn view_source(mut self: Self) -> Self {
        self.info.view_matrix = DrawRectInfoViewSource::Source;
        self
    }
    /// Uses a matrix that maps the entire target to a rectangle of (0., 0., 1., 1.)
    pub fn view_one(mut self: Self) -> Self {
        self.info.view_matrix = DrawRectInfoViewSource::One;
        self
    }
}
