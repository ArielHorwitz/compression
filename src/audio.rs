use std::{error::Error, fs::File, path::Path};
use thiserror::Error;
use wav::{BitDepth, Header};

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("multiple channels not supported - convert to mono")]
    UnsupportedChannels,
}

pub fn load_wav_file(path: &str) -> Result<(wav::Header, Vec<f32>), Box<dyn Error>> {
    let mut inp_file = File::open(Path::new(path))?;
    let (header, data) = wav::read(&mut inp_file)?;
    if header.channel_count != 1 {
        return Err(Box::new(FormatError::UnsupportedChannels));
    }
    let data: Vec<f32> = match data {
        BitDepth::Eight(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::Sixteen(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::TwentyFour(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::ThirtyTwoFloat(d) => d.iter().map(|x| x.clone() as f32).collect(),
        BitDepth::Empty => Vec::from([0.]),
    };
    Ok((header, data))
}

pub fn write_wav_file(
    path: &str,
    waveform: Vec<i16>,
    sample_rate: u32,
) -> Result<(), std::io::Error> {
    // TODO take bitdepth argument and match for `BitDepth` type
    let mut out_file = File::create(Path::new(path))?;
    let header = Header::new(1, 1, sample_rate, 16);
    let track = BitDepth::Sixteen(waveform);
    wav::write(header, &track, &mut out_file)?;
    Ok(())
}
