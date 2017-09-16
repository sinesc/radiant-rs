use prelude::*;
use std::borrow::Cow;
use glium;
use glium::{DisplayBuild, Surface};
use glium::uniforms::{Uniforms, AsUniformValue};
use core;
use maths::*;

macro_rules! implement_wrapped_vertex {
    ($struct_name:ident, $($field_name:ident),+) => (
        impl glium::vertex::Vertex for $struct_name {
            #[inline]
            fn build_bindings() -> glium::vertex::VertexFormat {
                use std::borrow::Cow;

                // TODO: use a &'static [] if possible

                Cow::Owned(vec![
                    $(
                        (
                            Cow::Borrowed(stringify!($field_name)),
                            {
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0usize) };
                                let dummy_field = &(dummy.0).$field_name;
                                let dummy_field: usize = unsafe { ::std::mem::transmute(dummy_field) };
                                dummy_field
                            },
                            {
                                fn attr_type_of_val<T: glium::vertex::Attribute>(_: &T)
                                    -> glium::vertex::AttributeType
                                {
                                    <T as glium::vertex::Attribute>::get_type()
                                }
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0usize) };
                                attr_type_of_val(&(dummy.0).$field_name)
                            },
                        )
                    ),+
                ])
            }
        }
    );

    ($struct_name:ident, $($field_name:ident),+,) => (
        implement_wrapped_vertex!($struct_name, $($field_name),+);
    );
}

// --------------
// Display
// --------------

#[derive(Clone)]
pub struct Display(glium::Display);

impl Display {
    pub fn new(descriptor: core::DisplayInfo) -> Display {
        let mut builder = glium::glutin::WindowBuilder::new()
            .with_dimensions(descriptor.width, descriptor.height)
            .with_title(descriptor.title)
            .with_transparency(descriptor.transparent)
            .with_decorations(descriptor.decorations)
            .with_visibility(descriptor.visible);

        match descriptor.monitor {
            Some(monitor) => {
                builder = builder.with_fullscreen(monitor.inner().inner().unwrap());
            }
            None => {}
        }

        if descriptor.vsync {
            builder = builder.with_vsync();
        }

        Display(builder.build_glium().unwrap())
    }
    pub fn draw(self: &Self) -> Frame {
        Frame(self.0.draw())
    }
    pub fn framebuffer_dimensions(self: &Self) -> Point2<u32> {
        self.0.get_framebuffer_dimensions().into()
    }
    pub fn window_dimensions(self: &Self) -> Point2<u32> {
        self.0.get_window().unwrap().get_inner_size_pixels().unwrap_or((0, 0)).into()
    }
    pub fn set_cursor_position(self: &Self, position: Point2<i32>) {
        self.0.get_window().unwrap().set_cursor_position(position.0, position.1).unwrap();
    }
    pub fn set_cursor_state(self: &Self, state: core::display::CursorState) {
        use core::display::CursorState as CS;
        self.0.get_window().unwrap().set_cursor_state(match state {
            CS::Normal => glium::glutin::CursorState::Normal,
            CS::Hide => glium::glutin::CursorState::Hide,
            CS::Grab => glium::glutin::CursorState::Grab,
        }).unwrap();
    }
    pub fn poll_events(self: &Self) -> EventIterator {
        EventIterator {
            it: self.0.poll_events(),
        }
    }
    pub fn show(self: &Self) {
        self.0.get_window().unwrap().show();
    }
    pub fn hide(self: &Self) {
        self.0.get_window().unwrap().hide()
    }
    pub fn set_title(self: &Self, title: &str) {
        self.0.get_window().unwrap().set_title(title);
    }
}

// --------------
// Display
// --------------

pub struct Frame(glium::Frame);

impl Frame {
    pub fn clear(self: &mut Self, color: core::Color) {
        let core::Color(r, g, b, a) = color;
        self.0.clear_color(r, g, b, a);
    }
    pub fn swap(self: Self) {
        self.0.finish().unwrap();
    }

    /// Copies given texture to given display.
    pub fn copy_from_texture(self: &Self, source: &core::Texture, filter: core::TextureFilter) {
        core::texture::handle(source).0.as_surface().fill(&self.0, magnify_filter(filter));
    }

    /// Copies the source rectangle to the target rectangle on the given display.
    pub fn copy_rect(self: &Self, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: core::TextureFilter) {
        let height = self.0.get_dimensions().1;
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, height, target_rect, height);
        self.0.blit_color(&glium_src_rect, &self.0, &glium_target_rect, magnify_filter(filter));
    }

    /// Copies the source rectangle from the given texture to the target rectangle on the given display.
    pub fn copy_rect_from_texture(self: &Self, source: &core::Texture, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: core::TextureFilter) {
        let target_height = self.0.get_dimensions().1;
        let source_height = core::texture::handle(source).0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        core::texture::handle(source).0.as_surface().blit_color(&glium_src_rect, &self.0, &glium_target_rect, magnify_filter(filter));
    }
}

