extern crate radiant_rs;
extern crate tiled;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::Path;
use std::fs::File;
use radiant_rs::*;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Sprites example").build();
    let renderer = Renderer::new(&display).unwrap();

    // Load tile-sheet as sprite, each frame will be a tile.
    let tileset = Sprite::from_file(&renderer.context(), r"res\iso_64x128.png").unwrap();

    // Create a HashMap that maps each tile-name to a frame_id. The sheet and the textfile were generated from a folder of images using tools/spritesheet.rs
    let name_to_frame_id = include_str!(r"../res/iso_64x128.txt").trim().lines().enumerate().map(|(id, line)| (line, id as u32)).collect::<HashMap<_, _>>();

    // Use rs-tiled to load a tilemap.
    let map = tiled::parse(File::open("res/iso.tmx").unwrap()).unwrap();

    // Create another HashMap that maps each of tiled's local tile ids to their image file name.
    let tile_to_name = map.tilesets[0].tiles.iter().map(|tile| (tile.id, Path::new(&tile.images[0].source).file_name().unwrap().to_str().unwrap()) ).collect::<HashMap<_, _>>();
    let first_gid = map.tilesets[0].first_gid;

    // Set up an isometric transformation matrix.
    let mut iso_transform = Mat4::identity();
    iso_transform.translate((320., 50., 0.));
    iso_transform.scale((64. / 2f32.sqrt(), 36. / 2f32.sqrt()));
    iso_transform.rotate(PI / 4.);

    // Draw each tile-layer onto a single (radiant) layer.
    let mut layers = Vec::new();

    for tile_layer in map.layers {
        layers.push(Layer::new((640., 480.)));
        for x in 0..10 {
            for y in 0..10 {
                let tile_id = tile_layer.tiles[y][x];
                if tile_id >= first_gid {
                    if let Some(ref name) = tile_to_name.get(&(tile_id - first_gid)).as_ref()  {
                        let pos = iso_transform * Vec2(x as f32, y as f32);
                        tileset.draw(&layers.last().unwrap(), name_to_frame_id[***name], (pos.0.round(), pos.1.round()), Color::white());
                    }
                }
            }
        }
    }

    utils::renderloop(|frame| {
        display.clear_frame(Color::black());

        // fade layers individually in
        for i in 0..layers.len() {
            let presentation = frame.elapsed_f32.floor() as usize % (layers.len() + 4);
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
