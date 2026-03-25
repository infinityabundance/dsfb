//! Nominal regime estimation and early-life baseline fitting.
//!
//! The baseline module estimates statistical properties of the signal
//! during the early nominal window: mean waveform shape, variance
//! profile, spectral characteristics, and autocorrelation structure.
//! These are used to construct the nominal model x_model(t) and the
//! admissibility envelope E_R.

/// Baseline statistics estimated from the nominal regime.
#[derive(Debug, Clone)]
pub struct NominalBaseline {
    /// Mean RMS across nominal windows.
    pub mean_rms: f64,
    /// Standard deviation of RMS across nominal windows.
    pub std_rms: f64,
    /// Mean variance across nominal windows.
    pub mean_variance: f64,
    /// Standard deviation of variance across nominal windows.
    pub std_variance: f64,
    /// Mean lag-1 autocorrelation across nominal windows.
    pub mean_autocorr: f64,
    /// Standard deviation of lag-1 autocorrelation.
    pub std_autocorr: f64,
    /// Mean spectral centroid (normalized frequency).
    pub mean_spectral_centroid: f64,
    /// Std of spectral centroid.
    pub std_spectral_centroid: f64,
    /// Mean kurtosis across nominal windows.
    pub mean_kurtosis: f64,
    /// Std of kurtosis across nominal windows.
    pub std_kurtosis: f64,
    /// Per-sample mean waveform (average across nominal snapshots).
    pub mean_waveform: Vec<f64>,
    /// Per-sample variance (used for envelope).
    pub waveform_variance: Vec<f64>,
}

/// Estimate the nominal baseline from a slice of channel data vectors.
///
/// Each entry in `nominal_windows` is one snapshot's channel data.
/// The model prediction x_model(t) is taken as the per-sample mean
/// across the nominal windows.
pub fn estimate_baseline(nominal_windows: &[&[f64]]) -> NominalBaseline {
    let n = nominal_windows.len();
    if n == 0 {
        return NominalBaseline {
            mean_rms: 0.0,
            std_rms: 0.0,
            mean_variance: 0.0,
            std_variance: 0.0,
            mean_autocorr: 0.0,
            std_autocorr: 0.0,
            mean_spectral_centroid: 0.0,
            std_spectral_centroid: 0.0,
            mean_kurtosis: 0.0,
            std_kurtosis: 0.0,
            mean_waveform: Vec::new(),
            waveform_variance: Vec::new(),
        };
    }

    // Compute per-window scalar statistics.
    let mut rms_vals = Vec::with_capacity(n);
    let mut var_vals = Vec::with_capacity(n);
    let mut ac_vals = Vec::with_capacity(n);
    let mut sc_vals = Vec::with_capacity(n);
    let mut kurt_vals = Vec::with_capacity(n);

    for &w in nominal_windows {
        rms_vals.push(rms(w));
        var_vals.push(variance(w));
        ac_vals.push(lag1_autocorrelation(w));
        sc_vals.push(spectral_centroid(w));
        kurt_vals.push(kurtosis(w));
    }

    // Per-sample mean waveform computed over the minimum shared length.
    let min_len = nominal_windows.iter().map(|w| w.len()).min().unwrap_or(0);
    let mut mean_wf = vec![0.0; min_len];
    let mut var_wf = vec![0.0; min_len];
    for &w in nominal_windows {
        for (i, &v) in w.iter().take(min_len).enumerate() {
            mean_wf[i] += v;
        }
    }
    for v in &mut mean_wf {
        *v /= n as f64;
    }
    // Per-sample variance.
    for &w in nominal_windows {
        for (i, &v) in w.iter().take(min_len).enumerate() {
            let d = v - mean_wf[i];
            var_wf[i] += d * d;
        }
    }
    for v in &mut var_wf {
        *v /= n as f64;
    }

    NominalBaseline {
        mean_rms: mean(&rms_vals),
        std_rms: std_dev(&rms_vals),
        mean_variance: mean(&var_vals),
        std_variance: std_dev(&var_vals),
        mean_autocorr: mean(&ac_vals),
        std_autocorr: std_dev(&ac_vals),
        mean_spectral_centroid: mean(&sc_vals),
        std_spectral_centroid: std_dev(&sc_vals),
        mean_kurtosis: mean(&kurt_vals),
        std_kurtosis: std_dev(&kurt_vals),
        mean_waveform: mean_wf,
        waveform_variance: var_wf,
    }
}

