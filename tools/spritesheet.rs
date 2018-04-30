extern crate image;
use std::{env, fs, cmp, path, collections, io, process};
use std::io::Write;
use image::GenericImage;

const OUTPUT_ASPECT: f32 = 1.0;

const INVALID_ARGUMENT: i32 = 1;
const INVALID_PATH: i32 = 2;
const FILE_EXISTS: i32 = 3;
const IMAGE_ERROR: i32 = 4;

fn main() {
    let mut extension_map = collections::HashMap::new();
    extension_map.insert("jpg", image::ImageFormat::JPEG);
    extension_map.insert("jpeg", image::ImageFormat::JPEG);
    extension_map.insert("png", image::ImageFormat::PNG);

    // get sourcec and target directory
    let source = env::args().nth(1).unwrap_or_else(|| error("Expected source directory as first argument", INVALID_ARGUMENT));
    let target = env::args().nth(2).unwrap_or_else(|| error("Expected target file as second argument", INVALID_ARGUMENT));
    let resize = env::args().nth(3).unwrap_or("0".to_string()).parse::<f32>().unwrap_or_else(|_| error("Expected floating point value scale as third argument", INVALID_ARGUMENT));

    // load images
    let files = find_images(&source, &extension_map.keys().map(|&s| s).collect::<Vec<_>>());
    print!("Loading image files");
    let ((max_width, max_height), images) = load_images(&files, resize);
    println!(".");

    // find nice layout
    let (cols, rows) = best_ratio(OUTPUT_ASPECT, images.len() as u32, (max_width, max_height));

    // create output image
    println!("Generating spritesheet...");
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
    let basename = target.file_stem().unwrap_or_else(|| error("Invalid output file name", INVALID_ARGUMENT));
    let extension = target.extension().unwrap_or_else(|| error("Invalid output file extension", INVALID_ARGUMENT));

    // write file
    {
        let fullname = format!("{}_{}x{}.{}", basename.to_str().unwrap(), max_width, max_height, extension.to_str().unwrap());

        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&fullname)
                            .unwrap_or_else(|_| error(&format!("Target image {} exists or cannot be written", &fullname), FILE_EXISTS));

        println!("Writing file {}...", &fullname);
        let format = extension_map.get(extension.to_str().unwrap()).unwrap_or_else(|| error("Output file type not supported", INVALID_ARGUMENT));
        dest.write_to(&mut file, *format).unwrap_or_else(|_| error("Failed to encode image", IMAGE_ERROR));
    }

    // write list of files
    {
        let fullname = format!("{}_{}x{}.txt", basename.to_str().unwrap(), max_width, max_height);

        let file = fs::OpenOptions::new().write(true).create_new(true).open(&fullname)
                            .unwrap_or_else(|_| error(&format!("Target sprite list {} exists or cannot be written", &fullname), FILE_EXISTS));

        println!("Writing file {}...", &fullname);
        let mut buffer = io::BufWriter::new(file);

        for file in files.iter() {
            buffer.write_all(file.file_name().unwrap().to_str().unwrap().as_bytes()).unwrap();
            buffer.write(&['\n' as u8]).unwrap();
        }
    }
}


// load images into vector
fn load_images(files: &Vec<path::PathBuf>, resize: f32) -> ((u32, u32), Vec<image::RgbaImage>) {
    let mut max_width = 0;
    let mut max_height = 0;
    let mut images = Vec::new();
    for file in files.iter() {
        let image = image::open(&file).unwrap_or_else(|_| error(&format!("Could not open image file {}", &file.to_str().unwrap()), IMAGE_ERROR));
        let mut image_dim = image.dimensions();
        let image = if resize > 0.0 {
            image_dim.0 = (image_dim.0 as f32 * resize) as u32;
            image_dim.1 = (image_dim.1 as f32 * resize) as u32;
            image::imageops::resize(&image, image_dim.0, image_dim.1, image::FilterType::Lanczos3)
        } else {
            image.to_rgba()
        };
        max_width = cmp::max(image_dim.0, max_width);
        max_height = cmp::max(image_dim.1, max_height);
        images.push(image);
        print!(".");
        io::stdout().flush().unwrap();
    }
    ((max_width, max_height), images)
}

// read image file names from source directory
fn find_images(source: &str, extensions: &[ &str ]) -> Vec<path::PathBuf> {
    let mut files = Vec::new();
    let entry_set = fs::read_dir(source).unwrap_or_else(|_| error("Cannot find source path", INVALID_PATH));
    let mut entries = entry_set.collect::<Result<Vec<_>, _>>().unwrap_or_else(|_| error("Cannot read source path", INVALID_PATH));
    entries.sort_by(|a, b| a.path().cmp(&b.path()));
    for entry in entries {
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

// together with unwrap_or_else replaces expect() to get some cheap error reporting
fn error(message: &str, code: i32) -> ! {
    if code == INVALID_ARGUMENT {
        println!("spritesheet: A tool to generate spritesheets. An accompanying textfile contains the input filenames in frame_id order.");
    }
    println!("Error: {}.", message);
    if code == INVALID_ARGUMENT {
        println!("Usage: spritesheet <source directory> <output basename>.<output filetype extension> [ <scaling factor> ]");
        println!("  e.g. spritesheet /path/to/images/ sheet.png 0.25");
    }
    process::exit(code)
}
