extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Font, FontInfo, Color, Point2, blendmodes, utils};

pub fn main() {

    // create a display to render to, a renderer to do the rendering
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: false, ..DisplayInfo::default() });
    let renderer = Renderer::new(&display);

    // create a sprite, a font (here from a known systemfont) and a layer
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/ball_v2_32x32x18.jpg");
    let font = Font::from_info(&renderer.context(), FontInfo { family: "Arial".to_string(), size: 16.0, ..FontInfo::default() } );
    let layer = Layer::new(640, 480, 0);

    // set how to blend the layer with the background
    layer.set_blendmode(blendmodes::LIGHTEN);

    // a simple mainloop helper (just an optional utility function)
    utils::renderloop(|state| {

        // clear the layer (layers can be drawn multiple times, e.g. a static UI might not need to be updated each frame)
        layer.clear();

        // colorize the whole layer
        let rainbow = Color::from_hsl((state.elapsed_f32/5.0).fract(), 1.0, 0.5, 1.0);
        layer.set_color(rainbow);

        // rotate the layer as a whole (by contrast, layer.model_matrix() would rotate the individual sprites)
        layer.view_matrix().rotate_at((320.0, 200.0), -state.delta_f32);

        // write some text
        font.write(&layer, &format!("It works. {} FPS", state.fps), Point2(260.0, 140.0));

        // draw a sprite (going though the spritesheet frames at 30 fps)
        let frame_id = (state.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, Point2(320.0, 200.0), Color::white());

        // draw the layer
        display.clear_frame(Color::black());
        renderer.draw_layer(&layer);
        display.swap_frame();

        // poll for new events on the display, exit loop if the window was closed
        !display.poll_events().was_closed()
    });
}
