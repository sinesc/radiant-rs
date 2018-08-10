extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::*;
use ru::Matrix;

#[path="res/effects/bloom.rs"]
mod bloom;

pub fn main() {
    // Setup input/display.
    let display = Display::builder().dimensions((640, 480)).vsync().title("Swirling blobs demo").build().unwrap();
    let renderer = Renderer::new(&display).unwrap();
    let input = Input::new(&display);

    // Create two layers to draw to.
    let text_layer = Layer::new((640., 480.));
    let spark_layer = Layer::new((640., 480.));
    spark_layer.set_blendmode(blendmodes::LIGHTEN);
    spark_layer.model_matrix().scale(4.0);

    // This is a userdefined postprocessor to add a bloom effect.
    let bloom_effect = bloom::Bloom::new(display.context(), display.dimensions(), 2, 5, 4.0);

    // Load sprite and fonts.
    let sprite = Sprite::from_file(display.context(), r"examples/res/sprites/sparkles2_64x64x1.png").unwrap();
    let font = Font::builder(display.context()).family("Arial").size(12.0).build().unwrap();
    let big_font = font.clone_with_size(24.0);

    // Draw the sprite three times, tinted red, green and blue. No need to do this each frame since we're
    // only going to manipulate the matrices. Also write some text.
    sprite.draw(&spark_layer, 0, (320., 180.), *Color::RED.scale(1.5));
    sprite.draw(&spark_layer, 0, (300., 200.), *Color::GREEN.scale(1.5));
    sprite.draw(&spark_layer, 0, (340., 200.), *Color::BLUE.scale(1.5));
    big_font.write(&text_layer, "blobs.rs", (355., 330.), Color::RED);
    font.write(&text_layer, "rotating colorful blobs since 2016", (370., 350.), Color::WHITE);

    // Clone a couple of layer matrices to play around with
    let mut view1 = spark_layer.view_matrix().clone();
    let mut view2 = spark_layer.view_matrix().clone();
    let mut view3 = spark_layer.view_matrix().clone();

    ru::renderloop(|frame| {
        display.clear_frame(Color::BLACK);

        // Rotate the model matrix.
        spark_layer.model_matrix().rotate(-frame.delta_f32 * 4.0);

        // Rotate the three viewmatrix clones at different rates.
        view1.rotate_at((320., 200.), frame.delta_f32 * 1.0);
        view2.rotate_at((320., 200.), frame.delta_f32 * 1.5);
        view3.rotate_at((320., 200.), frame.delta_f32 * 2.0);

        // Draw the spark layer three times with different matrices and alpha levels.
        renderer.postprocess(&bloom_effect, &blendmodes::COPY, || {
            renderer.fill().color(Color(0.0, 0.0, 0.0, 0.02)).draw();
            renderer.draw_layer(spark_layer.set_color(Color::alpha(0.125)).set_view_matrix(view1), 0);
            renderer.draw_layer(spark_layer.set_color(Color::alpha(0.5)).set_view_matrix(view2), 0);
            renderer.draw_layer(spark_layer.set_color(Color::alpha(1.0)).set_view_matrix(view3), 0);
        });

        // Draw the text layer.
        renderer.draw_layer(&text_layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
