extern crate radiant_rs;
use radiant_rs::*;

#[path="../res/bloom.rs"]
mod bloom;

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let font = Font::from_info(&renderer.context(), FontInfo { family: "Arial".to_string(), size: 12.0, ..FontInfo::default() } );

    // Load spritesheet containing components for rgba and a "lightmap". Create custom postprocessor.
    let sprite = Sprite::from_file(&renderer.context(), r"res/battery_lightmapped_128x128x15x2.png").unwrap();
    let bloom_effect = bloom::Bloom::new(&renderer.context(), 2, 5, blendmodes::ALPHA);

    // A bunch of layers. The lightmap layers use component 1 (the "lightmap") of the sprite.
    let color_layer = Layer::new((640., 480.));
    let lightmap_layer = Layer::new((640., 480.));
    let unprocessed_lightmap_layer = Layer::new((640., 480.));

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());

        color_layer.clear();
        lightmap_layer.clear();
        unprocessed_lightmap_layer.clear();

        // Draw top row of sprites: unprocessed components.
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&color_layer, frame_id, (120., 100.), Color::white());
        font.write(&color_layer, "Sprite component 0", (80., 140.));

        sprite.draw(&unprocessed_lightmap_layer, frame_id, (320., 100.), Color::white());
        font.write(&color_layer, "Sprite component 1", (280., 140.));

        sprite.draw(&color_layer, frame_id, (520., 100.), Color::white());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (520., 100.), Color::white());
        font.write(&color_layer, "Both components", (480., 140.));

        // Draw bottom sprite: this one uses lightmap_layer, which will be postprocessed below.
        sprite.draw(&color_layer, frame_id, (320., 380.), Color::white());
        sprite.draw(&lightmap_layer, frame_id, (320., 380.), Color::white());
        font.write(&color_layer, "Component 0 wihout postprocessing\nComponent 1 with postprocessing", (220., 440.));

        // Draw unprocessed layers.
        renderer.draw_layer(&color_layer, 0);
        renderer.draw_layer(&lightmap_layer, 1);
        renderer.draw_layer(&unprocessed_lightmap_layer, 1);

        // Draw light_map layer to postprocessor.
        renderer.postprocess(&bloom_effect, &blendmodes::LIGHTEN, || {
            renderer.clear(Color(0., 0., 0., 0.05));
            renderer.draw_layer(&lightmap_layer, 1);
        });

        display.swap_frame();
        !display.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
