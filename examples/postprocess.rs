extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, utils};

#[path="res/bloom.rs"]
mod bloom;

pub fn main() {

    // create a display to render to, a renderer to do the rendering
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let context = renderer.context();

    // load spritesheet containing rgba + lightmap
    let sprite = Sprite::from_file(&context, r"examples/res/battery_lightmapped_128x128x15x2.png").unwrap();
    let color_layer = Layer::new((640., 480.), 0);
    let lightmap_layer = Layer::new((640., 480.), 1);
    let unprocessed_lightmap_layer = Layer::new((640., 480.), 1);

    // custom postprocessor
    let mut postprocessor = bloom::Bloom::new(&context);

    // a simple mainloop helper (just an optional utility function)
    utils::renderloop(|state| {

        // clear layers, draw a bunch of sprites
        color_layer.clear();
        lightmap_layer.clear();
        unprocessed_lightmap_layer.clear();

        // top row: unprocessed components
        let frame_id = (state.elapsed_f32 * 1.0) as u32;
        sprite.draw(&color_layer, frame_id, (120., 120.), Color::green());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (520., 120.), Color::white());
        sprite.draw(&color_layer, frame_id, (320., 120.), Color::green());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (320., 120.), Color::white());

        // bottom: this one uses lightmap_layer, which will be postprocessed below
        let frame_id = (state.elapsed_f32 * 30.0) as u32;
        sprite.draw(&color_layer, frame_id, (320., 320.), Color::green());
        sprite.draw(&lightmap_layer, frame_id, (320., 320.), Color::white());

        // clear frame, draw color layer and postprocessed lightmap layer
        display.clear_frame(Color::black());
        renderer.draw_layer(&color_layer);
        renderer.draw_layer_processed(&lightmap_layer, &mut postprocessor);
        renderer.draw_layer(&unprocessed_lightmap_layer);
        display.swap_frame();

        // poll for new events on the display, exit loop if the window was closed
        !display.poll_events().was_closed()
    });
}
