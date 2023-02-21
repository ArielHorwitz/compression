use compression::{AnalysisConfig, Round};
use inquire::Select;

const OUTPUT_DIR: &str = "data/";

fn prompt_file() -> String {
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
    Select::new("File:", files).prompt().unwrap()
}

fn prompt_rounding() -> Round {
    let selection = Select::new("Round to 2^n:", vec!["up", "down"])
        .prompt()
        .unwrap();
    if selection == "up" {
        return Round::Up;
    }
    Round::Down
}

fn get_wav_file() -> (Vec<f32>, usize, String) {
    let file = prompt_file();
    let (header, waveform) = compression::audio::load_wav_file(&file).unwrap();
    println!("Sample size: {}\n{:?}", waveform.len(), header);
    (waveform, header.sampling_rate as usize, file)
}

fn get_custom() -> (Vec<f32>, usize, String) {
    let waveform = [0., 22937., 32767., 22937., 0., -22937., -32767., -22937.].to_vec();
    let sample_rate = 44100;
    (waveform, sample_rate, String::from("custom waveform"))
}

fn main() {
    let options = vec!["Custom", "File"];
    let mode = Select::new("Select Mode:", options).prompt().unwrap();
    let (waveform, sample_rate, name) = match mode {
        "File" => get_wav_file(),
        _ => get_custom(),
    };
    let rounding = prompt_rounding();
    let config = AnalysisConfig::new(&name, rounding, OUTPUT_DIR, true);
    compression::analyze_waveform(waveform, sample_rate, config);
}
