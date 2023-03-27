use crate::fft;
use num_complex::Complex32;
use plotly::{
    color::NamedColor,
    common::{Line, Mode, Title},
    layout::{Axis, GridPattern, LayoutGrid, RowOrder},
    Layout, Plot, Scatter,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{error::Error, path::PathBuf};
use thiserror::Error;
use wav::{BitDepth, Header};

/// Returned when file formats are not supported.
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("multiple channels not supported")]
    UnsupportedChannels,
    #[error("unrecognized format not supported")]
    UnsupportedFormat,
}

/// Compress a .wav file for later decompression using [`decompress_wav`].
///
/// The frequency cutoff is the highest frequency to maintain: lower = smaller compressed size,
/// higher = better quality.
pub fn compress_wav(
    wav_file: &PathBuf,
    output_file: &PathBuf,
    freq_cutoff: usize,
) -> Result<(), Box<dyn Error>> {
    let (metadata, mut waveform) = load_wav_file(&wav_file)?;
    let original_size = waveform.len();
    fft::round_sample_size_up(&mut waveform);
    let time_domain = fft::convert_sample(&waveform);
    let mut freq_domain = fft::fft(&time_domain);
    let freq_resolution = metadata.freq_resolution(waveform.len());
    let highest_bin = f32::ceil(freq_cutoff as f32 / freq_resolution) as usize;
    let highest_bin = highest_bin.min(freq_domain.len()).max(0);
    let cutoff_zeros = freq_domain.len() - highest_bin;
    freq_domain.drain(highest_bin..);
    let frequencies: Vec<(f32, f32)> = freq_domain.iter().map(|c| (c.re, c.im)).collect();
    let compressed = CompressedData::new(
        metadata.sample_rate,
        original_size,
        metadata.bit_rate,
        frequencies,
        cutoff_zeros,
    );
    let encoded = bincode::serialize(&compressed)?;
    let mut file = File::create(output_file)?;
    file.write_all(&encoded)?;
    Ok(())
}

/// Decompress a .wav file from [`compress_wav`].
pub fn decompress_wav(
    compressed_file: &PathBuf,
    output_file: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut encoded: Vec<u8> = Vec::new();
    let mut file = File::open(compressed_file)?;
    file.read_to_end(&mut encoded)?;
    let decoded: CompressedData = bincode::deserialize(&encoded)?;
    let mut freq_domain: Vec<Complex32> = decoded
        .frequencies
        .iter()
        .map(|(r, i)| Complex32::new(r.clone(), i.clone()))
        .collect();
    freq_domain.append(&mut vec![Complex32::default(); decoded.cutoff_zeros]);
    let time_domain = fft::fft_inverse(&freq_domain);
    let mut waveform: Vec<f32> = time_domain.iter().map(|c| c.re as f32).collect();
    waveform.drain(decoded.original_size..);
    let metadata = WaveformMetadata::new(decoded.sample_rate, decoded.bit_rate);
    write_wav_file(output_file, waveform, &metadata)?;
    Ok(())
}

