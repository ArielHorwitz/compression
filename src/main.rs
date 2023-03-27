/// Proof of concept for compressing and decompressing media files.
use clap::Parser;
use compression::{bmp, wav};
use std::cmp::Ordering;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (.wav or .bmp)
    #[arg(short, long)]
    file: String,
    /// Compression level (higher: smaller file size, lower: better quality)
    #[arg(short = 'c', long, default_value_t = 10.)]
    compression: f32,
    /// Analyze frequencies
    #[arg(short, long, default_value_t = false)]
    analyze: bool,
    /// Log factor (when analyzing)
    #[arg(short = 'l', long, default_value_t = 5.)]
    log_factor: f32,
    /// Output directory
    #[arg(short, long, default_value_t = String::from("./data/"))]
    output_dir: String,
}

fn main() {
    let args = Args::parse();
    let (_, file) = args.file.rsplit_once("/").unwrap_or(("", &args.file));
    let (stem, suffix) = file.rsplit_once(".").unwrap_or((file, ""));
    match suffix {
        "wav" => {
            if args.analyze {
                let analysis = wav::analyze_waveform(&args.file, &args.output_dir).unwrap();
                Command::new("xdg-open").arg(analysis).spawn().unwrap();
            } else {
                let compression_level = match args.compression.partial_cmp(&1.).expect(&format!(
                    "expected a number for compression level, got: {}",
                    args.compression
                )) {
                    Ordering::Greater => args.compression,
                    _ => 1.,
                };
                let freq_cutoff = (22050. / compression_level).ceil() as usize;
                let compressed_output = format!("{}{}_compressed.cmp", args.output_dir, stem);
                println!("Compressing to: {compressed_output}");
                wav::compress_wav(&args.file, &compressed_output, freq_cutoff).unwrap();
                let decompressed_output = format!("{}{}_decompressed.wav", args.output_dir, stem);
                println!("Decompressing to: {decompressed_output}");
                wav::decompress_wav(&compressed_output, &decompressed_output).unwrap();
            }
        }
        "bmp" => {
            if args.analyze {
                let log_factor = 1. / args.log_factor;
                bmp::analyze_image(&args.file, log_factor, &args.output_dir, true);
            } else {
                let compression_level = match args.compression.partial_cmp(&0.).expect(&format!(
                    "expected a number for compression level, got: {}",
                    args.compression
                )) {
                    Ordering::Greater => args.compression,
                    _ => 0.01,
                };
                let compressed_output = format!("{}{}_compressed.cmp", args.output_dir, stem);
                let decompressed_output = format!("{}{}_decompressed.bmp", args.output_dir, stem);
                bmp::compress_bmp(&args.file, &compressed_output, compression_level);
                bmp::decompress_bmp(&compressed_output, &decompressed_output);
            }
        }
        _ => panic!("File suffix unrecognized: {file} expected .wav or .bmp"),
    }
}
