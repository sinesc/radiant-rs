extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, blendmodes};
use ru::Matrix;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Matrix example").build();
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.));
    layer.set_blendmode(blendmodes::LIGHTEN);

    // Draw the usual sprites to the layer just once. We won't ever clear it, so we don't have to continuously redraw them.
    sprite.draw(&layer, 0, (160., 120.), Color::WHITE);
    sprite.draw(&layer, 0, (130., 100.), Color::WHITE);
    sprite.draw(&layer, 0, (190., 100.), Color::WHITE);
    sprite.draw(&layer, 0, (160., 155.), Color::WHITE);

    ru::renderloop(|frame| {
        display.clear_frame(Color::BLACK);
        let presentation_id = (frame.elapsed_f32 / 1.5) as u32 % 4;

        // Draw the layer in red, rotating only its view matrix.
        // The red sprites will all rotate around the center of the window.
        // Transformations to the view matrix apply globally to the layer.
        if presentation_id == 1 || presentation_id == 0 {
            layer.set_color(Color::RED);
            layer.view_matrix().push().rotate_at((160., 120.), frame.elapsed_f32);
            renderer.draw_layer(&layer, 0);
            layer.view_matrix().pop();
        }

        // Draw the same layer in green again, rotating only its model matrix.
        // The green sprites will all rotate around their own, individual centers.
        // Transformations to the model matrix apply locally to each sprite on the layer.
        if presentation_id == 2 || presentation_id == 0 {
            layer.set_color(Color::GREEN);
            layer.model_matrix().push().rotate_at((0., 0.), frame.elapsed_f32);
            renderer.draw_layer(&layer, 0);
            layer.model_matrix().pop();
        }

        // Draw the same layer in blue yet again, rotating both matrices.
        if presentation_id == 3 || presentation_id == 0 {
            layer.set_color(Color::BLUE);
            layer.view_matrix().push().rotate_at((160., 120.), frame.elapsed_f32);
            layer.model_matrix().push().rotate_at((0., 0.), frame.elapsed_f32);
            renderer.draw_layer(&layer, 0);
            layer.view_matrix().pop();
            layer.model_matrix().pop();
        }

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
