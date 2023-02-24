use clap::Parser;
use compression::{fft, plotting::plot};
use std::process::Command;

/// Proof of concept for compressing and decompressing media files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to compress / decompress / analyze
    #[arg(short, long)]
    file: String,

    /// Frequency cutoff (when compressing)
    #[arg(short = 'c', long, default_value_t = 3000)]
    freq_cutoff: usize,

    /// Analyze frequencies
    #[arg(short, long, default_value_t = false)]
    analyze: bool,

    /// Output file
    #[arg(short, long, default_value_t = String::from("./"))]
    output_dir: String,
}

fn main() {
    let args = Args::parse();
    // TODO: use crate fsio
    let (_, file) = args.file.rsplit_once("/").unwrap_or(("", &args.file));
    let (stem, suffix) = file.rsplit_once(".").unwrap();
    if args.analyze {
        analyze_waveform(&args.file, &args.output_dir);
    } else if suffix == "wav" {
        let output_file = format!("{}{}.cmp", args.output_dir, stem);
        println!("Compressing to: {output_file}");
        compression::compress_wav(&args.file, &output_file, args.freq_cutoff).unwrap();
    } else if suffix == "cmp" {
        let output_file = format!("{}{}.wav", args.output_dir, stem);
        println!("Decompressing to: {output_file}");
        compression::decompress_wav(&args.file, &output_file).unwrap();
    } else {
        panic!("File suffix unrecognized: {file}");
    }
}

fn analyze_waveform(wav_file: &str, output_dir: &str) {
    let file_path = format!("{output_dir}analysis.html");
    let (metadata, mut waveform) = compression::audio::load_wav_file(wav_file).unwrap();
    fft::round_sample_size_up(&mut waveform);
    let time_domain = fft::convert_sample(&waveform);
    let freq_bins = fft::frequency_bins(&fft::fft(&time_domain));
    println!("Writing analysis to: {file_path}");
    plot(waveform.clone(), freq_bins, &metadata, &file_path);
    Command::new("xdg-open").arg(file_path).spawn().unwrap();
}
