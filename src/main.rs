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
    match suffix.as_str() {
        "wav" => {
            if args.analyze {
                let analysis = wav::analyze_waveform(&file, &output_dir)?;
                Command::new("xdg-open").arg(analysis).spawn()?;
            } else {
                let compression_level = match args.compression.partial_cmp(&1.) {
                    Some(Ordering::Greater) => args.compression,
                    _ => 1.,
                };
                let freq_cutoff = (22050. / compression_level).ceil() as usize;
                let compressed_output = output_dir.join(format!("{stem}_compressed.cmp"));
                println!("Compressing to: {compressed_output:?}");
                wav::compress_wav(&file, &compressed_output, freq_cutoff)?;
                let decompressed_output = output_dir.join(format!("{stem}_decompressed.wav"));
                println!("Decompressing to: {decompressed_output:?}");
                wav::decompress_wav(&compressed_output, &decompressed_output)?;
            }
        }
        "bmp" => {
            if args.analyze {
                let log_factor = 1. / args.log_factor;
                let analysis = bmp::analyze_image(&file, log_factor, &output_dir)?;
                Command::new("xdg-open").arg(analysis).spawn()?;
            } else {
                let compression_level = match args.compression.partial_cmp(&0.) {
                    Some(Ordering::Greater) => args.compression,
                    _ => 0.01,
                };
                let compressed_output = output_dir.join(format!("{stem}_compressed.cmp"));
                let decompressed_output = output_dir.join(format!("{stem}_decompressed.bmp"));
                bmp::compress_bmp(&file, &compressed_output, compression_level)?;
                bmp::decompress_bmp(&compressed_output, &decompressed_output)?;
            }
        }
        _ => {
            return Err(BoxedError::from(
                "file suffix unrecognized: expected .wav or .bmp",
            ))
        }
    }
    Ok(())
}
