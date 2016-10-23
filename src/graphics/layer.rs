use prelude::*;
use avec::AVec;
use maths::Mat4;
use color::Color;
use graphics::{blendmodes, BlendMode, Point, Rect};

pub use Layer;

impl Layer {

    /// Creates a new layer with given dimensions and object limit.
    pub fn new(max_sprites: u32, dimensions: (u32, u32)) -> Self {
        Layer {
            view_matrix     : Mutex::new(Mat4::<f32>::viewport(dimensions.0 as f32, dimensions.1 as f32)),
            model_matrix    : Mutex::new(Mat4::<f32>::identity()),
            blend           : Mutex::new(blendmodes::ALPHA),
            color           : Mutex::new(Color::white()),
            vertex_data     : AVec::new(max_sprites * 4),
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
}

/// draws a rectangle on given layer
pub fn add_rect(layer: &Layer, bucket_id: u32, texture_id: u32, uv: Rect, pos: Point, anchor: Point, dim: Point, color: Color, rotation: f32, scale: Point) {

    // get vertex_data slice and draw into it

    let mut guard = layer.vertex_data.map(4);
    let mut vertex = guard.deref_mut();
    layer.dirty.store(true, Ordering::Relaxed);

    // corner positions relative to x/y

    let anchor_x = anchor.x * dim.x;
    let anchor_y = anchor.y * dim.y;

    let offset_x0 = -anchor_x * scale.x;
    let offset_x1 = (dim.x - anchor_x) * scale.x;
    let offset_y0 = -anchor_y * scale.y;
    let offset_y1 = (dim.y - anchor_y) * scale.y;

    // fill vertex array

    vertex[0].position[0]   = pos.x;
    vertex[0].position[1]   = pos.y;
    vertex[0].offset[0]     = offset_x0;
    vertex[0].offset[1]     = offset_y0;
    vertex[0].rotation      = rotation;
    vertex[0].bucket_id     = bucket_id;
    vertex[0].texture_id    = texture_id;
    vertex[0].color         = color;
    vertex[0].texture_uv[0] = uv.0.x;
    vertex[0].texture_uv[1] = uv.0.y;

    vertex[1].position[0]   = pos.x;
    vertex[1].position[1]   = pos.y;
    vertex[1].offset[0]     = offset_x1;
    vertex[1].offset[1]     = offset_y0;
    vertex[1].rotation      = rotation;
    vertex[1].bucket_id     = bucket_id;
    vertex[1].texture_id    = texture_id;
    vertex[1].color         = color;
    vertex[1].texture_uv[0] = uv.1.x;
    vertex[1].texture_uv[1] = uv.0.y;

    vertex[2].position[0]   = pos.x;
    vertex[2].position[1]   = pos.y;
    vertex[2].offset[0]     = offset_x0;
    vertex[2].offset[1]     = offset_y1;
    vertex[2].rotation      = rotation;
    vertex[2].bucket_id     = bucket_id;
    vertex[2].texture_id    = texture_id;
    vertex[2].color         = color;
    vertex[2].texture_uv[0] = uv.0.x;
    vertex[2].texture_uv[1] = uv.1.y;

    vertex[3].position[0]   = pos.x;
    vertex[3].position[1]   = pos.y;
    vertex[3].offset[0]     = offset_x1;
    vertex[3].offset[1]     = offset_y1;
    vertex[3].rotation      = rotation;
    vertex[3].bucket_id     = bucket_id;
    vertex[3].texture_id    = texture_id;
    vertex[3].color         = color;
    vertex[3].texture_uv[0] = uv.1.x;
    vertex[3].texture_uv[1] = uv.1.y;
}
