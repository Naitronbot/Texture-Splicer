use std::{env, io::{Cursor}};
use image::{io::Reader as ImageReader, Rgba, ImageBuffer, RgbaImage};

fn main() {
    // Read arguments
    let args: Vec<String> = env::args().collect();

    if args.len() == 4 {
        manage_swap(args);
    } else {
        println!("Invalid amount of arguments provided, arguments take the form of:");
        println!("<Texture Image> <Palette Image> <Use Interpolation (true/false)>");
    }
}

fn manage_swap(args: Vec<String>) {
    // Open images and set flags
    let texture_image = ImageReader::open(&args[1]).expect("Error opening texture").decode().expect("Error decoding texture").to_rgba8();
    let palette_image = ImageReader::open(&args[2]).expect("Error opening palette").decode().expect("Error decoding palette").to_rgba8();
    let interpolate = match &args[3].to_lowercase()[..] {
        "true" => true,
        "false" => false,
        _ => false,
    };

    // Get lum sorted palettes from both inputs
    let palette = get_palette(&palette_image);
    let texture_palette = get_palette(&texture_image);

    // Ensure both palettes are not empty
    if palette.len() == 0 || texture_palette.len() == 0 {
        println!("One or more images are empty");
    }

    // Generate a new image by mapping the palette onto the texture
    let combined_image = merge_palette(texture_image, &texture_palette, &palette, interpolate);

    // Encode final image as png
    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    combined_image.write_to(&mut cursor, image::ImageOutputFormat::Png).expect("PNG format error");
    let output = base64::encode(buffer);
    println!("{}", output);
}

fn get_palette(img: &image::RgbaImage) -> Vec<Rgba<u8>> {
    // Create two vectors to store each pixel and it's corresponding luminance value
    let mut lum: Vec<f32> = Vec::new();
    let mut pixels: Vec<Rgba<u8>> = Vec::new();

    // Get each pixel of the image, and insert it into the array (ensuring the array stays sorted)
    for px in img.enumerate_pixels() {
        // Don't include pixels with zero alpha
        if px.2[3] == 0 {
            continue;
        }

        let pixel_lum = get_lum(&px.2);
        let insert_location = lum.partition_point(|&x| x < pixel_lum);

        // Pushes pixel to the end of vector if it's larger than every other value
        if insert_location == pixels.len() {
            lum.push(pixel_lum);
            pixels.push(*px.2);
        }

        // Ensure no duplicates
        if pixels[insert_location] != *px.2 {
            lum.insert(insert_location, pixel_lum);
            pixels.insert(insert_location, *px.2);
        }
    }

    return pixels;
}

// Takes in an Rgba pixel, and returns the relative luminance
fn get_lum(px: &Rgba<u8>) -> f32 {
    let gamma_r = f32::powf(px[0] as f32, 2.2);
    let gamma_g = f32::powf(px[1] as f32, 2.2);
    let gamma_b = f32::powf(px[2] as f32, 2.2);
    0.2126 * gamma_r + 0.7152 * gamma_g + 0.0722 * gamma_b
}

// Takes in a texture and its palette, and maps another palette onto it
fn merge_palette(img: image::RgbaImage, texture: &Vec<Rgba<u8>>, palette: &Vec<Rgba<u8>>, interpolate: bool) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut new_image: RgbaImage = ImageBuffer::new(width, height);
    
    // Ratio to convert indices from texture palette to new palette
    let conversion_constant = (palette.len() - 1) as f32 / (texture.len() - 1) as f32;

    // Place all pixels into new image
    if interpolate {
        for y in 0..height {
            for x in 0..width {
                let current_pixel = img.get_pixel(x, y);

                // Skip pixels with zero alpha
                if current_pixel[3] == 0 {
                    continue;
                }

                let pixel_index = f32::min(conversion_constant * find_index(texture, &current_pixel) as f32, (palette.len() - 1) as f32);
                let interpolation_factor = pixel_index % 1.0;
                if interpolation_factor == 0.0 {
                    let mut new_pixel = *palette.get(pixel_index as usize).unwrap();
                    new_pixel[3] = current_pixel[3];
                    new_image.put_pixel(x, y, new_pixel);
                } else {
                    let pixel_floor = pixel_index.floor() as usize;
                    let mut new_pixel = color_interpolate(palette.get(pixel_floor).unwrap(), palette.get(pixel_floor + 1).unwrap(), interpolation_factor);
                    new_pixel[3] = current_pixel[3];
                    new_image.put_pixel(x, y, new_pixel);
                }
            }
        }
    } else {
        for y in 0..height {
            for x in 0..width {
                let current_pixel = img.get_pixel(x, y);

                // Skip pixels with zero alpha
                if current_pixel[3] == 0 {
                    continue;
                }
                
                let pixel_index = (conversion_constant * find_index(texture, &current_pixel) as f32).round() as usize;
                let mut new_pixel = *palette.get(pixel_index).unwrap();
                new_pixel[3] = current_pixel[3];
                new_image.put_pixel(x, y, new_pixel);
            }
        }
    }

    return new_image;
}

// Finds the index of a specific color in a vector
fn find_index(vector: &Vec<Rgba<u8>>, value: &Rgba<u8>) -> usize {
    for i in 0..vector.len() {
        if value.eq(vector.get(i).unwrap()) {
            return i;
        }
    }
    return usize::MAX;
}

// Linearly interpolates between two colors with gamma correction 
fn color_interpolate(px1: &Rgba<u8>, px2: &Rgba<u8>, factor: f32) -> Rgba<u8> {
    let gamma_r1 = f32::powf(px1[0] as f32, 2.2);
    let gamma_g1 = f32::powf(px1[1] as f32, 2.2);
    let gamma_b1 = f32::powf(px1[2] as f32, 2.2);
    let gamma_r2 = f32::powf(px2[0] as f32, 2.2);
    let gamma_g2 = f32::powf(px2[1] as f32, 2.2);
    let gamma_b2 = f32::powf(px2[2] as f32, 2.2);

    let correction = 1.0/2.2;
    let final_r = f32::powf(gamma_r1 + factor * (gamma_r2 - gamma_r1), correction) as u8;
    let final_g = f32::powf(gamma_g1 + factor * (gamma_g2 - gamma_g1), correction) as u8;
    let final_b = f32::powf(gamma_b1 + factor * (gamma_b2 - gamma_b1), correction) as u8;

    return Rgba([final_r, final_g, final_b, 1]);
}