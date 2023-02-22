use compression::{audio, common::WaveformMetadata, fft, plotting::plot};
use inquire::Select;

const DATA_DIR: &str = "data/";

fn main() {
    load_data();
}

fn load_data() {
    loop {
        let options = vec!["Custom", "File", "Exit"];
        let mode = Select::new("Select Data:", options)
            .prompt_skippable()
            .unwrap();
        let (metadata, mut waveform) = match mode {
            Some("File") => {
                if let Some(file) = prompt_file() {
                    get_wav_file(file)
                } else {
                    break;
                }
            }
            Some("Custom") => get_custom(),
            _ => break,
        };
        analyze_waveform(metadata, &mut waveform);
    }
}

fn get_wav_file(file: String) -> (WaveformMetadata, Vec<f32>) {
    let (metadata, waveform) = compression::audio::load_wav_file(&file).unwrap();
    println!("{:?}", metadata);
    (metadata, waveform)
}

fn get_custom() -> (WaveformMetadata, Vec<f32>) {
    let waveform = [0., 22937., 32767., 22937., 0., -22937., -32767., -22937.].to_vec();
    let sample_rate = 44100;
    let bit_rate = 16;
    let metadata = WaveformMetadata::new("custom waveform", waveform.len(), sample_rate, bit_rate);
    (metadata, waveform)
}

fn analyze_waveform(metadata: WaveformMetadata, waveform: &mut Vec<f32>) {
    prompt_round_sample(waveform);
    let metadata = WaveformMetadata {
        sample_size: waveform.len(),
        ..metadata
    };
    let time_domain = fft::convert_sample(&waveform);
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
    Export,
}

fn prompt_analysis_option() -> Option<AnalysisOption> {
    let option_names: Vec<String> = [
        "Plot domains".to_string(),
        "Export waveform".to_string(),
        "Back".to_string(),
    ]
    .to_vec();
    let uinput = Select::new("Options:", option_names)
        .prompt_skippable()
        .unwrap();
    if let Some(option) = uinput {
        match option.as_str() {
            "Plot domains" => Some(AnalysisOption::Plot),
            "Export waveform" => Some(AnalysisOption::Export),
            _ => None,
        }
    } else {
        None
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
    let selection = Select::new("Round to 2^n:", vec!["up", "down"])
        .prompt_skippable()
        .unwrap();
    match selection {
        Some("up") => fft::round_sample_size_up(vec),
        Some("down") => fft::round_sample_size_down(vec),
        _ => (),
    }
    println!("New sample size: {}", vec.len());
}
