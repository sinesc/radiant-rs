use prelude::*;
use misc::AVec;
use maths::{Mat4, Vec2};
use core::{Renderer, RenderContext, Layer, Font, Sprite, Color};
use BlendMode;

/// An operation-id returned from Scene::op.
#[derive(Copy, Clone)]
pub struct OpId(usize);

/// A layer-id returned from layer registration or creation methods.
#[derive(Copy, Clone)]
pub struct LayerId(usize);

/// A sprite-id returned from sprite registration or creation methods.
#[derive(Copy, Clone)]
pub struct SpriteId(usize);

/// A font-id returned from font registration or creation methods.
#[derive(Copy, Clone)]
pub struct FontId(usize);

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

/// A container for layers, sprites and fonts intended to simplify usage.
pub struct Scene {
    operations      : AVec<Op>,
    layers          : AVec<Layer>,
    sprites         : AVec<Sprite>,
    fonts           : AVec<Font>,
    context         : RenderContext,
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
        }
    }

    /// Push a layer operation on the scene operation stack.
    pub fn op(&self, op: Op) -> OpId {
        let insert_position = self.operations.push(op);
        OpId(insert_position)
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
    pub fn sprite(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, x: f32, y: f32, color: Color) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        sprites[sprite_id.0].draw(&layers[layer_id.0], frame_id, x, y, color);
        self
    }

    /// Draws a sprite with given rotation and scaling onto given layer.
    pub fn sprite_transformed(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, x: f32, y: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        sprites[sprite_id.0].draw_transformed(&layers[layer_id.0], frame_id, x, y, color, rotation, scale_x, scale_y);
        self
    }

    /// Writes a string onto given layer.
    pub fn write(&self, layer_id: LayerId, font_id: FontId, text: &str, x: f32, y: f32) -> &Self {
        let layers = self.layers.get();
        let fonts = self.fonts.get();
        fonts[font_id.0].write(&layers[layer_id.0], text, x, y);
        self
    }

    /// Create and register a layer to the scene.
    pub fn register_layer(&self, width: u32, height: u32) -> LayerId {
        let insert_position = self.layers.push(Layer::new(width, height));
        LayerId(insert_position)
    }

    /// Create and register a sprite to the scene
    pub fn register_sprite_from_file(self: &Self, file: &str) -> SpriteId {
        let sprite = Sprite::from_file(&self.context, file);
        SpriteId(self.sprites.push(sprite))
    }

    /// Register a sprite for the scene.
    pub fn register_sprite(self: &Self, sprite: Sprite) -> SpriteId {
        SpriteId(self.sprites.push(sprite))
    }

    /// Register a font for the scene.
    pub fn register_font(self: &Self, font: Font) -> FontId {
        let insert_position = self.fonts.push(font);
        FontId(insert_position)
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
                layers[layer_id.0 as usize].set_color(color);
            }
            Op::SetBlendmode(layer_id, blendmode) => {
                layers[layer_id.0 as usize].set_blendmode(blendmode);
            }
            Op::Draw(layer_id) => {
                renderer.draw_layer(&layers[layer_id.0]);
            }
            Op::Clear(layer_id) => {
                layers[layer_id.0 as usize].clear();
            }

            Op::SetViewMatrix(layer_id, matrix) => {
                layers[layer_id.0 as usize].set_view_matrix(matrix);
            }
            Op::TranslateViewMatrix(layer_id, delta_factor, vector) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.translate(vector * multiplier);
            }
            Op::RotateViewMatrix(layer_id, delta_factor, radians) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.rotate(radians * multiplier);
            }
            Op::RotateViewMatrixAt(layer_id, delta_factor, origin, radians) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.translate(origin);
                current.rotate(radians * multiplier);
                current.translate(-origin);
            }
            Op::ScaleViewMatrix(layer_id, delta_factor, vector) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].view_matrix();
                current.scale(vector * multiplier);
            }

            Op::SetModelMatrix(layer_id, matrix) => {
                layers[layer_id.0 as usize].set_model_matrix(matrix);
            }
            Op::TranslateModelMatrix(layer_id, delta_factor, vector) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.translate(vector * multiplier);
            }
            Op::RotateModelMatrix(layer_id, delta_factor, radians) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.rotate(radians * multiplier);
            }
            Op::RotateModelMatrixAt(layer_id, delta_factor, origin, radians) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.translate(origin);
                current.rotate(radians * multiplier);
                current.translate(-origin);
            }
            Op::ScaleModelMatrix(layer_id, delta_factor, vector) => {
                let multiplier = (1.0 - delta_factor) + (delta_factor * per_frame_multiplier);
                let mut current = layers[layer_id.0 as usize].model_matrix();
                current.scale(vector * multiplier);
            }

            _ => ()
        }
    }
}
