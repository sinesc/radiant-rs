use prelude::*;
use glium;
use core::{
    self, texture, layer, scene, rendercontext, blendmode, program, uniform,
    Display, Layer, Texture, TextureFilter, BlendMode, Color, Program, Postprocessor,
    RenderContext, RenderContextData, AsRenderTarget, RenderTarget,
    GliumUniform, blendmodes,
};
use maths::{Rect, Mat4};

/// Default fragment shader program
const DEFAULT_FS: &'static str = include_str!("../shader/default.fs");

/// A renderer is used to render [`Layer`](struct.Layer.html)s or [`Scene`](scene/struct.Scene.html)s to the
/// [`Display`](struct.Display.html).
///
/// The renderer itself is not thread-safe. Instead, draw or write onto layers (from any one or
/// more threads)  and present those layers using the renderer once your threads have concluded
/// drawing.
#[derive(Clone)]
pub struct Renderer {
    context         : RenderContext,
    program         : Rc<Program>,
    target          : Rc<RefCell<Vec<RenderTarget>>>,
    empty_texture   : Texture,
}

impl Renderer {

    /// Returns a new renderer instance.
    pub fn new(display: &Display) -> core::Result<Self> {

        let context_data = RenderContextData::new(display, rendercontext::INITIAL_CAPACITY)?;
        let context = rendercontext::new(context_data);
        let target = vec![ RenderTarget::Display(display.clone()) ];
        let identity_texture = Texture::new(&context, 1, 1);
        identity_texture.as_render_target().clear(Color::white());

        Ok(Renderer {
            empty_texture   : identity_texture,
            context         : context,
            program         : Rc::new(program::create(display, DEFAULT_FS)?),
            target          : Rc::new(RefCell::new(target)),
        })
    }

    /// Returns a reference to the renderers' context. The [`RenderContext`](struct.RenderContext.html)
    /// implements send+sync and is required by [`Font`](struct.Font.html), [`Sprite`](struct.Sprite.html)
    /// and [`Texture`](struct.Texture.html) to create new instances.
    pub fn context(self: &Self) -> RenderContext {
        self.context.clone()
    }

    /// Clears the current target.
    pub fn clear(self: &Self, color: Color) -> &Self {
        self.target.borrow().last().unwrap().clear(color);
        self
    }

    /// Draws given layer to the current target. Component refers to the sprite component to draw.
    /// All sprites support at least component 0. Sprites that do not support
    /// the given component will not be drawn.
    pub fn draw_layer(self: &Self, layer: &Layer, component: u32) -> &Self {
        use glium::uniforms::{MagnifySamplerFilter, SamplerWrapFunction};

        // open context
        let mut context = rendercontext::lock(&self.context);
        let mut context = context.deref_mut();

        // update sprite texture arrays, font texture and vertex buffer as required
        context.update_tex_array();
        context.update_font_cache();
        let (vertex_buffer, num_vertices) = layer::upload(&layer, context);
        context.update_index_buffer(num_vertices / 4);

        // draw the layer, unless it is empty
        if num_vertices <= 0 {
            return self;
        }

        // use default or custom program
        let program = layer::program(layer).unwrap_or(&self.program);

        // set up uniforms
        let uniforms = program::uniforms(program);
        let mut glium_uniforms = uniform::to_glium_uniforms(&uniforms);
        glium_uniforms
            .add("u_view", GliumUniform::Mat4(layer.view_matrix().deref().into()))
            .add("u_model", GliumUniform::Mat4(layer.model_matrix().deref().into()))
            .add("_rd_color", GliumUniform::Vec4(layer.color().deref().into()))
            .add("_rd_tex", GliumUniform::Sampled2d(context.font_texture.sampled().magnify_filter(MagnifySamplerFilter::Nearest).wrap_function(SamplerWrapFunction::Clamp)))
            .add("_rd_comp", GliumUniform::UnsignedInt(component))
            .add("_rd_tex1", GliumUniform::Texture2dArray(context.tex_array[1].data.deref()))
            .add("_rd_tex2", GliumUniform::Texture2dArray(context.tex_array[2].data.deref()))
            .add("_rd_tex3", GliumUniform::Texture2dArray(context.tex_array[3].data.deref()))
            .add("_rd_tex4", GliumUniform::Texture2dArray(context.tex_array[4].data.deref()))
            .add("_rd_tex5", GliumUniform::Texture2dArray(context.tex_array[5].data.deref()));

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : blendmode::inner(layer.blendmode().deref()),
            .. Default::default()
        };

