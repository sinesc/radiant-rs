use glium;
use prelude::*;
use misc::AVec;
use maths::Mat4;
use core::{blendmodes, BlendMode, Point, Rect, RenderContextData, Color, display};

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    position    : [f32; 2],
    offset      : [f32; 2],
    rotation    : f32,
    color       : Color,
    bucket_id   : u32,
    texture_id  : u32,
    texture_uv  : [f32; 2],
}
implement_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv);

/// A non-blocking, thread-safe drawing target.
///
/// In radiant_rs, all drawing happens on layers. Layers provide transformation capabilities in
/// the form of model- and view-matrices and the layer's blendmode and color determine
/// how sprites are rendered onto the display. Layers can be rendered multiple times using
/// different matrices, blendmodes or colors without having to redraw their contents first.
///
/// Multiple threads can draw onto the same layer without blocking. However, manipulating layer
/// properties may block other threads from manipulating the same property.
pub struct Layer {
    view_matrix     : Mutex<Mat4<f32>>,
    model_matrix    : Mutex<Mat4<f32>>,
    blend           : Mutex<BlendMode>,
    color           : Mutex<Color>,
    vertex_data     : AVec<Vertex>,
    vertex_buffer   : Mutex<Option<glium::VertexBuffer<Vertex>>>,
    dirty           : AtomicBool,
}
unsafe impl Send for Layer { }
unsafe impl Sync for Layer { }

impl Layer {

    /// Creates a new layer with given dimensions and object limit.
    pub fn new(width: u32, height: u32) -> Self {
        Layer {
            view_matrix     : Mutex::new(Mat4::<f32>::viewport(width as f32, height as f32)),
            model_matrix    : Mutex::new(Mat4::<f32>::identity()),
            blend           : Mutex::new(blendmodes::ALPHA),
            color           : Mutex::new(Color::white()),
            vertex_data     : AVec::new(1024 * 4),  // todo: "1024" add some sort of configurable?
            vertex_buffer   : Mutex::new(None),
            dirty           : AtomicBool::new(true),
        }
    }

    /// Sets a global color multiplicator. Setting this to white means that the layer contents
    /// are renderered in their original colors.
    ///
    /// Note that [`Color`](struct.Color.html)s contain
    /// alpha information and are not clamped to any range, so it is possible to use an overbright
    /// color to brighten the result or use the alpha channel to apply global transparency.
    pub fn set_color(&self, color: Color) -> &Self {
        self.color().set(color);
        self
    }

    /// Returns a mutex guarded mutable reference to the global color multiplicator.
    pub fn color(&self) -> MutexGuard<Color> {
        self.color.lock().unwrap()
    }

