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
use std::{fmt::Debug, fs::File};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn compress_bmp(
    bmp_file: &Path,
    compressed_file: &Path,
    compression_level: f32,
) -> Result<(), BoxedError> {
    let original_image = ComplexImage::from_bitmap(bmp_file)?;
    let rounded_image = original_image.round_up();
    let transformed_image = ComplexImage::new(
        fft_2d(&rounded_image.red),
        fft_2d(&rounded_image.green),
        fft_2d(&rounded_image.blue),
    );
    let new_width = (transformed_image.width() as f32 / compression_level) as usize;
    let new_height = (transformed_image.height() as f32 / compression_level) as usize;
    let compressed_image = &transformed_image
        .corners(new_width, new_height)
        .map_err(|_| "compression must be no smaller than 1")?;
    let compressed_data = CompressedData::new(
        convert_complex_to_raw(&compressed_image.red),
        convert_complex_to_raw(&compressed_image.green),
        convert_complex_to_raw(&compressed_image.blue),
        transformed_image.size(),
        original_image.size(),
    );
    let encoded = bincode::serialize(&compressed_data)?;
    let mut file = File::create(compressed_file)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn decompress_bmp(compressed_file: &Path, output_file: &Path) -> Result<(), BoxedError> {
    let encoded: Vec<u8> = fs::read(compressed_file)?;
    let compressed_data: CompressedData = bincode::deserialize(&encoded)?;
    let compressed_image = ComplexImage::new(
        convert_raw_to_complex(&compressed_data.red),
        convert_raw_to_complex(&compressed_data.green),
        convert_raw_to_complex(&compressed_data.blue),
    );
    let transformed_image = compressed_image.fill_from_corners(compressed_data.transformed_size);
    let rounded_image = ComplexImage::new(
        fft_2d_inverse(&transformed_image.red),
        fft_2d_inverse(&transformed_image.green),
        fft_2d_inverse(&transformed_image.blue),
    );
    let restored_image = rounded_image.truncate(compressed_data.original_size);
    ComplexImage::save_bitmap(&restored_image, output_file)?;
    Ok(())
}

pub fn analyze_image(
    filepath: &Path,
    log_factor: f32,
    output_dir: &Path,
) -> Result<PathBuf, BoxedError> {
    println!("Analyzing {filepath:?}... ");
    let image = ComplexImage::from_bitmap(filepath)?.round_up();
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
        fft_2d_vertical(&horizontal.red),
        fft_2d_vertical(&horizontal.green),
        fft_2d_vertical(&horizontal.blue),
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

type BoxedError = Box<dyn std::error::Error>;
type Channel<T> = Vec<Vec<T>>;
type ComplexChannel = Channel<Complex32>;
type RawChannel = Channel<(f32, f32)>;

#[derive(Clone)]
struct ComplexImage {
    pub red: ComplexChannel,
    pub green: ComplexChannel,
    pub blue: ComplexChannel,
}

impl Debug for ComplexImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComplexImage {{ {}x{} }}", self.width(), self.height())
    }
}

