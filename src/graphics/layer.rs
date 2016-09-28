use prelude::*;
use avec::AVec;
use maths::{Vec3, Mat4};
use color::Color;
use graphics::{Sprite, blendmodes, BlendMode};

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

    /// adds a sprite to the layer
    pub fn sprite(&self, sprite: Sprite, frame_id: u32, x: u32, y: u32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {

        // increase local part of hash to mark this layer as modified against cached state in Renderer
        self.lid.fetch_add(1, Ordering::Relaxed);

        let texture_id = sprite.texture_id(frame_id);
        let bucket_id = sprite.bucket_id();

        // corner positions relative to x/y

        let x = x as f32;
        let y = y as f32;
        let anchor_x = sprite.anchor_x * sprite.width() as f32;
        let anchor_y = sprite.anchor_y * sprite.height() as f32;

        let offset_x0 = -anchor_x * scale_x;
        let offset_x1 = (sprite.width() as f32 - anchor_x) * scale_x;
        let offset_y0 = -anchor_y * scale_y;
        let offset_y1 = (sprite.height() as f32 - anchor_y) * scale_y;

        {
            let mut vertex = self.vertex_data.map(4);

            // fill vertex array

            vertex[0].position[0] = x;
            vertex[0].position[1] = y;
            vertex[0].offset[0] = offset_x0;
            vertex[0].offset[1] = offset_y0;
            vertex[0].rotation = rotation;
            vertex[0].bucket_id = bucket_id;
            vertex[0].texture_id = texture_id;
            vertex[0].color = color;
            vertex[0].texture_uv[0] = 0.0;
            vertex[0].texture_uv[1] = 0.0;

            vertex[1].position[0] = x;
            vertex[1].position[1] = y;
            vertex[1].offset[0] = offset_x1;
            vertex[1].offset[1] = offset_y0;
            vertex[1].rotation = rotation;
            vertex[1].bucket_id = bucket_id;
            vertex[1].texture_id = texture_id;
            vertex[1].color = color;
            vertex[1].texture_uv[0] = sprite.u_max();
            vertex[1].texture_uv[1] = 0.0;

            vertex[2].position[0] = x;
            vertex[2].position[1] = y;
            vertex[2].offset[0] = offset_x0;
            vertex[2].offset[1] = offset_y1;
            vertex[2].rotation = rotation;
            vertex[2].bucket_id = bucket_id;
            vertex[2].texture_id = texture_id;
            vertex[2].color = color;
            vertex[2].texture_uv[0] = 0.0;
            vertex[2].texture_uv[1] = sprite.v_max();

            vertex[3].position[0] = x;
            vertex[3].position[1] = y;
            vertex[3].offset[0] = offset_x1;
            vertex[3].offset[1] = offset_y1;
            vertex[3].rotation = rotation;
            vertex[3].bucket_id = bucket_id;
            vertex[3].texture_id = texture_id;
            vertex[3].color = color;
            vertex[3].texture_uv[0] = sprite.u_max();
            vertex[3].texture_uv[1] = sprite.v_max();
        }

        self
    }

    /// removes previously added sprites from the drawing queue. typically invoked after draw()
    pub fn reset(self: &Self) -> &Self {

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
