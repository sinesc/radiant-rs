use prelude::*;
use glium;
use glium::Surface;
use core::{Display, rendercontext, RenderContext, RenderContextData, layer, Layer, blendmode, scene, Color, display};

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
pub struct Renderer<'a> {
    capacity    : usize,
    context     : RenderContext<'a>,
}

impl<'a> Renderer<'a> {

    /// Returns a new renderer instance.
    pub fn new(display: &Display) -> Self {

        let context_data = RenderContextData::new(display, 1024);  // todo: "1024" add some sort of configurable?

        Renderer {
            capacity: 1024,
            context : rendercontext::new(context_data),
        }
    }

    /// Returns a reference to the renderers' context. The [`RenderContext`](struct.RenderContext)
    /// is thread-safe and required by [`Font`](struct.Font) and [`Sprite`](struct.Sprite) to
    /// create new instances.
    pub fn context(&self) -> RenderContext<'a> {
        self.context.clone()
    }

    /// Prepares a new target for drawing without clearing it.
    pub fn prepare_target(&self) {
        let mut context = rendercontext::lock(&self.context);
        context.target = Some(display::handle(&context.display).draw());
    }

    /// Prepares a new target and clears it with given color.
    pub fn clear_target(&self, color: Color) {
        let mut context = rendercontext::lock(&self.context);
        let (r, g, b, a) = color.as_tuple();
        let mut target = display::handle(&context.display).draw();
        target.clear_color(r, g, b, a);
        context.target = Some(target);
    }

    /// Finishes drawing and swaps the drawing target to front.
    pub fn swap_target(&self) {
        let mut context = rendercontext::lock(&self.context);
        context.target.take().unwrap().finish().unwrap();
    }
/*
    /// Takes the target frame from the renderer.
    pub fn take_target(&self) -> glium::Frame {
        let mut context = self.context.lock();
        context.target.take().unwrap()
    }
*/
    /// Draws given scene.
    pub fn draw_scene(&self, scene: &scene::Scene, per_frame_multiplier: f32) -> &Self {
        scene::draw(scene, self, per_frame_multiplier);
        self
    }

    /// Draws given layer.
    pub fn draw_layer(&self, layer: &Layer) -> &Self {

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

            let uniforms = uniform! {
                view_matrix     : *layer.view_matrix().deref_mut(),
                model_matrix    : *layer.model_matrix().deref_mut(),
                global_color    : *layer.color().deref_mut(),
                font_cache      : context.font_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex1            : &context.tex_array[1].data,
                tex2            : &context.tex_array[2].data,
                tex3            : &context.tex_array[3].data,
                tex4            : &context.tex_array[4].data,
            };

            // set up draw parameters for given blend options

            let draw_parameters = glium::draw_parameters::DrawParameters {
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                blend           : blendmode::access_blendmode(layer.blendmode().deref_mut()),
                .. Default::default()
            };

            // draw up to container.size

            let ib_slice = context.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();
            context.target.as_mut().unwrap().draw(vertex_buffer.as_ref().unwrap(), &ib_slice, &context.program, &uniforms, &draw_parameters).unwrap();
        }

        self
    }
}

/// returns the appropriate bucket_id and padded texture size for the given texture size
pub fn bucket_info(width: u32, height: u32) -> (u32, u32) {
    let ln2 = (cmp::max(width, height) as f32).log2().ceil() as u32;
    let size = 2u32.pow(ln2);
    // skip first five sizes 1x1 to 16x16, use id 0 for font-cache
    let bucket_id = cmp::max(0, ln2 - 4 + 1);
    (bucket_id, size)
}
