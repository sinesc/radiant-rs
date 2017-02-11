use prelude::*;
use avec::AVec;
use maths::{Mat4, Vec2, Point2};
use core::{self, Renderer, RenderContext, Layer, Font, Sprite, Color};
use BlendMode;

/// An operation-id returned from Scene::op.
#[derive(Copy, Clone)]
pub struct OpId(u16, u16);

/// A layer-id returned from layer registration or creation methods.
#[derive(Copy, Clone)]
pub struct LayerId(u16, u16);

/// A sprite-id returned from sprite registration or creation methods.
#[derive(Copy, Clone)]
pub struct SpriteId(u16, u16);

/// A font-id returned from font registration or creation methods.
#[derive(Copy, Clone)]
pub struct FontId(u16, u16);

/// A scene-operation.
///
/// Each time a scene is drawn, all of the registered scene operations will be performed.
/// Transformation-, rotation- and scaling-operations take a delta-factor which determines how the operation is affected by frame delta.
/// A value of 1.0 will scale the relevant parts of the operation by frame-delta, a value of 0.0 will not apply any modifications to the operation.
#[derive(Copy, Clone)]
pub enum Op {
    /// No op
    None,
    /// Set multiplication color for given layer.
    SetColor(LayerId, Color),
    /// Sets the blendmode for given layer.
    SetBlendmode(LayerId, BlendMode),
    /// Draws given layer.
    Draw(LayerId),
    /// Clears given layer.
    Clear(LayerId),
    /// Set a layer's view matrix.
    SetViewMatrix(LayerId, Mat4<f32>),
    /// Translate view matrix by given vector.
    TranslateViewMatrix(LayerId, f32, Vec2<f32>),
    /// Rotate view matrix by given radians.
    RotateViewMatrix(LayerId, f32, f32),
    /// Rotate view matrix by given radians around given origin.
    RotateViewMatrixAt(LayerId, f32, Vec2<f32>, f32),
    /// Scale view matrix by given vector.
    ScaleViewMatrix(LayerId, f32, Vec2<f32>),
    /// Set a layer's model matrix.
    SetModelMatrix(LayerId, Mat4<f32>),
    /// Translate model matrix by given vector.
    TranslateModelMatrix(LayerId, f32, Vec2<f32>),
    /// Rotate model matrix by given radians.
    RotateModelMatrix(LayerId, f32, f32),
    /// Rotate view matrix by given radians around given origin.
    RotateModelMatrixAt(LayerId, f32, Vec2<f32>, f32),
    /// Scale model matrix by given vector.
    ScaleModelMatrix(LayerId, f32, Vec2<f32>),
}

impl Default for Op {
    fn default() -> Op {
        Op::None
   }
}

static SCENE_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

/// A container for layers, sprites and fonts intended to simplify usage.
pub struct Scene {
    operations      : AVec<Op>,
    layers          : AVec<Layer>,
    sprites         : AVec<Sprite>,
    fonts           : AVec<Font>,
    context         : RenderContext,
    scene_id        : u16,
}

impl Scene {
    /// Create a new scene instance.
    pub fn new(context: &RenderContext) -> Self {
        Scene {
            operations  : AVec::new(1024),
            layers      : AVec::new(64),
            sprites     : AVec::new(64),
            fonts       : AVec::new(64),
            context     : context.clone(),
            scene_id    : SCENE_COUNTER.fetch_add(1, Ordering::Relaxed) as u16,
        }
    }

    /// Push a layer operation on the scene operation stack.
    pub fn op(&self, op: Op) -> OpId {
        let insert_position = self.operations.push(op);
        OpId(insert_position as u16, self.scene_id)
    }

    /// Push multiple operations on the scene operation stack.
    pub fn ops(&self, ops: &[Op]) -> &Self {
        for op in ops {
            self.op(*op);
        }
        self
    }

    /// Clear operation stack.
    pub fn clear(&self) -> &Self {
        self.operations.clear();
        self
    }

