extern crate radiant_rs;
use radiant_rs::{Display, Renderer, Layer, Font, Color, utils};

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Text example").build();
    let renderer = Renderer::new(&display).unwrap();
    let layer = Layer::new((640., 480.));

    // A few fonts (here from a known systemfont, could also come from a file)
    let small = Font::builder(&renderer.context()).family("Arial").size(16.0).build().unwrap();
    let large = small.with_size(48.0);
    let tiny_it = Font::builder(&renderer.context()).family("Arial").italic().size(12.0).build().unwrap();

    utils::renderloop(|frame| {
        layer.clear();

        // Colorize entire layer. This is applied multiplicatively to the contents of the layer on draw.
        layer.set_color(Color::from_hsl((frame.elapsed_f32/5.0).fract(), 0.5, 0.5, 1.0));

        // Write some text
        large.write(&layer, "Nine squared", (210., 100.));
        small.write(&layer, include_str!("../res/03_text.txt"), (210., 160.));
        tiny_it.write(&layer, "https://en.wikipedia.org/wiki/Leigh_Mercer", (10., 460.));

        // The usual clear, draw, swap, repeat.
        display.clear_frame(Color::black());
        renderer.draw_layer(&layer, 0);
        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
