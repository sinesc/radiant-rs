extern crate radiant_rs;
use radiant_rs::*;

#[test]
fn build_default_window() {
    let display = Display::builder().build();
    display.prepare_frame();
    display.swap_frame();
}
