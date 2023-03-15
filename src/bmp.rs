use crate::fft::{
    fft_2d, fft_2d_horizontal, fft_2d_horizontal_inverse, fft_2d_inverse, fft_2d_vertical,
    fft_2d_vertical_inverse,
};
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

struct ComplexImage {
    pub red: Vec<Vec<Complex32>>,
    pub green: Vec<Vec<Complex32>>,
    pub blue: Vec<Vec<Complex32>>,
}

pub fn analyze_image(filepath: &str, log_factor: f32, output_dir: &str) -> String {
    let image = bitmap_to_image(filepath);
    let horizontal = ComplexImage {
        red: fft_2d_horizontal(&image.red),
        green: fft_2d_horizontal(&image.green),
        blue: fft_2d_horizontal(&image.blue),
    };
    let vertical = ComplexImage {
        red: fft_2d_vertical(&image.red),
        green: fft_2d_vertical(&image.green),
        blue: fft_2d_vertical(&image.blue),
    };
    let transformed = ComplexImage {
        red: fft_2d(&image.red),
        green: fft_2d(&image.green),
        blue: fft_2d(&image.blue),
    };
    let output_path = format!("{output_dir}analysis.html");
    plot(
        &image,
        &vertical,
        &horizontal,
        &transformed,
        log_factor,
        &output_path,
    );
    let inverse_horizontal = ComplexImage {
        red: fft_2d_horizontal_inverse(&transformed.red),
        green: fft_2d_horizontal_inverse(&transformed.green),
        blue: fft_2d_horizontal_inverse(&transformed.blue),
    };
    let inverse_vertical = ComplexImage {
        red: fft_2d_vertical_inverse(&transformed.red),
        green: fft_2d_vertical_inverse(&transformed.green),
        blue: fft_2d_vertical_inverse(&transformed.blue),
    };
    let inverse_transformed_image = ComplexImage {
        red: fft_2d_inverse(&transformed.red),
        green: fft_2d_inverse(&transformed.green),
        blue: fft_2d_inverse(&transformed.blue),
    };
    plot(
        &inverse_transformed_image,
        &inverse_vertical,
        &inverse_horizontal,
        &transformed,
        log_factor,
        &format!("{output_dir}analysis_inverse.html"),
    );
    image_to_bitmap(
        inverse_transformed_image,
        &format!("{output_dir}detransformed_image.bmp"),
    );
    output_path
}

fn bitmap_to_image(filepath: &str) -> ComplexImage {
    let bmp_data = bmp::open(filepath).unwrap();
    println!("bmp: {:?}", bmp_data);
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
    ComplexImage { red, green, blue }
}

fn image_to_bitmap(image: ComplexImage, filepath: &str) {
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
    bmp_image.save(filepath).unwrap();
}

fn image_to_trace(image: &ComplexImage, log_factor: f32) -> Box<Image> {
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
    let normalized_image: Vec<Vec<Rgb>> = converted_image
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
    Image::new(normalized_image).color_model(ColorModel::RGB)
}

fn plot(
    image: &ComplexImage,
    vertical: &ComplexImage,
    horizontal: &ComplexImage,
    transformed: &ComplexImage,
    log_factor: f32,
    file_path: &str,
) {
    // Plot
    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .rows(2)
                .columns(2)
                .pattern(GridPattern::Independent),
        )
        .title(Title::new(file_path))
        .width(1700)
        .height(1700);
    let mut plot = Plot::new();
    plot.set_layout(layout);
    plot.add_trace(
        image_to_trace(image, 1.)
            .name("Color domain")
            .x_axis("x1")
            .y_axis("y1"),
    );
    plot.add_trace(
        image_to_trace(transformed, log_factor)
            .name("Frequency domain")
            .x_axis("x2")
            .y_axis("y2"),
    );
    plot.add_trace(
        image_to_trace(horizontal, log_factor)
            .name("Horizontal frequency domain")
            .x_axis("x3")
            .y_axis("y3"),
    );
    plot.add_trace(
        image_to_trace(vertical, log_factor)
            .name("Vertical frequency domain")
            .x_axis("x4")
            .y_axis("y4"),
    );
    // Write to file
    plot.write_html(&file_path);
}
