use prelude::*;
use glium;
use glium::Surface;
use color::Color;
use graphics::{Display, RenderContext, RenderContextData, RenderContextTextureArray, LayerBufferContainer, Layer, font, blendmode};
use scene;

#[derive(Clone)]
pub struct Renderer<'a> {
    max_sprites     : u32,
    context         : Arc<RenderContext<'a>>,
}

impl<'a> Renderer<'a> {

    /// Returns a new sprite renderer instance
    pub fn new(display: &Display, max_sprites: u32) -> Self {

        let mut context_data = RenderContextData {
            index_buffer    : Self::create_index_buffer(&display.handle, max_sprites),
            program         : Self::create_program(&display.handle),
            tex_array       : Vec::new(),
            target          : Option::None,
            display         : display.clone(),
            layer_buffers   : HashMap::new(),
        };

        for _ in 0..5 {
            context_data.tex_array.push(RenderContextTextureArray::new(display));
        }

        Renderer {
            max_sprites     : max_sprites,
            context         : Arc::new(RenderContext::new(context_data)),
        }
    }

    /// Returns a reference to the renderers' context.
    pub fn context(&self) -> Arc<RenderContext<'a>> {
        self.context.clone()
    }

    /// prepares a new target for drawing without clearing it
    pub fn prepare_target(&self) {
        let mut context = self.context.lock();
        context.target = Some(context.display.handle.draw());
    }

    /// prepares a new target and clears it with given color
    pub fn clear_target(&self, color: Color) {
        let mut context = self.context.lock();
        let (r, g, b, a) = color.as_tuple();
        let mut target = context.display.handle.draw();
        target.clear_color(r, g, b, a);
        context.target = Some(target);
    }

    /// finishes drawing and swaps the drawing target to front
    pub fn swap_target(&self) {
        let mut context = self.context.lock();
        context.target.take().unwrap().finish().unwrap();
    }

    /// takes the target frame from radiant-rs
    pub fn take_target(&self) -> glium::Frame {
        let mut context = self.context.lock();
        context.target.take().unwrap()
    }

    /// draws given scene
    pub fn draw_scene(&self, scene: &scene::Scene) -> &Self {
        scene::draw(scene, self);
        self
    }

    /// draws all sprites on given layer
    pub fn draw_layer(&self, layer: &Layer) -> &Self {

        // make sure texture arrays have been generated from raw images

        self.create_texture_arrays();

        // load layer local id, guard against writes to vertex_data

        let lid = layer.lid.load(Ordering::Relaxed);

        {
            // prepare texture array uniforms

            let mut context = self.context.lock();
            let mut context = context.deref_mut();

            // set up draw parameters for given blend options

            let draw_parameters = glium::draw_parameters::DrawParameters {
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                blend           : blendmode::access_blendmode(layer.blend.lock().unwrap().deref_mut()),
                .. Default::default()
            };

            // copy layer data to vertexbuffer

            if context.layer_buffers.contains_key(&layer.gid) == false {
                context.layer_buffers.insert(layer.gid, LayerBufferContainer {
                    lid     : 0,
                    size    : 0,
                    vb      : glium::VertexBuffer::empty_dynamic(&context.display.handle, self.max_sprites as usize * 4).unwrap(),
                    tc      : font::create_cache_texture(&context.display.handle, 512, 512), // !todo
                });
            }

            let container = context.layer_buffers.get_mut(&layer.gid).unwrap();

            if container.lid != lid {
                let vertex_data = layer.vertex_data.get();
                container.size = vertex_data.len() / 4;
                if container.size > 0 {
                    let num_vertices = container.size as usize * 4;
                    let vb_slice = container.vb.slice(0 .. num_vertices).unwrap();
                    vb_slice.write(&vertex_data[0 .. num_vertices]);
                    container.lid = lid;
                }
            }

            // update font texture from layer

            layer.font_cache.update(&mut container.tc);

            let uniforms = uniform! {
                view_matrix     : *layer.view_matrix.lock().unwrap().deref_mut(),
                model_matrix    : *layer.model_matrix.lock().unwrap().deref_mut(),
                global_color    : *layer.color.lock().unwrap().deref_mut(),
                font_cache      : container.tc.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex1            : &context.tex_array[1].data,
                tex2            : &context.tex_array[2].data,
                tex3            : &context.tex_array[3].data,
                tex4            : &context.tex_array[4].data,
            };

            // draw up to container.size (!todo try to check this earlier)

            if container.size > 0 {
                let ib_slice = context.index_buffer.slice(0 .. container.size as usize * 6).unwrap();
                context.target.as_mut().unwrap().draw(&container.vb, &ib_slice, &context.program, &uniforms, &draw_parameters).unwrap();
            }
        }

        self
    }

    /// creates texture arrays from registered textures
    fn create_texture_arrays(&self) {

        let mut context = self.context.lock();
        let mut context = context.deref_mut();

        for bucket_id in 0..context.tex_array.len() {
            if context.tex_array[bucket_id].dirty {
                context.tex_array[bucket_id].dirty = false;
                if context.tex_array[bucket_id].raw.len() > 0 {
                    let mut raw_images = Vec::new();
                    for texture_id in 0..context.tex_array[bucket_id].raw.len() {
                        let frame = &context.tex_array[bucket_id].raw[texture_id];
                        raw_images.push(glium::texture::RawImage2d {
                            data: frame.data.clone(),
                            width: frame.width,
                            height: frame.height,
                            format: frame.format,
                        });
                    }
                    let array = glium::texture::Texture2dArray::new(&context.display.handle, raw_images).unwrap();
                    context.tex_array[bucket_id].data = array;
                } else {
                    context.tex_array[bucket_id].data = glium::texture::Texture2dArray::empty(&context.display.handle, 2, 2, 1).unwrap();
                }
            }
        }
    }

    /// creates vertex pool for given number of sprites
    fn create_index_buffer(display: &glium::Display, max_sprites: u32) -> glium::index::IndexBuffer<u32> {

        let mut ib_data = Vec::with_capacity(max_sprites as usize * 6);

        for i in 0..max_sprites {
            let num = i as u32;
            ib_data.push(num * 4);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 2);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 3);
            ib_data.push(num * 4 + 2);
        }

        glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap()
    }

    /// creates the shader program
    fn create_program(display: &glium::Display) -> glium::Program {
        program!(display,
            140 => {
                vertex: include_str!("../shader/default.vs"),
                fragment: include_str!("../shader/default.fs")
            }
        ).unwrap()
    }
}

/// returns the appropriate bucket_id for the given texture size
pub fn bucket_info(width: u32, height: u32) -> (u32, u32) {
    let ln2 = (cmp::max(width, height) as f32).log2().ceil() as u32;
    let size = 2u32.pow(ln2);
    // skip first five sizes 1x1 to 16x16, use id 0 for font-cache
    let bucket_id = cmp::max(0, ln2 - 4 + 1);
    (bucket_id, size)
}
