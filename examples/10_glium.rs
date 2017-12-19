#[macro_use]
extern crate glium;
extern crate radiant_rs;
use radiant_rs::{backend, Renderer, Layer, Sprite, Color, utils, blendmodes};
use glium::Surface;
use glium::index::PrimitiveType;
use std::cell::RefCell;
use std::rc::Rc;

pub fn main() {

    // Set up glium display. Much of the glium code in this example was taken from
    // https://github.com/glium/glium/blob/master/examples/triangle.rs
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("Glium example");

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true);

    let events_loop = glium::glutin::EventsLoop::new();
    let glium_display = glium::Display::new(window, context, &events_loop).unwrap();

    // Building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&glium_display,
            &[
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    // Building the index buffer
    let index_buffer = glium::IndexBuffer::new(&glium_display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    // Compiling shaders and linking them together
    let program = program!(&glium_display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec3 color;
                out vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",
            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },
    ).unwrap();

    // Building the uniforms
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    };
    
    // Create a Radiant Display from an existing glium Display and glutin EventsLoop.
    let display = backend::create_display(&glium_display, Rc::new(RefCell::new(events_loop)));

    // Create the renderer, a sprite and a layer
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    utils::renderloop(|frame| {

        // Clear the frame
        display.clear_frame(Color::BLACK);

        // Clear the layer, draw a few sprites to it
        layer.clear();
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::WHITE);
        sprite.draw(&layer, frame_id, (130., 100.), Color::RED);
        sprite.draw(&layer, frame_id, (190., 100.), Color::GREEN);
        sprite.draw(&layer, frame_id, (160., 155.), Color::BLUE);

        // "Borrow" Radiant's current frame to draw to it using Glium
        backend::get_frame(&display, |target| {
            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        });

        // Draw the sprites layer on top of the glium triangle
        renderer.draw_layer(&layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
