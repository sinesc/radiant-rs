extern crate radiant_rs;
use radiant_rs::*;

#[path="../res/effects/bloom.rs"]
mod bloom;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Glare effect demo").build();
    let renderer = Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let font = Font::builder(&renderer.context()).family("Arial").size(12.0).build().unwrap();

    // Load spritesheet containing components for rgba and a "lightmap". Create custom postprocessor.
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/battery_lightmapped_128x128x15x2.png").unwrap();
    let bloom_effect = bloom::Bloom::new(&renderer.context(), 2, 5, 5.0);

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
        font.write(&color_layer, "Sprite component 0", (80., 140.), Color::white());

        sprite.draw(&unprocessed_lightmap_layer, frame_id, (320., 100.), Color::white());
        font.write(&color_layer, "Sprite component 1", (280., 140.), Color::white());

        sprite.draw(&color_layer, frame_id, (520., 100.), Color::white());
        sprite.draw(&unprocessed_lightmap_layer, frame_id, (520., 100.), Color::white());
        font.write(&color_layer, "Both components", (480., 140.), Color::white());

        // Draw bottom sprite: this one uses lightmap_layer, which will be postprocessed below.
        sprite.draw(&color_layer, frame_id, (320., 380.), Color::white());
        sprite.draw(&lightmap_layer, frame_id, (320., 380.), Color::white());
        font.write(&color_layer, "Component 0 wihout postprocessing\nComponent 1 with postprocessing", (220., 440.), Color::white());

        // Draw unprocessed layers.
        renderer.draw_layer(&color_layer, 0);
        renderer.draw_layer(&lightmap_layer, 1);
        renderer.draw_layer(&unprocessed_lightmap_layer, 1);

        // Draw light_map layer to postprocessor.
        renderer.postprocess(&bloom_effect, &blendmodes::SCREEN, || {
            renderer.clear(Color(0., 0., 0., 0.05));
            renderer.draw_layer(&lightmap_layer, 1);
        });

        display.swap_frame();
        !display.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
