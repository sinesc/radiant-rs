extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, Program, utils, blendmodes, postprocessors};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Basic postprocessor example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((320., 240.), 0);

    // Load a shader progam
    let program = Program::from_string(&renderer.context(), include_str!("../res/ripple.fs")).unwrap();

    // Use a default Basic postprocessor with the given program. It simply draws the input
    // using the given program, but there is a trait to implement custom postprocessors.
    let mut ripple_effect = postprocessors::Basic::new(&renderer.context(), program, "tex");

    utils::renderloop(|frame| {
        layer.clear();

        // Draw the four sprites from sprite.rs again
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::white());
        sprite.draw(&layer, frame_id, (130., 100.), Color::red());
        sprite.draw(&layer, frame_id, (190., 100.), Color::green());
        sprite.draw(&layer, frame_id, (160., 155.), Color::blue());

        display.clear_frame(Color::black());

        // Drawing within Renderer::postprocess() applies the given postprocessor to the result
        renderer.postprocess(&mut ripple_effect, &blendmodes::LIGHTEN, || {
            renderer.draw_layer(&layer);
        });

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
