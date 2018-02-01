#[macro_use]
extern crate glium;
extern crate radiant_rs;
use radiant_rs::{backend, Layer, Sprite, Color, utils, blendmodes};
use glium::Surface;

#[path="res/glium_utils.rs"]
mod glium_utils;

// This example uses Radiant (and Glium) for rendering and relies on glium/glutin for display/events handling.
pub fn main() {

    // Set up glium display. Much of the glium code in this example was taken from
    // https://github.com/glium/glium/blob/master/examples/triangle.rs
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("Glium example 2: Glium with a little Radiant");

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true);

    let mut events_loop = glium::glutin::EventsLoop::new();
    let glium_display = glium::Display::new(window, context, &events_loop).unwrap();

    // Build glium buffers, program and uniforms (see res/glium_utils.rs)
    let vertex_buffer = glium_utils::build_vertex_buffer(&glium_display);
    let index_buffer = glium_utils::build_index_buffer(&glium_display);
    let program = glium_utils::build_program(&glium_display);
    let uniforms = glium_utils::build_uniforms();
    
    // Create a Radiant Renderer from an existing glium Display
    let renderer = backend::create_renderer(&glium_display).unwrap();

    // Create a sprite and a layer
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    utils::renderloop(|frame| {

        // Clear the layer, draw a few sprites to it
        layer.clear();
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::WHITE);
        sprite.draw(&layer, frame_id, (130., 100.), Color::RED);
        sprite.draw(&layer, frame_id, (190., 100.), Color::GREEN);
        sprite.draw(&layer, frame_id, (160., 155.), Color::BLUE);

        // Get and clear a glium frame
        let mut target = glium_display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        // Draw to to the glium frame using Glium        
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        
        // Draw the sprites layer on top of the glium triangle
        backend::target_frame(&renderer, &mut target, || {
            renderer.draw_layer(&layer, 0);
        });

        // Finish the frame and swap
        target.finish().unwrap();

        // Handle window close event
        !glium_utils::was_closed(&mut events_loop)
    });
}
