use num_complex::Complex32;
use rustfft::{algorithm::Dft, Fft, FftDirection};
use std::f32::consts::PI;

/// Convert a sequence of floats to complex numbers.
pub fn convert_sample(sample: &[f32]) -> Vec<Complex32> {
    sample.iter().map(|x| Complex32::from(x.clone())).collect()
}

/// Add default values to round sample size up to 2^n.
pub fn round_sample_size_up<T: Default + Clone>(sample: &mut Vec<T>) {
    let original_size = sample.len();
    let nearest_power2 = 2f64.powf((original_size as f64).log2().ceil()) as usize;
    let padding = nearest_power2 - original_size;
    sample.append(&mut vec![T::default(); padding]);
}

/// Removes items to round sample size down to 2^n.
pub fn round_sample_size_down<T: Default + Clone>(sample: &mut Vec<T>) {
    let nearest_power2 = 2f64.powf((sample.len() as f64).log2().floor()) as usize;
    sample.drain(nearest_power2..);
}

/// Perform a 2D FFT on a 2D sample of complex numbers (horizontal then vertical).
pub fn fft_2d(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    fft_2d_vertical(&fft_2d_horizontal(samples))
}

/// Perform an inverse 2D FFT on a 2D sample of complex numbers (vertical then horizontal).
pub fn fft_2d_inverse(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    fft_2d_horizontal_inverse(&fft_2d_vertical_inverse(samples))
}

pub fn fft_2d_horizontal(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    samples.iter().map(|y| fft(y)).collect()
}

pub fn fft_2d_horizontal_inverse(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    samples.iter().map(|y| fft_inverse(y)).collect()
}

pub fn fft_2d_vertical(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    let (height, width) = (samples.len(), samples[0].len());
    let transposed: Vec<Vec<Complex32>> = (0..width)
        .map(|x| fft(&(0..height).map(|y| samples[y][x]).collect()))
        .collect();
    (0..height)
        .map(|y| (0..width).map(|x| transposed[x][y]).collect())
        .collect()
}

pub fn fft_2d_vertical_inverse(samples: &Vec<Vec<Complex32>>) -> Vec<Vec<Complex32>> {
    let (height, width) = (samples.len(), samples[0].len());
    let transposed: Vec<Vec<Complex32>> = (0..width)
        .map(|x| fft_inverse(&(0..height).map(|y| samples[y][x]).collect()))
        .collect();
    (0..height)
        .map(|y| (0..width).map(|x| transposed[x][y]).collect())
        .collect()
}

/// Perform an FFT on a sample of complex numbers.
pub fn fft(samples: &Vec<Complex32>) -> Vec<Complex32> {
    assert_sample_size(&samples);
    fft_recursive(samples.clone(), 1.)
}

/// Perform an inverse FFT on a sample of complex numbers.
pub fn fft_inverse(samples: &Vec<Complex32>) -> Vec<Complex32> {
    assert_sample_size(&samples);
    let sample_size = samples.len() as f32;
    fft_recursive(samples.clone(), -1.)
        .iter()
        .map(|x| x / sample_size)
        .collect()
}

/// Returns the amplitudes of the discernable frequencies in bins (by the frequency resolution).
pub fn frequency_bins(sample: &[Complex32]) -> Vec<f32> {
    let sample_size = sample.len() as f32;
    let alias_index = (sample_size / 2.) as usize;
    sample[0..alias_index]
        .iter()
        .map(|x| x.norm() * 2. / sample_size)
        .collect()
}

fn fft_recursive(sample: Vec<Complex32>, coeff: f32) -> Vec<Complex32> {
    // WARNING: will fail if sample size is not 2^n
    let sample_size = sample.len();
    if sample_size == 1 {
        return sample;
    }
    let half_size = sample_size / 2;

    // Collect transforms of even and odd samples (recursive)
    let mut evens = Vec::with_capacity(half_size);
    let mut odds = Vec::with_capacity(half_size);
    for i in 0..half_size {
        evens.push(sample[2 * i]);
        odds.push(sample[2 * i + 1]);
    }
    let freq_evens = fft_recursive(evens, coeff);
    let freq_odds = fft_recursive(odds, coeff);

    // Calculate frequency bins
    let mut freq_bins = vec![Complex32::default(); sample_size];
    let coeff_const = Complex32::new(0., coeff * -2. * PI / sample_size as f32);
    for k in 0..half_size {
        let k2 = k + half_size;
        let ek1 = coeff_const * k as f32;
        let ek2 = coeff_const * k2 as f32;
        freq_bins[k] = freq_evens[k] + ek1.exp() * freq_odds[k];
        freq_bins[k2] = freq_evens[k] + ek2.exp() * freq_odds[k];
    }
    freq_bins
}

fn assert_sample_size(samples: &Vec<Complex32>) {
    let sample_log = f32::log2(samples.len() as f32);
    assert_eq!(
        sample_log,
        (sample_log as i32 as f32),
        "Sample size is not a power of 2: {}",
        samples.len()
    );
}

#[allow(dead_code)] // For testing
fn basefft(samples: &[Complex32]) -> Vec<Complex32> {
    // Computes a forward FFT
    let mut result = samples.to_vec();
    let fft = Dft::new(result.len(), FftDirection::Forward);
    fft.process(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_builtin() {
        let sample = convert_sample(&[0., 1., 0., -1.]);
        let result = fft(&sample);
        let expected = basefft(&sample);
        let epsilon = 10f32.powi(-5);
        println!("result {:?}\nexpected {:?}", result, expected);
        for i in 0..expected.len() {
            let diff = result[i].l1_norm() - expected[i].l1_norm();
            assert!(f32::abs(diff) < epsilon);
        }
    }

    #[test]
    fn inversion() {
        let sample = convert_sample(&[1., 2., 3., 4., 5., 6., 7., 8.]);
        let result = fft(&fft_inverse(&sample));
        let epsilon = 10f32.powi(-5);
        println!("result {:?}\nexpected {:?}", sample, result);
        for i in 0..result.len() {
            let diff = sample[i].l1_norm() - result[i].l1_norm();
            assert!(f32::abs(diff) < epsilon);
        }
    }
}
