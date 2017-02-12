use prelude::*;
use glium;
use core::{
    self, texture, layer, scene, rendercontext, blendmode, program, uniform,
    Display, Layer, Texture, BlendMode, Color, Program,
    RenderContext, RenderContextData, RenderTarget, RenderTargetType,
    GliumUniform,
};
use maths::Vec2;

/// Default fragment shader program
const DEFAULT_FS: &'static str = include_str!("../shader/default.fs");

/// A renderer is used to render [`Layer`](struct.Layer.html)s or [`Scene`](struct.Scene.html)s to the
/// [`Display`](struct.Display.html).
///
/// The renderer itself is not thread-safe. Instead, draw or write onto layers (from any one or
/// more threads)  and present those layers using the renderer once your threads have concluded
/// drawing.
///
/// Alternatively to directly drawing on layers, [`Scene`](struct.Scene.html) provides a higher
/// level abstraction.
#[derive(Clone)]
pub struct Renderer {
    context: RenderContext,
    program: Rc<Program>,
    target: Rc<RefCell<RenderTargetType>>,
}

impl<'a> Renderer {

    /// Returns a new renderer instance.
    pub fn new(display: &Display) -> core::Result<Self> {

        let context_data = RenderContextData::new(display, rendercontext::INITIAL_CAPACITY)?;

        Ok(Renderer {
            context: rendercontext::new(context_data),
            program: Rc::new(program::create(display, DEFAULT_FS)?),
            target: Rc::new(RefCell::new(display.get_target())),
        })
    }

    /// Returns a reference to the renderers' context. The [`RenderContext`](struct.RenderContext)
    /// is thread-safe and required by [`Font`](struct.Font) and [`Sprite`](struct.Sprite) to
    /// create new instances.
    pub fn context(self: &Self) -> RenderContext {
        self.context.clone()
    }

    /// Sets a new rendering target. Valid targets are the display and textures.
    pub fn set_target<T>(self: &Self, target: &T) where T: RenderTarget {
        *self.target.borrow_mut() = target.get_target().clone();
    }

    /// Clears the current target.
    pub fn clear(self: &Self, color: &Color) {
        self.target.borrow().clear(color);
    }

    /// Draws given scene to the current target..
    pub fn draw_scene(self: &Self, scene: &scene::Scene, per_frame_multiplier: f32) -> &Self {
        scene::draw(scene, self, per_frame_multiplier);
        self
    }

    /// Draws given layer to the current target..
    pub fn draw_layer(self: &Self, layer: &Layer) -> &Self {

        // open context

        let mut context = rendercontext::lock(&self.context);
        let mut context = context.deref_mut();

        // update sprite texture arrays, font texture and vertex buffer as required

        context.update_tex_array();
        context.update_font_cache();
        let (vertex_buffer, num_vertices) = layer::upload(&layer, context);
        context.update_index_buffer(num_vertices / 4);

        // draw the layer, unless it is empty

        if num_vertices > 0 {

            // set up uniforms

            let uniforms = program::uniforms(&self.program);
            let mut glium_uniforms = uniform::to_glium_uniforms(&uniforms);

            glium_uniforms.add_glium("u_view", GliumUniform::Mat4(layer.view_matrix().deref().into()));
            glium_uniforms.add_glium("u_model", GliumUniform::Mat4(layer.model_matrix().deref().into()));
            glium_uniforms.add_glium("_rd_color", GliumUniform::Vec4(layer.color().deref().into()));
            glium_uniforms.add_glium("_rd_tex", GliumUniform::Sampled2d(context.font_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)));
            glium_uniforms.add_glium("_rd_tex1", GliumUniform::Texture2dArray(context.tex_array[1].data.deref()));
            glium_uniforms.add_glium("_rd_tex2", GliumUniform::Texture2dArray(context.tex_array[2].data.deref()));
            glium_uniforms.add_glium("_rd_tex3", GliumUniform::Texture2dArray(context.tex_array[3].data.deref()));
            glium_uniforms.add_glium("_rd_tex4", GliumUniform::Texture2dArray(context.tex_array[4].data.deref()));
            glium_uniforms.add_glium("_rd_tex5", GliumUniform::Texture2dArray(context.tex_array[5].data.deref()));

            // set up draw parameters for given blend options

            let draw_parameters = glium::draw_parameters::DrawParameters {
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                blend           : blendmode::access_blendmode(layer.blendmode().deref_mut()),
                .. Default::default()
            };

            // draw up to container.size

            let ib_slice = context.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();
            self.target.borrow().draw(vertex_buffer.as_ref().unwrap(), &ib_slice, &program::sprite(&self.program), &glium_uniforms, &draw_parameters).unwrap();
        }

        self
    }

    /// Draws given texture to the current target. Note: This function will likely change.
    pub fn draw_texture<T, S>(self: &Self, texture: &Texture, program: &Program, blendmode: BlendMode, position: T, size: S) -> &Self where Vec2<f32>: From<T>+From<S> {

        // open context

        let mut context = rendercontext::lock(&self.context);
        let mut context = context.deref_mut();

        context.update_index_buffer(1);

        // set up uniforms

        let uniforms = program::uniforms(program);
        let mut glium_uniforms = uniform::to_glium_uniforms(&uniforms);

        glium_uniforms.add_glium("offset", GliumUniform::Vec2(Vec2::from(position).into()));
        glium_uniforms.add_glium("size", GliumUniform::Vec2(Vec2::from(size).into()));
        glium_uniforms.add_glium("_rd_tex", GliumUniform::Texture2d(texture::handle(texture)));

        // set up draw parameters for given blend options

        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : blendmode::access_blendmode(&blendmode),
            .. Default::default()
        };

        // draw up to container.size

        let ib_slice = context.index_buffer.slice(0..6).unwrap();
        self.target.borrow().draw(&context.vertex_buffer_single, &ib_slice, &program::texture(program), &glium_uniforms, &draw_parameters).unwrap();

        self
    }

    /// Returns a reference to the default rendering program.
    pub fn default_program(self: &Self) -> &Program {
        self.program.deref()
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
