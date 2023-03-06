/// Proof of concept for compressing and decompressing media files.
use clap::Parser;
use compression::utils;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WAV file
    #[arg(short, long)]
    file: String,
    /// Analyze frequencies
    #[arg(short, long, default_value_t = false)]
    analyze: bool,
    /// Frequency cutoff (when compressing)
    #[arg(short = 'c', long, default_value_t = 3000)]
    freq_cutoff: usize,
    /// Output folder
    #[arg(short, long, default_value_t = String::from("./"))]
    output_dir: String,
}

fn main() {
    let args = Args::parse();
    let (_, file) = args.file.rsplit_once("/").unwrap_or(("", &args.file));
    let (stem, suffix) = file.rsplit_once(".").unwrap_or((file, ""));
    if suffix != "wav" {
        panic!("File suffix unrecognized: {file} expected .wav");
    }
    if args.analyze {
        let analysis = utils::analyze_waveform(&args.file, &args.output_dir).unwrap();
        Command::new("xdg-open").arg(analysis).spawn().unwrap();
    } else {
        let compressed_output = format!("{}{}_compressed.cmp", args.output_dir, stem);
        println!("Compressing to: {compressed_output}");
        utils::compress_wav(&args.file, &compressed_output, args.freq_cutoff).unwrap();
        let decompressed_output = format!("{}{}_decompressed.wav", args.output_dir, stem);
        println!("Decompressing to: {decompressed_output}");
        utils::decompress_wav(&compressed_output, &decompressed_output).unwrap();
    }
}
