extern crate radiant_rs;

use std::thread;
use std::time::{Duration, Instant};
use radiant_rs::{Input, Color, Renderer, Vec3, Descriptor, Display};

fn main() {

    let display = Display::new(Descriptor { /*monitor: 0,*/ width: 1024, height: 768, ..Descriptor::default() });
    let mut input = Input::new(&display);
    let renderer = Renderer::new(&display, 1000);

    let asteroid = renderer.texture(r"C:\Users\nyda\Projekte\#js\ferocitylib\demo\www\sprite\asteroid\type1_64x64x60.png");
    let mine = renderer.texture(r"C:\Users\nyda\Projekte\#js\ferocitylib\demo\www\sprite\hostile\radial_64x64x1.png");
    let powerup = renderer.texture(r"C:\Users\nyda\Projekte\#js\ferocitylib\demo\www\sprite\powerup\ball_h_32x32x18.jpg");
    let test = renderer.texture(r"C:\Users\nyda\Projekte\radiant-rs\res\test_64x32x1.png");
    let test2 = renderer.texture(r"C:\Users\nyda\Projekte\radiant-rs\res\test_32x64x1.png");
    let test3 = renderer.texture(r"C:\Users\nyda\Projekte\radiant-rs\res\test_59x30x1.png");

    let mut layer = renderer.layer();
    let mut testlayer = renderer.layer();
    let mut testlayer2 = renderer.layer();
    testlayer2.blend_overlay();

    let mut rot = 0.0;
    let mut scale = 1.0;
    let mut scaler = 0.1;
    let mut frame = 0;

    // the main loop
    start_loop(|| {

        // basic input

        input.poll();

        if input.alt_left {
            println!("hello!");
        }

        // add some sprites to render

        layer.sprite(&asteroid, 0, 40, 40, Color(255, 255, 0, 255), 0.0, 1.0, 1.0);
        layer.sprite(&asteroid, 0, 80, 80, Color(255, 0, 255, 255), 0.0, 1.0, 1.0);
        layer.sprite(&asteroid, 10, 100, 100, Color(0, 255, 255, 255), 0.0, scale, scale);
        layer.sprite(&asteroid, 20, 150, 150, Color(255, 255, 255, 255), 0.0, 1.0, 1.0);
        layer.sprite(&asteroid, 50, 300, 300, Color(127, 0, 127, 127), 0.0, scale, 1.0);
        layer.sprite(&asteroid, frame, 320, 320, Color(127, 0, 127, 127), 0.0, 1.0, 1.0);
        layer.sprite(&mine, 50, 220, 420, Color(127, 0, 127, 127), rot, 1.0, 1.0);
        layer.sprite(&test, 50, 600, 600, Color::white(), rot, 1.0, 1.0);
        layer.sprite(&test2, 50, 650, 650, Color::white(), rot, 1.0, 1.0);
        layer.sprite(&test3, 50, 700, 700, Color::white(), rot, 1.0, 1.0);
        //testlayer.sprite(&asteroid, 50, 420, 320, Color::white(), 0.0, scale, 1.0);
        //testlayer.sprite(&asteroid, 50, 420, 320, Color::white(), 0.0, 1.0, scale);
        testlayer.sprite(&asteroid, 50, 420, 320, Color::white(), rot, 1.0, 1.0);
        let big = if input.button.0 { 3.0 } else { 1.0 };
        testlayer2.sprite(&powerup, 50, input.mouse.x, input.mouse.y, Color::white(), 0.0, scale * big, 1.0);
        testlayer2.sprite(&powerup, 50, input.mouse.x, input.mouse.y, Color::white(), 0.0, 1.0, scale * big);
        testlayer2.sprite(&powerup, 50, input.mouse.x, input.mouse.y, Color::white(), rot, 1.0, 1.0);

        testlayer.matrix
            .translate(&Vec3(320.0, 220.0, 0.0))
            .rotate_z(0.01f32)
            .translate(&Vec3(-320.0, -220.0, 0.0));

        rot += 0.01;
        scale += scaler;
        if scale > 3.0 || scale < 0.0 {
            scaler = -scaler;
        }
        frame += 1;

        // prepare render target, draw layers and swap

        renderer.prepare_and_clear_target(&Color::black());

        for i in 0..50 {
            testlayer
                .blend_alpha_const(255-(i*5))
                .matrix
                .translate(&Vec3(320.0, 220.0, 0.0))
                .rotate_z(-0.04f32)
                .translate(&Vec3(-320.0, -220.0, 0.0));
            testlayer
                .draw();
        }

        testlayer
            .reset()
            .matrix
            .translate(&Vec3(320.0, 220.0, 0.0))
            .rotate_z(2f32)
            .translate(&Vec3(-320.0, -220.0, 0.0));

        layer.draw().reset();
        //testlayer.draw().reset();
        testlayer2.draw().reset();

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

    let frame_interval = Duration::new(0, 166667); //16666667

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
