use prelude::*;
use glium;
use glium::Surface;
use color::Color;
use graphics::{Display, Layer, sprite, Sprite, font, Font, RawFrame, Vertex, blendmode};
use scene;

struct LayerBufferContainer {
    lid     : usize,
    size    : usize,
    vb      : glium::VertexBuffer<Vertex>,
    tc      : glium::texture::Texture2d,
}

// !todo use multiple structs glium (or better something else so it isn't so easy to confuse with glium::...), sprite, font
struct GliumState {
    index_buffer    : glium::IndexBuffer<u32>,
    program         : glium::Program,
    tex_array       : Vec<Option<glium::texture::Texture2dArray>>,
    raw_tex_data    : Vec<Vec<RawFrame>>,
    target          : Option<glium::Frame>,
    display         : Display,
    layer_buffers   : HashMap<usize, LayerBufferContainer>,
}

#[derive(Clone)]
pub struct Renderer {
    max_sprites     : u32,
    glium           : Rc<RefCell<GliumState>>,
}

impl Renderer {

    /// returns a new sprite renderer instance
    ///
    /// display is a glium Display obtained by i.e. glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap()
    pub fn new(display: &Display, max_sprites: u32) -> Self {

        let glium = GliumState {
            index_buffer    : Self::create_index_buffer(&display.handle, max_sprites),
            program         : Self::create_program(&display.handle),
            tex_array       : Vec::new(),
            raw_tex_data    : Vec::new(),
            target          : Option::None,
            display         : display.clone(),
            layer_buffers   : HashMap::new(),
        };
        Renderer {
            max_sprites     : max_sprites,
            glium           : Rc::new(RefCell::new(glium)),
        }
    }

    /// registers a sprite texture for drawing
    ///
    /// must be done before first draw, calling  this function after draw will reset existing
    /// sprites and register new ones
    /// filename is epected to end on _<width>x<height>x<frames>.<extension>, i.e. asteroid_64x64x24.png
    pub fn create_sprite(&self, file: &str) -> Sprite {

        let mut glium = self.glium.borrow_mut();

        // load spritesheet into RawFrames

        let (frame_width, frame_height, frames) = sprite::load_spritesheet(file);

        // identify bucket_id (which texture array) and texture index in the array

        let (bucket_id, _) = bucket_info(frame_width, frame_height);

        while glium.raw_tex_data.len() <= bucket_id as usize {
            glium.raw_tex_data.push(Vec::new());
        }

        let bucket_pos = glium.raw_tex_data[bucket_id as usize].len() as u32;

        // append frames to the array

        let frame_count = frames.len() as u32;

        for frame in frames {
            glium.raw_tex_data[bucket_id as usize].push(frame);
        }

        sprite::create_sprite(frame_width as f32, frame_height as f32, frame_count, bucket_pos)
    }

    pub fn create_font<'a>(&self, file: &str) -> Font<'a> {
        font::create_font(/*file*/)
    }

    /// prepares a new target for drawing
    pub fn prepare_target(&self) {
        let mut glium = self.glium.borrow_mut();
        glium.target = Some(glium.display.handle.draw());
    }

    /// clears the prepared target with given color
    pub fn clear_target(&self, color: Color) {
        let mut glium = self.glium.borrow_mut();
        let (r, g, b, a) = color.as_tuple();
        glium.target.as_mut().unwrap().clear_color(r, g, b, a);
    }

    /// prepares a new target and clears it with given color
    pub fn prepare_and_clear_target(&self, color: Color) {
        let mut glium = self.glium.borrow_mut();
        let (r, g, b, a) = color.as_tuple();
        let mut target = glium.display.handle.draw();
        target.clear_color(r, g, b, a);
        glium.target = Some(target);
    }

    /// finishes drawing and swaps the drawing target to front
    pub fn swap_target(&self) {
        let mut glium = self.glium.borrow_mut();
        glium.target.take().unwrap().finish().unwrap();
    }

    /// takes the target frame from radiant-rs
    pub fn take_target(&self) -> glium::Frame {
        let mut glium = self.glium.borrow_mut();
        glium.target.take().unwrap()
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

            let mut glium_mutexguard = self.glium.borrow_mut();
            let mut glium = glium_mutexguard.deref_mut(); // !todo blah

            let empty = &glium::texture::Texture2dArray::empty(&glium.display.handle, 2, 2, 1).unwrap();
            let mut arrays = Vec::<&glium::texture::Texture2dArray>::new();

            for i in 0..5 {
                arrays.push(if glium.tex_array.len() > i && glium.tex_array[i].is_some() {
                    glium.tex_array[i].as_ref().unwrap()
                } else {
                    empty
                });
            }

            // set up draw parameters for given blend options

            let draw_parameters = glium::draw_parameters::DrawParameters {
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                blend           : blendmode::access_blendmode(layer.blend.lock().unwrap().deref_mut()),
                .. Default::default()
            };

            // copy layer data to vertexbuffer

            if glium.layer_buffers.contains_key(&layer.gid) == false {
                glium.layer_buffers.insert(layer.gid, LayerBufferContainer {
                    lid     : 0,
                    size    : 0,
                    vb      : glium::VertexBuffer::empty_dynamic(&glium.display.handle, self.max_sprites as usize * 4).unwrap(),
                    tc      : font::create_cache_texture(&glium.display.handle, 512, 512), // !todo
                });
            }

            let container = glium.layer_buffers.get_mut(&layer.gid).unwrap();

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

            font::update_cache_texture(layer, &mut container.tc);

            let uniforms = uniform! {
                view_matrix     : *layer.view_matrix.lock().unwrap().deref_mut(),
                model_matrix    : *layer.model_matrix.lock().unwrap().deref_mut(),
                global_color    : *layer.color.lock().unwrap().deref_mut(),
                font_cache      : container.tc.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex1            : arrays[1],
                tex2            : arrays[2],
                tex3            : arrays[3],
                tex4            : arrays[4],
            };

            // draw up to container.size (!todo try to check this earlier)

            if container.size > 0 {
                let ib_slice = glium.index_buffer.slice(0 .. container.size as usize * 6).unwrap();
                glium.target.as_mut().unwrap().draw(&container.vb, &ib_slice, &glium.program, &uniforms, &draw_parameters).unwrap();
            }
        }

        self
    }

    /// creates texture arrays from registered textures
    fn create_texture_arrays(&self) {

        let mut glium = self.glium.borrow_mut();

        if glium.tex_array.len() == 0 {

            for bucket_id in 0..glium.raw_tex_data.len() {
                if glium.raw_tex_data[bucket_id].len() > 0 {
                    let raw_data = mem::replace(&mut glium.raw_tex_data[bucket_id as usize], Vec::new());
                    let array = glium::texture::Texture2dArray::new(&glium.display.handle, raw_data).unwrap();
                    glium.tex_array.push(Some(array));
                } else {
                    glium.tex_array.push(None);
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
