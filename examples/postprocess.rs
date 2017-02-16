extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, utils, blendmodes};

#[path="res/bloom.rs"]
mod bloom;

pub fn main() {

    // Create a display to render to and a renderer to do the rendering.
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let context = renderer.context();

    // Load spritesheet containing components for rgba and a "lightmap". Create custom postprocessor.
    let sprite = Sprite::from_file(&context, r"examples/res/battery_lightmapped_128x128x15x2.png").unwrap();
    let mut bloom_effect = bloom::Bloom::new(&context);

    // A bunch of layers. The lightmap layers use component 1 (the "lightmap") of the sprite.
    let color_layer = Layer::new((640., 480.), 0);
    let lightmap_layer = Layer::new((640., 480.), 1);
    let unprocessed_lightmap_layer = Layer::new((640., 480.), 1);

    // A simple mainloop helper (just an optional utility function).
    utils::renderloop(|state| {

        let frame_id = (state.elapsed_f32 * 30.0) as u32;

        // Clear layers.
        color_layer.clear();
        lightmap_layer.clear();
        unprocessed_lightmap_layer.clear();

        // Draw top row of sprites: unprocessed components.
        sprite.draw(&color_layer, frame_id, (120., 120.), Color::white());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (320., 120.), Color::white());
        sprite.draw(&color_layer, frame_id, (520., 120.), Color::white());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (520., 120.), Color::white());

        // Draw bottom sprite: this one uses lightmap_layer, which will be postprocessed below.
        sprite.draw(&color_layer, frame_id, (320., 320.), Color::white());
        sprite.draw(&lightmap_layer, frame_id, (320., 320.), Color::white());

        // Clear frame, draw unprocesses layers and postprocessed lightmap layer.
        display.clear_frame(Color::black());
        renderer.draw_layer(&color_layer);
        renderer.draw_layer(&lightmap_layer);
        renderer.draw_layer(&unprocessed_lightmap_layer);

        renderer.postprocess(blendmodes::LIGHTEN, &mut bloom_effect, || {
            renderer.clear(Color(0., 0., 0., 0.05));
            renderer.draw_layer(&lightmap_layer);
        });

        display.swap_frame();

        // Poll for new events on the display, exit loop if the window was closed
        !display.poll_events().was_closed()
    });
}
