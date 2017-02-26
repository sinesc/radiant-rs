extern crate image;
use std::{env, fs, cmp, path, collections};
use image::GenericImage;

static ASPECT_RATIO: f32 = 1.0;

fn main() {
    let mut extension_map = collections::HashMap::new();
    extension_map.insert("jpg", image::ImageFormat::JPEG);
    extension_map.insert("jpeg", image::ImageFormat::JPEG);
    extension_map.insert("png", image::ImageFormat::PNG);

    // get sourcec and target directory
    let source = env::args().nth(1).expect("Expected source directory as first argument");
    let target = env::args().nth(2).expect("Expected target file as second argument");

    // load images
    let files = find_images(&source, &extension_map.keys().map(|&s| s).collect::<Vec<_>>());
    let ((max_width, max_height), images) = load_images(files);

    // find nice layout
    let (cols, rows) = best_ratio(ASPECT_RATIO, images.len() as u32, (max_width, max_height));

    // create output image
    let mut dest = image::DynamicImage::new_rgba8(cols * max_width, rows * max_height);
    let mut image_id = 0;

    for row in 0..rows {
        for col in 0..cols {
            if image_id < images.len() {
                dest.copy_from(&images[image_id], col * max_width, row * max_height);
                image_id += 1;
            }
        }
    }

    // construct file name
    let target = path::Path::new(&target);
    let basename = target.file_stem().expect("Invalid output file name");
    let extension = target.extension().expect("Invalid output file extension");
    let fullname = format!("{}_{}x{}.{}", basename.to_str().unwrap(), max_width, max_height, extension.to_str().unwrap());

    // write file
    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(fullname)
                        .expect("Target exists or cannot be written");

    let format = extension_map.get(extension.to_str().unwrap()).expect("Output file type not supported");
    dest.save(&mut file, *format).expect("Failed to encode image");
}


// load images into vector
fn load_images(files: Vec<path::PathBuf>) -> ((u32, u32), Vec<image::DynamicImage>) {
    let mut max_width = 0;
    let mut max_height = 0;
    let mut images = Vec::new();
    for file in files.iter() {
        let image = image::open(&file).expect("Could not open image file.");// !todo name file here
        let image_dim = image.dimensions();
        max_width = cmp::max(image_dim.0, max_width);
        max_height = cmp::max(image_dim.1, max_height);
        images.push(image);
    }
    ((max_width, max_height), images)
}

// read image file names from source directory
fn find_images(source: &str, extensions: &[ &str ]) -> Vec<path::PathBuf> {
    let mut files = Vec::new();
    let entries = fs::read_dir(source).expect("Cannot find source path");
    for entry in entries {
        let entry = entry.unwrap();
        let extension = entry.path().extension().map_or("", |p| p.to_str().unwrap()).to_string(); // !todo better solution or intended user experience ?
        if extensions.iter().find(|ext| **ext == extension).is_some() && entry.path().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files
}

// finds number of rows/columns required to make output as close to desirect aspect as possible
fn best_ratio(desired_aspect: f32, num_files: u32, max_dim: (u32, u32)) -> (u32, u32) {
    let mut best_ratio_diff = (1, 999999.0); // !todo inf
    for rows in 1..num_files {
        let cols = (num_files as f32 / rows as f32).ceil() as u32;
        let ratio = (cols * max_dim.0) as f32 / (rows * max_dim.1) as f32;
        let diff = (ratio - desired_aspect).abs();
        if diff < best_ratio_diff.1 {
            best_ratio_diff = (rows, diff);
        }
    }
    ((num_files as f32 / best_ratio_diff.0 as f32).ceil() as u32, best_ratio_diff.0)
}
