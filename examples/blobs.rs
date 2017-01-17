extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Input, InputId, Layer, Sprite, Font, FontInfo, Color, blendmodes, utils};

pub fn main() {

    // create a window, a renderer and some basic input handler for the window
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: false, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display);
    let input = Input::new(&display);
    let context = renderer.context();

    // create three layers, change one to use an overlay blend mode
    let text_layer = Layer::new(640, 480);
    let spark_layer = Layer::new(640, 480);
    let fps_layer = Layer::new(640, 480);
    spark_layer.set_blendmode(blendmodes::LIGHTEN);

    // create a sprite and some fonts
    let sprite = Sprite::from_file(&context, r"examples/res/sparkles_64x64x1.png");
    let font = Font::from_info(&context, FontInfo { family: "Arial".to_string(), size: 12.0, ..FontInfo::default() } );
    let big_red_font = font.with_size(24.0).with_color(Color::red());

    // write text to layer only once and reuse every frame
    big_red_font.write(&text_layer, "examples/blobs.rs", 355.0, 330.0);
    font.write(&text_layer, "rotating colorful blobs since 2016", 370.0, 350.0);

    // clone a couple of layer matrices to play around with
    let mut view1 = spark_layer.view_matrix().clone();
    let mut view2 = spark_layer.view_matrix().clone();
    let mut view3 = spark_layer.view_matrix().clone();
    let mut model = *spark_layer.model_matrix().clone().scale(4.0);

    // a simple mainloop helper (just an optional utility function)
    utils::renderloop(|state| {

        // clear the layer containing the sparks and rotate its model matrix  (per-sprite matrix)
        spark_layer.clear();
        spark_layer.set_model_matrix(*model.rotate(-state.delta_f32 * 4.0));

        font.write(&fps_layer.clear(), &format!("{}FPS", state.fps), 10.0, 10.0);

        // rotate the three viewmatrix clones at different rates
        view1.rotate_at((320.0, 200.0), state.delta_f32 * 1.0);
        view2.rotate_at((320.0, 200.0), state.delta_f32 * 1.5);
        view3.rotate_at((320.0, 200.0), state.delta_f32 * 2.0);

        // draw the sprite three times, tinted red, green and blue
        sprite.draw(&spark_layer, 0, 320.0, 180.0, Color::red().scale(1.5));
        sprite.draw(&spark_layer, 0, 300.0, 200.0, Color::green().scale(1.5));
        sprite.draw(&spark_layer, 0, 340.0, 200.0, Color::blue().scale(1.5));

        // draw the spark layer three times with different matrices and alpha levels as well as the text layer
        renderer.clear_target(Color::black());
        renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.125)).set_view_matrix(view1));
        renderer.draw_layer(&spark_layer.set_color(Color::alpha(0.5)).set_view_matrix(view2));
        renderer.draw_layer(&spark_layer.set_color(Color::alpha(1.0)).set_view_matrix(view3));
        renderer.draw_layer(&text_layer);
        renderer.draw_layer(&fps_layer);
        renderer.swap_target();

        !display.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