    /// Sets the view matrix.
    ///
    /// View matrix transformation is applied after the objects are fully positioned on the layer.
    /// As a result, manipulating the view matrix has the effect of manipulating the layer itself,
    /// e.g. rotating the entire layer.
    pub fn set_view_matrix(&self, matrix: Mat4<f32>) -> &Self {
        self.view_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the view matrix.
    /// See [`set_model_matrix()`](#method.set_model_matrix) for a description of the model matrix.
    pub fn view_matrix(&self) -> MutexGuard<Mat4<f32>> {
        self.view_matrix.lock().unwrap()
    }

    /// Sets the model matrix.
    ///
    /// Model matrix transformation is applied before each object is transformed to its position
    /// on the layer. As a result, manipulating the model matrix has the effect of manipulating
    /// every object on the layer in the same way, e.g. rotating every individual object on the
    /// layer around a point relative to the individual object.
    pub fn set_model_matrix(&self, matrix: Mat4<f32>) -> &Self {
        self.model_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the model matrix.
    /// See [`set_view_matrix()`](#method.set_view_matrix) for a description of the view matrix.
    pub fn model_matrix(&self) -> MutexGuard<Mat4<f32>> {
        self.model_matrix.lock().unwrap()
    }

    /// Sets the blendmode.
    pub fn set_blendmode(&self, blendmode: BlendMode) -> &Self {
        self.blendmode().set(blendmode);
        self
    }

    /// Returns a mutex guarded mutable reference to the blendmode.
    pub fn blendmode(&self) -> MutexGuard<BlendMode> {
        self.blend.lock().unwrap()
    }

    /// Removes all previously added object from the layer. Typically invoked after the layer has
    /// been rendered.
    pub fn clear(self: &Self) -> &Self {
        self.dirty.store(true, Ordering::Relaxed);
        self.vertex_data.clear();
        self
    }

    /// Returns the number of sprites the layer can hold without having to perform a blocking reallocation.
    pub fn capacity(self: &Self) -> usize {
        self.vertex_data.capacity() / 4
    }

    /// Returns the number of sprites currently stored the layer.
    pub fn len(self: &Self) -> usize {
        self.vertex_data.len() / 4
    }
}

/// Draws a rectangle on given layer
pub fn add_rect(layer: &Layer, bucket_id: u32, texture_id: u32, uv: Rect, pos: Point, anchor: Point, dim: Point, color: Color, rotation: f32, scale: Point) {

    layer.dirty.store(true, Ordering::Relaxed);

    // corner positions relative to x/y

    let anchor_x = anchor.x * dim.x;
    let anchor_y = anchor.y * dim.y;

    let offset_x0 = -anchor_x * scale.x;
    let offset_x1 = (dim.x - anchor_x) * scale.x;
    let offset_y0 = -anchor_y * scale.y;
    let offset_y1 = (dim.y - anchor_y) * scale.y;

    // get vertex_data slice and draw into it

    let map = layer.vertex_data.map(4);

    map.set(0, Vertex {
        position    : [pos.x, pos.y],
        offset      : [offset_x0, offset_y0],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [uv.0.x, uv.0.y],
    });

    map.set(1, Vertex {
        position    : [pos.x, pos.y],
        offset      : [offset_x1, offset_y0],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [uv.1.x, uv.0.y],
    });

    map.set(2, Vertex {
        position    : [pos.x, pos.y],
        offset      : [offset_x0, offset_y1],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [uv.0.x, uv.1.y],
    });

    map.set(3, Vertex {
        position    : [pos.x, pos.y],
        offset      : [offset_x1, offset_y1],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [uv.1.x, uv.1.y],
    });
}

/// Uploads vertex data to the vertex buffer and returns number of vertices uploaded and the mutex-guarded vertex-buffer.
pub fn upload<'a>(layer: &'a Layer, context: &RenderContextData) -> (MutexGuard<'a, Option<glium::VertexBuffer<Vertex>>>, usize) {

    // prepare vertexbuffer if not already done

    let mut vertex_buffer_guard = layer.vertex_buffer.lock().unwrap();

    let num_vertices = {
        let mut vertex_buffer = vertex_buffer_guard.deref_mut();

        // prepare vertexbuffer if not already done

        if vertex_buffer.is_none() {
            *vertex_buffer = Some(glium::VertexBuffer::empty_dynamic(display::handle(&context.display), layer.vertex_data.capacity()).unwrap());
        }

        // copy layer data to vertexbuffer

        if layer.dirty.swap(false, Ordering::Relaxed) {
            let vertex_data = layer.vertex_data.get();
            let num_vertices = vertex_data.len();
            if num_vertices > 0 {
                // resize as neccessary
                if num_vertices > vertex_buffer.as_ref().unwrap().len() {
                    *vertex_buffer = Some(glium::VertexBuffer::empty_dynamic(display::handle(&context.display), layer.vertex_data.capacity()).unwrap());
                }
                // copy data to buffer
                let vb_slice = vertex_buffer.as_ref().unwrap().slice(0 .. num_vertices).unwrap();
                vb_slice.write(&vertex_data[0 .. num_vertices]);
            }
            num_vertices
        } else {
            layer.vertex_data.len()
        }
    };

    (vertex_buffer_guard, num_vertices)
}
