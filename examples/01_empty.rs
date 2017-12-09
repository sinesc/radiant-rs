extern crate radiant_rs;
use radiant_rs::{Display, Color, utils};

pub fn main() {

    // Create a display to render to
    let display = Display::builder().dimensions((640, 480)).vsync().title("Empty window example").build();

    // A simple mainloop helper (just an optional utility function)
    utils::renderloop(|_| {

        // Clear current backbuffer frame (black)
        display.clear_frame(Color::BLACK);

        // Here would be a good place to draw stuff

        // Swap back- and frontbuffer, making the changes visible
        display.swap_frame();

        // Poll for new events on the display, exit loop if the window was closed
        !display.poll_events().was_closed()
    });
}
