extern crate radiant_rs;
use radiant_rs::*;

#[path="../res/effects/bloom.rs"]
mod bloom;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Swirling blobs demo").build();
    let renderer = Renderer::new(&display).unwrap();
    let input = Input::new(&display);

    let text_layer = Layer::new((640., 480.));
    let spark_layer = Layer::new((640., 480.));
    spark_layer.set_blendmode(blendmodes::LIGHTEN);

    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/sparkles_64x64x1.png").unwrap();
    let font = Font::builder(&renderer.context()).family("Arial").size(12.0).build().unwrap();
    let big_font = font.with_size(24.0);

    // Clone a couple of layer matrices to play around with
    let mut view1 = spark_layer.view_matrix().clone();
    let mut view2 = spark_layer.view_matrix().clone();
    let mut view3 = spark_layer.view_matrix().clone();
    let mut model = *spark_layer.model_matrix().clone().scale(4.0);

    // This is a userdefined postprocessor to add a bloom effect
    let bloom_effect = bloom::Bloom::new(&renderer.context(), 2, 5, 4.0);

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());
        spark_layer.clear();
        text_layer.clear();

        // Rotate the model matrix
        spark_layer.set_model_matrix(*model.rotate(-frame.delta_f32 * 4.0));

        // Rotate the three viewmatrix clones at different rates
        view1.rotate_at((320., 200.), frame.delta_f32 * 1.0);
        view2.rotate_at((320., 200.), frame.delta_f32 * 1.5);
        view3.rotate_at((320., 200.), frame.delta_f32 * 2.0);

        // Draw the sprite three times, tinted red, green and blue
        sprite.draw(&spark_layer, 0, (320., 180.), *Color::red().scale(1.5));
        sprite.draw(&spark_layer, 0, (300., 200.), *Color::green().scale(1.5));
        sprite.draw(&spark_layer, 0, (340., 200.), *Color::blue().scale(1.5));

        // Draw the spark layer three times with different matrices and alpha levels
        if (frame.elapsed_f32 / 1.5) as u32 % 2 == 0 {
            // Postprocesses version
            renderer.postprocess(&bloom_effect, &blendmodes::COPY, || {
                renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.125)).set_view_matrix(view1), 0);
                renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.5)).set_view_matrix(view2), 0);
                renderer.draw_layer(&spark_layer.set_color(Color::alpha(1.0)).set_view_matrix(view3), 0);
            });
            font.write(&text_layer, "Custom postprocessor: enabled", (240., 450.), Color::white());
        } else {
            // Unprocessed version
            renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.125)).set_view_matrix(view1), 0);
            renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.5)).set_view_matrix(view2), 0);
            renderer.draw_layer(&spark_layer.set_color(Color::alpha(1.0)).set_view_matrix(view3), 0);
            font.write(&text_layer, "Custom postprocessor: disabled", (240., 450.), Color::white());
        }

        // Draw text
        big_font.write(&text_layer, "blobs.rs", (355., 330.), Color::red());
        font.write(&text_layer, "rotating colorful blobs since 2016", (370., 350.), Color::white());
        renderer.draw_layer(&text_layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