// --------------
// Program
// --------------

pub struct Program(glium::Program);

impl Program {
    /// Creates a shader program from given vertex- and fragment-shader sources.
    pub fn new(display: &Display, vertex_shader: &str, fragment_shader: &str) -> core::Result<Program> {
        use glium::program::ProgramCreationError;
        use core::Error;
        match glium::Program::from_source(&display.0, vertex_shader, fragment_shader, None) {
            Err(ProgramCreationError::CompilationError(message)) => { Err(Error::ShaderError(format!("Shader compilation failed with: {}", message))) }
            Err(ProgramCreationError::LinkingError(message))     => { Err(Error::ShaderError(format!("Shader linking failed with: {}", message))) }
            Err(_)                                               => { Err(Error::ShaderError("No shader support found".to_string())) }
            Ok(program)                                          => { Ok(Program(program)) }
        }
    }
}

// --------------
// Monitor
// --------------

// #[derive(Clone)] // see inner
pub struct Monitor(glium::glutin::MonitorId);

impl Clone for Monitor {
    fn clone(&self) -> Monitor {
        Monitor(self.inner().unwrap())
    }
}

impl Monitor {
    pub fn get_dimensions(self: &Self) -> (u32, u32) {
        self.0.get_dimensions()
    }
    pub fn get_name(self: &Self) -> Option<String> {
        self.0.get_name()
    }
    pub fn inner(self: &Self) -> Option<glium::glutin::MonitorId> { // !todo non-pub

        // MonitorId is not currently cloneable as of winit-0.6.4, but a pull request was recently merged, so this hack can probably be removed for winit > 0.6.4
        let iter = glium::glutin::get_available_monitors();
        for monitor in iter {
            if monitor.get_native_identifier() == self.0.get_native_identifier() {
                return Some(monitor);
            }
        }
        None
    }
}

pub struct MonitorIterator(glium::glutin::AvailableMonitorsIter);

impl MonitorIterator {
    pub fn new() -> Self {
        MonitorIterator(glium::glutin::get_available_monitors())
    }
}

impl Iterator for MonitorIterator {
    type Item = Monitor;
    fn next(&mut self) -> Option<Monitor> {
        let current = self.0.next();
        match current {
            Some(monitor) => Some(Monitor(monitor)),
            None => None,
        }
    }
}

// --------------
// Texture2d
// --------------

pub struct Texture2d(glium::texture::Texture2d);

