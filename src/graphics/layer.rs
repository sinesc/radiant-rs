use prelude::*;
use avec::AVec;
use maths::{Vec3, Mat4};
use color::Color;
use graphics::{blendmodes, font, BlendMode, Point, Rect};

static LAYER_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
pub use Layer;

impl Layer {

    /// creates a new layer with given dimensions and sprite limit
    pub fn new(max_sprites: u32, dimensions: (u32, u32)) -> Self {
        Layer {
            view_matrix     : Mutex::new(Self::viewport_matrix(dimensions.0, dimensions.1)),
            model_matrix    : Mutex::new(Mat4::<f32>::identity()),
            blend           : Mutex::new(blendmodes::ALPHA),
            color           : Mutex::new(Color::white()),
            gid             : LAYER_COUNTER.fetch_add(1, Ordering::Relaxed),
            lid             : ATOMIC_USIZE_INIT,
            vertex_data     : AVec::new(max_sprites * 4),
            font_cache      : Mutex::new(font::FontCache::new(512, 512, 0.01, 0.01)),
        }
    }

    /// sets global color multiplicator
    pub fn set_color(&self, color: Color) -> &Self {
        self.color().set(color);
        self
    }

    /// returns a mutex guarded mutable reference to the global color multiplicator
    pub fn color(&self) -> MutexGuard<Color> {
        self.color.lock().unwrap()
    }

    /// sets the view matrix
    pub fn set_view_matrix(&self, matrix: Mat4<f32>) -> &Self {
        self.view_matrix().set(matrix);
        self
    }

    /// returns a mutex guarded mutable reference to the view matrix
    pub fn view_matrix(&self) -> MutexGuard<Mat4<f32>> {
        self.view_matrix.lock().unwrap()
    }

    /// sets the model matrix
    pub fn set_model_matrix(&self, matrix: Mat4<f32>) -> &Self {
        self.model_matrix().set(matrix);
        self
    }

    /// returns a mutex guarded mutable reference to the model matrix
    pub fn model_matrix(&self) -> MutexGuard<Mat4<f32>> {
        self.model_matrix.lock().unwrap()
    }

    /// sets the blendmode
    pub fn set_blendmode(&self, blendmode: BlendMode) -> &Self {
        self.blendmode().set(blendmode);
        self
    }

    /// returns a mutex guarded mutable reference to the blendmode
    pub fn blendmode(&self) -> MutexGuard<BlendMode> {
        self.blend.lock().unwrap()
    }

    /// removes previously added sprites from the drawing queue. typically invoked after draw()
    pub fn clear(self: &Self) -> &Self {

        // increase local part of hash to mark this layer as modified against cached state in Renderer
        self.lid.fetch_add(1, Ordering::Relaxed);
        self.vertex_data.clear();
        self
    }

    /// compute the default view matrix
    fn viewport_matrix(width: u32, height: u32) -> Mat4<f32> {
        let mut matrix = Mat4::<f32>::identity();
        *matrix
            .translate(Vec3(-1.0, 1.0, 0.0))
            .scale(Vec3(2.0 / width as f32, -2.0 / height as f32, 1.0))
    }
}

/// draws a rectangle on given layer
pub fn add_rect(layer: &Layer, bucket_id: u32, texture_id: u32, uv: Rect, pos: Point, anchor: Point, dim: Point, color: Color, rotation: f32, scale: Point) {

    // increase local part of hash to mark this layer as modified against cached state in Renderer

    layer.lid.fetch_add(1, Ordering::Relaxed);

    // get vertex_data slice and draw into it

    let mut guard = layer.vertex_data.map(4);
    let mut vertex = guard.deref_mut();

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
