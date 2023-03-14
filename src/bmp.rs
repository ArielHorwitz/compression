use crate::fft::{fft_2d_horizontal, fft_2d_vertical};
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
    let full = ComplexImage {
        red: fft_2d_vertical(&horizontal.red),
        green: fft_2d_vertical(&horizontal.green),
        blue: fft_2d_vertical(&horizontal.blue),
    };
    let output_path = format!("{output_dir}analysis.html");
    plot(
        &image,
        &horizontal,
        &vertical,
        &full,
        log_factor,
        &output_path,
    );
    output_path
}

fn bitmap_to_image(filepath: &str) -> ComplexImage {
    let bmp_data = bmp::open(filepath).unwrap();
    println!("bmp: {:?}", bmp_data);
    let width = bmp_data.get_width() as usize;
    let height = bmp_data.get_height() as usize;
    let mut red = Vec::with_capacity(width);
    let mut green = Vec::with_capacity(width);
    let mut blue = Vec::with_capacity(width);
    for x in 0..width {
        let mut r_column = Vec::with_capacity(height);
        let mut g_column = Vec::with_capacity(height);
        let mut b_column = Vec::with_capacity(height);
        for y in 0..height {
            let pix = bmp_data.get_pixel(y as u32, x as u32);
            r_column.push(Complex32::from(pix.r as f32));
            g_column.push(Complex32::from(pix.g as f32));
            b_column.push(Complex32::from(pix.b as f32));
        }
        red.push(r_column);
        green.push(g_column);
        blue.push(b_column);
    }
    ComplexImage { red, green, blue }
}

fn image_to_trace(image: &ComplexImage, log_factor: f32) -> Box<Image> {
    // Assumes image is properly formed
    let (width, height) = (image.red.len(), image.red[0].len());
    let mut converted_image = Vec::with_capacity(width);
    let mut max_value = 0.;
    for x in 0..width {
        let mut column = Vec::with_capacity(height);
        for y in 0..height {
            let r = image.red[x][y].norm();
            let g = image.green[x][y].norm();
            let b = image.blue[x][y].norm();
            column.push((r, g, b));
            max_value = f32::max(f32::max(f32::max(max_value, r), g), b);
        }
        converted_image.push(column);
    }
    let normalized_image: Vec<Vec<Rgb>> = converted_image
        .iter()
        .map(|row| {
            row.iter()
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
