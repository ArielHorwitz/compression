use crate::audio;
use crate::common::WaveformMetadata;
use crate::fft;
use num_complex::Complex32;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
struct CompressedData {
    sample_rate: usize,
    original_size: usize,
    frequencies: Vec<(f32, f32)>,
    cutoff_zeros: usize,
}

impl CompressedData {
    fn new(
        sample_rate: usize,
        original_size: usize,
        frequencies: Vec<(f32, f32)>,
        cutoff_zeros: usize,
    ) -> CompressedData {
        CompressedData {
            sample_rate,
            original_size,
            frequencies,
            cutoff_zeros,
        }
    }
}

/// Compress a .wav file for later decompression using [`decompress`].
///
/// The frequency cutoff is the highest frequency to maintain: lower = smaller compressed size,
/// higher = better quality.
pub fn compress_wav(
    wav_file: &str,
    output_file: &str,
    freq_cutoff: usize,
) -> Result<(), Box<dyn Error>> {
    let (metadata, mut waveform) = audio::load_wav_file(&wav_file)?;
    fft::round_sample_size_up(&mut waveform);
    let time_domain = fft::convert_sample(&waveform);
    let mut freq_domain = fft::fft(&time_domain);
    let highest_bin = f32::ceil(freq_cutoff as f32 / metadata.freq_resolution) as usize;
    let highest_bin = highest_bin.min(freq_domain.len()).max(0);
    let cutoff_zeros = freq_domain.len() - highest_bin;
    freq_domain.drain(highest_bin..);
    let freq: Vec<(f32, f32)> = freq_domain.iter().map(|c| (c.re, c.im)).collect();
    let compressed = CompressedData::new(
        metadata.sample_rate,
        metadata.sample_size,
        freq,
        cutoff_zeros,
    );
    let encoded = bincode::serialize(&compressed)?;
    let mut file = File::create(output_file)?;
    file.write_all(&encoded)?;
    Ok(())
}

/// Decompress a .wav file from [`compress`].
pub fn decompress_wav(compressed_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
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
    let mut waveform: Vec<i16> = time_domain.iter().map(|c| c.re as i16).collect();
    waveform.drain(decoded.original_size..);
    let metadata = WaveformMetadata::new("", waveform.len(), decoded.sample_rate, 16);
    audio::write_wav_file(output_file, waveform, &metadata)?;
    Ok(())
}