/// Produce an html page with interactive plots of the time domain and frequency domain.
pub fn analyze_waveform(
    wav_file: &PathBuf,
    output_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn Error>> {
    let file_path = output_dir.join("analysis.html");
    let (metadata, mut waveform) = load_wav_file(&wav_file)?;
    fft::round_sample_size_up(&mut waveform);
    let time_domain = fft::convert_sample(&waveform);
    let freq_bins = fft::frequency_bins(&fft::fft(&time_domain));
    println!("Writing analysis to: {:?}", file_path);
    plot(
        waveform.clone(),
        freq_bins,
        &metadata,
        &file_path,
        &wav_file.as_path().to_string_lossy().to_string(),
    );
    Ok(file_path)
}

#[derive(Serialize, Deserialize, Debug)]
struct WaveformMetadata {
    pub sample_rate: usize,
    pub bit_rate: usize,
}

impl WaveformMetadata {
    pub fn new(sample_rate: usize, bit_rate: usize) -> WaveformMetadata {
        WaveformMetadata {
            sample_rate,
            bit_rate,
        }
    }

    pub fn freq_resolution(&self, sample_size: usize) -> f32 {
        self.sample_rate as f32 / sample_size as f32
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CompressedData {
    sample_rate: usize,
    original_size: usize,
    bit_rate: usize,
    frequencies: Vec<(f32, f32)>,
    cutoff_zeros: usize,
}

impl CompressedData {
    fn new(
        sample_rate: usize,
        original_size: usize,
        bit_rate: usize,
        frequencies: Vec<(f32, f32)>,
        cutoff_zeros: usize,
    ) -> CompressedData {
        CompressedData {
            sample_rate,
            original_size,
            bit_rate,
            frequencies,
            cutoff_zeros,
        }
    }
}

fn load_wav_file(path: &PathBuf) -> Result<(WaveformMetadata, Vec<f32>), Box<dyn Error>> {
    let mut inp_file = File::open(Path::new(path))?;
    let (header, data) = wav::read(&mut inp_file)?;
    if header.channel_count != 1 {
        return Err(Box::new(FormatError::UnsupportedChannels));
    }
    let waveform: Vec<f32> = match data {
        BitDepth::Eight(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::Sixteen(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::TwentyFour(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::ThirtyTwoFloat(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::Empty => return Err(Box::new(FormatError::UnsupportedFormat)),
    };
    let metadata = WaveformMetadata::new(
        header.sampling_rate as usize,
        header.bits_per_sample as usize,
    );
    Ok((metadata, waveform))
}

fn write_wav_file(
    path: &PathBuf,
    waveform: Vec<f32>,
    metadata: &WaveformMetadata,
) -> Result<(), Box<dyn Error>> {
    let mut out_file = File::create(Path::new(path))?;
    let header = Header::new(1, 1, metadata.sample_rate as u32, metadata.bit_rate as u16);
    let track = match metadata.bit_rate {
        8 => BitDepth::Eight(waveform.iter().map(|x| x.clone() as u8).collect()),
        16 => BitDepth::Sixteen(waveform.iter().map(|x| x.clone() as i16).collect()),
        24 => BitDepth::TwentyFour(waveform.iter().map(|x| x.clone() as i32).collect()),
        32 => BitDepth::ThirtyTwoFloat(waveform),
        _ => return Err(Box::new(FormatError::UnsupportedFormat)),
    };
    wav::write(header, &track, &mut out_file)?;
    Ok(())
}

fn plot(
    waveform: Vec<f32>,
    freq_bins: Vec<f32>,
    metadata: &WaveformMetadata,
    file_path: &PathBuf,
    title: &str,
) {
    let sample_size = waveform.len();
    let waveform_legend = (0..sample_size)
        .map(|x| x as f32 / metadata.sample_rate as f32)
        .collect();
    let waveform_trace = Scatter::new(waveform_legend, waveform)
        .mode(Mode::Lines)
        .name("")
        .line(Line::new().color(NamedColor::Blue))
        .x_axis("x1")
        .y_axis("y1");
    let freq_legend = (0..freq_bins.len())
        .map(|x| x as f32 * metadata.freq_resolution(sample_size))
        .collect();
    let freq_bins_trace = Scatter::new(freq_legend, freq_bins)
        .mode(Mode::Lines)
        .name("")
        .line(Line::new().color(NamedColor::IndianRed))
        .x_axis("x2")
        .y_axis("y2");
    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .rows(2)
                .columns(1)
                .pattern(GridPattern::Independent)
                .row_order(RowOrder::TopToBottom),
        )
        .title(Title::new(title))
        .x_axis(Axis::new().title(Title::new("Time (seconds)")))
        .y_axis(Axis::new().title(Title::new("Amplitude")))
        .x_axis2(Axis::new().title(Title::new("Frequency (Hz)")))
        .y_axis2(Axis::new().title(Title::new("Amplitude")))
        .show_legend(false)
        .width(1900)
        .height(800);
    let mut plot = Plot::new();
    plot.add_trace(waveform_trace);
    plot.add_trace(freq_bins_trace);
    plot.set_layout(layout);
    plot.write_html(file_path);
}
