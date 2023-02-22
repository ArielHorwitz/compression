use num_complex::Complex32;
use std::{error::Error, fs::File, ops::RangeInclusive, path::Path};
use thiserror::Error;
use wav::{BitDepth, Header};

use crate::common::WaveformMetadata;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("multiple channels not supported - convert to mono")]
    UnsupportedChannels,
}

pub fn load_wav_file(path: &str) -> Result<(WaveformMetadata, Vec<f32>), Box<dyn Error>> {
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
        BitDepth::Empty => Vec::from([0.]),
    };
    let modified_name = path.strip_suffix(".wav").unwrap_or("unknown");
    let (_, modified_name) = modified_name
        .rsplit_once("/")
        .unwrap_or(("", modified_name));
    let metadata = WaveformMetadata::new(
        modified_name,
        waveform.len(),
        header.sampling_rate as usize,
        header.bits_per_sample as usize,
    );
    Ok((metadata, waveform))
}

pub fn write_wav_file(
    path: &str,
    waveform: Vec<i16>,
    metadata: &WaveformMetadata,
) -> Result<(), std::io::Error> {
    let mut out_file = File::create(Path::new(path))?;
    let header = Header::new(1, 1, metadata.sample_rate as u32, metadata.bit_rate as u16);
    let track = BitDepth::Sixteen(waveform);
    wav::write(header, &track, &mut out_file)?;
    Ok(())
}

pub fn flatten_freq_range(
    freq_domain: &mut Vec<Complex32>,
    metadata: &WaveformMetadata,
    range: RangeInclusive<f32>,
) -> Result<(), ()> {
    let start = (range.start() / metadata.freq_resolution) as usize;
    let end = (range.end() / metadata.freq_resolution) as usize;
    let range = start..=end;
    let replace_with = vec![Complex32::default(); end - start + 1];
    freq_domain.splice(range, replace_with);
    Ok(())
}
