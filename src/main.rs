use compression::{audio, fft, plotting};
use inquire::Select;
use num_complex::Complex32;

const OUTPUT_DIR: &str = "data/";

fn main() {
    load_data();
}

fn load_data() {
    loop {
        let options = vec!["Custom", "File", "Exit"];
        let mode = Select::new("Select Data:", options)
            .prompt_skippable()
            .unwrap();
        let (mut waveform, sample_rate, name) = match mode {
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
        analyze_waveform(&mut waveform, sample_rate, &name);
    }
}

fn get_wav_file(file: String) -> (Vec<f32>, usize, String) {
    let (header, waveform) = compression::audio::load_wav_file(&file).unwrap();
    println!("{:?}", header);
    (waveform, header.sampling_rate as usize, file)
}

fn get_custom() -> (Vec<f32>, usize, String) {
    let waveform = [0., 22937., 32767., 22937., 0., -22937., -32767., -22937.].to_vec();
    let sample_rate = 44100;
    (waveform, sample_rate, String::from("custom waveform"))
}

fn analyze_waveform(waveform: &mut Vec<f32>, sample_rate: usize, name: &str) {
    prompt_round_sample(waveform);
    let sample_size = waveform.len();
    let time_domain = fft::convert_sample(&waveform);
    loop {
        let option = match prompt_analysis_option() {
            Some(opt) => opt,
            None => break,
        };
        match option {
            AnalysisOption::Plot => {
                plot_time_domain(waveform.clone(), sample_rate as u32, name);
                let freq_domain = fft::fft(&time_domain);
                let freq_resolution = sample_rate as f32 / sample_size as f32;
                plot_freqbins(&freq_domain, freq_resolution, name);
            }
            AnalysisOption::Export => {
                let modified_waveform = time_domain.iter().map(|x| x.re as i16).collect();
                export_waveform(modified_waveform, sample_rate as u32);
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

fn plot_time_domain(time_domain: Vec<f32>, sample_rate: u32, name: &str) {
    plotting::plot(
        time_domain,
        1. / sample_rate as f32,
        &format!("{} - time domain", &name),
        &format!("{}time.html", OUTPUT_DIR),
        "Time (seconds)",
        "Amplitude",
    );
}

fn plot_freqbins(freq_domain: &Vec<Complex32>, freq_resolution: f32, name: &str) {
    println!("Drawing frequencies...");
    let freq_bins = fft::frequency_bins(freq_domain);
    plotting::plot(
        freq_bins,
        freq_resolution,
        &format!("{} - frequency domain", name),
        &format!("{}freq.html", OUTPUT_DIR),
        "Frequency (Hz)",
        "Amplitude",
    );
}

fn export_waveform(waveform: Vec<i16>, sample_rate: u32) {
    let path = format!("{}waveform.wav", OUTPUT_DIR);
    println!("Exporting {path}");
    audio::write_wav_file(&path, waveform, sample_rate).expect("failed to write file");
}

fn prompt_file() -> Option<String> {
    let mut files = Vec::new();
    for path in std::fs::read_dir(OUTPUT_DIR).unwrap() {
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
