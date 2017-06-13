extern crate radiant_rs;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, Program, Texture, TextureFilter, utils, blendmodes, postprocessors};

#[path="../res/effects/bloom.rs"]
mod bloom;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Draw to texture and postprocess example").build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    sprite.draw(&layer, 0, (160., 120.), Color::white());
    sprite.draw(&layer, 0, (130., 100.), Color::red());
    sprite.draw(&layer, 0, (190., 100.), Color::green());
    sprite.draw(&layer, 0, (160., 155.), Color::blue());

    // The basic postprocessor previously introduced.
    let program = Program::from_string(&renderer.context(), include_str!("../res/effects/ripple.fs")).unwrap();
    let ripple_effect = postprocessors::Basic::new(&renderer.context(), program);

    // A custom example bloom effect postprocessor. Here, the arguments define the bloom
    // quality, bloom spread and brightness.
    let bloom_effect = bloom::Bloom::new(&renderer.context(), 2, 5, 10.0);

    let surface = Texture::new(&renderer.context(), 640, 480);
    let thumbnail = Texture::new(&renderer.context(), 640, 480);
    let darken = Color(0., 0., 0., 0.06);

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());
        layer.view_matrix().rotate_at((160., 120.), frame.delta_f32);
        layer.model_matrix().rotate(frame.delta_f32 * 1.1);

        // Back up view matrix, then scale it based on elapsed time
        let prev_view_matrix = layer.view_matrix().clone();
        layer.view_matrix().scale_at((160., 120.), frame.elapsed_f32.sin() + 1.0 * 2.0);

        // This example simply combines rendering to textures with postprocessors.
        renderer.render_to(&surface, || {
            renderer.postprocess(&bloom_effect, &blendmodes::LIGHTEN, || {
                renderer.postprocess(&ripple_effect, &blendmodes::LIGHTEN, || {
                    renderer.render_to(&thumbnail, || {
                        renderer.clear(Color::transparent());
                        renderer.draw_layer(&layer, 0);
                    });
                    renderer.fill().blendmode(blendmodes::ALPHA).color(darken).draw();
                    renderer.copy_from(&thumbnail, TextureFilter::Linear);
                });
            });
            renderer.fill().blendmode(blendmodes::ALPHA).color(darken).draw();
        });

        // Draw processed texture to display. Also draw the original layer ontop.
        renderer.copy_from(&surface, TextureFilter::Linear);
        renderer.draw_layer(&layer, 0);

        // Draw small thumbnails of the intermediate and final surface
        renderer.copy_rect_from(&thumbnail, (0, 0, 640, 480), (512, 288, 128, 96), TextureFilter::Linear);
        renderer.copy_rect_from(&surface, (0, 0, 640, 480), (512, 384, 128, 96), TextureFilter::Linear);

        layer.set_view_matrix(prev_view_matrix);
        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
