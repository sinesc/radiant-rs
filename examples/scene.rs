extern crate radiant_rs;
use radiant_rs::*;
use radiant_rs::scene::*;

fn main() {

    // create display and renderer
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display);

    // create a scene with one layer and load a sprite for later use
    let scene = Scene::new(&renderer.context());
    let layer_id = scene.create_layer(640, 480);
    let sprite_id = scene.create_sprite_from_file("res/sparkles_64x64x1.png");

    // define a few scene operations to be run each frame
    scene.op(Op::SetBlendmode(layer_id, blendmodes::MAX));
    scene.op(Op::RotateViewMatrixAt(layer_id, 1.0, Vec2(320.0, 240.0), 1.0));
    scene.op(Op::RotateModelMatrix(layer_id, 1.0, -2.0));
    scene.op(Op::Draw(layer_id));

    // randomly draw some sprites onto the scene's layer
    let mut rand = utils::Rng::new(5339.0);
    for _ in 0..10000 {
        scene.sprite(layer_id, sprite_id, 0, rand.range(-160.0, 800.0), rand.range(-160.0, 800.0), Color(rand.get(), rand.get(), rand.get(), rand.get()));
    }

    // keep drawing the scene
    utils::renderloop(|frame| {
        renderer.clear_target(Color::black());
        renderer.draw_scene(&scene, frame.delta_f32);
        renderer.swap_target();
        !display.poll_events().was_closed()
    });
}