impl ComplexImage {
    pub fn new(red: ComplexChannel, green: ComplexChannel, blue: ComplexChannel) -> ComplexImage {
        ComplexImage { red, green, blue }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    pub fn width(&self) -> usize {
        if self.red.is_empty() {
            return 0;
        }
        assert_eq!(self.red[0].len(), self.green[0].len());
        assert_eq!(self.red[0].len(), self.blue[0].len());
        self.red[0].len()
    }

    pub fn height(&self) -> usize {
        assert_eq!(self.red.len(), self.green.len());
        assert_eq!(self.red.len(), self.blue.len());
        self.red.len()
    }

    pub fn round_up(&self) -> Self {
        let new_width = 2f64.powf((self.width() as f64).log2().ceil()) as usize;
        let new_height = 2f64.powf((self.height() as f64).log2().ceil()) as usize;
        let extra_width = new_width - self.width();
        let extra_height = new_height - self.height();
        Self::from_iter(self.channels().iter().map(|channel| {
            let mut new_channel = channel.to_owned().to_owned();
            new_channel
                .iter_mut()
                .for_each(|row| row.extend(vec![Complex32::default(); extra_width]));
            new_channel.extend(vec![vec![Complex32::default(); new_width]; extra_height]);
            new_channel
        }))
    }

    pub fn truncate(&self, new_size: (usize, usize)) -> Self {
        Self::from_iter(self.channels().iter().map(|channel| {
            channel[..new_size.1]
                .iter()
                .map(|row| row[..new_size.0].to_vec())
                .collect()
        }))
    }

    pub fn from_bitmap(filepath: &Path) -> Result<ComplexImage, BoxedError> {
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

    pub fn save_bitmap(&self, filepath: &Path) -> Result<(), BoxedError> {
        let (width, height) = (self.red[0].len(), self.red.len());
        let mut bmp_image = bmp::Image::new(width as u32, height as u32);
        for y in 0..height {
            for x in 0..width {
                bmp_image.set_pixel(
                    x as u32,
                    y as u32,
                    bmp::Pixel::new(
                        (self.red[y][x].norm()) as u8,
                        (self.green[y][x].norm()) as u8,
                        (self.blue[y][x].norm()) as u8,
                    ),
                );
            }
        }
        bmp_image.save(filepath)?;
        Ok(())
    }

    /// Returns a new ComplexImage containing only the corners of this image.
    /// Returns an error if the new_width or new_height are larger than the current width and height.
    fn corners(&self, new_width: usize, new_height: usize) -> Result<Self, ()> {
        if new_width >= self.width() || new_height >= self.height() {
            return Err(());
        }
        let corner_width = new_width / 2;
        let corner_height = new_height / 2;
        let channels = self.channels();
        let new_channels = channels
            .iter()
            .map(|c| self.channel_corners(c, corner_width, corner_height));
        Ok(Self::from_iter(new_channels))
    }

    fn channel_corners(
        &self,
        channel: &ComplexChannel,
        corner_width: usize,
        corner_height: usize,
    ) -> ComplexChannel {
        let inverse_width = self.width() - corner_width;
        let inverse_height = self.height() - corner_height;
        let vert_slice =
            (0usize..corner_height).chain(inverse_height..self.height());
        let mut new_channel = ComplexChannel::new();
        for y in vert_slice {
            let mut row: Vec<Complex32> = Vec::with_capacity(corner_width * 2);
            row.extend_from_slice(&channel[y][..corner_width]);
            row.extend_from_slice(&channel[y][inverse_width..self.width()]);
            new_channel.push(row);
        }
        new_channel
    }

    fn fill_from_corners(&self, original_size: (usize, usize)) -> Self {
        ComplexImage::from_iter(
            self.channels()
                .iter()
                .map(|channel| self.fill_from_channel_corners(channel, original_size)),
        )
    }

    fn fill_from_channel_corners(
        &self,
        channel: &ComplexChannel,
        original_size: (usize, usize),
    ) -> ComplexChannel {
        let mid_width = self.size().0 / 2;
        let mid_height = self.size().1 / 2;
        let missing_width = original_size.0 - self.size().0;
        let missing_height = original_size.1 - self.size().1;
        let pad_width = vec![Complex32::default(); missing_width];
        let pad_height = vec![vec![Complex32::default(); original_size.0]; missing_height];
        let mut new_channel = channel.clone();
        new_channel
            .iter_mut()
            .map(|row| {
                row.splice(mid_width..mid_width, pad_width.clone());
            })
            .for_each(drop);
        new_channel.splice(mid_height..mid_height, pad_height);
        new_channel
    }

    pub fn channels(&self) -> [&ComplexChannel; 3] {
        [&self.red, &self.green, &self.blue]
    }
}

impl FromIterator<ComplexChannel> for ComplexImage {
    fn from_iter<T: IntoIterator<Item = ComplexChannel>>(iterable: T) -> Self {
        let mut iter = iterable.into_iter();
        Self::new(
            iter.next().expect("expected red channel"),
            iter.next().expect("expected green channel"),
            iter.next().expect("expected blue channel"),
        )
    }
}

#[derive(Serialize, Deserialize)]
struct CompressedData {
    red: RawChannel,
    green: RawChannel,
    blue: RawChannel,
    transformed_size: (usize, usize),
    original_size: (usize, usize),
}

impl CompressedData {
    pub fn new(
        red: RawChannel,
        green: RawChannel,
        blue: RawChannel,
        transformed_size: (usize, usize),
        original_size: (usize, usize),
    ) -> Self {
        CompressedData {
            red,
            green,
            blue,
            transformed_size,
            original_size,
        }
    }

    pub fn width(&self) -> usize {
        if self.red.is_empty() {
            return 0;
        }
        assert_eq!(self.red[0].len(), self.green[0].len());
        assert_eq!(self.red[0].len(), self.blue[0].len());
        self.red[0].len()
    }

    pub fn height(&self) -> usize {
        assert_eq!(self.red.len(), self.green.len());
        assert_eq!(self.red.len(), self.blue.len());
        self.red.len()
    }
}

impl Debug for CompressedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SerializableComplexImage {{ {}x{} -> {}x{} -> {}x{} }}",
            self.width(),
            self.height(),
            self.transformed_size.0,
            self.transformed_size.1,
            self.original_size.0,
            self.original_size.1,
        )
    }
}

fn convert_complex_to_raw(channel: &ComplexChannel) -> RawChannel {
    channel
        .iter()
        .map(|row| row.iter().map(|c| (c.re, c.im)).collect())
        .collect()
}

fn convert_raw_to_complex(channel: &RawChannel) -> ComplexChannel {
    channel
        .iter()
        .map(|row| {
            row.iter()
                .map(|(re, im)| Complex32::new(*re, *im))
                .collect()
        })
        .collect()
}

fn shift_vector<T>(channel: &mut Channel<T>) {
    let (width, height) = (channel.len(), channel[0].len());
    let (half_width, half_height) = (width / 2, height / 2);
    let mut x2;
    let mut y2;
    for x in 0..half_width {
        x2 = x + half_width;
        for y in 0..half_height {
            y2 = y + half_height;
            channel[x].swap(y, y2);
            channel[x2].swap(y, y2);
        }
        channel.swap(x, x2);
    }
}

fn image_to_trace(image: &ComplexImage, log_factor: f32, shift: bool) -> Box<Image> {
    // Assumes image is properly formed
    let (width, height) = (image.width(), image.height());
    let mut converted_image = Vec::with_capacity(height);
    let mut max_value = 0.;
    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let r = image.red[y][x].norm();
            let g = image.green[y][x].norm();
            let b = image.blue[y][x].norm();
            row.push((r, g, b));
            max_value = f32::max(f32::max(f32::max(max_value, r), g), b);
        }
        converted_image.push(row);
    }
    let mut normalized_image: Channel<Rgb> = converted_image
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
    if shift {
        shift_vector(&mut normalized_image);
    }
    Image::new(normalized_image).color_model(ColorModel::RGB)
}
