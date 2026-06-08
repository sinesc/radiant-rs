use radiant_utils as ru;
use std::collections::HashMap;
use std::f32::consts::PI;
use radiant_rs::*;
use ru::Matrix;

// Note: Recommended to be run with --release. This example loads a large tileset
// using the image library which appears to be a bit sedate in debug mode.

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Tiles example").build().unwrap();
    let renderer = Renderer::new(&display).unwrap();

    // Load tile-sheet as sprite, each frame will be a tile.
    let tileset = Sprite::from_file(display.context(), r"examples/res/tiles/iso_64x128.png").unwrap();

    // Create a HashMap that maps each tile-name to a frame_id. The sheet and the textfile were generated from a folder of images using tools/spritesheet.rs
    let name_to_frame_id = include_str!(r"res/tiles/iso_64x128.txt").trim().lines().enumerate().map(|(id, line)| (line, id as u32)).collect::<HashMap<_, _>>();

    // Use tiled to load a tilemap (free tiles from http://www.kenney.nl/)
    let map = tiled::Loader::new().load_tmx_map("examples/res/tiles/iso.tmx").unwrap();

    // Create a HashMap that maps each of tiled's local tile ids to their image file name.
    let map_tileset = &map.tilesets()[0];
    let tile_to_name: HashMap<tiled::TileId, String> = map_tileset.tiles()
        .filter_map(|(id, tile)| {
            tile.image.as_ref().map(|img| {
                (id, img.source.file_name().unwrap().to_str().unwrap().to_string())
            })
        })
        .collect();

    // Set up an isometric transformation matrix.
    let mut iso_transform = ru::Mat4::identity();
    iso_transform.translate((320., 50., 0.));
    iso_transform.scale((64. / 2f32.sqrt(), 36. / 2f32.sqrt()));
    iso_transform.rotate(PI / 4.);

    // Draw each tile-layer onto a single (radiant) layer.
    let mut layers = Vec::new();

    for layer in map.layers() {
        layers.push(Layer::new((640., 480.)));
        if let Some(tile_layer) = layer.as_tile_layer() {
            for x in 0..map.width as i32 {
                for y in 0..map.height as i32 {
                    if let Some(layer_tile) = tile_layer.get_tile(x, y) {
                        let id = layer_tile.id();
                        if let Some(name) = tile_to_name.get(&id) {
                            let pos = iso_transform * ru::Vec2(x as f32, y as f32);
                            tileset.draw(&layers.last().unwrap(), name_to_frame_id[name.as_str()], (pos.0.round(), pos.1.round()), Color::WHITE);
                        }
                    }
                }
            }
        }
    }

    ru::renderloop(|frame| {
        display.clear_frame(Color::BLACK);

        // fade layers individually in
        let presentation = frame.elapsed_f32.floor() as usize % (layers.len() + 4);

        for i in 0..layers.len() {
            if presentation >= i {
                if presentation == i {
                    layers[i].set_color(Color::alpha_pm( frame.elapsed_f32.fract() ));
                }
                renderer.draw_layer(&layers[i], 0);
            }
        }

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
