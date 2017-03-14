#![feature(test)]
extern crate test;
use test::Bencher;

extern crate radiant_rs;
use radiant_rs::*;

#[bench]
fn sprite_drawing(b: &mut Bencher) {

    let display = Display::builder().build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((640., 480.));

    display.clear_frame(Color::black());

    // make sure layer is full allocated
    for i in 0..50000 {
        sprite.draw(&layer, i, (320., 200.), Color::white());
    }

    b.iter(|| {
        layer.clear();
        for i in 0..50000 {
            sprite.draw(&layer, i, (320., 200.), Color::white());
        }
    });

    display.swap_frame();
}

#[bench]
fn sprite_transformed_drawing(b: &mut Bencher) {

    let display = Display::builder().build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sprites/ball_v2_32x32x18.jpg").unwrap();
    let layer = Layer::new((640., 480.));

    display.clear_frame(Color::black());

    // make sure layer is full allocated
    for i in 0..50000 {
        sprite.draw(&layer, i, (320., 200.), Color::white());
    }

    b.iter(|| {
        layer.clear();
        for i in 0..50000 {
            sprite.draw_transformed(&layer, i, (320., 200.), Color::white(), 1.23, (2.34, 3.45));
        }
    });

    display.swap_frame();
}
