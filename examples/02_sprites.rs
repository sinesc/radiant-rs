extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, utils, blendmodes};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Sprites example".to_string(), ..DisplayInfo::default() });

    // Create a renderer. It is used to draw to a rendertarget (usually a frame).
    let renderer = Renderer::new(&display).unwrap();

    // Create a sprite from a spritesheet file, extracting frame layout from filename.
    let sprite = Sprite::from_file(&renderer.context(), r"res/ball_v2_32x32x18.jpg").unwrap();

    // A layer where 320x240 units correspond to the full window (which measures 640x480 pixels, so that one unit = two pixel).
    let layer = Layer::new((320., 240.));

    // Layers have a blendmode setting that defines how their contents will be blended with the background on draw.
    layer.set_blendmode(blendmodes::LIGHTEN);

    utils::renderloop(|frame| {

        // Clear the layer (layers could also be drawn multiple times, e.g. a static UI might not need to be updated each frame)
        layer.clear();

        // Draw three sprites to the layer, multiplied by colors red, green and blue as well as the original sprite (multiplied by white, which is the identity)
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::white());
        sprite.draw(&layer, frame_id, (130., 100.), Color::red());
        sprite.draw(&layer, frame_id, (190., 100.), Color::green());
        sprite.draw(&layer, frame_id, (160., 155.), Color::blue());

        // draw the layer to the frame after clearing it with solid black.
        display.clear_frame(Color::black());
        renderer.draw_layer(&layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
