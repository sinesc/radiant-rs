extern crate radiant_rs;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, Program, utils};

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Custom program example").build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/ball_v2_32x32x18.jpg").unwrap();

    // A custom shader program.
    let program = Program::from_string(&renderer.context(), include_str!("../res/effects/ripple.fs")).unwrap();

    // Two layers, one with the default program, the other one with the custom program.
    // Cloning a layer like this creates a new layer that references the contents of the
    // source layer but has its own matrices, program, and so on.
    let layer = Layer::new((320., 240.));
    let layer_custom = layer.clone_with_program(program);

    // Translate them to the left/right.
    layer.view_matrix().translate((-80., 0.));
    layer_custom.view_matrix().translate((80., 0.));

    utils::renderloop(|frame| {
        display.clear_frame(Color::BLACK);
        layer.clear();

        // Draw to "both" layers.
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::WHITE);
        sprite.draw(&layer, frame_id, (130., 100.), Color::RED);
        sprite.draw(&layer, frame_id, (190., 100.), Color::GREEN);
        sprite.draw(&layer, frame_id, (160., 155.), Color::BLUE);

        // Draw both layers.
        renderer.draw_layer(&layer, 0);
        renderer.draw_layer(&layer_custom, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
