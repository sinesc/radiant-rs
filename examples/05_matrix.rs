extern crate radiant_rs;
use radiant_rs::{DisplayInfo, Display, Renderer, Layer, Sprite, Color, utils, blendmodes};

pub fn main() {
    let display = Display::new(DisplayInfo { width: 640, height: 480, vsync: true, title: "Matrix example".to_string(), ..DisplayInfo::default() });
    let renderer = Renderer::new(&display).unwrap();
    let sprite = Sprite::from_file(&renderer.context(), r"res/sparkles_64x64x1.png").unwrap();
    let layer = Layer::new((320., 240.), 0);
    layer.set_blendmode(blendmodes::LIGHTEN);

    // Draw the usual sprites to the layer just once. We won't ever clear it, so we don't have to continuously redraw them.
    sprite.draw(&layer, 0, (160., 120.), Color::white());
    sprite.draw(&layer, 0, (130., 100.), Color::white());
    sprite.draw(&layer, 0, (190., 100.), Color::white());
    sprite.draw(&layer, 0, (160., 155.), Color::white());

    // Layers have a view and a model matrix. Make a backup of them here.
    // Transformations to the view matrix apply globally to the layer.
    // Transformations to the model matrix apply locally to each sprite on the layer.
    let original_view_matrix = layer.view_matrix().clone();
    let original_model_matrix = layer.model_matrix().clone();

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());
        let presentation_id = (frame.elapsed_f32 / 1.5) as u32 % 4;

        // Draw the layer in red, rotating only its view matrix.
        // The red sprites will all rotate around the center of the window.
        if presentation_id == 1 || presentation_id == 0 {
            layer.set_color(Color::red());
            layer.view_matrix().rotate_at((160., 120.), frame.elapsed_f32);
            renderer.draw_layer(&layer);
            layer.set_view_matrix(original_view_matrix);
        }

        // Draw the same layer in green again, rotating only its model matrix.
        // The green sprites will all rotate around their own, individual centers.
        if presentation_id == 2 || presentation_id == 0 {
            layer.set_color(Color::green());
            layer.model_matrix().rotate_at((0., 0.), frame.elapsed_f32);
            renderer.draw_layer(&layer);
            layer.set_model_matrix(original_model_matrix);
        }

        // Draw the same layer in blue yet again, rotating both matrices.
        if presentation_id == 3 || presentation_id == 0 {
            layer.set_color(Color::blue());
            layer.view_matrix().rotate_at((160., 120.), frame.elapsed_f32);
            layer.model_matrix().rotate_at((0., 0.), frame.elapsed_f32);
            renderer.draw_layer(&layer);
            layer.set_view_matrix(original_view_matrix);
            layer.set_model_matrix(original_model_matrix);
        }

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
