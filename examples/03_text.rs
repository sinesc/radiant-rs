extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Font, FontInfo, Color, utils};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Text example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let layer = Layer::new((640., 480.), 0);

    // A few fonts (here from a known systemfont, could also come from a file)
    let small = Font::from_info(&renderer.context(), FontInfo { family: "Arial".to_string(), size: 16.0, ..FontInfo::default() } );
    let large = small.with_size(48.0);
    let tiny_it = Font::from_info(&renderer.context(), FontInfo { family: "Arial".to_string(), italic: true, size: 12.0, ..FontInfo::default() } );

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
        renderer.draw_layer(&layer);
        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
