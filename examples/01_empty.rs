extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Color, utils};

pub fn main() {

    // Create a display to render to
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Empty window example".to_string(), ..DisplayInfo::default() });

    // A simple mainloop helper (just an optional utility function)
    utils::renderloop(|_| {

        // Clear current backbuffer frame (black)
        display.clear_frame(Color::black());

        // Here would be a good place to draw stuff

        // Swap back- and frontbuffer, making the changes visible
        display.swap_frame();

        // Poll for new events on the display, exit loop if the window was closed
        !display.poll_events().was_closed()
    });
}
