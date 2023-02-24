use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WaveformMetadata {
    pub name: String,
    pub sample_size: usize,
    pub sample_rate: usize,
    pub freq_resolution: f32,
    pub bit_rate: usize,
}

impl WaveformMetadata {
    pub fn new(
        name: &str,
        sample_size: usize,
        sample_rate: usize,
        bit_rate: usize,
    ) -> WaveformMetadata {
        let freq_resolution = sample_rate as f32 / sample_size as f32;
        WaveformMetadata {
            name: name.to_string(),
            sample_size,
            sample_rate,
            freq_resolution,
            bit_rate,
        }
    }
}
