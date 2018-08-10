extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::*;
use ru::Matrix;

pub fn main() {

    // Set up shared context
    let context = Context::new();

    // Create three displays using the shared context.
    let display1 = Display::builder().dimensions((640, 480)).vsync().title("Window 1").context(&context).build().unwrap();
    let display2 = Display::builder().dimensions((640, 480)).vsync().title("Window 2").context(&context).build().unwrap();
    let display3 = Display::builder().dimensions((640, 480)).vsync().title("Window 3").context(&context).build().unwrap();

    // Setup renderer defaulting to window 1.
    let renderer = Renderer::headless(&context).unwrap();
    let input = Input::new(&display1);

    // Create a layers to draw to.
    let spark_layer = Layer::new((640., 480.));
    spark_layer.set_blendmode(blendmodes::LIGHTEN);
    spark_layer.model_matrix().scale(4.0);

    // Load sprite and draw it three times, tinted red, green and blue. No need to do this each frame since we're
    // only going to manipulate the matrices.
    let sprite = Sprite::from_file(&context, r"examples/res/sprites/sparkles2_64x64x1.png").unwrap();
    sprite.draw(&spark_layer, 0, (320., 180.), *Color::RED.scale(1.5));
    sprite.draw(&spark_layer, 0, (300., 200.), *Color::GREEN.scale(1.5));
    sprite.draw(&spark_layer, 0, (340., 200.), *Color::BLUE.scale(1.5));

    // Clone a couple of layer matrices to play around with.
    let mut view1 = spark_layer.view_matrix().clone();
    let mut view2 = spark_layer.view_matrix().clone();
    let mut view3 = spark_layer.view_matrix().clone();

    ru::renderloop(|frame| {
        display1.prepare_frame();
        display2.prepare_frame();
        display3.prepare_frame();

        // Prepare some matrices.
        spark_layer.model_matrix().rotate(-frame.delta_f32 * 4.0);
        view1.rotate_at((320., 200.), frame.delta_f32 * 1.0);
        view2.rotate_at((320., 200.), frame.delta_f32 * 1.5);
        view3.rotate_at((320., 200.), frame.delta_f32 * 2.0);

        // Render to window 1.
        renderer.render_to(&display1, || {
            renderer.fill().color(Color(0.0, 0.0, 0.0, 0.03)).draw();
            renderer.draw_layer(spark_layer.set_color(Color::RED).set_view_matrix(view1), 0);
            renderer.draw_layer(spark_layer.set_color(Color::RED).set_view_matrix(view2), 0);
            renderer.draw_layer(spark_layer.set_color(Color::RED).set_view_matrix(view3), 0);
        });

        // Render to window 2.
        renderer.render_to(&display2, || {
            renderer.fill().color(Color(0.0, 0.0, 0.0, 0.03)).draw();
            renderer.draw_layer(spark_layer.set_color(Color::GREEN).set_view_matrix(view1), 0);
            renderer.draw_layer(spark_layer.set_color(Color::GREEN).set_view_matrix(view2), 0);
            renderer.draw_layer(spark_layer.set_color(Color::GREEN).set_view_matrix(view3), 0);
        });

        // Render to window 3.
        renderer.render_to(&display3, || {
            renderer.fill().color(Color(0.0, 0.0, 0.0, 0.03)).draw();
            renderer.draw_layer(spark_layer.set_color(Color::BLUE).set_view_matrix(view1), 0);
            renderer.draw_layer(spark_layer.set_color(Color::BLUE).set_view_matrix(view2), 0);
            renderer.draw_layer(spark_layer.set_color(Color::BLUE).set_view_matrix(view3), 0);
        });

        display1.swap_frame();
        display2.swap_frame();
        display3.swap_frame();
        !display1.poll_events().was_closed() && !input.down(InputId::Escape)
    });
}