        // draw up to container.size
        let ib_slice = context.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();
        self.target.borrow().last().unwrap().draw(vertex_buffer.as_ref().unwrap(), &ib_slice, &program::sprite(program), &glium_uniforms, &draw_parameters);
        self
    }

    /// Draws a rectangle to the current target. See [`DrawRectBuilder`](builders/struct.DrawRectBuilder.html) for available options.
    pub fn rect<T>(self: &Self, target_rect: T) -> DrawRectBuilder where Rect<f32>: From<T> {
        DrawRectBuilder::new(self, Rect::<f32>::from(target_rect))
    }

    fn draw_rect(self: &Self, builder: DrawRectBuilder) -> &Self {

        // open context
        let mut context = rendercontext::lock(&self.context);
        let mut context = context.deref_mut();

        context.update_index_buffer(1);

        // use default or custom program and texture
        let target_rect = builder.rect;
        let program = builder.program.unwrap_or(&self.program);
        let texture = builder.texture.unwrap_or(&self.empty_texture);
        let color = builder.color.map_or(Color::white(), |c| *c);
        let blendmode = builder.blendmode.map_or(blendmodes::ALPHA, |b| *b);
        let model_matrix = builder.model_matrix.map_or(Mat4::identity(), |m| *m);
        let view_matrix = match builder.view_matrix {
            DrawRectViewSource::Matrix(matrix) => *matrix,
            DrawRectViewSource::Display => {
                let dim = context.display.dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
            DrawRectViewSource::Target => {
                let dim = self.target.borrow().last().unwrap().dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
            DrawRectViewSource::Source => {
                let dim = texture.dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
        };

        // set up uniforms
        let uniforms = program::uniforms(program);
        let mut glium_uniforms = uniform::to_glium_uniforms(&uniforms);
        glium_uniforms
            .add("u_view", GliumUniform::Mat4(view_matrix.into()))
            .add("u_model", GliumUniform::Mat4(model_matrix.into()))
            .add("_rd_color", GliumUniform::Vec4(color.into()))
            .add("_rd_tex", GliumUniform::Texture2d(texture::handle(texture)))
            .add("_rd_offset", GliumUniform::Vec2(target_rect.0.into()))
            .add("_rd_dimensions", GliumUniform::Vec2(target_rect.1.into()));

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : blendmode::inner(&blendmode),
            .. Default::default()
        };

        // draw up to container.size
        let ib_slice = context.index_buffer.slice(0..6).unwrap();
        self.target.borrow().last().unwrap().draw(&context.vertex_buffer_single, &ib_slice, &program::texture(program), &glium_uniforms, &draw_parameters);
        self
    }

    /// Copies a rectangle from the source to the current target. This is a blitting operation that uses integral pixel coordinates (top/left = 0/0).
    /// Coordinates must be entirely contained within their respective sources. No blending is performed.
    pub fn copy_rect_from<R, S, T>(self: &Self, source: &R, source_rect: S, target_rect: T, filter: TextureFilter) where R: AsRenderTarget, Rect<i32>: From<S> + From<T> {
        self.target.borrow().last().unwrap().blit_rect(&source.as_render_target(), source_rect.into(), target_rect.into(), filter);
    }

    /// Copies the entire source, overwriting the entire current target. This is a blitting operation, no blending is performed.
    pub fn copy_from<R>(self: &Self, source: &R, filter: TextureFilter) where R: AsRenderTarget {
        self.target.borrow().last().unwrap().blit(&source.as_render_target(), filter);
    }

    /// Reroutes draws issued within `draw_func()` to given Texture.
    pub fn render_to<F>(self: &Self, texture: &Texture, mut draw_func: F) -> &Self where F: FnMut() {
        self.push_target(texture);
        draw_func();
        self.pop_target();
        self
    }

    /// Reroutes draws issued within `draw_func()` through the given postprocessor.
    pub fn postprocess<P, F>(self: &Self, postprocessor: &P, arg: &<P as Postprocessor>::T, mut draw_func: F) -> &Self where F: FnMut(), P: Postprocessor {

        // draw to temporary target using given draw_func
        self.push_target(postprocessor.target());
        draw_func();

        // postprocess draw result
        postprocessor.process(self, arg);

        // restore previous target and draw postprocessor result
        self.pop_target();
        postprocessor.draw(self, arg);
        self
    }

    /// Returns a reference to the default rendering program.
    pub fn default_program(self: &Self) -> &Program {
        self.program.deref()
    }

    /// Pushes a target onto the target stack
    fn push_target<T>(self: &Self, target: &T) where T: AsRenderTarget {
        self.target.borrow_mut().push(target.as_render_target().clone());
    }

    /// Pops a target from the target stack
    fn pop_target(self: &Self) {
        self.target.borrow_mut().pop();
    }

    /// Draws given scene to the current target.
    #[deprecated(note="Removed for being out of scope of this library")]
    #[allow(deprecated)]
    pub fn draw_scene(self: &Self, scene: &scene::Scene, per_frame_multiplier: f32) -> &Self {
        scene::draw(scene, self, per_frame_multiplier);
        self
    }
}

/// returns the appropriate bucket_id and padded texture size for the given texture size
pub fn bucket_info(width: u32, height: u32) -> (u32, u32) {
    let ln2 = (cmp::max(width, height) as f32).log2().ceil() as u32;
    let size = 2u32.pow(ln2);
    // skip first five sizes 1x1 to 16x16, use id 0 for font-cache
    let bucket_id = cmp::max(0, ln2 - 4 + 1);
    assert!(bucket_id < rendercontext::NUM_BUCKETS as u32, "texture size exceeded configured maximum");
    (bucket_id, size)
}

// Below implements the builder anti-pattern. !todo There seems to be no good method
// to extract this into a submodule because rect() needs access to the private
// fields and build() needs access to the renderer's private draw_rect().
// An alternative would be to do the drawing in build() but that would move even
// more code where it doesn't belong.

/// A rectangle builder.
#[must_use]
pub struct DrawRectBuilder<'a> {
    renderer    : &'a Renderer,
    rect        : Rect,
    color       : Option<&'a Color>,
    texture     : Option<&'a Texture>,
    blendmode   : Option<&'a BlendMode>,
    view_matrix : DrawRectViewSource<'a>,
    model_matrix: Option<&'a Mat4>,
    program     : Option<&'a Program>,
}

