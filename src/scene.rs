#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use color::Color;
use maths::Mat4;
use graphics::{Renderer, Layer};
use BlendMode;

#[derive(Copy, Clone)]
pub struct OperationId(usize);

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

pub struct Scene {
    renderer        : Renderer,
    operations      : Vec<Operation>,
    layers          : Vec<Layer>,
}

impl Scene {
    /// create a new scene instance
    pub fn new(renderer: &Renderer) -> Scene {
        Scene {
            renderer    : renderer.clone(),
            operations  : Vec::new(),
            layers      : Vec::new(),
        }
    }

    /// push a layer operation on the scene operation stack
    pub fn push(&mut self, op: Operation) -> OperationId {
        let insert_position = self.operations.len();
        self.operations.push(op);
        OperationId(insert_position)
    }

    /// pop the last layer operation off the scene operation stack
    pub fn pop(&mut self) -> Operation {
        self.operations.pop().unwrap()
    }

    /// replace given layer opation with another operation
    pub fn replace(&mut self, id: OperationId, op: Operation) {
        self.operations[id.0] = op;
    }

    /// clear operation stack
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// create and add a layer to the scene
    pub fn add_layer(&mut self) -> LayerId {
        let insert_position = self.layers.len();
        self.layers.push(Layer::new(&self.renderer));
        LayerId(insert_position)
    }

    /// returns an existing layer
    pub fn layer(&mut self, id: LayerId) -> &mut Layer {
        &mut self.layers[id.0]
    }

    // draw entire scene. required to be called from the thread that created this instance
    pub fn draw(&mut self) {
        for operation in &self.operations {
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
