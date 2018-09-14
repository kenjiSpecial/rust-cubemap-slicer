extern crate image;

use std::fs;

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;

use image::{GenericImage, imageops};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("{}", args.len());
        println!("[ERROR] Call with image name");
        std::process::exit(1);
    }

    let cube_image_name = &args[1];
    

    let target_dir = format!("assets/{}", cube_image_name);
    fs::create_dir_all(target_dir);

    let open_image_src = format!("assets/{}.png", cube_image_name);
    // println!("{}", open_image_src);

    let ref mut img = image::open(&Path::new(&open_image_src)).expect("Failed to load image");
    let img_size: u32 = img.width() / 4 as u32;
    let mut output_imgSize: u32 = 0;
    if args.len() == 3 {
        output_imgSize = args[2].parse::<u32>().unwrap();
    } else {
        output_imgSize = img_size;
    }


    println!("{}, {}, {}", img_size, img.width(), img.height());

    let image_names = vec!["negx", "posx", "negy", "posy", "negz", "posz"];

    for i in 0..image_names.len() {
        let target_dir = format!("assets/{}", cube_image_name);
        let image_src_name = format!("{}/{}.jpg", target_dir.clone(), image_names[i]);
        // println!("{}", image_src_name);

        let mut x:u32 = 0;
        let mut y:u32 = 0;

        match i {
            0 => {x = 0; y = img_size;},
            1 => {x = img_size * 2; y = img_size;},
            2 => {x = img_size; y = img_size * 2;},
            3 => {x = img_size; y = 0;},
            4 => {x = img_size; y = img_size;},
            _ => {x = img_size * 3; y = img_size;},
        }

        let croped_image = imageops::crop(img, x, y, img_size, img_size).to_image();
        let output_image = imageops::resize(&croped_image, output_imgSize, output_imgSize, image::FilterType::Nearest);

        output_image.save(image_src_name);
    }
}
