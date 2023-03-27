use crate::fft::{fft_2d, fft_2d_horizontal, fft_2d_inverse, fft_2d_vertical};
use bmp;
use num_complex::Complex32;
use plotly::{
    self,
    color::Rgb,
    common::Title,
    image::ColorModel,
    layout::{GridPattern, LayoutGrid},
    Image, Layout, Plot,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File};
use std::{
    io::{Read, Write},
    path::PathBuf,
};

pub fn analyze_image(
    filepath: &PathBuf,
    log_factor: f32,
    output_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn Error>> {
    let image = bitmap_to_image(filepath)?;
    let horizontal = ComplexImage::new(
        fft_2d_horizontal(&image.red),
        fft_2d_horizontal(&image.green),
        fft_2d_horizontal(&image.blue),
    );
    let vertical = ComplexImage::new(
        fft_2d_vertical(&image.red),
        fft_2d_vertical(&image.green),
        fft_2d_vertical(&image.blue),
    );
    let transformed = ComplexImage::new(
        fft_2d(&image.red),
        fft_2d(&image.green),
        fft_2d(&image.blue),
    );
    // Plot
    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .columns(4)
                .rows(1)
                .pattern(GridPattern::Independent),
        )
        .title(Title::new(&filepath.to_string_lossy()))
        .width(1900)
        .height(900);
    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(
        image_to_trace(&image, 1., false)
            .name("Uncompressed color domain")
            .x_axis("x1")
            .y_axis("y1"),
    );
    plot.add_trace(
        image_to_trace(&transformed, log_factor, true)
            .name("Uncompressed frequency domain")
            .x_axis("x2")
            .y_axis("y2"),
    );
    plot.add_trace(
        image_to_trace(&horizontal, log_factor, true)
            .name("Uncompressed horizontal frequency domain")
            .x_axis("x3")
            .y_axis("y3"),
    );
    plot.add_trace(
        image_to_trace(&vertical, log_factor, true)
            .name("Uncompressed vertical frequency domain")
            .x_axis("x4")
            .y_axis("y4"),
    );
    // Write to file
    let output_path = output_dir.join("analysis.html");
    plot.write_html(&output_path);
    Ok(output_path)
}

pub fn compress_bmp(
    bmp_file: &PathBuf,
    compressed_file: &PathBuf,
    compression_level: f32,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Compressing {:?} at level {:?}... ",
        bmp_file, compression_level
    );
    let original_image = bitmap_to_image(&bmp_file)?;
    println!("Transforming... ");
    let mut transformed_image = ComplexImage::new(
        fft_2d(&original_image.red),
        fft_2d(&original_image.green),
        fft_2d(&original_image.blue),
    );
    println!("Compressing... ");
    cut_image(&mut transformed_image, compression_level);
    let compressed = SerializableComplexImage::from_image(&transformed_image);
    let encoded = bincode::serialize(&compressed)?;
    let mut file = File::create(compressed_file)?;
    file.write_all(&encoded)?;
    println!("Compressed to: {:?}", compressed_file);
    Ok(())
}

pub fn decompress_bmp(
    compressed_file: &PathBuf,
    output_file: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    println!("Decompressing {:?}... ", compressed_file);
    let mut encoded: Vec<u8> = Vec::new();
    let mut file = File::open(compressed_file)?;
    file.read_to_end(&mut encoded)?;
    let decoded: SerializableComplexImage = bincode::deserialize(&encoded)?;
    let mut transformed_image = decoded.to_image();
    println!("Decompressing... ");
    restore_image(&mut transformed_image);
    println!("Transforming... ");
    let restored_image = ComplexImage::new(
        fft_2d_inverse(&transformed_image.red),
        fft_2d_inverse(&transformed_image.green),
        fft_2d_inverse(&transformed_image.blue),
    );
    image_to_bitmap(&restored_image, output_file)?;
    println!("Decompressed to: {:?}", output_file);
    Ok(())
}

