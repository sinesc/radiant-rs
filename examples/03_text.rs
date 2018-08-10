extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Display, Renderer, Layer, Font, Color};

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Text example").build().unwrap();
    let renderer = Renderer::new(&display).unwrap();
    let layer = Layer::new((640., 480.));

    // A few fonts (here from a known systemfont, could also come from a file)
    let small = Font::builder(display.context()).family("Arial").size(16.0).build().unwrap();
    let large = small.clone_with_size(48.0);
    let tiny_it = Font::builder(display.context()).family("Arial").italic().size(12.0).build().unwrap();

    ru::renderloop(|frame| {
        layer.clear();

        // Colorize entire layer. This is applied multiplicatively to the contents of the layer on draw.
        layer.set_color(Color::from_hsl((frame.elapsed_f32/5.0).fract(), 0.5, 0.5, 1.0));

        // Write some text
        large.write(&layer, "Nine squared", (210., 100.), Color::WHITE);
        small.write(&layer, include_str!("res/limerick.txt"), (210., 160.), Color::WHITE);
        tiny_it.write(&layer, "https://en.wikipedia.org/wiki/Leigh_Mercer", (10., 460.), Color::WHITE);

        // The usual clear, draw, swap, repeat.
        display.clear_frame(Color::BLACK);
        renderer.draw_layer(&layer, 0);
        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
