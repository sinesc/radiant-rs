extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Color, Renderer, Layer, Display, Font, blendmodes};
use std::thread;
use std::sync::{Arc, Barrier};

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Threads example").build();
    let renderer = Renderer::new(&display).unwrap();

    // Create a layer and a font, wrapped in Arcs
    let layer = Arc::new(Layer::new((640., 480.)));
    layer.set_blendmode(blendmodes::LIGHTEN);
    let font = Arc::new(Font::builder(&renderer.context()).family("Arial").size(20.0).build().unwrap());

    // Even though it would be safe to draw without coordination from multiple threads
    // while continously rendering from the main thread, you still want to present
    // completed frames.
    // Set up two barriers to ensure 1) all threads are done and before we show the frame
    // and 2) threads don't restart drawing before the main thread clears the layer.
    let num_threads = 15;
    let draw_start = Arc::new(Barrier::new(num_threads + 1));
    let draw_done = Arc::new(Barrier::new(num_threads + 1));

    for i in 0..num_threads {
        let (font, layer, draw_start, draw_done) = (font.clone(), layer.clone(), draw_start.clone(), draw_done.clone());

        // draw from a bunch of threads in parallel
        thread::spawn(move || {
            let mut rot = 0.0;
            let pos = 120.0 + (i as f32) * 20.0;
            loop {
                font.write_transformed(&layer, &format!("Thread {} !", i+1), (pos, pos / 1.33), Color::WHITE, 0.0, rot, (1.0, 1.0));
                rot += 0.01;

                // wait until all other threads have also drawn, then wait until the layers have been rendered
                draw_start.wait();
                draw_done.wait();
            }
        });
    }

    ru::renderloop(|_| {

        // Unblock once all threads have finished drawing.
        draw_start.wait();

        display.clear_frame(Color::BLACK);
        renderer.draw_layer(&layer, 0);
        layer.clear();

        // Layer is drawn, let threads render their next frame.
        draw_done.wait();

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