struct ComplexImage {
    pub red: Vec<Vec<Complex32>>,
    pub green: Vec<Vec<Complex32>>,
    pub blue: Vec<Vec<Complex32>>,
    pub original_width: usize,
    pub original_height: usize,
}

impl ComplexImage {
    pub fn new(
        red: Vec<Vec<Complex32>>,
        green: Vec<Vec<Complex32>>,
        blue: Vec<Vec<Complex32>>,
    ) -> ComplexImage {
        let original_width = red[0].len();
        let original_height = red.len();
        ComplexImage {
            red,
            green,
            blue,
            original_width,
            original_height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializableComplexImage {
    red: Vec<Vec<(f32, f32)>>,
    green: Vec<Vec<(f32, f32)>>,
    blue: Vec<Vec<(f32, f32)>>,
    width: usize,
    height: usize,
}

impl SerializableComplexImage {
    pub fn from_image(image: &ComplexImage) -> SerializableComplexImage {
        SerializableComplexImage {
            red: image
                .red
                .iter()
                .map(|row| row.iter().map(|c| (c.re, c.im)).collect())
                .collect(),
            green: image
                .green
                .iter()
                .map(|row| row.iter().map(|c| (c.re, c.im)).collect())
                .collect(),
            blue: image
                .blue
                .iter()
                .map(|row| row.iter().map(|c| (c.re, c.im)).collect())
                .collect(),
            width: image.original_width,
            height: image.original_height,
        }
    }

    pub fn to_image(&self) -> ComplexImage {
        ComplexImage {
            red: self
                .red
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(re, im)| Complex32::new(re.clone(), im.clone()))
                        .collect()
                })
                .collect(),
            green: self
                .green
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(re, im)| Complex32::new(re.clone(), im.clone()))
                        .collect()
                })
                .collect(),
            blue: self
                .blue
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(re, im)| Complex32::new(re.clone(), im.clone()))
                        .collect()
                })
                .collect(),
            original_width: self.width,
            original_height: self.height,
        }
    }
}

fn bitmap_to_image(filepath: &PathBuf) -> Result<ComplexImage, Box<dyn Error>> {
    let bmp_data = bmp::open(filepath)?;
    let width = bmp_data.get_width() as usize;
    let height = bmp_data.get_height() as usize;
    let mut red = Vec::with_capacity(height);
    let mut green = Vec::with_capacity(height);
    let mut blue = Vec::with_capacity(height);
    for y in 0..height {
        let mut r_row = Vec::with_capacity(width);
        let mut g_row = Vec::with_capacity(width);
        let mut b_row = Vec::with_capacity(width);
        for x in 0..width {
            let pix = bmp_data.get_pixel(x as u32, y as u32);
            r_row.push(Complex32::from(pix.r as f32));
            g_row.push(Complex32::from(pix.g as f32));
            b_row.push(Complex32::from(pix.b as f32));
        }
        red.push(r_row);
        green.push(g_row);
        blue.push(b_row);
    }
    Ok(ComplexImage::new(red, green, blue))
}

fn image_to_bitmap(image: &ComplexImage, filepath: &PathBuf) -> Result<(), Box<dyn Error>> {
    let (width, height) = (image.red[0].len(), image.red.len());
    let mut bmp_image = bmp::Image::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            bmp_image.set_pixel(
                x as u32,
                y as u32,
                bmp::Pixel::new(
                    (image.red[y][x].norm()) as u8,
                    (image.green[y][x].norm()) as u8,
                    (image.blue[y][x].norm()) as u8,
                ),
            );
        }
    }
    bmp_image.save(filepath)?;
    Ok(())
}

fn shift_image(image: &mut ComplexImage) {
    shift_vectors(&mut image.red);
    shift_vectors(&mut image.green);
    shift_vectors(&mut image.blue);
}

