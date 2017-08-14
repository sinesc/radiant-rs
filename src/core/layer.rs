use prelude::*;
use avec;
use maths::{Mat4, Point2, Rect};
use core::{blendmodes, BlendMode, rendercontext, Color, Program, Vertex};
use maths::Vec2;

static LAYER_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

/// A drawing surface for text and sprites that implements send+sync and is wait-free for drawing operations.
///
/// In radiant_rs, sprite drawing happens on layers. Layers provide transformation capabilities in
/// the form of model- and view-matrices and the layer's blendmode and color determine
/// how sprites are rendered to the drawing target. Layers can be rendered multiple times using
/// different matrices, blendmodes or colors without having to redraw their contents first.
///
/// For convenience, layers are created with a view-matrix that maps the given dimensions to the
/// entirety of the drawing target. The layer itself is infinite though, and can be transformed at any
/// time before rendering.
///
/// Drawing to a layer is a wait-free atomic operation that can be safely performed from multiple threads at
/// the same time. Modifying layer properties like the matrices may cause other threads to wait.
pub struct Layer {
    view_matrix     : Mutex<Mat4<f32>>,
    model_matrix    : Mutex<Mat4<f32>>,
    blend           : Mutex<BlendMode>,
    color           : Mutex<Color>,
    contents        : Arc<LayerContents>,
    program         : Option<Program>,
}

unsafe impl Send for Layer { }
unsafe impl Sync for Layer { }

impl Clone for Layer {
    /// Creates a new layer that references the contents of this layer but has its own
    /// color, blendmode and set of matrices.
    fn clone(self: &Self) -> Self {
        self.create_clone(None)
    }
}

/// Layer contents, shared among layer clones.
struct LayerContents {
    vertex_data     : avec::AVec<Vertex>,
    dirty           : AtomicBool,
    generation      : AtomicUsize,
    layer_id        : usize,
}

impl Layer {

    /// Creates a new layer with given dimensions, meaning that is is created with
    /// a view matrix that maps the given dimensions to the entirety of the drawing target.
    pub fn new<T>(dimensions: T) -> Self where Vec2<f32>: From<T> {
        Self::create(dimensions, None)
    }

    /// Creates a new layer with given dimensions and fragment program.
    pub fn with_program<T>(dimensions: T, program: Program) -> Self where Vec2<f32>: From<T> {
        Self::create(dimensions, Some(program))
    }

    /// Creates a new layer that references the contents of this layer but has its own
    /// color, blendmode, program and set of matrices.
    pub fn clone_with_program(self: &Self, program: Program) -> Self {
        self.create_clone(Some(program))
    }

    /// Sets a global color multiplicator. Setting this to white means that the layer contents
    /// are renderered in their original colors.
    ///
    /// Note that [`Colors`](struct.Color.html) contain
    /// alpha information and are not clamped to any range, so it is possible to use an overbright
    /// color to brighten the result or use the alpha channel to apply global transparency.
    pub fn set_color(self: &Self, color: Color) -> &Self {
        self.color().set(color);
        self
    }

    /// Returns a mutex guarded mutable reference to the global color multiplicator.
    pub fn color(self: &Self) -> MutexGuard<Color> {
        self.color.lock().unwrap()
    }

