use std::ops::RangeInclusive;

use compression::{audio, common::WaveformMetadata, fft, plotting::plot};
use inquire::{validator::Validation, Select, Text};

const DATA_DIR: &str = "data/";

fn main() {
    load_data();
}

fn load_data() {
    loop {
        if let Some(file) = prompt_file() {
            let (metadata, mut waveform) = get_wav_file(file);
            analyze_waveform(metadata, &mut waveform);
        } else {
            break;
        }
    }
}

fn get_wav_file(file: String) -> (WaveformMetadata, Vec<f32>) {
    let (metadata, waveform) = compression::audio::load_wav_file(&file).unwrap();
    println!("{:?}", metadata);
    (metadata, waveform)
}

fn analyze_waveform(metadata: WaveformMetadata, waveform: &mut Vec<f32>) {
    prompt_round_sample(waveform);
    let metadata = WaveformMetadata {
        sample_size: waveform.len(),
        ..metadata
    };
    let mut time_domain = fft::convert_sample(&waveform);
    loop {
        let option = match prompt_analysis_option() {
            Some(opt) => opt,
            None => break,
        };
        match option {
            AnalysisOption::Plot => {
                let freq_bins = fft::frequency_bins(&fft::fft(&time_domain));
                let file_path = format!("{DATA_DIR}analysis.html");
                plot(waveform.clone(), freq_bins, &metadata, &file_path);
            }
            AnalysisOption::FlattenRange(range) => {
                let mut freq_domain = fft::fft(&time_domain);
                audio::flatten_freq_range(&mut freq_domain, &metadata, range).unwrap_or_default();
                time_domain = fft::fft_inverse(&freq_domain);
            }
            AnalysisOption::Export => {
                let modified_waveform = time_domain.iter().map(|x| x.re as i16).collect();
                let modified_metadata = &WaveformMetadata {
                    name: format!("{}_modified", metadata.name),
                    ..metadata
                };
                export_waveform(modified_waveform, modified_metadata);
            }
        }
    }
}

enum AnalysisOption {
    Plot,
    FlattenRange(RangeInclusive<f32>),
    Export,
}

fn prompt_analysis_option() -> Option<AnalysisOption> {
    let option_names: Vec<String> = [
        "Plot domains".to_string(),
        "Flatten range".to_string(),
        "Export waveform".to_string(),
    ]
    .to_vec();
    let uinput = Select::new("Options:", option_names)
        .prompt_skippable()
        .unwrap();
    if let Some(option) = uinput {
        match option.as_str() {
            "Plot domains" => Some(AnalysisOption::Plot),
            "Flatten range" => {
                if let Some(range) = prompt_range() {
                    Some(AnalysisOption::FlattenRange(range))
                } else {
                    None
                }
            }
            "Export waveform" => Some(AnalysisOption::Export),
            _ => None,
        }
    } else {
        None
    }
}

fn prompt_range() -> Option<RangeInclusive<f32>> {
    let start = Text::new("Start:")
        .with_validator(validate_numbers)
        .prompt_skippable()
        .unwrap();
    if let Some(start) = start {
        let end = Text::new("End:")
            .with_validator(validate_numbers)
            .prompt_skippable()
            .unwrap();
        if let Some(end) = end {
            Some(start.parse::<f32>().unwrap()..=end.parse::<f32>().unwrap())
        } else {
            None
        }
    } else {
        None
    }
}

fn validate_numbers(
    s: &str,
) -> Result<Validation, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
    match s.parse::<f32>() {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            inquire::validator::ErrorMessage::Custom("Not a valid number".to_string()),
        )),
    }
}

fn export_waveform(waveform: Vec<i16>, metadata: &WaveformMetadata) {
    let path = format!("{}{}.wav", DATA_DIR, metadata.name);
    println!("Exporting {path}");
    audio::write_wav_file(&path, waveform, metadata).expect("failed to write file");
}

fn prompt_file() -> Option<String> {
    let mut files = Vec::new();
    for path in std::fs::read_dir(DATA_DIR).unwrap() {
        let p = path
            .unwrap()
            .path()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned();
        if p.ends_with(".wav") {
            files.push(p);
        }
    }
    Select::new("File:", files).prompt_skippable().unwrap()
}

fn prompt_round_sample(vec: &mut Vec<f32>) {
    let selection = Select::new("Round to 2^n:", vec!["down", "up"])
        .prompt_skippable()
        .unwrap();
    match selection {
        Some("down") => fft::round_sample_size_down(vec),
        _ => fft::round_sample_size_up(vec),
    }
    println!("New sample size: {}", vec.len());
}
