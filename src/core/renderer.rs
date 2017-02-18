use prelude::*;
use glium;
use core::{
    self, texture, layer, scene, rendercontext, blendmode, program, uniform,
    Display, Layer, Texture, BlendMode, Color, Program, Postprocessor,
    RenderContext, RenderContextData, AsRenderTarget, RenderTarget,
    GliumUniform,
};
use maths::Vec2;

/// Default fragment shader program
const DEFAULT_FS: &'static str = include_str!("../shader/default.fs");

/// A renderer is used to render [`Layer`](struct.Layer.html)s or [`Scene`](scene/struct.Scene.html)s to the
/// [`Display`](struct.Display.html).
///
/// The renderer itself is not thread-safe. Instead, draw or write onto layers (from any one or
/// more threads)  and present those layers using the renderer once your threads have concluded
/// drawing.
///
/// Alternatively to directly drawing on layers, [`Scene`](scene/struct.Scene.html) provides a higher
/// level abstraction.
#[derive(Clone)]
pub struct Renderer {
    context         : RenderContext,
    program         : Rc<Program>,
    target          : Rc<RefCell<(RenderTarget, bool)>>,
    empty_texture   : Texture,
}

impl<'a> Renderer {

    /// Returns a new renderer instance.
    pub fn new(display: &Display) -> core::Result<Self> {

        let context_data = RenderContextData::new(display, rendercontext::INITIAL_CAPACITY)?;
        let context = rendercontext::new(context_data);

        Ok(Renderer {
            empty_texture   : Texture::new(&context, 2, 2),
            context         : context,
            program         : Rc::new(program::create(display, DEFAULT_FS)?),
            target          : Rc::new(RefCell::new((RenderTarget::Display(display.clone()), false))),
        })
    }

    /// Returns a reference to the renderers' context. The [`RenderContext`](struct.RenderContext.html)
    /// is thread-safe and required by [`Font`](struct.Font.html) and [`Sprite`](struct.Sprite.html) to
    /// create new instances.
    pub fn context(self: &Self) -> RenderContext {
        self.context.clone()
    }

    /// Sets a new rendering target. Valid targets are the display and textures.
    pub fn set_target<T>(self: &Self, target: &T) -> &Self where T: AsRenderTarget {
        if self.target.borrow().1 {
            panic!("Attempted to set_target from within postrenderer function");
        }
        *self.target.borrow_mut() = (target.as_render_target().clone(), false);
        self
    }

    /// Clears the current target.
    pub fn clear(self: &Self, color: Color) -> &Self {
        self.target.borrow().0.clear(color);
        self
    }

    /// Draws given scene to the current target.
    #[deprecated(note="Removed for being out of scope of this library")]
    #[allow(deprecated)]
    pub fn draw_scene(self: &Self, scene: &scene::Scene, per_frame_multiplier: f32) -> &Self {
        scene::draw(scene, self, per_frame_multiplier);
        self
    }

    /// Draws given layer to the current target..
    pub fn draw_layer(self: &Self, layer: &Layer) -> &Self {
        self.draw(layer, None);
        self
    }

    /// Reroutes draws issued within draw_func through the given postprocessor.
    pub fn postprocess<P, F>(self: &Self, blendmode: BlendMode, postprocessor: &mut P, mut draw_func: F) -> &Self where F: FnMut(), P: Postprocessor {

        // backup previous target and set temporary target
        let previous_target = {
            let texture = postprocessor.target();
            mem::replace(&mut self.target.borrow_mut().0, texture.as_render_target().clone())
        };

        // draw to temporary target using given draw_func
        self.target.borrow_mut().1 = true;
        draw_func();
        self.target.borrow_mut().1 = false;

        // postprocess draw result
        postprocessor.process(self);

        // restore previous target and draw postprocessor result
        self.target.borrow_mut().0 = previous_target;
        postprocessor.draw(self, blendmode);
        self
    }

    /// Draws a rectangle to the current target. The optionally specified texture is available via sheet*() in the shader. Note that you can
    /// pass custom textures via the optional shader program's uniforms.
    pub fn draw_rect<T, S>(self: &Self, position: T, dimensions: S, blendmode: BlendMode, program: Option<&Program>, texture: Option<&Texture>) -> &Self where Vec2<f32>: From<T>+From<S> {

        // open context
        let mut context = rendercontext::lock(&self.context);
        let mut context = context.deref_mut();

        context.update_index_buffer(1);

        // use default or custom program and texture
        let program = match program {
            Some(program)   => program,
            None            => &self.program,
        };
        let texture = match texture {
            Some(texture)   => texture,
            None            => &self.empty_texture,
        };

        // set up uniforms
        let uniforms = program::uniforms(program);
        let mut glium_uniforms = uniform::to_glium_uniforms(&uniforms);
        glium_uniforms.add("_rd_offset", GliumUniform::Vec2(Vec2::from(position).into()));
        glium_uniforms.add("_rd_dimensions", GliumUniform::Vec2(Vec2::from(dimensions).into()));
        glium_uniforms.add("_rd_tex", GliumUniform::Texture2d(texture::handle(texture)));

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : blendmode::inner(&blendmode),
            .. Default::default()
        };

        // draw up to container.size
        let ib_slice = context.index_buffer.slice(0..6).unwrap();
        self.target.borrow().0.draw(&context.vertex_buffer_single, &ib_slice, &program::texture(program), &glium_uniforms, &draw_parameters);

        self
    }

    /// Returns a reference to the default rendering program.
    pub fn default_program(self: &Self) -> &Program {
        self.program.deref()
    }

    fn draw(self: &Self, layer: &Layer, override_blendmode: Option<&BlendMode>) {

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
            return;
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
            .add("_rd_tex1", GliumUniform::Texture2dArray(context.tex_array[1].data.deref()))
            .add("_rd_tex2", GliumUniform::Texture2dArray(context.tex_array[2].data.deref()))
            .add("_rd_tex3", GliumUniform::Texture2dArray(context.tex_array[3].data.deref()))
            .add("_rd_tex4", GliumUniform::Texture2dArray(context.tex_array[4].data.deref()))
            .add("_rd_tex5", GliumUniform::Texture2dArray(context.tex_array[5].data.deref()));

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : blendmode::inner( override_blendmode.unwrap_or(layer.blendmode().deref_mut()) ),
            .. Default::default()
        };

        // draw up to container.size
        let ib_slice = context.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();
        self.target.borrow().0.draw(vertex_buffer.as_ref().unwrap(), &ib_slice, &program::sprite(program), &glium_uniforms, &draw_parameters);
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
