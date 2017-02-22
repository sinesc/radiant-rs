extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Color, blendmodes, utils, Vec2};
use radiant_rs::scene::{Scene, Op};

// !note that the Scene API is WIP and will likely change a lot. Scenes are intended as thread-safe
// convenience containers to reduce the amount of individual items the programmer has to pass around
// in his application. They are not intended to become classical scene graphs.

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Scene example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();

    // Create a scene with one layer and load a sprite for later use
    let scene = Scene::new(&renderer.context());
    let layer_id = scene.register_layer((640., 480.), 0);
    let sprite_id = scene.register_sprite_from_file("res/sparkles_64x64x1.png").unwrap();

    // Define a few scene operations to be run each frame
    scene.op(Op::SetBlendmode(layer_id, blendmodes::MAX));
    scene.op(Op::RotateViewMatrixAt(layer_id, 1.0, Vec2(320., 240.), 1.0));
    scene.op(Op::RotateModelMatrix(layer_id, 1.0, -2.0));
    scene.op(Op::Draw(layer_id));

    // Randomly draw some sprites onto the scene's layer
    let mut rand = utils::Rng::new(1234.0);

    for _ in 0..10000 {
        scene.sprite(layer_id, sprite_id, 0, (rand.range(-160., 800.), rand.range(-160., 800.)), Color(rand.get(), rand.get(), rand.get(), rand.get()));
    }

    utils::renderloop(|frame| {

        // Instead of drawing individual layers, we draw the entire scene.
        display.clear_frame(Color::black());
        renderer.draw_scene(&scene, frame.delta_f32);
        display.swap_frame();

        !display.poll_events().was_closed()
    });
}
