extern crate radiant_rs;

//use std::thread;
use std::time::{Duration, Instant};
use radiant_rs::{Input, Color, Renderer, Descriptor, Display};

fn main() {

    // initialize a display, and input source and a renderer

    let max_sprites = 1500;
    let display = Display::new(Descriptor { /*monitor: 0,*/ width: 1024, height: 768, vsync: true, ..Descriptor::default() });
    let mut input = Input::new(&display);
    let renderer = Renderer::new(&display, max_sprites);

    // load some textures

    let test1 = renderer.texture(r"res/test_64x32x1.png");
    let test2 = renderer.texture(r"res/test_32x64x1.png");
    let test3 = renderer.texture(r"res/test_59x30x1.png");
    let sparkles = renderer.texture(r"res/sparkles_64x64x1.png");
    let spark = renderer.texture(r"res/basic_64x64x1.png");

    // set up two rendering layers, switch to lighten blend mode for one

    let mut layer = renderer.layer();
    let mut persistent_layer = renderer.layer();
    persistent_layer.blend_lighten();

    // put some random sparkles on the persistent_layer (we'll draw to it only once, hence the name)

    let mut rand_state = 0.0;
    let radius = 600.0;

    for i in 0..max_sprites {
        let l = sinrand(&mut rand_state);
        let r = l * radius / 2.0;
        let a = sinrand(&mut rand_state) * 2.0 * 3.14157;
        let x = (radius / 2.0) + a.sin() * r;
        let y = (radius / 2.0) + a.cos() * r;
        let s = sinrand(&mut rand_state);
        if sinrand(&mut rand_state) > 0.90 {
            let temperature = sinrand(&mut rand_state) * (40000.0 - 2000.0) + 2000.0;
            persistent_layer.sprite(spark, i, x as u32, y as u32, Color::temperature(temperature, 1.0).scale(2.0-l), r, 0.2, 0.2);
        } else {
            let temperature = sinrand(&mut rand_state) * (15000.0 - 2000.0) + 2000.0;
            persistent_layer.sprite(sparkles, i, x as u32, y as u32, Color::temperature(temperature, 1.0).scale(1.0-l), r, s, s);
        }
    }

    persistent_layer.view_matrix.translate((150.0, 100.0));

    // clone a couple of view matricies

    let mut pv1 = persistent_layer.view_matrix.clone();
    let mut pv2 = persistent_layer.view_matrix.clone();
    let mut pv3 = persistent_layer.view_matrix.clone();
    pv1.rotate_z_at((radius / 2.0, radius / 2.0), 1.0);
    pv2.rotate_z_at((radius / 2.0, radius / 2.0), 2.0);
    pv3.rotate_z_at((radius / 2.0, radius / 2.0), 3.0);
    pv1.scale((0.9, 0.9)).translate((15.0, 10.0));

    // model matricies as well

    let mut pm1 = persistent_layer.model_matrix.clone();
    let mut pm2 = persistent_layer.model_matrix.clone();
    let mut pm3 = persistent_layer.model_matrix.clone();

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

        // some matrix games: prepare 3 view and model matricies to rotate the entire layer and each sprite per layer

        layer.view_matrix.rotate_z_at((650.0, 650.0), 0.005);

        pv1.rotate_z_at((radius / 2.0, radius / 2.0), 0.0009);
        pv2.rotate_z_at((radius / 2.0, radius / 2.0), 0.0007);
        pv3.rotate_z_at((radius / 2.0, radius / 2.0), 0.0004);
        pm1.rotate_z(-0.005);
        pm2.rotate_z(0.004);
        pm3.rotate_z(0.003);

        // prepare render target, required before drawing

        renderer.prepare_and_clear_target(Color::black());

        // draw the boring layer once

        layer.draw().reset();

        // draw the persistent layer 3 times with different model- and view matricies and brightness

        persistent_layer
            .set_view_matrix(pv3)
            .set_model_matrix(pm3)
            .set_color(Color::lightness(0.25))
            .draw();

        persistent_layer
            .set_view_matrix(pv2)
            .set_model_matrix(pm2)
            .set_color(Color::lightness(0.5))
            .draw();

        persistent_layer
            .set_view_matrix(pv1)
            .set_model_matrix(pm1)
            .set_color(Color::lightness(1.0))
            .draw();

        renderer.swap_target();

        // exit on window close

        if input.should_close { Action::Stop } else { Action::Continue }
    });
}



/* to avoid rand dependency just for this "demo", not suitable for general use */
fn sinrand(state: &mut f64) -> f32 /* 0..1 */ {
    let large = (*state as f64).sin() * 100000000.0;
    *state += 1.0;
    (large - large.floor())  as f32
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
