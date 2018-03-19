extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, Program, blendmodes, postprocessors};
use ru::Matrix;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Basic postprocessor example").build().unwrap();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    sprite.draw(&layer, 0, (160., 120.), Color::WHITE);
    sprite.draw(&layer, 0, (130., 100.), Color::RED);
    sprite.draw(&layer, 0, (190., 100.), Color::GREEN);
    sprite.draw(&layer, 0, (160., 155.), Color::BLUE);

    // Load a shader progam.
    let program = Program::from_string(&renderer.context(), include_str!("res/effects/ripple.fs")).unwrap();

    // Use a default Basic postprocessor with the given program. It simply draws the input
    // using the given program, but there is a trait to implement custom postprocessors.
    let ripple_effect = postprocessors::Basic::new(&renderer.context(), program);

    ru::renderloop(|frame| {
        display.clear_frame(Color::BLACK);
        layer.view_matrix().rotate_at((160., 120.), frame.delta_f32);
        layer.model_matrix().rotate(frame.delta_f32 * 1.1);

        // Drawing within Renderer::postprocess() applies the given postprocessor to the result
        // This particular postprocessor takes a blendmode as argument, which is provided here with blendmodes::LIGHTEN.
        renderer.postprocess(&ripple_effect, &blendmodes::LIGHTEN, || {
            renderer.clear(Color::TRANSPARENT);
            renderer.draw_layer(&layer, 0);
        });

        renderer.draw_layer(&layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
