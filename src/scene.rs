#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use prelude::*;
use avec::AVec;
use color::Color;
use maths::Mat4;
use graphics::{Renderer, Layer};
use BlendMode;

#[derive(Copy, Clone)]
pub struct OperationId(u32);

#[derive(Copy, Clone)]
pub struct LayerId(usize);

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

pub struct Scene {
    renderer        : Renderer,
    operations      : AVec<Operation>,
    layers          : Vec<Layer>,
    layer_id        : Mutex<usize>,
    dimensions      : (u32, u32),
}

impl Scene {
    /// create a new scene instance
    pub fn new(renderer: &Renderer, dimensions: (u32, u32)) -> Scene {
        Scene {
            renderer    : renderer.clone(),
            operations  : AVec::new(1024),  // !todo
            layers      : Vec::new(),
            layer_id    : Mutex::new(0),
            dimensions  : dimensions,
        }
    }

    /// push a layer operation on the scene operation stack
    pub fn push(&mut self, op: Operation) -> OperationId {
        let insert_position = self.operations.len();
        self.operations.push(op);
        OperationId(insert_position)
    }

    /// clear operation stack
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// create and add a layer to the scene
    pub fn add_layer(&mut self) -> LayerId {
        let mut lock = self.layer_id.lock().unwrap();
        let mut layer_id = lock.deref_mut();

        let insert_position = self.layers.len();
        self.layers.push(Layer::new(&self.renderer, self.dimensions));

        *layer_id += 1;
        assert!(*layer_id == self.layers.len());

        LayerId(insert_position)
    }

    /// returns an existing layer
    pub fn layer(&mut self, id: LayerId) -> &mut Layer {
        &mut self.layers[id.0]
    }

    // draw entire scene. required to be called from the thread that created this instance
    pub fn draw(&mut self) {
        let operations_guard = self.operations.get();
        let operations = operations_guard.deref();

        for operation in operations {
            match *operation {
                Operation::SetColor(layer_id, color) => {
                    self.layers[layer_id.0 as usize].set_color(color);
                }
                Operation::SetViewMatrix(layer_id, matrix) => {
                    self.layers[layer_id.0 as usize].set_view_matrix(matrix);
                }
                Operation::SetModelMatrix(layer_id, matrix) => {
                    self.layers[layer_id.0 as usize].set_model_matrix(matrix);
                }
                Operation::SetBlendmode(layer_id, blendmode) => {
                    self.layers[layer_id.0 as usize].set_blendmode(blendmode);
                }
                Operation::Draw(layer_id) => {
                    self.layers[layer_id.0 as usize].draw();
                }
                Operation::Reset(layer_id) => {
                    self.layers[layer_id.0 as usize].reset();
                }
                _ => ()
            }
        }
    }
}