/// The view matrix used when drawing a rectangle.
enum DrawRectViewSource<'a> {
    Display,
    Target,
    Source,
    Matrix(&'a Mat4)
}

impl<'a> DrawRectBuilder<'a> {
    fn new(renderer: &'a Renderer, rect: Rect) -> Self {
        DrawRectBuilder {
            renderer: renderer,
            rect: rect,
            color: None,
            texture: None,
            blendmode: None,
            view_matrix: DrawRectViewSource::Display,
            model_matrix: None,
            program: None,
        }
    }
    /// Sets a color for drawing.
    pub fn color(mut self: Self, color: &'a Color) -> Self {
        self.color = Some(color);
        self
    }
    /// Sets a model matrix for drawing.
    pub fn model_matrix(mut self: Self, model_matrix: &'a Mat4) -> Self {
        self.model_matrix = Some(model_matrix);
        self
    }
    /// Sets a view matrix for drawing.
    pub fn view_matrix(mut self: Self, view_matrix: &'a Mat4) -> Self {
        self.view_matrix = DrawRectViewSource::Matrix(view_matrix);
        self
    }
    /// Uses the view matrix of the display for drawing.
    pub fn view_display(mut self: Self) -> Self {
        self.view_matrix = DrawRectViewSource::Display;
        self
    }
    /// Uses the view matrix of the target for drawing.
    pub fn view_target(mut self: Self) -> Self {
        self.view_matrix = DrawRectViewSource::Target;
        self
    }
    /// Uses the view matrix of the source for drawing.
    pub fn view_source(mut self: Self) -> Self {
        self.view_matrix = DrawRectViewSource::Source;
        self
    }
    /// Sets the texture for drawing.
    pub fn texture(mut self: Self, texture: &'a Texture) -> Self {
        self.texture = Some(texture);
        self
    }
    /// Sets the blendmode for drawing.
    pub fn blendmode(mut self: Self, blendmode: &'a BlendMode) -> Self {
        self.blendmode = Some(blendmode);
        self
    }
    /// Sets the program used to draw.
    pub fn program(mut self: Self, program: &'a Program) -> Self {
        self.program = Some(program);
        self
    }
    /// Draws the rectangle.
    pub fn draw(self: Self) {
        self.renderer.draw_rect(self);
    }
}