impl Texture2d {
    pub fn new(context: &mut core::RenderContextData, info: &core::TextureInfo) -> Texture2d {
        let texture = glium::texture::Texture2d::empty_with_format(
            &context.display.handle().0,
            Self::convert_format(info.format),
            glium::texture::MipmapsOption::NoMipmap,
            info.width,
            info.height,
        ).unwrap();
        texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
        Texture2d(texture)
    }
    /// Creates a new cache texture for the renderer.
    pub fn font_cache(display: &Display, width: u32, height: u32) -> Texture2d {
        Texture2d(glium::texture::Texture2d::with_format(
            &display.0,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; width as usize * height as usize]),
                width: width,
                height: height,
                format: glium::texture::ClientFormat::U8
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap
        ).unwrap())
    }
    pub fn clear(self: &Self, color: core::Color) {
        let core::Color(r, g, b, a) = color;
        self.0.as_surface().clear_color(r, g, b, a);
    }
    pub fn write(self: &Self, rect: &Rect<u32>, data: &Vec<u8>) {
        self.0.main_level().write(
            glium::Rect {
                left: (rect.0).0,
                bottom: (rect.0).1,
                width: (rect.1).0 - (rect.0).0, // !todo Rect is terrible
                height: (rect.1).1 - (rect.0).1,
            },
            glium::texture::RawImage2d {
                data: Cow::Borrowed(&data),
                width: (rect.1).0 - (rect.0).0,
                height: (rect.1).1 - (rect.0).1,
                format: glium::texture::ClientFormat::U8
            }
        );
    }
    pub fn copy_from(self: &Self, src_texture: &Texture2d, filter: core::TextureFilter) {
        src_texture.0.as_surface().fill(&self.0.as_surface(), magnify_filter(filter))
    }
    pub fn copy_rect_from(self: &Self, src_texture: &Texture2d, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: core::TextureFilter) {
        let target_height = self.0.height();
        let source_height = src_texture.0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        src_texture.0.as_surface().blit_color(&glium_src_rect, &self.0.as_surface(), &glium_target_rect, magnify_filter(filter));
    }
    pub fn copy_from_frame(self: &Self, src_frame: &Frame, filter: core::TextureFilter) {
        src_frame.0.fill(&self.0.as_surface(), magnify_filter(filter));
    }
    pub fn copy_rect_from_frame(self: &Self, src_frame: &Frame, source_rect: Rect<i32>, target_rect: Rect<i32>, filter: core::TextureFilter) {
        let source_height = src_frame.0.get_dimensions().1;
        let target_height = self.0.height();
        let (glium_src_rect, glium_target_rect) = blit_coords(source_rect, source_height, target_rect, target_height);
        src_frame.0.blit_color(&glium_src_rect, &self.0.as_surface(), &glium_target_rect, magnify_filter(filter));
    }

    /// Converts TextureFormat to the supported gliums texture formats
    fn convert_format(format: core::TextureFormat) -> glium::texture::UncompressedFloatFormat {
        use glium::texture::UncompressedFloatFormat as GF;
        use core::TextureFormat as RF;
        match format {
            RF::U8              => GF::U8,
            RF::U16             => GF::U16,
            RF::U8U8            => GF::U8U8,
            RF::U16U16          => GF::U16U16,
            RF::U10U10U10       => GF::U10U10U10,
            RF::U12U12U12       => GF::U12U12U12,
            RF::U16U16U16       => GF::U16U16U16,
            RF::U2U2U2U2        => GF::U2U2U2U2,
            RF::U4U4U4U4        => GF::U4U4U4U4,
            RF::U5U5U5U1        => GF::U5U5U5U1,
            RF::U8U8U8U8        => GF::U8U8U8U8,
            RF::U10U10U10U2     => GF::U10U10U10U2,
            RF::U12U12U12U12    => GF::U12U12U12U12,
            RF::U16U16U16U16    => GF::U16U16U16U16,
            RF::I16I16I16I16    => GF::I16I16I16I16,
            RF::F16             => GF::F16,
            RF::F16F16          => GF::F16F16,
            RF::F16F16F16F16    => GF::F16F16F16F16,
            RF::F32             => GF::F32,
            RF::F32F32          => GF::F32F32,
            RF::F32F32F32F32    => GF::F32F32F32F32,
            RF::F11F11F10       => GF::F11F11F10,
        }
    }
}

// --------------
// Texture2dArray
// --------------

#[derive(Clone)]
struct RawFrame(core::RawFrame);

impl<'a> glium::texture::Texture2dDataSource<'a> for RawFrame {
    type Data = u8;
    fn into_raw(self: Self) -> glium::texture::RawImage2d<'a, Self::Data> {
        glium::texture::RawImage2d {
            data: Cow::Owned(self.0.data),
            width: self.0.width,
            height: self.0.height,
            format: glium::texture::ClientFormat::U8U8U8U8,
        }
    }
}

pub struct Texture2dArray(glium::texture::Texture2dArray);

impl Texture2dArray {
    /// Generates glium texture array from given vector of textures
    pub fn new(display: &Display, raw: &Vec<core::RawFrame>) -> Self {

        use glium::texture;
        use std::mem::transmute;

        let raw_wrapped: Vec<RawFrame> = unsafe { transmute(raw.clone()) };

        Texture2dArray(
            if raw_wrapped.len() > 0 {
                texture::Texture2dArray::with_format(&display.0, raw_wrapped, texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap).unwrap()
            } else {
                texture::Texture2dArray::empty_with_format(&display.0, texture::UncompressedFloatFormat::U8U8U8U8, texture::MipmapsOption::NoMipmap, 2, 2, 1).unwrap()
            }
        )
    }
}

// --------------
// Context
// --------------

struct VertexBufferCacheItem {
    hint: usize,
    age: usize,
    buffer: glium::VertexBuffer<Vertex>,
}

pub struct Context {
    display         : glium::Display,
    index_buffer    : glium::IndexBuffer<u32>,
    vertex_buffers  : Vec<VertexBufferCacheItem>,
}

impl Context {
    pub fn new(display: &Display, initial_capacity: usize) -> Self {
        Context {
            display: display.0.clone(),
            index_buffer: Self::create_index_buffer(&display.0, initial_capacity),
            vertex_buffers: Vec::new(),
        }
    }

