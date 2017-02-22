extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, Program, utils};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Custom program example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/ball_v2_32x32x18.jpg").unwrap();

    // A custom shader program.
    let program = Program::from_string(&renderer.context(), include_str!("../res/ripple.fs")).unwrap();

    // Two layers, one with the default program, the other one with the custom program.
    let layer = Layer::new((320., 240.));
    let layer_custom = Layer::with_program((320., 240.), program);

    // Translate them to the left/right.
    layer.view_matrix().translate((-80., 0.));
    layer_custom.view_matrix().translate((80., 0.));

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());

        let frame_id = (frame.elapsed_f32 * 30.0) as u32;

        // Draw to both layers.
        for target in [ &layer_custom, &layer ].iter() {
            target.clear();
            sprite.draw(&target, frame_id, (160., 120.), Color::white());
            sprite.draw(&target, frame_id, (130., 100.), Color::red());
            sprite.draw(&target, frame_id, (190., 100.), Color::green());
            sprite.draw(&target, frame_id, (160., 155.), Color::blue());
            target.model_matrix().rotate(frame.delta_f32);
        }

        // Draw both layers.
        renderer.draw_layer(&layer, 0);
        renderer.draw_layer(&layer_custom, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
