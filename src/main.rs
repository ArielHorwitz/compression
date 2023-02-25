/// Proof of concept for compressing and decompressing media files.
use clap::Parser;
use compression::utils;
use std::process::Command;

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
        let analysis = utils::analyze_waveform(&args.file, &args.output_dir).unwrap();
        Command::new("xdg-open").arg(analysis).spawn().unwrap();
    } else if suffix == "wav" {
        let output_file = format!("{}{}.cmp", args.output_dir, stem);
        println!("Compressing to: {output_file}");
        utils::compress_wav(&args.file, &output_file, args.freq_cutoff).unwrap();
    } else if suffix == "cmp" {
        let output_file = format!("{}{}.wav", args.output_dir, stem);
        println!("Decompressing to: {output_file}");
        utils::decompress_wav(&args.file, &output_file).unwrap();
    } else {
        panic!("File suffix unrecognized: {file}");
    }
}