fn shift_vectors<T>(vvec: &mut Vec<Vec<T>>) {
    let (width, height) = (vvec.len(), vvec[0].len());
    let (half_width, half_height) = (width / 2, height / 2);
    let mut x2;
    let mut y2;
    for x in 0..half_width {
        x2 = x + half_width;
        for y in 0..half_height {
            y2 = y + half_height;
            vvec[x].swap(y, y2);
            vvec[x2].swap(y, y2);
        }
        vvec.swap(x, x2);
    }
}

fn restore_image(image: &mut ComplexImage) {
    fn pad_vvec(vvec: &mut Vec<Vec<Complex32>>, width: usize, height: usize) {
        fn pad_horizontal(vvec: &mut Vec<Vec<Complex32>>, width_padding: usize) {
            for row in vvec {
                row.append(&mut vec![Complex32::default(); width_padding]);
                row.splice(0..0, vec![Complex32::default(); width_padding]);
            }
        }
        let width_padding = (width - vvec[0].len()) / 2;
        pad_horizontal(vvec, width_padding);
        let height_padding = (height - vvec.len()) / 2;
        vvec.append(&mut vec![vec![Complex32::default(); width]; height_padding]);
        vvec.splice(
            0..0,
            vec![vec![Complex32::default(); width]; height_padding],
        );
    }
    pad_vvec(&mut image.red, image.original_width, image.original_height);
    pad_vvec(
        &mut image.green,
        image.original_width,
        image.original_height,
    );
    pad_vvec(&mut image.blue, image.original_width, image.original_height);
    shift_image(image);
}

fn cut_image(image: &mut ComplexImage, compression_level: f32) {
    shift_image(image);
    let (width, height) = (image.red[0].len(), image.red.len());
    let compress_x = ((width as f32 - width as f32 / compression_level) / 2.) as usize;
    let compress_y = ((height as f32 - height as f32 / compression_level) / 2.) as usize;
    image.original_width = width;
    image.original_height = height;
    drain_vectors(&mut image.red, width, height, compress_x, compress_y);
    drain_vectors(&mut image.green, width, height, compress_x, compress_y);
    drain_vectors(&mut image.blue, width, height, compress_x, compress_y);
}

fn drain_vectors<T>(
    vvec: &mut Vec<Vec<T>>,
    width: usize,
    height: usize,
    compress_x: usize,
    compress_y: usize,
) {
    vvec.truncate(height - compress_y);
    vvec.drain(0..compress_y);
    for row in vvec {
        row.truncate(width - compress_x);
        row.drain(0..compress_x);
    }
}

fn image_to_trace(image: &ComplexImage, log_factor: f32, shift: bool) -> Box<Image> {
    // Assumes image is properly formed
    let (width, height) = (image.red.len(), image.red[0].len());
    let mut converted_image = Vec::with_capacity(width);
    let mut max_value = 0.;
    for y in 0..width {
        let mut column = Vec::with_capacity(height);
        for x in 0..height {
            let r = image.red[y][x].norm();
            let g = image.green[y][x].norm();
            let b = image.blue[y][x].norm();
            column.push((r, g, b));
            max_value = f32::max(f32::max(f32::max(max_value, r), g), b);
        }
        converted_image.push(column);
    }
    let mut normalized_image: Vec<Vec<Rgb>> = converted_image
        .iter()
        .map(|y| {
            y.iter()
                .map(|pixel| {
                    let (r, g, b) = pixel;
                    Rgb::new(
                        ((r / max_value).powf(log_factor) * 255.) as u8,
                        ((g / max_value).powf(log_factor) * 255.) as u8,
                        ((b / max_value).powf(log_factor) * 255.) as u8,
                    )
                })
                .collect()
        })
        .collect();
    if shift == true {
        shift_vectors(&mut normalized_image);
    }
    Image::new(normalized_image).color_model(ColorModel::RGB)
}
