#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate radiant_rs;

//use std::thread;
use std::time::{Duration, Instant};
use std::ops::Deref;
use std::sync::mpsc::sync_channel;
use radiant_rs::{Input, Color, Renderer, Layer, Descriptor, Display, Scene, Operation, blendmodes, utils};

//use radiant_rs::avec::AVec;
use std::thread;
use std::sync::Arc;

fn main() {
/*
    let myvec = Arc::new(AVec::<u32>::new(100));

    for i in 0..8 {
        let local_vec = myvec.clone();
        let thread_id = i;
        thread::spawn(move || {
            let mut vec = local_vec.map(thread_id);
            for n in 0..thread_id {
                thread::sleep(Duration::new(0, 16666667));
                vec[n as usize] = thread_id;
            }
        });
    }

    for _ in 0..10 {

        let vector = myvec.get();
        for i in 0..vector.len() {
            print!("{} ", vector[i]);
        }

        thread::sleep(Duration::new(0, 16666667));

        println!("--");
    }

    let another = myvec.get();
    for i in 0..another.len() {
        print!("x{} ", another[i]);
    }

    println!("--");
*/

    // initialize a display, and input source and a renderer

    let max_sprites = 15000;
    let display = Display::new(Descriptor { /*monitor: 0,*/ width: 1024, height: 768, vsync: false, ..Descriptor::default() });
    let mut input = Input::new(&display);
    let renderer = Renderer::new(&display, max_sprites);

    // load some textures

    let test1 = renderer.create_sprite(r"res/test_64x32x1.png");
    let test2 = renderer.create_sprite(r"res/test_32x64x1.png");
    let test3 = renderer.create_sprite(r"res/test_59x30x1.png");
    let sparkles = renderer.create_sprite(r"res/sparkles_64x64x1.png");
    let spark = renderer.create_sprite(r"res/basic_64x64x1.png");
    let font = renderer.create_font("who cares");

    let (tx, rx) = sync_channel(1);

    // create a scene

    let main_scene = Arc::new(Scene::new(max_sprites, display.dimensions()));

    let scene = main_scene.clone();
    thread::spawn(move || {

        let logo = scene.add_layer();
        let galaxy = scene.add_layer();

        // put some random sparkles on the persistent_layer (we'll draw to it only once, hence the name)

        let mut rng = utils::Rng::new(0.0);
        let radius = 600.0;

        for i in 0..1500 {
            let l = rng.get::<f32>();
            let r = l * radius / 2.0;
            let a = rng.range(0.0f32, 2.0 * 3.14157);
            let x = (radius / 2.0) + a.sin() * r;
            let y = (radius / 2.0) + a.cos() * r;
            let s = rng.get::<f32>();
            if rng.get::<f32>() > 0.90 {
                let temperature = rng.range(4000.0f32, 10000.0);
                scene.sprite(galaxy, spark, i, x, y, Color::temperature(temperature, 1.0).scale(2.0-l), r, 0.2, 0.2);
            } else {
                let temperature = rng.range(4000.0f32, 10000.0);
                scene.sprite(galaxy, sparkles, i, x, y, Color::temperature(temperature, 1.0).scale(1.0-l), r, s, s);
            }
        }
let persistent_layer = scene.layer(galaxy);
let layer = scene.layer(logo);
// !todo
        persistent_layer.set_blendmode(blendmodes::OVERLAY);
        persistent_layer.view_matrix().translate((150.0, 100.0));

        // clone a couple of view matricies

        let mut pv1 = persistent_layer.view_matrix().clone();
        let mut pv2 = persistent_layer.view_matrix().clone();
        let mut pv3 = persistent_layer.view_matrix().clone();
        pv1.rotate_z_at((radius / 2.0, radius / 2.0), 1.0);
        pv2.rotate_z_at((radius / 2.0, radius / 2.0), 2.0);
        pv3.rotate_z_at((radius / 2.0, radius / 2.0), 3.0);
        pv1.scale((0.9, 0.9)).translate((15.0, 10.0));

        // model matricies as well

        let mut pm1 = persistent_layer.model_matrix().clone();
        let mut pm2 = persistent_layer.model_matrix().clone();
        let mut pm3 = persistent_layer.model_matrix().clone();

        font.write(persistent_layer, "Hello world how are you", 100.0, 100.0, 200, Color::white(), 1.0, 1.0, 1.0);

        utils::mainloop(Duration::new(0, 16666666), |state| { true }, |state| {

            // add some sprites to render

            layer.clear();
            test1.draw(layer, 50, 600., 600., Color::white(), 0.0, 1.0, 1.0);
            test2.draw(layer, 50, 650., 650., Color::white(), 0.0, 1.0, 1.0);
            test3.draw(layer, 50, 700., 700., Color::white(), 0.0, 1.0, 1.0);

            // some matrix games: prepare 3 view and model matricies to rotate the entire layer and each sprite per layer

            layer.view_matrix().rotate_z_at((650.0, 650.0), 0.3 * state.delta_f32);

            pv1.rotate_z_at((radius / 2.0, radius / 2.0), 0.054 * state.delta_f32);
            pv2.rotate_z_at((radius / 2.0, radius / 2.0), 0.042 * state.delta_f32);
            pv3.rotate_z_at((radius / 2.0, radius / 2.0), 0.024 * state.delta_f32);
            pm1.rotate_z(-0.15 * state.delta_f32);
            pm2.rotate_z(0.12 * state.delta_f32);
            pm3.rotate_z(0.09 * state.delta_f32);

            scene.clear().ops(&[
                Operation::Draw(logo),
/*
                Operation::SetViewMatrix(galaxy, pv3),
                Operation::SetModelMatrix(galaxy, pm3),
                Operation::SetColor(galaxy, Color::lightness(0.25)),
                Operation::Draw(galaxy),

                Operation::SetViewMatrix(galaxy, pv2),
                Operation::SetModelMatrix(galaxy, pm2),
                Operation::SetColor(galaxy, Color::lightness(0.5)),
                Operation::Draw(galaxy),
*/
                //Operation::SetViewMatrix(galaxy, pv1),
                //Operation::SetModelMatrix(galaxy, pm1),
                Operation::SetColor(galaxy, Color::lightness(1.0)),
                Operation::Draw(galaxy),

            ]);

            // this will panic when the main thread exists
            tx.send(1).unwrap(); // start drawing
            tx.send(1).unwrap(); // drawing finished

            true
        });
    });


    // the main loop

    utils::mainloop(Duration::new(0, 16666666), |state| { true }, |state| {

        // basic input

        input.poll();

        if input.alt_left {
            println!("hello!");
        }

        // prepare render target, required before drawing

        let x = rx.recv();
        renderer.prepare_and_clear_target(Color::black());
        renderer.draw_scene(main_scene.deref());
        renderer.swap_target();
        let y = rx.recv();

        !input.should_close
    });
}