    /// Sets the view matrix.
    ///
    /// View matrix transformation is applied after the objects are fully positioned on the layer.
    /// As a result, manipulating the view matrix has the effect of manipulating the layer itself,
    /// e.g. rotating the entire layer.
    pub fn set_view_matrix(self: &Self, matrix: Mat4<f32>) -> &Self {
        self.view_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the view matrix.
    /// See [`set_view_matrix()`](#method.set_view_matrix) for a description of the view matrix.
    pub fn view_matrix(self: &Self) -> MutexGuard<Mat4<f32>> {
        self.view_matrix.lock().unwrap()
    }

    /// Sets the model matrix.
    ///
    /// Model matrix transformation is applied before each object is transformed to its position
    /// on the layer. As a result, manipulating the model matrix has the effect of manipulating
    /// every object on the layer in the same way, e.g. rotating every individual object on the
    /// layer around a point relative to the individual object.
    pub fn set_model_matrix(self: &Self, matrix: Mat4<f32>) -> &Self {
        self.model_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the model matrix.
    /// See [`set_model_matrix()`](#method.set_model_matrix) for a description of the model matrix.
    pub fn model_matrix(self: &Self) -> MutexGuard<Mat4<f32>> {
        self.model_matrix.lock().unwrap()
    }

    /// Sets the blendmode.
    pub fn set_blendmode(self: &Self, blendmode: BlendMode) -> &Self {
        *self.blendmode() = blendmode;
        self
    }

    /// Returns a mutex guarded mutable reference to the blendmode.
    pub fn blendmode(self: &Self) -> MutexGuard<BlendMode> {
        self.blend.lock().unwrap()
    }

    /// Removes all previously added objects from the layer. Typically invoked after the layer has
    /// been rendered.
    pub fn clear(self: &Self) -> &Self {
        self.set_dirty(true);
        self.set_generation(0);
        self.contents.vertex_data.clear();
        self
    }

    /// Returns the number of sprites the layer can hold without having to perform a blocking reallocation.
    pub fn capacity(self: &Self) -> usize {
        self.contents.vertex_data.capacity() / 4
    }

    /// Returns the number of sprites currently stored the layer.
    pub fn len(self: &Self) -> usize {
        self.contents.vertex_data.len() / 4
    }

    /// Returns the layer wrapped in an std::Arc.
    pub fn arc(self: Self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Creates a new layer
    fn create<T>(dimensions: T, program: Option<Program>) -> Self where Vec2<f32>: From<T> {
        let dimensions = Vec2::from(dimensions);
        Layer {
            view_matrix     : Mutex::new(Mat4::viewport(dimensions.0, dimensions.1)),
            model_matrix    : Mutex::new(Mat4::identity()),
            blend           : Mutex::new(blendmodes::ALPHA),
            color           : Mutex::new(Color::white()),
            contents        : Arc::new(LayerContents {
                vertex_data     : avec::AVec::new(rendercontext::INITIAL_CAPACITY * 4),
                dirty           : AtomicBool::new(true),
                generation      : AtomicUsize::new(0),
                layer_id        : 1 + LAYER_COUNTER.fetch_add(1, Ordering::Relaxed),
            }),
            program         : program,
        }
    }

    /// Creates a clone.
    fn create_clone(self: &Self, program: Option<Program>) -> Self {
        Layer {
            view_matrix     : Mutex::new(self.view_matrix().deref().clone()),
            model_matrix    : Mutex::new(self.model_matrix().deref().clone()),
            blend           : Mutex::new(self.blendmode().deref().clone()),
            color           : Mutex::new(self.color().deref().clone()),
            contents        : self.contents.clone(),
            program         : program,
        }
    }

    /// Sets or unsets the layer content generation. A generation can only be set
    /// if the current generation is unset (generation=0). Returns true on success.
    fn set_generation(self: &Self, generation: usize) -> bool {
        let previous = self.contents.generation.swap(generation, Ordering::Relaxed);
        previous == generation || generation == 0 || previous == 0
    }

    /// Sets or unsets the layers dirty state
    fn set_dirty(self: &Self, value: bool) {
        self.contents.dirty.store(value, Ordering::Relaxed);
    }
}

/// Returns a reference to the layer's program, if it has any.
pub fn program(layer: &Layer) -> Option<&Program> {
    layer.program.as_ref()
}

pub fn vertices(layer: &Layer) -> avec::AVecReadGuard<Vertex> {
    layer.contents.vertex_data.get()
}

pub fn layer_id(layer: &Layer) -> usize {
    layer.contents.layer_id
}

/// Draws a rectangle on given layer.
pub fn add_rect(layer: &Layer, generation: Option<usize>, bucket_id: u8, texture_id: u32, components: u8, uv: Rect, pos: Point2, anchor: Point2<f32>, dim: Point2, color: Color, rotation: f32, scale: Vec2) {

    layer.set_dirty(true);
    if generation.is_some() && !layer.set_generation(generation.unwrap()) {
        panic!("Layer contains garbage data. Note: Layers need to be cleared after performing a RenderContext::prune().");
    }

    // corner positions relative to x/y

    let offset_x0 = -anchor.0 * scale.0;
    let offset_x1 = (dim.0 - anchor.0) * scale.0;
    let offset_y0 = -anchor.1 * scale.1;
    let offset_y1 = (dim.1 - anchor.1) * scale.1;

    let bucket_id = bucket_id as u32;
    let components = components as u32;

    // get vertex_data slice and draw into it

    let map = layer.contents.vertex_data.map(4);

    map.set(0, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x0, offset_y0],
        rotation    : rotation,
        color       : color.into(),
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : uv.top_left().into(),
        components  : components,
    });

    map.set(1, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x1, offset_y0],
        rotation    : rotation,
        color       : color.into(),
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : uv.top_right().into(),
        components  : components,
    });

    map.set(2, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x0, offset_y1],
        rotation    : rotation,
        color       : color.into(),
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : uv.bottom_left().into(),
        components  : components,
    });

    map.set(3, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x1, offset_y1],
        rotation    : rotation,
        color       : color.into(),
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : uv.bottom_right().into(),
        components  : components,
    });
}
