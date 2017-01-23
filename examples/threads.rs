extern crate radiant_rs;
use radiant_rs::{Color, Renderer, Layer, DisplayInfo, Display, Font, FontInfo, Point2, Vec2, blendmodes, utils};
use std::thread;
use std::sync::{Arc, Barrier};

pub fn main() {

    // create a window, a renderer and a threadsafe context (required for sprite/font creation)
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display);

    // create a single layer and a font
    let layer = Arc::new(Layer::new(640, 480));
    layer.set_blendmode(blendmodes::LIGHTEN);
    let big_font = Arc::new(Font::from_info(&renderer.context(), FontInfo { family: "Arial".to_string(), size: 20.0, ..FontInfo::default() }));
    let font = big_font.with_size(12.0);

    // set up two barriers to ensure 1) all threads are done and before we show the frame
    // and 2) threads don't restart drawing while the layer is still being rendererd
    let num_threads = 10;
    let draw_start = Arc::new(Barrier::new(num_threads + 1));
    let draw_done = Arc::new(Barrier::new(num_threads + 1));

    for i in 0..num_threads {
        let (big_font, layer, draw_start, draw_done) = (big_font.clone(), layer.clone(), draw_start.clone(), draw_done.clone());

        // draw from a bunch of threads in parallel
        thread::spawn(move || {
            let mut rot = 0.0;
            let pos = 120.0 + (i as f32) * 20.0;
            loop {
                big_font.write_transformed(&layer, &format!("Thread {} !", i+1), Point2(pos, pos), 0.0, rot, Vec2(1.0, 1.0));
                rot += 0.01;

                // wait until all other threads have also drawn, then wait until the layers have been rendered
                draw_start.wait();
                draw_done.wait();
            }
        });
    }

    // a simple mainloop helper (just an optional utility function)
    utils::renderloop(|state| {

        // this will unblock once all threads have finished drawing
        draw_start.wait();

        renderer.clear_target(Color::black());
        font.write(&layer, &format!("{}FPS", state.fps), Point2(10.0, 10.0));
        renderer.draw_layer(&layer);
        layer.clear();

        // layer is drawn, let threads render their next frame
        draw_done.wait();

        renderer.swap_target();

        !display.poll_events().was_closed()
    });
}
