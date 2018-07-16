use glium;
use glium::index::PrimitiveType;
use glium::glutin;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

pub fn build_vertex_buffer(glium_display: &glium::Display) -> glium::VertexBuffer<Vertex> {
    glium::VertexBuffer::new(glium_display,
        &[
            Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
        ]
    ).unwrap()
}

pub fn build_program(glium_display: &glium::Display) -> glium::Program {
    program!(glium_display,
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
    ).unwrap()
}

pub fn build_index_buffer(glium_display: &glium::Display) -> glium::IndexBuffer<u16> {
    glium::IndexBuffer::new(glium_display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap()
}

pub fn build_uniforms<'a>() -> glium::uniforms::UniformsStorage<'a, [[f32; 4]; 4], glium::uniforms::EmptyUniforms> {
    uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    }
}

#[allow(dead_code)]
pub fn was_closed(events_loop: &mut glium::glutin::EventsLoop) -> bool {

    let mut was_closed = false;

    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::WindowEvent::CloseRequested => { was_closed = true; },
                _ => (),
            },
            _ => (),
        }
    });

    was_closed
}