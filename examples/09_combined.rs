extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, Texture, TextureFilter, blendmodes};
use ru::Matrix;

#[path="res/effects/bloom.rs"]
mod bloom;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Draw to texture and postprocess example").build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::SCREEN);

    sprite.draw(&layer, 0, (160., 120.), Color::WHITE);
    sprite.draw(&layer, 0, (130., 100.), Color::RED);
    sprite.draw(&layer, 0, (190., 100.), Color::GREEN);
    sprite.draw(&layer, 0, (160., 155.), Color::BLUE);

    // A custom example bloom effect postprocessor. The arguments define
    // bloom quality, bloom spread and brightness.
    // note: Radiant now also includes a predefined Bloom postprocessor. This example uses a similar implementation.
    let bloom_effect = bloom::Bloom::new(&renderer.context(), display.dimensions(), 2, 5, 10.0);

    let surface = Texture::new(&renderer.context(), 640, 480);
    let thumbnail = Texture::new(&renderer.context(), 640, 480);

    ru::renderloop(|frame| {
        display.prepare_frame();
        layer.view_matrix().rotate_at((160., 120.), frame.delta_f32);
        layer.model_matrix().rotate(frame.delta_f32 * 1.1);

        // Back up view matrix, then scale it based on elapsed time
        let prev_view_matrix = layer.view_matrix().clone();
        layer.view_matrix().scale_at((160., 120.), (frame.elapsed_f32.sin() + 2.) * 0.5);

        // This example simply combines rendering to textures with postprocessors.
        renderer.render_to(&surface, || {
            renderer.postprocess(&bloom_effect, &blendmodes::ALPHA, || {
                // Render to thumbnail...
                renderer.render_to(&thumbnail, || {
                    renderer.clear(Color::BLACK);
                    renderer.draw_layer(&layer, 0);
                });
                // ...but also copy to current render-target (the postprocessor input)
                renderer.copy_from(&thumbnail, TextureFilter::Linear);
            });
            renderer.draw_layer(&layer, 0);
        });

        // Draw processed texture to display. Also draw the original layer ontop.
        renderer.fill().blendmode(blendmodes::ALPHA).color(Color::alpha_mask(0.15)).draw();
        renderer.fill().blendmode(blendmodes::SCREEN).texture(&surface).draw();

        // Draw small thumbnails of the intermediate and final surface.
        // Note: copy_* are fast pixel copy operations (no shaders/blending/transforms). Coordinates are in pixels (integers).
        renderer.copy_rect_from(&thumbnail, ((0, 0), (640, 480)), ((512, 288), (128, 96)), TextureFilter::Linear);
        renderer.copy_rect_from(&surface, ((0, 0), (640, 480)), ((512, 384), (128, 96)), TextureFilter::Linear);

        // Draw color filtered variants of the thumbnail.
        renderer.rect(((469., 384.), (43., 32.))).blendmode(blendmodes::ALPHA).texture(&surface).color(Color::RED).draw();
        renderer.rect(((469., 416.), (43., 32.))).blendmode(blendmodes::ALPHA).texture(&surface).color(Color::GREEN).draw();
        renderer.rect(((469., 448.), (43., 32.))).blendmode(blendmodes::ALPHA).texture(&surface).color(Color::BLUE).draw();

        layer.set_view_matrix(prev_view_matrix);
        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