// ----- Helper math functions -----

/// Arithmetic mean.
pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

/// Population standard deviation.
pub fn std_dev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
    var.sqrt()
}

/// Root mean square.
pub fn rms(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
}

/// Population variance.
pub fn variance(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let m = mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64
}

/// Excess kurtosis.
pub fn kurtosis(v: &[f64]) -> f64 {
    let n = v.len();
    if n < 4 {
        return 0.0;
    }
    let m = mean(v);
    let var = variance(v);
    if var < 1e-30 {
        return 0.0;
    }
    let m4 = v.iter().map(|x| (x - m).powi(4)).sum::<f64>() / n as f64;
    m4 / (var * var) - 3.0
}

/// Lag-1 autocorrelation.
pub fn lag1_autocorrelation(v: &[f64]) -> f64 {
    let n = v.len();
    if n < 2 {
        return 0.0;
    }
    let m = mean(v);
    let var = variance(v);
    if var < 1e-30 {
        return 0.0;
    }
    let mut ac = 0.0;
    for i in 0..n - 1 {
        ac += (v[i] - m) * (v[i + 1] - m);
    }
    ac /= (n - 1) as f64 * var;
    ac
}

/// Crest factor: peak absolute value / RMS.
pub fn crest_factor(v: &[f64]) -> f64 {
    let r = rms(v);
    if r < 1e-30 {
        return 0.0;
    }
    let peak = v.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    peak / r
}

/// Simplified spectral centroid using DFT magnitude squared.
///
/// Returns the centroid as a normalized frequency (0..0.5).
pub fn spectral_centroid(v: &[f64]) -> f64 {
    let n = v.len();
    if n < 4 {
        return 0.0;
    }
    // Use a simple DFT on the first chunk (up to 2048 samples for efficiency).
    let m = n.min(2048);
    let half = m / 2;
    let mut power = vec![0.0; half];
    let mean_val = mean(&v[..m]);
    for k in 0..half {
        let mut re = 0.0;
        let mut im = 0.0;
        let w = 2.0 * std::f64::consts::PI * k as f64 / m as f64;
        for (j, &x) in v[..m].iter().enumerate() {
            let xc = x - mean_val;
            re += xc * (w * j as f64).cos();
            im -= xc * (w * j as f64).sin();
        }
        power[k] = re * re + im * im;
    }
    let total: f64 = power.iter().sum();
    if total < 1e-30 {
        return 0.0;
    }
    let centroid: f64 = power
        .iter()
        .enumerate()
        .map(|(k, &p)| k as f64 * p)
        .sum::<f64>()
        / total;
    centroid / half as f64 * 0.5
}

/// Spectral band energy: fraction of energy above the half-Nyquist mark.
pub fn spectral_band_energy(v: &[f64]) -> f64 {
    let n = v.len();
    if n < 4 {
        return 0.0;
    }
    let m = n.min(2048);
    let half = m / 2;
    let mut power = vec![0.0; half];
    let mean_val = mean(&v[..m]);
    for k in 0..half {
        let mut re = 0.0;
        let mut im = 0.0;
        let w = 2.0 * std::f64::consts::PI * k as f64 / m as f64;
        for (j, &x) in v[..m].iter().enumerate() {
            let xc = x - mean_val;
            re += xc * (w * j as f64).cos();
            im -= xc * (w * j as f64).sin();
        }
        power[k] = re * re + im * im;
    }
    let total: f64 = power.iter().sum();
    if total < 1e-30 {
        return 0.0;
    }
    let upper: f64 = power[half / 2..].iter().sum();
    upper / total
}
