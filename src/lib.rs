pub mod fft;
pub mod plotting;
use std::{error::Error, fs::File, path::Path};
use thiserror::Error;
use wav::{self, BitDepth, Header};

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

pub enum Round {
    Up,
    Down,
}

pub struct AnalysisConfig {
    pub name: String,
    pub rounding: Round,
    pub output_dir: String,
    pub print_progress: bool,
}

impl AnalysisConfig {
    pub fn new(
        name: &str,
        rounding: Round,
        output_dir: &str,
        print_progress: bool,
    ) -> AnalysisConfig {
        AnalysisConfig {
            name: name.to_owned(),
            rounding,
            output_dir: output_dir.to_owned(),
            print_progress,
        }
    }
}

pub fn analyze_waveform(waveform: Vec<f32>, sample_rate: usize, config: AnalysisConfig) {
    if config.print_progress {
        println!("Converting...");
    }
    let mut waveform = waveform;
    match config.rounding {
        Round::Up => fft::round_sample_size_up(&mut waveform),
        Round::Down => fft::round_sample_size_down(&mut waveform),
    }
    let time_domain = fft::convert_sample(&waveform);
    if config.print_progress {
        println!("Transforming...");
    }
    let freq_domain = fft::fft(&time_domain);
    if config.print_progress {
        println!("Analysing...");
    }
    let freq_bins = fft::frequency_bins(&freq_domain);
    if config.print_progress {
        println!("Drawing waveform...");
    }
    let sample_size = waveform.len();
    plotting::plot(
        waveform,
        1. / sample_rate as f32,
        &format!("{} - time domain", config.name),
        &format!("{}time.html", config.output_dir),
        "Time (seconds)",
        "Amplitude",
    );
    if config.print_progress {
        println!("Drawing frequencies...");
    }
    let freq_resolution = sample_rate as f32 / sample_size as f32;
    plotting::plot(
        freq_bins,
        freq_resolution,
        &format!("{} - frequency domain", config.name),
        &format!("{}freq.html", config.output_dir),
        "Frequency (Hz)",
        "Amplitude",
    );
    if config.print_progress {
        println!("Writing wav to file...");
    }
    let waveform32 = time_domain.iter().map(|x| x.re as i16).collect();
    write_wav_file(
        &format!("{}waveform.wav", config.output_dir),
        waveform32,
        sample_rate as u32,
    )
    .expect("failed to write file");
    if config.print_progress {
        println!("Analysis written to disk.");
    }
}
