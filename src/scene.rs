use prelude::*;
use avec::AVec;
use color::Color;
use maths::Mat4;
use graphics::{Renderer, RenderContext, Layer, Font, Sprite};
use BlendMode;

#[derive(Copy, Clone)]
pub struct OperationId(usize);

#[derive(Copy, Clone)]
pub struct LayerId(usize);

#[derive(Copy, Clone)]
pub struct SpriteId(usize);

#[derive(Copy, Clone)]
pub struct FontId(usize);

#[derive(Copy, Clone)]
pub enum Operation {
    None,
    SetColor(LayerId, Color),
    SetViewMatrix(LayerId, Mat4<f32>),
    SetModelMatrix(LayerId, Mat4<f32>),
    SetBlendmode(LayerId, BlendMode),
    Draw(LayerId),
    Reset(LayerId),
}

impl Default for Operation {
    fn default() -> Operation {
        Operation::None
   }
}

/// bla
#[allow(dead_code)]
pub struct Scene<'a> {
    operations      : AVec<Operation>,
    layers          : AVec<Layer>,
    sprites         : AVec<Sprite<'a>>,
    fonts           : AVec<Font<'a>>,
    context         : Arc<RenderContext<'a>>,
}

unsafe impl<'a> Send for Scene<'a> { }
unsafe impl<'a> Sync for Scene<'a> { }

impl<'a> Scene<'a> {
    /// Create a new scene instance.
    pub fn new(context: &Arc<RenderContext<'a>>) -> Scene<'a> {
        Scene {
            operations  : AVec::new(1024),
            layers      : AVec::new(64),
            sprites     : AVec::new(64),
            fonts       : AVec::new(64),
            context     : context.clone(),
        }
    }

    /// Push a layer operation on the scene operation stack.
    pub fn op(&self, op: Operation) -> OperationId {
        let insert_position = self.operations.push(op);
        OperationId(insert_position)
    }

    /// Push multiple operations on the scene operation stack.
    pub fn ops(&self, ops: &[Operation]) -> &Self {
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

    /// Draws a sprite with given rotation and scaling onto given layer.
    pub fn sprite_transformed(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, x: f32, y: f32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        sprites[sprite_id.0].draw_transformed(&layers[layer_id.0], frame_id, x, y, color, rotation, scale_x, scale_y);
        self
    }

    /// Draws a sprite onto given layer.
    pub fn sprite(&self, layer_id: LayerId, sprite_id: SpriteId, frame_id: u32, x: f32, y: f32, color: Color) -> &Self {
        let layers = self.layers.get();
        let sprites = self.sprites.get();
        sprites[sprite_id.0].draw(&layers[layer_id.0], frame_id, x, y, color);
        self
    }

    /// Writes a string onto given layer.
    pub fn write(&self, layer_id: LayerId, font_id: FontId, text: &str, x: f32, y: f32) -> &Self {
        let layers = self.layers.get();
        let fonts = self.fonts.get();
        fonts[font_id.0].write(&layers[layer_id.0], text, x, y);
        self
    }

    /// Create and add a layer to the scene.
    pub fn create_layer(&self, width: u32, height: u32) -> LayerId {
        let insert_position = self.layers.push(Layer::new(width, height));
        LayerId(insert_position)
    }

    /// Register a sprite for the scene.
    pub fn register_sprite(self: &Self, sprite: Sprite<'a>) -> SpriteId {
        let insert_position = self.sprites.push(sprite);
        SpriteId(insert_position)
    }

    /// Register a font for the scene.
    pub fn register_font(self: &Self, font: Font<'a>) -> FontId {
        let insert_position = self.fonts.push(font);
        FontId(insert_position)
    }

    // !todo how to deal with fonts "with_xxx" mechanics here?
}

/// Draw entire scene. As this function is required to be called from the thread that created this
/// instance, it's not available in the implementation. Instead use renderer::draw_scene().
pub fn draw(this: &Scene, renderer: &Renderer) {
    let operations_guard = this.operations.get();
    let operations = operations_guard.deref();
    let layers = this.layers.get();

    for operation in operations {
        match *operation {
            Operation::SetColor(layer_id, color) => {
                layers[layer_id.0 as usize].set_color(color);
            }
            Operation::SetViewMatrix(layer_id, matrix) => {
                layers[layer_id.0 as usize].set_view_matrix(matrix);
            }
            Operation::SetModelMatrix(layer_id, matrix) => {
                layers[layer_id.0 as usize].set_model_matrix(matrix);
            }
            Operation::SetBlendmode(layer_id, blendmode) => {
                layers[layer_id.0 as usize].set_blendmode(blendmode);
            }
            Operation::Draw(layer_id) => {
                renderer.draw_layer(&layers[layer_id.0]);
            }
            Operation::Reset(layer_id) => {
                layers[layer_id.0 as usize].clear();
            }
            _ => ()
        }
    }
}
