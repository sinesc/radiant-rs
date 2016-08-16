use glium;
use glium::draw_parameters::*;
use glium::Surface;
use std::ops::DerefMut;

use maths::*;
use color::Color;
use renderer::Renderer;
use renderer::Sprite;
use super::Vertex;

pub struct Layer {
    pub matrix      : Mat4<f32>,
    pub blend       : Blend,
    vertex_buffer   : glium::VertexBuffer<Vertex>,
    sprite_id       : u32,
    renderer        : Renderer,
}

impl Layer {

    /// creates a new layer for the given renderer. use Renderer::layer() instead.
    pub fn new(renderer: Renderer) -> Self {

        let glium = renderer.glium.lock().unwrap();
        let (width, height) = glium.display.get_framebuffer_dimensions();
        let mut matrix = Mat4::<f32>::init_identity();
        matrix.translate(&Vec3(-1.0, 1.0, 0.0));
        matrix.scale(&Vec3(2.0 / width as f32, -2.0 / height as f32, 1.0));

        Layer {
            matrix          : matrix,
            blend           : Blend::alpha_blending(),
            vertex_buffer   : glium::VertexBuffer::empty_persistent(&glium.display, renderer.max_sprites as usize * 4).unwrap(),
            sprite_id       : 0,
            renderer        : renderer.clone(),
        }
    }

    /// sets the drawing matrix
    pub fn matrix(&mut self, matrix: Mat4<f32>) -> &mut Mat4<f32> {
        self.matrix = matrix;
        &mut self.matrix
    }

    /// sets the blend function for the layer (standard alpha blending)
    pub fn blend_alpha(&mut self) -> &mut Self {
        self.blend = Blend {
             color: BlendingFunction::Addition {
                 source: LinearBlendingFactor::One,
                 destination: LinearBlendingFactor::OneMinusSourceAlpha,
             },
             alpha: BlendingFunction::Addition {
                 source: LinearBlendingFactor::One,
                 destination: LinearBlendingFactor::OneMinusSourceAlpha,
             },
             constant_value: (0.0, 0.0, 0.0, 0.0)
         };
        self
    }

    /// sets the blend function for the layer (like alpha, but adds brightness value)
    pub fn blend_alpha_const(&mut self, brightness: u8) -> &mut Self {
       self.blend = Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::ConstantAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            constant_value: (0.0, 0.0, 0.0, brightness as f32 / 255.0)
        };
        self
    }

    /// sets the blend function for the layer
    pub fn blend_max(&mut self) -> &mut Self {
        self.blend = Blend {
            color: BlendingFunction::Max,
            alpha: BlendingFunction::Max,
            .. Default::default()
        };
        self
    }

    /// sets the blend function for the layer
    pub fn blend_min(&mut self) -> &mut Self {
        self.blend = Blend {
            color: BlendingFunction::Min,
            alpha: BlendingFunction::Min,
            .. Default::default()
        };
        self
    }

