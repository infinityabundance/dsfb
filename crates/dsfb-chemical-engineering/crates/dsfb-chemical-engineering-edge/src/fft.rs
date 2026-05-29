//! Minimal deterministic radix-2 FFT and spectral helpers for the spectral detector family.
//!
//! Pure `f64`, no `unsafe`, no external deps. Inputs are zero-padded to the next power of two.
//! Used for spectral-entropy and band-energy detectors over sliding windows.

use std::f64::consts::PI;

/// In-place iterative radix-2 Cooley–Tukey FFT on interleaved (re, im) pairs.
/// `data.len()` must be `2 * n` where `n` is a power of two.
fn fft_in_place(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(im.len(), n);
    // Bit-reversal permutation.
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    // Danielson–Lanczos.
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / len as f64;
        let (wl_re, wl_im) = (ang.cos(), ang.sin());
        let mut i = 0;
        while i < n {
            let (mut w_re, mut w_im) = (1.0f64, 0.0f64);
            for k in 0..len / 2 {
                let u_re = re[i + k];
                let u_im = im[i + k];
                let t_re = re[i + k + len / 2] * w_re - im[i + k + len / 2] * w_im;
                let t_im = re[i + k + len / 2] * w_im + im[i + k + len / 2] * w_re;
                re[i + k] = u_re + t_re;
                im[i + k] = u_im + t_im;
                re[i + k + len / 2] = u_re - t_re;
                im[i + k + len / 2] = u_im - t_im;
                let nw_re = w_re * wl_re - w_im * wl_im;
                let nw_im = w_re * wl_im + w_im * wl_re;
                w_re = nw_re;
                w_im = nw_im;
            }
            i += len;
        }
        len <<= 1;
    }
}

/// One-sided power spectrum of a real signal (length-`m` bins for input padded to `2m`).
pub fn power_spectrum(signal: &[f64]) -> Vec<f64> {
    let n = signal.len().next_power_of_two().max(2);
    let mut re = vec![0.0; n];
    let mut im = vec![0.0; n];
    // Remove DC offset to focus on fluctuation structure.
    let m = signal.iter().sum::<f64>() / signal.len().max(1) as f64;
    for (i, &v) in signal.iter().enumerate() {
        re[i] = v - m;
    }
    fft_in_place(&mut re, &mut im);
    (0..n / 2).map(|k| re[k] * re[k] + im[k] * im[k]).collect()
}

/// Spectral entropy (normalised to `[0,1]`) of a window: high = white/disordered, low = tonal.
pub fn spectral_entropy(signal: &[f64]) -> f64 {
    let ps = power_spectrum(signal);
    let total: f64 = ps.iter().sum();
    if total <= 1e-30 || ps.len() < 2 {
        return 0.0;
    }
    let mut h = 0.0;
    for &p in &ps {
        let pn = p / total;
        if pn > 1e-30 {
            h -= pn * pn.ln();
        }
    }
    h / (ps.len() as f64).ln()
}

/// Fraction of spectral power in the high-frequency half-band (a coarse band-energy detector).
pub fn high_band_energy_fraction(signal: &[f64]) -> f64 {
    let ps = power_spectrum(signal);
    let total: f64 = ps.iter().sum();
    if total <= 1e-30 {
        return 0.0;
    }
    let half = ps.len() / 2;
    let hi: f64 = ps.iter().skip(half).sum();
    hi / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_tone_has_low_entropy() {
        let s: Vec<f64> = (0..64)
            .map(|i| (2.0 * PI * 4.0 * i as f64 / 64.0).sin())
            .collect();
        let white: Vec<f64> = (0..64)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 } * ((i * 7 % 13) as f64 - 6.0))
            .collect();
        assert!(spectral_entropy(&s) < spectral_entropy(&white));
    }
}
