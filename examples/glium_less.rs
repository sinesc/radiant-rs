#[macro_use]
extern crate glium;
extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{backend, Renderer, Layer, Sprite, Color, blendmodes};
use glium::Surface;

#[path="res/glium_utils.rs"]
mod glium_utils;

// This example uses Radiant for display/events handling and uses Glium (and Radiant) for drawing.
pub fn main() {

    // Set up glium display. Much of the glium code in this example was taken from
    // https://github.com/glium/glium/blob/master/examples/triangle.rs
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions((640, 480).into())
        .with_title("Glium example 1: Radiant with a little Glium");

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true);

    let events_loop = glium::glutin::EventsLoop::new();
    let glium_display = glium::Display::new(window, context, &events_loop).unwrap();

    // Build glium buffers, program and uniforms (see res/glium_utils.rs)
    let vertex_buffer = glium_utils::build_vertex_buffer(&glium_display);
    let index_buffer = glium_utils::build_index_buffer(&glium_display);
    let program = glium_utils::build_program(&glium_display);
    let uniforms = glium_utils::build_uniforms();

    // Create a Radiant Display from an existing glium Display and glutin EventsLoop.
    backend::set_events_loop(events_loop);
    let display = backend::create_display(&glium_display);

    // Create the renderer, a sprite and a layer
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(display.context(), r"examples/res/sprites/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    ru::renderloop(|frame| {

        // Clear the layer, draw a few sprites to it
        layer.clear();
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::WHITE);
        sprite.draw(&layer, frame_id, (130., 100.), Color::RED);
        sprite.draw(&layer, frame_id, (190., 100.), Color::GREEN);
        sprite.draw(&layer, frame_id, (160., 155.), Color::BLUE);

        // Clear the frame
        display.clear_frame(Color::BLACK);

        // "Borrow" Radiant's current frame to draw to it using Glium
        backend::take_frame(&display, |target| {
            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        });

        // Draw the sprites layer on top of the glium triangle
        renderer.draw_layer(&layer, 0);

        // Finish the frame and swap
        display.swap_frame();

        // Handle window close event
        !display.poll_events().was_closed()
    });
}