// !todo set up some nice blend modes

    /// sets the blend function for the layer
    pub fn blend_lighten(&mut self) -> &mut Self {
        self.blend = Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::One,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One,
            },
            .. Default::default()
        };
        self
    }

    /// sets the blend function for the layer
    pub fn blend_overlay(&mut self) -> &mut Self {
        self.blend = Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::SourceAlpha,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One,
            },
            .. Default::default()
        };
        self
    }

    /// adds a sprite to the draw queue
    pub fn sprite(&mut self, sprite: &Sprite, frame_id: u32, x: u32, y: u32, color: Color, rotation: f32, scale_x: f32, scale_y: f32) -> &mut Self {

        assert!(self.sprite_id < self.renderer.max_sprites);

        let texture_id = sprite.texture_id(frame_id);
        let bucket_id = sprite.bucket_id();
        let vertex_id = self.sprite_id as usize * 4;

        {
            let mut vertex = self.vertex_buffer.map();

            // corner positions relative to x/y

            let x = x as f32;
            let y = y as f32;
            let anchor_x = sprite.anchor_x * sprite.width() as f32;
            let anchor_y = sprite.anchor_y * sprite.height() as f32;

            let offset_x0 = -anchor_x * scale_x;
            let offset_x1 = (sprite.width() as f32 - anchor_x) * scale_x;
            let offset_y0 = -anchor_y * scale_y;
            let offset_y1 = (sprite.height() as f32 - anchor_y) * scale_y;

            // fill vertex array

            vertex[vertex_id].position[0] = x;
            vertex[vertex_id].position[1] = y;
            vertex[vertex_id].offset[0] = offset_x0;
            vertex[vertex_id].offset[1] = offset_y0;
            vertex[vertex_id].rotation = rotation;
            vertex[vertex_id].bucket_id = bucket_id;
            vertex[vertex_id].texture_id = texture_id;
            vertex[vertex_id].color = color;
            vertex[vertex_id].texture_uv[0] = 0.0;
            vertex[vertex_id].texture_uv[1] = 0.0;

            vertex[vertex_id+1].position[0] = x;
            vertex[vertex_id+1].position[1] = y;
            vertex[vertex_id+1].offset[0] = offset_x1;
            vertex[vertex_id+1].offset[1] = offset_y0;
            vertex[vertex_id+1].rotation = rotation;
            vertex[vertex_id+1].bucket_id = bucket_id;
            vertex[vertex_id+1].texture_id = texture_id;
            vertex[vertex_id+1].color = color;
            vertex[vertex_id+1].texture_uv[0] = sprite.u_max();
            vertex[vertex_id+1].texture_uv[1] = 0.0;

            vertex[vertex_id+2].position[0] = x;
            vertex[vertex_id+2].position[1] = y;
            vertex[vertex_id+2].offset[0] = offset_x0;
            vertex[vertex_id+2].offset[1] = offset_y1;
            vertex[vertex_id+2].rotation = rotation;
            vertex[vertex_id+2].bucket_id = bucket_id;
            vertex[vertex_id+2].texture_id = texture_id;
            vertex[vertex_id+2].color = color;
            vertex[vertex_id+2].texture_uv[0] = 0.0;
            vertex[vertex_id+2].texture_uv[1] = sprite.v_max();

            vertex[vertex_id+3].position[0] = x;
            vertex[vertex_id+3].position[1] = y;
            vertex[vertex_id+3].offset[0] = offset_x1;
            vertex[vertex_id+3].offset[1] = offset_y1;
            vertex[vertex_id+3].rotation = rotation;
            vertex[vertex_id+3].bucket_id = bucket_id;
            vertex[vertex_id+3].texture_id = texture_id;
            vertex[vertex_id+3].color = color;
            vertex[vertex_id+3].texture_uv[0] = sprite.u_max();
            vertex[vertex_id+3].texture_uv[1] = sprite.v_max();
        }

        self.sprite_id += 1;
        self
    }

    /// draws all previously added sprites. does not clear sprites.
    pub fn draw(&mut self) -> &mut Self {

        // make sure texture arrays have been generated from raw images

        self.renderer.create_texture_arrays();

        {
            // prepare texture array uniforms

            let mut lock = self.renderer.glium.lock().unwrap();
            let mut glium = lock.deref_mut();

            let empty = &glium::texture::Texture2dArray::empty(&glium.display, 2, 2, 1).unwrap();
            let mut arrays = Vec::<&glium::texture::Texture2dArray>::new();

            for i in 0..5 {
                arrays.push(if glium.tex_array.len() > i && glium.tex_array[i].is_some() {
                    glium.tex_array[i].as_ref().unwrap()
                } else {
                    empty
                });
            }

            let uniforms = uniform! {
                matrix: self.matrix,
                tex0: arrays[0],
                tex1: arrays[1],
                tex2: arrays[2],
                tex3: arrays[3],
                tex4: arrays[4]
            };

            // set up draw parameters for given blend options

            let draw_parameters = glium::draw_parameters::DrawParameters {
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                blend: self.blend,
                .. Default::default()
            };

            // draw up to sprite_id

            let ib_slice = glium.index_buffer.slice(0 .. self.sprite_id as usize * 4 * 6).unwrap();
            glium.target.as_mut().unwrap().draw(&self.vertex_buffer, &ib_slice, &glium.program, &uniforms, &draw_parameters).unwrap();
        }

        self
    }

    /// removes previously added sprites from the drawing queue. typically invoked after draw()
    pub fn reset(self: &mut Self) -> &mut Self {
        self.sprite_id = 0;
        self
    }
}