    /// Draws a sprite onto given layer.
    pub fn sprite(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, position: Point2, color: Color) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        assert_eq!(self.scene_id, layer_id.1);
        assert_eq!(self.scene_id, sprite_id.1);
        sprites[sprite_id.0 as usize].draw(&layers[layer_id.0 as usize], frame_id, position, color);
        self
    }

    /// Draws a sprite with given rotation and scaling onto given layer.
    pub fn sprite_transformed(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, position: Point2, color: Color, rotation: f32, scale: Vec2) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        assert_eq!(self.scene_id, layer_id.1);
        assert_eq!(self.scene_id, sprite_id.1);
        sprites[sprite_id.0 as usize].draw_transformed(&layers[layer_id.0 as usize], frame_id, position, color, rotation, scale);
        self
    }

    /// Writes a string onto given layer.
    pub fn write(&self, layer_id: LayerId, font_id: FontId, text: &str, position: Point2) -> &Self {
        let layers = self.layers.get();
        let fonts = self.fonts.get();
        assert_eq!(self.scene_id, layer_id.1);
        assert_eq!(self.scene_id, font_id.1);
        fonts[font_id.0 as usize].write(&layers[layer_id.0 as usize], text, position);
        self
    }

    /// Create and register a layer to the scene.
    pub fn register_layer(&self, width: u32, height: u32, channel: u32) -> LayerId {
        let insert_position = self.layers.push(Layer::new(width, height, channel));
        LayerId(insert_position as u16, self.scene_id)
    }

    /// Create and register a sprite to the scene
    pub fn register_sprite_from_file(self: &Self, file: &str) -> core::Result<SpriteId> {
        let sprite = Sprite::from_file(&self.context, file)?;
        result::Result::Ok(SpriteId(self.sprites.push(sprite) as u16, self.scene_id))
    }

    /// Register a sprite for the scene.
    pub fn register_sprite(self: &Self, sprite: Sprite) -> SpriteId {
        SpriteId(self.sprites.push(sprite) as u16, self.scene_id)
    }

    /// Register a font for the scene.
    pub fn register_font(self: &Self, font: Font) -> FontId {
        let insert_position = self.fonts.push(font);
        FontId(insert_position as u16, self.scene_id)
    }

    // !todo how to deal with fonts "with_xxx" mechanics here?
}

/// Draw entire scene. As this function is required to be called from the thread that created this
/// instance, it's not available in the implementation. Instead use renderer::draw_scene().
pub fn draw(this: &Scene, renderer: &Renderer, per_frame_multiplier: f32) {
    let operations_guard = this.operations.get();
    let operations = operations_guard.deref();
    let layers = this.layers.get();

    for operation in operations {
        match *operation {
            Op::SetColor(layer_id, color) => {
                assert_eq!(this.scene_id, layer_id.1);
                layers[layer_id.0 as usize].set_color(color);
            }
            Op::SetBlendmode(layer_id, blendmode) => {
                assert_eq!(this.scene_id, layer_id.1);
                layers[layer_id.0 as usize].set_blendmode(blendmode);
            }
            Op::Draw(layer_id) => {
                assert_eq!(this.scene_id, layer_id.1);
                renderer.draw_layer(&layers[layer_id.0 as usize]);
            }
            Op::Clear(layer_id) => {
                assert_eq!(this.scene_id, layer_id.1);
                layers[layer_id.0 as usize].clear();
            }

            Op::SetViewMatrix(layer_id, matrix) => {
                assert_eq!(this.scene_id, layer_id.1);
                layers[layer_id.0 as usize].set_view_matrix(matrix);
            }
            Op::TranslateViewMatrix(layer_id, delta_factor, vector) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.translate(vector * multiplier);
            }
            Op::RotateViewMatrix(layer_id, delta_factor, radians) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.rotate(radians * multiplier);
            }
            Op::RotateViewMatrixAt(layer_id, delta_factor, origin, radians) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.translate(origin);
                current.rotate(radians * multiplier);
                current.translate(-origin);
            }
            Op::ScaleViewMatrix(layer_id, delta_factor, vector) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.scale(vector * multiplier);
            }

            Op::SetModelMatrix(layer_id, matrix) => {
                assert_eq!(this.scene_id, layer_id.1);
                layers[layer_id.0 as usize].set_model_matrix(matrix);
            }
            Op::TranslateModelMatrix(layer_id, delta_factor, vector) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.translate(vector * multiplier);
            }
            Op::RotateModelMatrix(layer_id, delta_factor, radians) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.rotate(radians * multiplier);
            }
            Op::RotateModelMatrixAt(layer_id, delta_factor, origin, radians) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.translate(origin);
                current.rotate(radians * multiplier);
                current.translate(-origin);
            }
            Op::ScaleModelMatrix(layer_id, delta_factor, vector) => {
                assert_eq!(this.scene_id, layer_id.1);
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.scale(vector * multiplier);
            }

            _ => ()
        }
    }
}
