use prelude::*;
use core::{
    self, rendercontext,
    Display, Layer, Texture, TextureFilter, BlendMode, Color, Program, Postprocessor,
    RenderContext, RenderContextData, AsRenderTarget, RenderTarget,
    blendmodes, TextureFormat
};
use core::builder::*;
use maths::{Rect, Mat4};
use backends::backend;

/// Default fragment shader program
pub const DEFAULT_FS: &'static str = include_str!("../shader/default.fs");

lazy_static! {
    static ref VIEWPORT_ONE: Mat4 = Mat4::viewport(1.0, 1.0);
}

/// A renderer is used to render [`Layers`](struct.Layer.html) or [`Textures`](struct.Texture.html) to the
/// [`Display`](struct.Display.html).
///
/// The renderer itself is not thread-safe. Instead, draw or write onto layers (from any one or
/// more threads)  and present those layers using the renderer once your threads have concluded
/// drawing.
#[derive(Clone)]
pub struct Renderer {
    pub(crate) context         : RenderContext,
    pub(crate) program         : Rc<Program>,
    pub(crate) target          : Rc<RefCell<Vec<RenderTarget>>>,
    pub(crate) empty_texture   : Texture,
}

impl Renderer {

    /// Returns a new renderer instance.
    pub fn new(display: &Display) -> core::Result<Self> {

        let context_data = RenderContextData::new(&display.handle, rendercontext::INITIAL_CAPACITY)?;
        let context = RenderContext::new(context_data);
        let target = vec![ RenderTarget::Display(display.clone()) ];
        let identity_texture = Texture::builder(&context).format(TextureFormat::F16F16F16F16).dimensions((1, 1)).build().unwrap();
        identity_texture.clear(Color::WHITE);

        Ok(Renderer {
            empty_texture   : identity_texture,
            context         : context,
            program         : Rc::new(Program::new(&display.handle, DEFAULT_FS)?),
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

        // open context
        let mut context = self.context.lock();
        let context = context.deref_mut();

        // update sprite texture arrays, font texture and vertex buffer as required
        context.update_tex_array();
        context.update_font_cache();

        // use default or custom program
        let program = layer.program().unwrap_or(&self.program);

        backend::draw_layer(self.target.borrow().last().unwrap(), program, context, layer, component);
        self
    }

    /// Draws a rectangle to the current target. See [`DrawBuilder`](support/struct.DrawBuilder.html) for available options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let tex = Texture::new(&renderer.context(), 1, 1);
    /// # display.prepare_frame();
    /// renderer.rect((0., 0., 640., 480.)).blendmode(blendmodes::ALPHA).texture(&tex).draw();
    /// # display.swap_frame();
    /// ```
    pub fn rect<T>(self: &Self, target_rect: T) -> DrawBuilder<DrawBuilderRect> where Rect<f32>: From<T> {
        DrawBuilderRect::new(self, Rect::<f32>::from(target_rect))
    }

    /// Fills the current target. See [`DrawBuilder`](support/struct.DrawBuilder.html) for available options.
    ///
    /// This is a specialization of `rect()` that simply fills the entire target.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let tex = Texture::new(&renderer.context(), 1, 1);
    /// # display.prepare_frame();
    /// renderer.fill().blendmode(blendmodes::ALPHA).texture(&tex).draw();
    /// # display.swap_frame();
    /// ```
    pub fn fill(self: &Self) -> DrawBuilder<DrawBuilderFill> {
        DrawBuilderFill::new(self)
    }

    /// Reroutes draws issued within `draw_func()` to given Texture.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let some_layer = Layer::new((1.0, 1.0));
    /// # let some_texture = Texture::new(&renderer.context(), 1, 1);
    /// # let rendercontext = renderer.context();
    /// // Create a texture to render to.
    /// let surface = Texture::new(&rendercontext, 640, 480);
    ///
    /// // Render something to it.
    /// # display.prepare_frame();
    /// renderer.render_to(&surface, || {
    ///     renderer.rect((0., 0., 640., 480.)).texture(&some_texture).draw();
    ///     renderer.draw_layer(&some_layer, 0);
    /// });
    /// # display.swap_frame();
    /// ```
    pub fn render_to<F>(self: &Self, texture: &Texture, mut draw_func: F) -> &Self where F: FnMut() {
        self.push_target(texture);
        draw_func();
        self.pop_target();
        self
    }

    /// Reroutes draws issued within `draw_func()` through the given postprocessor.
    ///
    /// The following example uses the [`Basic`](postprocessors/struct.Basic.html) postprocessor provided by the library.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use radiant_rs::*;
    /// # let display = Display::builder().hidden().build();
    /// # let renderer = Renderer::new(&display).unwrap();
    /// # let some_layer = Layer::new((1.0, 1.0));
    /// # let some_texture = Texture::new(&renderer.context(), 1, 1);
    /// # let rendercontext = renderer.context();
    /// # let program_source = "#version 140\nout vec4 f_color;\nvoid main() { f_color = vec4(0.0, 0.0, 0.0, 0.0); }";
    /// # let my_layer = Layer::new((1.0, 1.0));
    /// // Load a shader progam.
    /// let my_program = Program::from_string(&rendercontext, &program_source).unwrap();
    ///
    /// // Create the postprocessor with the program.
    /// let my_postprocessor = postprocessors::Basic::new(&rendercontext, my_program);
    ///
    /// // ... in your renderloop...
    /// # display.prepare_frame();
    /// renderer.postprocess(&my_postprocessor, &blendmodes::ALPHA, || {
    ///     renderer.clear(Color::black());
    ///     renderer.draw_layer(&my_layer, 0);
    /// });
    /// # display.swap_frame();
    /// ```
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

    /// Copies a rectangle from the source to the current target.
    ///
    /// This is a blitting operation that uses integral pixel coordinates (top/left = 0/0).
    /// Coordinates must be entirely contained within their respective sources. No blending is performed.
    pub fn copy_rect_from<R, S, T>(self: &Self, source: &R, source_rect: S, target_rect: T, filter: TextureFilter) where R: AsRenderTarget, Rect<i32>: From<S> + From<T> {
        self.target.borrow().last().unwrap().blit_rect(&source.as_render_target(), source_rect.into(), target_rect.into(), filter);
    }

    /// Copies the entire source, overwriting the entire current target.
    ///
    /// This is a blitting operation, no blending is performed.
    pub fn copy_from<R>(self: &Self, source: &R, filter: TextureFilter) where R: AsRenderTarget {
        self.target.borrow().last().unwrap().blit(&source.as_render_target(), filter);
    }

    /// Returns a reference to the default rendering program.
    pub fn default_program(self: &Self) -> &Program {
        self.program.deref()
    }

    /// Draws the rectangle described info to the current target.
    pub(crate) fn draw_rect(self: &Self, target: DrawRectInfo) -> &Self {

        // open context
        let mut context = self.context.lock();
        let context = context.deref_mut();

        // use default or custom program and texture
        let program = target.program.unwrap_or(&self.program);
        let texture = target.texture.unwrap_or(&self.empty_texture);
        let color = target.color.unwrap_or(Color::WHITE);
        let blendmode = target.blendmode.unwrap_or(blendmodes::ALPHA);
        let model_matrix = target.model_matrix.unwrap_or(Mat4::identity());
        let view_matrix = match target.view_matrix {
            DrawRectInfoViewSource::Matrix(matrix) => matrix,
            DrawRectInfoViewSource::One => *VIEWPORT_ONE,
            DrawRectInfoViewSource::Target => {
                let dim = self.target.borrow().last().unwrap().dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
            DrawRectInfoViewSource::Display => {
                let dim = context.display.framebuffer_dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
            DrawRectInfoViewSource::Source => {
                let dim = texture.dimensions();
                Mat4::viewport(dim.0 as f32, dim.1 as f32)
            }
        };

        backend::draw_rect(self.target.borrow().last().unwrap(), program, context, blendmode, target, view_matrix, model_matrix, color, texture);
        self
    }

    /// Returns the appropriate bucket_id and padded texture size for the given texture size
    pub(crate) fn bucket_info(width: u32, height: u32) -> (u32, u32) {
        let ln2 = (cmp::max(width, height) as f32).log2().ceil() as u32;
        let size = 2u32.pow(ln2);
        // skip first five sizes 1x1 to 16x16, use id 0 for font-cache
        let bucket_id = cmp::max(0, ln2 - 4 + 1);
        assert!(bucket_id < rendercontext::NUM_BUCKETS as u32, "texture size exceeded configured maximum");
        (bucket_id, size)
    }

    /// Pushes a target onto the target stack
    fn push_target<T>(self: &Self, target: &T) where T: AsRenderTarget {
        self.target.borrow_mut().push(target.as_render_target().clone());
    }

    /// Pops a target from the target stack
    fn pop_target(self: &Self) {
        self.target.borrow_mut().pop();
    }
}

/// A struct used to describe a rectangle for Renderer::rect
#[derive(Clone)]
pub struct DrawRectInfo<'a> {
    pub rect        : Rect,
    pub color       : Option<Color>,
    pub texture     : Option<&'a Texture>,
    pub blendmode   : Option<BlendMode>,
    pub view_matrix : DrawRectInfoViewSource,
    pub model_matrix: Option<Mat4>,
    pub program     : Option<&'a Program>,
}

/// The view matrix used when drawing a rectangle.
#[derive(Clone)]
pub enum DrawRectInfoViewSource {
    Display,
    Target,
    Source,
    One,
    Matrix(Mat4)
}
