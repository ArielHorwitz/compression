use compression::{audio, fft, plotting};
use inquire::Select;

const OUTPUT_DIR: &str = "data/";

enum Round {
    Up,
    Down,
}

struct AnalysisConfig {
    pub name: String,
    pub rounding: Round,
    pub output_dir: String,
    pub print_progress: bool,
}

impl AnalysisConfig {
    fn new(name: &str, rounding: Round, output_dir: &str, print_progress: bool) -> AnalysisConfig {
        AnalysisConfig {
            name: name.to_owned(),
            rounding,
            output_dir: output_dir.to_owned(),
            print_progress,
        }
    }
}

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

fn analyze_waveform(waveform: Vec<f32>, sample_rate: usize, config: AnalysisConfig) {
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
    audio::write_wav_file(
        &format!("{}waveform.wav", config.output_dir),
        waveform32,
        sample_rate as u32,
    )
    .expect("failed to write file");
    if config.print_progress {
        println!("Analysis written to disk.");
    }
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
    analyze_waveform(waveform, sample_rate, config);
}