    fn create_index_buffer(display: &glium::Display, max_sprites: usize) -> glium::IndexBuffer<u32> {

        let mut data = Vec::with_capacity(max_sprites as usize * 6);

        for i in 0..max_sprites {
            let num = i as u32;
            data.push(num * 4);
            data.push(num * 4 + 1);
            data.push(num * 4 + 2);
            data.push(num * 4 + 1);
            data.push(num * 4 + 3);
            data.push(num * 4 + 2);
        }

        glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &data).unwrap()
    }

    /// Update index buffer to given size
    fn update_index_buffer(self: &mut Self, max_sprites: usize) {
        if max_sprites * 6 > self.index_buffer.len() {
            self.index_buffer = Self::create_index_buffer(&self.display, max_sprites);
        }
    }

    fn select_vertex_buffer(self: &mut Self, buffer_hint: usize, num_vertices: usize) -> (usize, bool) {

        const MAX_BUFFERS: usize = 10;

        for buffer in self.vertex_buffers.iter_mut() {
            buffer.age += 1;
        }

        if let Some(id) = self.vertex_buffers.iter().position(|ref item| item.hint == buffer_hint && item.buffer.len() >= num_vertices) {
            self.vertex_buffers[id].age = 0;
            (id, false)
        } else if self.vertex_buffers.len() < MAX_BUFFERS {
            self.vertex_buffers.push(VertexBufferCacheItem {
                hint: buffer_hint,
                age: 0,
                buffer: if buffer_hint == 0 {
                    glium::VertexBuffer::empty(&self.display, num_vertices).unwrap()
                } else {
                    glium::VertexBuffer::empty_dynamic(&self.display, num_vertices).unwrap()
                }
            });
            (self.vertex_buffers.len() - 1, true)
        } else {
            if let Some((id, _)) = self.vertex_buffers.iter().enumerate().max_by(|&(_, a), &(_, b)| a.age.cmp(&b.age)) {
                self.vertex_buffers[id].age = 0;
                (id, true)
            } else {
                (1, true)
            }
        }
    }

    fn draw(self: &mut Self, target: &core::RenderTarget, vertices: &[Vertex], dirty: bool, buffer_hint: usize, program: &Program, uniforms: &GliumUniformList, blendmode: &core::BlendMode) {

        let num_vertices = vertices.len();
        let num_sprites = num_vertices / 4;

        // set up vertex buffer

        let (vb_index, vb_dirty) = self.select_vertex_buffer(buffer_hint, num_vertices);
        {
            if dirty || vb_dirty {
                let vb_slice = self.vertex_buffers[vb_index].buffer.slice(0 .. num_vertices).unwrap();
                vb_slice.write(&vertices[0 .. num_vertices]);
            }
        }

        // set up index buffer

        self.update_index_buffer(num_sprites);
        let ib_slice = self.index_buffer.slice(0..num_vertices as usize / 4 * 6).unwrap();

        // set up draw parameters for given blend options
        let draw_parameters = glium::draw_parameters::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            blend           : glium_blendmode(blendmode),
            .. Default::default()
        };

        // draw

        match *target {
            core::RenderTarget::Display(ref display) => {
                display.frame.borrow_mut().as_mut().unwrap().0.draw(&self.vertex_buffers[vb_index].buffer, &ib_slice, &program.0, uniforms, &draw_parameters).unwrap()
            }
            core::RenderTarget::Texture(ref texture) => {
                texture.handle.0.as_surface().draw(&self.vertex_buffers[vb_index].buffer, &ib_slice, &program.0, uniforms, &draw_parameters).unwrap();
            }
            core::RenderTarget::None => { }
        }
    }
}

// --------------
// Uniforms
// --------------

