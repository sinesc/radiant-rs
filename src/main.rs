extern crate radiant_rs;

use std::time::{Duration, Instant};
use radiant_rs::{Input, Color, Renderer, Vec3, Descriptor, Display};

/* to avoid rand dependency just for this "demo", not suitable for general use */
fn dummyrand(state: &mut f64) -> f32 /* 0..1 */ {
    let large = (*state as f64).sin() * 100000000.0;
    *state += 1.0;
    (large - large.floor())  as f32
}

fn main() {

    // initialize a display, and input source and a renderer

    let display = Display::new(Descriptor { /*monitor: 0,*/ width: 1024, height: 768, vsync: true, ..Descriptor::default() });
    let mut input = Input::new(&display);
    let renderer = Renderer::new(&display, 1500);

    // load some textures

    let test1 = renderer.texture(r"res/test_64x32x1.png");
    let test2 = renderer.texture(r"res/test_32x64x1.png");
    let test3 = renderer.texture(r"res/test_59x30x1.png");
    let sparkles = renderer.texture(r"res/sparkles_64x64x1.png");

    // set up two rendering layers

    let mut layer = renderer.layer();
    let mut persistent_layer = renderer.layer();
    persistent_layer.blend_lighten();

    // put some random sparkles on the persistent_layer (we'll draw it a couple of times, hence the name)

    let mut rand_state = 0.0;
    let radius = 500.0;

    for i in 0..1000 {
        let r = dummyrand(&mut rand_state) * radius / 2.0;
        let a = dummyrand(&mut rand_state) * 2.0 * 3.14157;
        let x = (radius / 2.0) + a.sin() * r;
        let y = (radius / 2.0) + a.cos() * r;
        let s = dummyrand(&mut rand_state);
        persistent_layer.sprite(sparkles, i, x as u32, y as u32, Color::white(), r, s, s);
    }

    let mut pm1 = persistent_layer.matrix.clone();
    let mut pm2 = persistent_layer.matrix.clone();
    let mut pm3 = persistent_layer.matrix.clone();
    let pos1 = 300.0;
    let pos2 = 300.0;
    let pos3 = 300.0;

    // the main loop
    start_loop(|| {

        // basic input

        input.poll();

        if input.alt_left {
            println!("hello!");
        }

        // add some sprites to render

        layer.sprite(test1, 50, 600, 600, Color::white(), 0.0, 1.0, 1.0);
        layer.sprite(test2, 50, 650, 650, Color::white(), 0.0, 1.0, 1.0);
        layer.sprite(test3, 50, 700, 700, Color::white(), 0.0, 1.0, 1.0);

        layer.matrix.rotate_z_at(Vec3(650.0, 650.0, 0.0), 0.005);

        // prepare render target, draw layers and swap

        renderer.prepare_and_clear_target(&Color::black());

        layer.draw().reset();

        pm1 .translate(Vec3(pos1, pos1, 0.0))
            .rotate_z(0.005)
            .translate(Vec3(-250.0, -250.0, 0.0));

        pm2 .translate(Vec3(pos2, pos2, 0.0))
            .rotate_z(0.004)
            .translate(Vec3(-250.0, -250.0, 0.0));

        pm3 .translate(Vec3(pos3, pos3, 0.0))
            .rotate_z(0.003)
            .translate(Vec3(-250.0, -250.0, 0.0));

        persistent_layer
            .model_matrix
            .rotate_z(0.01);

        persistent_layer.set_matrix(pm3).set_color(Color::lightness(0.25)).draw();

        persistent_layer
            .model_matrix
            .rotate_z(-0.005);

        persistent_layer.set_matrix(pm2).set_color(Color::lightness(0.5)).draw();
        persistent_layer.set_matrix(pm1).set_color(Color::lightness(1.0)).draw();


        pm1 .translate(Vec3(250.0, 250.0, 0.0))
            .translate(Vec3(-pos1, -pos1, 0.0));

        pm2 .translate(Vec3(250.0, 250.0, 0.0))
            .translate(Vec3(-pos2, -pos2, 0.0));

        pm3 .translate(Vec3(250.0, 250.0, 0.0))
            .translate(Vec3(-pos3, -pos3, 0.0));

        renderer.swap_target();

        if input.should_close { Action::Stop } else { Action::Continue }
    });
}

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    let frame_interval = Duration::new(0, 16666667);

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;

    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => ()
        };

        let now = Instant::now();

        // determine thread sleep to maintain X FPS
        accumulator += now - previous_clock;

        while accumulator >= frame_interval {
            accumulator -= frame_interval;
            // if you have a game, update the state here
        }

        // framerate print
        second_elapsed += now - previous_clock;
        frames_elapsed += 1;
        if second_elapsed >= second {
            println!("Frames rendered: {}", frames_elapsed);
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
        //thread::sleep(frame_interval - accumulator);
    }
}
