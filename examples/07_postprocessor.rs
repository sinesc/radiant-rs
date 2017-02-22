extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, Program, utils, blendmodes, postprocessors};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Basic postprocessor example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    sprite.draw(&layer, 0, (160., 120.), Color::white());
    sprite.draw(&layer, 0, (130., 100.), Color::red());
    sprite.draw(&layer, 0, (190., 100.), Color::green());
    sprite.draw(&layer, 0, (160., 155.), Color::blue());

    // Load a shader progam.
    let program = Program::from_string(&renderer.context(), include_str!("../res/ripple.fs")).unwrap();

    // Use a default Basic postprocessor with the given program. It simply draws the input
    // using the given program, but there is a trait to implement custom postprocessors.
    let ripple_effect = postprocessors::Basic::new(&renderer.context(), program);

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());
        layer.view_matrix().rotate_at((160., 120.), frame.delta_f32);
        layer.model_matrix().rotate(frame.delta_f32 * 1.1);

        // Drawing within Renderer::postprocess() applies the given postprocessor to the result
        // This particular postprocessor takes a blendmode as argument, which is provided here with blendmodes::LIGHTEN.
        // Notice the similarity to rendering to textures.
        renderer.postprocess(&ripple_effect, &blendmodes::LIGHTEN, || {
            renderer.clear(Color::transparent());
            renderer.draw_layer(&layer, 0);
        });

        renderer.draw_layer(&layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
