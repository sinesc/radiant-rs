extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, Texture, TextureFilter, utils, blendmodes};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Drawing to textures example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    sprite.draw(&layer, 0, (160., 120.), Color::white());
    sprite.draw(&layer, 0, (130., 100.), Color::red());
    sprite.draw(&layer, 0, (190., 100.), Color::green());
    sprite.draw(&layer, 0, (160., 155.), Color::blue());

    // Two textures. We'll draw the sprites to "surface". The "darken" texture only
    // contains black with a low opacity. We'll blend this with surface's contents.
    // Note: there are more optimal solutions to do this (using Program). This is just to make the example pretty.
    let surface = Texture::new(&renderer.context(), 640, 480);
    let darken = Texture::new(&renderer.context(), 1, 1);
    darken.clear(Color(0., 0., 0., 0.04));

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());

        // Rotate the sprite matrices
        layer.view_matrix().rotate_at((160., 120.), frame.delta_f32);
        layer.model_matrix().rotate(frame.delta_f32 * 1.1);

        // Drawing within Renderer::render_to() redirects the output to the given rendertarget.
        // First we draw the sprites, then we blend the low opacity black on top (to fade previously drawn contents)
        renderer.render_to(&surface, || {
            renderer.draw_layer(&layer, 0);
            renderer.rect((0., 0., 640., 480.)).blendmode(&blendmodes::ALPHA).texture(&darken).draw();
        });

        if (frame.elapsed_f32 / 1.5) as u32 % 2 == 0 {
            // Copies surface to the display.
            renderer.copy_from(&surface, TextureFilter::Linear);
        } else {
            // Draw the sprites to display.
            renderer.draw_layer(&layer, 0);
        }

        // Draw a small thumbnail of surface
        renderer.copy_rect_from(&surface, (0, 0, 640, 480), (512, 384, 128, 96), TextureFilter::Linear);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
