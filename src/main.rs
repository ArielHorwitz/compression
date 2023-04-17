/// Proof of concept for compressing and decompressing media files.
use clap::Parser;
use compression::{bmp, wav};
use std::cmp::Ordering;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

type BoxedError = Box<dyn std::error::Error>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (.wav or .bmp)
    #[arg()]
    file: String,
    /// Compression level (higher: smaller file size, lower: better quality)
    #[arg(short = 'c', long, default_value_t = 10.)]
    compression: f32,
    /// Analyze frequencies
    #[arg(short, long, default_value_t = false)]
    analyze: bool,
    /// Log factor (when analyzing)
    #[arg(short = 'l', long, default_value_t = 2.5)]
    log_factor: f32,
    /// Output directory
    #[arg(short, long, default_value_t = String::from("data"))]
    output_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let file = PathBuf::from(args.file);
    if !file.is_file() {
        return Err(BoxedError::from("Not a file."));
    }
    let stem = file
        .file_stem()
        .expect("cannot get file stem")
        .to_string_lossy()
        .to_string();
    let suffix = file
        .extension()
        .expect("cannot get file suffix")
        .to_string_lossy()
        .to_string();
    let output_dir = PathBuf::from(args.output_dir);
    let wav_freq_cutoff = match args.compression.partial_cmp(&1.) {
        Some(Ordering::Greater) => (22050. / args.compression).ceil() as usize,
        _ => 22050,
    };
    let bmp_compression_level = match args.compression.partial_cmp(&0.) {
        Some(Ordering::Greater) => args.compression,
        _ => 0.01,
    };
    match (suffix.as_str(), args.analyze) {
        // Compress
        ("wav", false) => {
            let compressed_output = output_dir.join(format!("{stem}.cwv"));
            wav::compress_wav(&file, &compressed_output, wav_freq_cutoff)?;
            println!("Compressed to: {compressed_output:?}");
        }
        ("bmp", false) => {
            let compressed_output = output_dir.join(format!("{stem}.cbm"));
            bmp::compress_bmp(&file, &compressed_output, bmp_compression_level)?;
            println!("Compressed to: {compressed_output:?}");
        }
        // Decompress
        ("cwv", false) => {
            let decompressed_output = output_dir.join(format!("{stem}_decompressed.wav"));
            wav::decompress_wav(&file, &decompressed_output)?;
            println!("Decompressed to: {decompressed_output:?}");
        }
        ("cbm", false) => {
            let decompressed_output = output_dir.join(format!("{stem}_decompressed.bmp"));
            bmp::decompress_bmp(&file, &decompressed_output)?;
            println!("Decompressed to: {decompressed_output:?}");
        }
        // Analyze
        ("wav", true) => {
            let analysis = wav::analyze_waveform(&file, &output_dir)?;
            println!("Analysis file: {analysis:?}");
            Command::new("xdg-open").arg(analysis).spawn()?;
        }
        ("bmp", true) => {
            let log_factor = 1. / args.log_factor;
            let analysis = bmp::analyze_image(&file, log_factor, &output_dir)?;
            Command::new("xdg-open").arg(analysis).spawn()?;
        }
        _ => return Err(BoxedError::from("file suffix unrecognized")),
    }
    Ok(())
}
