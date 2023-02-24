//! # Compression
//! Proof of concept for compressing and decompressing media files.
//!

mod compression;

pub mod audio;
pub mod common;
pub mod fft;
pub mod plotting;
pub use crate::compression::{compress_wav, decompress_wav};