enum GliumUniform<'a> {
    Bool(bool),
    SignedInt(i32),
    UnsignedInt(u32),
    Float(f32),
    Mat4([[f32; 4]; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Double(f64),
    DoubleMat4([[f64; 4]; 4]),
    DoubleVec2([f64; 2]),
    DoubleVec3([f64; 3]),
    DoubleVec4([f64; 4]),
    Texture2d(&'a glium::texture::Texture2d),
    Texture2dArray(&'a glium::texture::Texture2dArray),
    Sampled2d(glium::uniforms::Sampler<'a, glium::texture::Texture2d>),
}

/// A structure to implement gliums Uniforms trait on.
struct GliumUniformList<'a>(Vec<(&'a str, GliumUniform<'a>)>);

impl<'a> GliumUniformList<'a> {
    pub fn from_uniform_list(list: &'a core::UniformList) -> Self {
        let mut result = GliumUniformList(Vec::new());
        for (name, uniform) in list.0.iter() {
            result.add_uniform(name, uniform);
        }
        result
    }
    pub fn add(self: &mut Self, name: &'a str, uniform: GliumUniform<'a>) -> &mut Self {
        self.0.push((name, uniform));
        self
    }
    fn add_uniform(self: &mut Self, name: &'a str, uniform: &'a core::Uniform) {
        use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter, SamplerWrapFunction};
        use core::Uniform as CU;
        use core::TextureWrap as TW;
        self.0.push((name, match *uniform {
            CU::Bool(val) => { GliumUniform::Bool(val) },
            CU::SignedInt(val) => { GliumUniform::SignedInt(val) },
            CU::UnsignedInt(val) => { GliumUniform::UnsignedInt(val) },
            CU::Float(val) => { GliumUniform::Float(val) },
            CU::Mat4(val) => { GliumUniform::Mat4(val) },
            CU::Vec2(val) => { GliumUniform::Vec2(val) },
            CU::Vec3(val) => { GliumUniform::Vec3(val) },
            CU::Vec4(val) => { GliumUniform::Vec4(val) },
            CU::Double(val) => { GliumUniform::Double(val) },
            CU::DoubleMat4(val) => { GliumUniform::DoubleMat4(val) },
            CU::DoubleVec2(val) => { GliumUniform::DoubleVec2(val) },
            CU::DoubleVec3(val) => { GliumUniform::DoubleVec3(val) },
            CU::DoubleVec4(val) => { GliumUniform::DoubleVec4(val) },
            CU::Texture(ref val) => {
                let (minify, magnify, wrap) = core::texture::filters(val);
                let glium_minify = if minify == core::TextureFilter::Linear { MinifySamplerFilter::Linear } else { MinifySamplerFilter::Nearest };
                let glium_magnify = if magnify == core::TextureFilter::Linear { MagnifySamplerFilter::Linear } else { MagnifySamplerFilter::Nearest };
                let glium_wrap = match wrap {
                    TW::Repeat         => SamplerWrapFunction::Repeat,
                    TW::Mirror         => SamplerWrapFunction::Mirror,
                    TW::Clamp          => SamplerWrapFunction::Clamp,
                    TW::MirrorClamp    => SamplerWrapFunction::MirrorClamp,
                };
                GliumUniform::Sampled2d(
                    core::texture::handle(val).0
                                .sampled()
                                .minify_filter(glium_minify)
                                .magnify_filter(glium_magnify)
                                .wrap_function(glium_wrap)
                )
            },
        }));
    }
}

impl<'b> Uniforms for GliumUniformList<'b> {
    fn visit_values<'a, F>(self: &'a Self, mut output: F) where F: FnMut(&str, glium::uniforms::UniformValue<'a>) {
        use glium::uniforms::UniformValue;
        for &(name, ref uniform) in &self.0 {
            output(name, match *uniform {
                GliumUniform::Bool(val) => { UniformValue::Bool(val) },
                GliumUniform::SignedInt(val) => { UniformValue::SignedInt(val) },
                GliumUniform::UnsignedInt(val) => { UniformValue::UnsignedInt(val) },
                GliumUniform::Float(val) => { UniformValue::Float(val) },
                GliumUniform::Mat4(val) => { UniformValue::Mat4(val) },
                GliumUniform::Vec2(val) => { UniformValue::Vec2(val) },
                GliumUniform::Vec3(val) => { UniformValue::Vec3(val) },
                GliumUniform::Vec4(val) => { UniformValue::Vec4(val) },
                GliumUniform::Double(val) => { UniformValue::Double(val) },
                GliumUniform::DoubleMat4(val) => { UniformValue::DoubleMat4(val) },
                GliumUniform::DoubleVec2(val) => { UniformValue::DoubleVec2(val) },
                GliumUniform::DoubleVec3(val) => { UniformValue::DoubleVec3(val) },
                GliumUniform::DoubleVec4(val) => { UniformValue::DoubleVec4(val) },
                GliumUniform::Sampled2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2d(ref val) => {
                    val.as_uniform_value()
                }
                GliumUniform::Texture2dArray(ref val) => {
                    val.as_uniform_value()
                }
            });
        }
    }
}

// --------------
// Vertex
// --------------

#[derive(Copy, Clone)]
struct Vertex(core::Vertex);

implement_wrapped_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv, components);

// --------------
// Drawing
// --------------

pub fn draw_layer(target: &core::RenderTarget, program: &core::Program, context: &mut core::RenderContextData, layer: &core::Layer, component: u32) {

    use glium::uniforms::{MagnifySamplerFilter, SamplerWrapFunction};
    use std::mem::transmute;

    let mut glium_uniforms = GliumUniformList::from_uniform_list(&program.uniforms);
    glium_uniforms
        .add("u_view", GliumUniform::Mat4(layer.view_matrix().deref().into()))
        .add("u_model", GliumUniform::Mat4(layer.model_matrix().deref().into()))
        .add("_rd_color", GliumUniform::Vec4(layer.color().deref().into()))
        .add("_rd_tex", GliumUniform::Sampled2d(context.font_texture.0.sampled().magnify_filter(MagnifySamplerFilter::Nearest).wrap_function(SamplerWrapFunction::Clamp)))
        .add("_rd_comp", GliumUniform::UnsignedInt(component))
        .add("_rd_tex1", GliumUniform::Texture2dArray(&context.tex_arrays[1].data.deref().0))
        .add("_rd_tex2", GliumUniform::Texture2dArray(&context.tex_arrays[2].data.deref().0))
        .add("_rd_tex3", GliumUniform::Texture2dArray(&context.tex_arrays[3].data.deref().0))
        .add("_rd_tex4", GliumUniform::Texture2dArray(&context.tex_arrays[4].data.deref().0))
        .add("_rd_tex5", GliumUniform::Texture2dArray(&context.tex_arrays[5].data.deref().0));

    let vertices = core::vertices(layer);
    let vertices = vertices.deref();

    context.backend_context.draw(target, unsafe { transmute(vertices) }, core::layer_undirty(layer), core::layer_id(layer), core::program::sprite(program), &glium_uniforms, &layer.blendmode());
}

pub fn draw_rect(target: &core::RenderTarget, program: &core::Program, context: &mut core::RenderContextData, blend: core::BlendMode, info: core::DrawRectInfo, view_matrix: Mat4, model_matrix: Mat4, color: core::Color, texture: &core::Texture) {

    use std::mem::transmute;

    // set up uniforms !todo FRONTEND

    let mut glium_uniforms = GliumUniformList::from_uniform_list(&program.uniforms);
    glium_uniforms
        .add("u_view", GliumUniform::Mat4(view_matrix.into()))
        .add("u_model", GliumUniform::Mat4(model_matrix.into()))
        .add("_rd_color", GliumUniform::Vec4(color.into()))
        .add("_rd_tex", GliumUniform::Texture2d(&core::texture::handle(texture).0))
        .add("_rd_offset", GliumUniform::Vec2(info.rect.0.into()))
        .add("_rd_dimensions", GliumUniform::Vec2(info.rect.1.into()))
        .add("_rd_has_tex", GliumUniform::Bool(info.texture.is_some()));

    let vertices = &context.single_rect;
    let vertices = &vertices[..];

    context.backend_context.draw(target, unsafe { transmute(vertices) }, false, 0, core::program::texture(program), &glium_uniforms, &blend);
}

// --------------
// Blending
// --------------

#[inline(always)]
fn glium_blendmode(blendmode: &core::BlendMode) -> glium::Blend {
    glium::Blend {
        color: blendfunc(blendmode.color),
        alpha: blendfunc(blendmode.alpha),
        constant_value: blendmode.constant_value.into(),
    }
}

#[inline(always)]
fn blendfunc(function: core::BlendingFunction) -> glium::BlendingFunction {
    use core::BlendingFunction as CF;
    use glium::BlendingFunction as GF;
    match function {
        CF::AlwaysReplace                               => GF::AlwaysReplace,
        CF::Min                                         => GF::Min,
        CF::Max                                         => GF::Max,
        CF::Addition { source, destination }            => GF::Addition { source: blendfactor(source), destination: blendfactor(destination) },
        CF::Subtraction { source, destination }         => GF::Subtraction { source: blendfactor(source), destination: blendfactor(destination) },
        CF::ReverseSubtraction { source, destination }  => GF::Subtraction { source: blendfactor(source), destination: blendfactor(destination) },
    }
}

#[inline(always)]
fn blendfactor(factor: core::LinearBlendingFactor) -> glium::LinearBlendingFactor {
    use core::LinearBlendingFactor as CB;
    use glium::LinearBlendingFactor as GB;
    match factor {
        CB::Zero                      => GB::Zero,
        CB::One                       => GB::One,
        CB::SourceColor               => GB::SourceColor,
        CB::OneMinusSourceColor       => GB::OneMinusSourceColor,
        CB::DestinationColor          => GB::DestinationColor,
        CB::OneMinusDestinationColor  => GB::OneMinusDestinationColor,
        CB::SourceAlpha               => GB::SourceAlpha,
        CB::OneMinusSourceAlpha       => GB::OneMinusSourceAlpha,
        CB::DestinationAlpha          => GB::DestinationAlpha,
        CB::OneMinusDestinationAlpha  => GB::OneMinusDestinationAlpha,
        CB::SourceAlphaSaturate       => GB::SourceAlphaSaturate,
        CB::ConstantColor             => GB::ConstantColor,
        CB::OneMinusConstantColor     => GB::OneMinusConstantColor,
        CB::ConstantAlpha             => GB::ConstantAlpha,
        CB::OneMinusConstantAlpha     => GB::OneMinusConstantAlpha,
    }
}

// --------------
// Misc
// --------------

fn blit_coords(source_rect: Rect<i32>, source_height: u32, target_rect: Rect<i32>, target_height: u32) -> (glium::Rect, glium::BlitTarget) {
    (glium::Rect {
        left: (source_rect.0).0 as u32,
        bottom: (source_height as i32 - (source_rect.1).1 as i32 - (source_rect.0).1 as i32) as u32,
        width: (source_rect.1).0 as u32,
        height: (source_rect.1).1 as u32,
    },
    glium::BlitTarget {
        left: (target_rect.0).0 as u32,
        bottom: (target_height as i32 - (target_rect.1).1 as i32 - (target_rect.0).1 as i32) as u32,
        width: (target_rect.1).0 as i32,
        height: (target_rect.1).1 as i32,
    })
}

fn magnify_filter(filter: core::TextureFilter) -> glium::uniforms::MagnifySamplerFilter {
    if filter == core::TextureFilter::Linear {
        glium::uniforms::MagnifySamplerFilter::Linear
    } else {
        glium::uniforms::MagnifySamplerFilter::Nearest
    }
}

// --------------
// Input Events
// --------------

pub enum Event {
    KeyboardInput(usize, bool),
    MouseInput(usize, bool),
    MouseMoved(i32, i32),
    Focused,
    Closed,
}

impl Event {
    fn map_key_code(key: glium::glutin::VirtualKeyCode) -> core::InputId {
        use glium::glutin::VirtualKeyCode as VK;
        use core::InputId as IID;
        match key {
            VK::Key1          => IID::Key1,
            VK::Key2          => IID::Key2,
            VK::Key3          => IID::Key3,
            VK::Key4          => IID::Key4,
            VK::Key5          => IID::Key5,
            VK::Key6          => IID::Key6,
            VK::Key7          => IID::Key7,
            VK::Key8          => IID::Key8,
            VK::Key9          => IID::Key9,
            VK::Key0          => IID::Key0,
            VK::A             => IID::A,
            VK::B             => IID::B,
            VK::C             => IID::C,
            VK::D             => IID::D,
            VK::E             => IID::E,
            VK::F             => IID::F,
            VK::G             => IID::G,
            VK::H             => IID::H,
            VK::I             => IID::I,
            VK::J             => IID::J,
            VK::K             => IID::K,
            VK::L             => IID::L,
            VK::M             => IID::M,
            VK::N             => IID::N,
            VK::O             => IID::O,
            VK::P             => IID::P,
            VK::Q             => IID::Q,
            VK::R             => IID::R,
            VK::S             => IID::S,
            VK::T             => IID::T,
            VK::U             => IID::U,
            VK::V             => IID::V,
            VK::W             => IID::W,
            VK::X             => IID::X,
            VK::Y             => IID::Y,
            VK::Z             => IID::Z,
            VK::Escape        => IID::Escape,
            VK::F1            => IID::F1,
            VK::F2            => IID::F2,
            VK::F3            => IID::F3,
            VK::F4            => IID::F4,
            VK::F5            => IID::F5,
            VK::F6            => IID::F6,
            VK::F7            => IID::F7,
            VK::F8            => IID::F8,
            VK::F9            => IID::F9,
            VK::F10           => IID::F10,
            VK::F11           => IID::F11,
            VK::F12           => IID::F12,
            VK::F13           => IID::F13,
            VK::F14           => IID::F14,
            VK::F15           => IID::F15,
            VK::Snapshot      => IID::Snapshot,
            VK::Scroll        => IID::Scroll,
            VK::Pause         => IID::Pause,
            VK::Insert        => IID::Insert,
            VK::Home          => IID::Home,
            VK::Delete        => IID::Delete,
            VK::End           => IID::End,
            VK::PageDown      => IID::PageDown,
            VK::PageUp        => IID::PageUp,
            VK::Left          => IID::CursorLeft,
            VK::Up            => IID::CursorUp,
            VK::Right         => IID::CursorRight,
            VK::Down          => IID::CursorDown,
            VK::Back          => IID::Backspace,
            VK::Return        => IID::Return,
            VK::Space         => IID::Space,
            VK::Numlock       => IID::Numlock,
            VK::Numpad0       => IID::Numpad0,
            VK::Numpad1       => IID::Numpad1,
            VK::Numpad2       => IID::Numpad2,
            VK::Numpad3       => IID::Numpad3,
            VK::Numpad4       => IID::Numpad4,
            VK::Numpad5       => IID::Numpad5,
            VK::Numpad6       => IID::Numpad6,
            VK::Numpad7       => IID::Numpad7,
            VK::Numpad8       => IID::Numpad8,
            VK::Numpad9       => IID::Numpad9,
            VK::AbntC1        => IID::AbntC1,
            VK::AbntC2        => IID::AbntC2,
            VK::Add           => IID::Add,
            VK::Apostrophe    => IID::Apostrophe,
            VK::Apps          => IID::Apps,
            VK::At            => IID::At,
            VK::Ax            => IID::Ax,
            VK::Backslash     => IID::Backslash,
            VK::Calculator    => IID::Calculator,
            VK::Capital       => IID::Capital,
            VK::Colon         => IID::Colon,
            VK::Comma         => IID::Comma,
            VK::Convert       => IID::Convert,
            VK::Decimal       => IID::Decimal,
            VK::Divide        => IID::Divide,
            VK::Equals        => IID::Equals,
            VK::Grave         => IID::Grave,
            VK::Kana          => IID::Kana,
            VK::Kanji         => IID::Kanji,
            VK::LAlt          => IID::LAlt,
            VK::LBracket      => IID::LBracket,
            VK::LControl      => IID::LControl,
            VK::LMenu         => IID::LMenu,
            VK::LShift        => IID::LShift,
            VK::LWin          => IID::LWin,
            VK::Mail          => IID::Mail,
            VK::MediaSelect   => IID::MediaSelect,
            VK::MediaStop     => IID::MediaStop,
            VK::Minus         => IID::Minus,
            VK::Multiply      => IID::Multiply,
            VK::Mute          => IID::Mute,
            VK::MyComputer    => IID::MyComputer,
            VK::NextTrack     => IID::NextTrack,
            VK::NoConvert     => IID::NoConvert,
            VK::NumpadComma   => IID::NumpadComma,
            VK::NumpadEnter   => IID::NumpadEnter,
            VK::NumpadEquals  => IID::NumpadEquals,
            VK::OEM102        => IID::OEM102,
            VK::Period        => IID::Period,
            VK::PlayPause     => IID::PlayPause,
            VK::Power         => IID::Power,
            VK::PrevTrack     => IID::PrevTrack,
            VK::RAlt          => IID::RAlt,
            VK::RBracket      => IID::RBracket,
            VK::RControl      => IID::RControl,
            VK::RMenu         => IID::RMenu,
            VK::RShift        => IID::RShift,
            VK::RWin          => IID::RWin,
            VK::Semicolon     => IID::Semicolon,
            VK::Slash         => IID::Slash,
            VK::Sleep         => IID::Sleep,
            VK::Stop          => IID::Stop,
            VK::Subtract      => IID::Subtract,
            VK::Sysrq         => IID::Sysrq,
            VK::Tab           => IID::Tab,
            VK::Underline     => IID::Underline,
            VK::Unlabeled     => IID::Unlabeled,
            VK::VolumeDown    => IID::VolumeDown,
            VK::VolumeUp      => IID::VolumeUp,
            VK::Wake          => IID::Wake,
            VK::WebBack       => IID::WebBack,
            VK::WebFavorites  => IID::WebFavorites,
            VK::WebForward    => IID::WebForward,
            VK::WebHome       => IID::WebHome,
            VK::WebRefresh    => IID::WebRefresh,
            VK::WebSearch     => IID::WebSearch,
            VK::WebStop       => IID::WebStop,
            VK::Yen           => IID::Yen,
            VK::Compose       => IID::Compose,
            VK::NavigateForward => IID::NavigateForward,
            VK::NavigateBackward => IID::NavigateBackward,
        }
    }
}

pub struct EventIterator<'a> {
    it: glium::backend::glutin_backend::PollEventsIter<'a>,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;

    fn next(self: &mut Self) -> Option<Event> {
        use glium::glutin::{ElementState, MouseButton};
        use glium::glutin::Event as GlutinEvent;

        let event = self.it.next();

        if let Some(event) = event {
            match event {
                GlutinEvent::KeyboardInput(element_state, _, Some(virtual_code)) => {
                    let key_id = Event::map_key_code(virtual_code) as usize;
                    if key_id < core::NUM_KEYS {
                        Some(Event::KeyboardInput(key_id, element_state == ElementState::Pressed))
                    } else {
                        None
                    }
                },
                GlutinEvent::MouseMoved(x, y) => {
                    Some(Event::MouseMoved(x, y))
                },
                GlutinEvent::MouseInput(element_state, button) => {
                    let button_id = match button {
                        MouseButton::Left => 0,
                        MouseButton::Middle => 1,
                        MouseButton::Right => 2,
                        MouseButton::Other(x) => (x - 1) as usize,
                    };
                    if button_id < core::NUM_BUTTONS {
                        Some(Event::MouseInput(button_id, element_state == ElementState::Pressed))
                    } else {
                        None
                    }
                },
                GlutinEvent::Focused(true) => {
                    Some(Event::Focused)
                }
                GlutinEvent::Closed => {
                    Some(Event::Closed)
                }
                _ => None
            }
        } else {
            None
        }
    }
}
