use prelude::*;
use avec::AVec;
use maths::{Vec3, Mat4};
use color::Color;
use graphics::{sprite, Sprite, blendmodes, BlendMode};

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
    pub fn sprite(&self, sprite: Sprite, frame_id: u32, x: f32, y: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {

        // increase local part of hash to mark this layer as modified against cached state in Renderer
        self.lid.fetch_add(1, Ordering::Relaxed);

        // get vertex_data slice and draw into it
        let mut guard = self.vertex_data.map(4);
        let mut vertices = guard.deref_mut();
        sprite::draw_sprite(&sprite, vertices, frame_id, x, y, color, rotation, scale_x, scale_y);

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
