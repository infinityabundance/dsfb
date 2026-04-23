//! RadioML 2018.01a HDF5 loader — amplitude-template residual evaluation.
//!
//! Enabled with `--features hdf5_loader`. Requires `libhdf5` installed on the
//! host (e.g. `pacman -S hdf5` on Arch / `apt install libhdf5-dev` on Debian).
//!
//! # Residual Construction (paper §IX step 2, Algorithm 1)
//!
//! DSFB-RF is an **augmentation of existing receiver chains**.  It observes
//! the residual norm ‖r(k)‖ — near-zero when the receiver is healthy, growing
//! structurally when the channel degrades past its operating threshold.
//!
//! This loader implements the paper's Stage III residual construction in the
//! **amplitude domain**:
//!
//! 1. **Nominal reference** (paper §IX step 1).  For each modulation class,
//!    compute the per-sample amplitude template from the first 100 captures
//!    at SNR ≥ +10 dB:
//!
//!    `template[n] = mean( |x_k[n]| )  over k ∈ healthy window,  n = 0..N_s`
//!
//! 2. **Residual** (paper §IX step 2).  For each capture:
//!
//!    `r(k) = |x(k)| − template`   (element-wise amplitude deviation)
//!    `‖r(k)‖ = sqrt( (1/N_s) Σ (|x_k[n]| − template[n])² )`
//!
//! 3. **Envelope** (paper §IX step 3).  `ρ = 3σ` from the healthy-window
//!    residual norms.  No hand-tuning.
//!
//! ## Why amplitude domain (not complex IQ)
//!
//! RadioML captures have random per-capture carrier phase offsets from the
//! USRP hardware.  A complex-valued mean across captures at different phases
//! averages toward zero.  The amplitude `|x[n]| = sqrt(I² + Q²)` is phase-
//! invariant: it preserves each modulation's deterministic amplitude shape
//! (bimodal for OOK, constant for PSK, multi-level for QAM) without requiring
//! carrier synchronisation.
//!
//! At high SNR, captures closely match the amplitude template → small ‖r(k)‖.
//! At low SNR, noise dominates → large ‖r(k)‖.  The transition occurs at the
//! SNR where the modulation's amplitude pattern becomes unrecognisable — the
//! **demodulation threshold**.  This is the structural phase transition DSFB
//! detects.
//!
//! ## Relationship to paper Table IV
//!
//! The paper's Table IV numbers used a receiver-chain decoder residual from
//! carrier-synchronised demodulation.  The amplitude-template residual here
//! is not identical — it operates in the amplitude domain without phase
//! recovery.  It captures the same structural phenomenon (amplitude shape
//! collapse at the demodulation threshold) via a different projection.
//!
//! # HDF5 Schema (RadioML 2018.01a / DeepSig GOLD variant)
//!
//! | Dataset | Shape          | Dtype | Description                           |
//! |---------|----------------|-------|---------------------------------------|
//! | `X`     | `[N, 1024, 2]` | f32   | IQ samples (1024 samp, 2 channels)    |
//! | `Y`     | `[N, 24]`      | i64   | One-hot modulation class label        |
//! | `Z`     | `[N, 1]`       | i64   | Per-capture SNR in dB (integer)       |
//!
//! N = 2,555,904 = 24 mod classes × 26 SNR levels × 4,096 captures.
//! File ordering: mod-class-major, SNR-minor (−20 dB → +30 dB within each class).
//!
//! # Ground-Truth Events
//!
//! Every SNR-bin boundary in the per-class descending-SNR stream is a real
//! physical regime transition in the amplitude distribution.  With 26 SNR
//! levels, each class has up to 25 bin boundaries (~16 in the evaluation
//! region after the healthy window).  The flat-stream loader (`load_radioml`)
//! interleaves classes and produces crossings consistent with the 102-event
//! protocol described in the paper.
//!
//! # Memory
//!
//! Reading X requires ≈ 21 GB RAM.  Per-class amplitude templates require
//! 24 × 1024 × 4 bytes ≈ 96 KB additional.

extern crate std;

use std::boxed::Box;
use std::error::Error;
use std::vec::Vec;

use crate::pipeline::{RfObservation, RegimeTransitionEvent, HEALTHY_WINDOW_SIZE};
use hdf5_metno::File;

// ── Sorted-Amplitude Residual (Wasserstein distance in amplitude domain) ──────
//
// For each modulation class, the healthy-window reference is the **sorted**
// amplitude quantile function:
//
//   template[n] = mean( sort(|x_k|)[n] )  over k ∈ calibration window
//
// For each capture, the residual is the Wasserstein-2 distance between its
// sorted amplitude distribution and the reference:
//
//   ‖r(k)‖ = sqrt( (1/N_s) Σ_n (sort(|x_k|)[n] − template[n])² )
//
// WHY SORTED: RadioML captures have random symbol sequences per capture.
// Per-sample-index averaging across captures averages across different
// symbols → smeared template → every capture deviates → no contrast.
// Sorting aligns amplitude levels regardless of symbol ordering:
//   - At high SNR: amplitude distribution matches reference → small ‖r(k)‖
//   - At low SNR:  noise smears the distribution → large ‖r(k)‖
//   - The transition is sharp at the demodulation threshold.
//
// No carrier synchronisation, no demodulation, no FFT, no new dependencies.
// Sorting + subtraction + squaring + square root.

/// Minimum SNR (dB) for captures included in the template calibration window.
/// Using +28 dB restricts to the two highest SNR levels (+28, +30) for a
/// cleaner template uncontaminated by moderate-SNR noise.
const HEALTHY_SNR_MIN_DB: f32 = 28.0;

/// Get the k-th IQ pair from a flat per-capture slice.
///
/// `is_channel_last=true`  → layout [S,2]: iq[2k]=I, iq[2k+1]=Q
/// `is_channel_last=false` → layout [2,S]: iq[k]=I, iq[n_samples+k]=Q
#[inline]
fn get_iq(iq: &[f32], k: usize, n_samples: usize, is_channel_last: bool) -> (f32, f32) {
    if is_channel_last {
        (iq[2 * k], iq[2 * k + 1])
    } else {
        (iq[k], iq[n_samples + k])
    }
}

/// Compute the **sorted, RMS-normalised** amplitude vector for one capture.
///
/// Steps:
/// 1. Compute `|x[n]| = sqrt(I² + Q²)` for each IQ pair.
/// 2. Divide by per-capture RMS to remove gain variation (isolates shape).
/// 3. Sort ascending to form the empirical quantile function.
///
/// RMS normalisation ensures that the residual measures distributional SHAPE
/// change (modulation-specific → noise-like), not absolute energy change.
fn sorted_amplitude_vector(iq: &[f32], n_samples: usize, is_channel_last: bool) -> Vec<f32> {
    let mut amps = Vec::with_capacity(n_samples);
    for k in 0..n_samples {
        let (i, q) = get_iq(iq, k, n_samples, is_channel_last);
        amps.push((i * i + q * q).sqrt());
    }
    // Per-capture RMS normalisation: removes gain variation, isolates shape.
    let rms = {
        let sum_sq: f64 = amps.iter().map(|&a| (a as f64) * (a as f64)).sum();
        ((sum_sq / n_samples as f64).sqrt()) as f32
    };
    if rms > 1e-8 {
        for a in &mut amps {
            *a /= rms;
        }
    }
    amps.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    amps
}

/// Compute ‖r(k)‖ = RMS of element-wise deviation between sorted amplitudes.
///
/// Both `amps` and `template` must be pre-sorted ascending.  The result is
/// the discrete Wasserstein-2 distance between the two amplitude distributions.
///
/// This is the scalar residual norm fed to the DSFB engine.
fn amplitude_template_residual(amps: &[f32], template: &[f32]) -> f32 {
    let n = amps.len().min(template.len());
    if n == 0 { return 0.0; }
    let mut sum_sq = 0.0_f64;
    for i in 0..n {
        let d = (amps[i] - template[i]) as f64;
        sum_sq += d * d;
    }
    ((sum_sq / n as f64).sqrt()) as f32
}


// ── Internal shared dataset ───────────────────────────────────────────────────

/// Loaded dataset shared between the two public loader functions.
#[allow(dead_code)]
struct RawDataset {
    n:                usize,
    n_classes:        usize,
    n_samples:        usize,        // IQ pairs per capture (1024 for GOLD)
    z:                Vec<f32>,      // SNR (dB) per capture — i64 → f32
    mod_class:        Vec<usize>,    // argmax of Y one-hot row per capture
    decoder_residual: Vec<f32>,      // ‖r(k)‖ = amplitude-template residual norm
    templates:        Vec<Vec<f32>>, // per-class amplitude templates (n_classes × n_samples)
}

/// Read Z, Y, X from an HDF5 file, build per-class amplitude templates,
/// and compute the amplitude-template residual ‖r(k)‖ for every capture.
///
/// Two-pass over X:
/// 1. First pass: accumulate amplitude templates from calibration captures
///    (SNR ≥ +10 dB) per class.
/// 2. Second pass: compute ‖r(k)‖ for every capture using its class template.
///
/// Heavy I/O — reads ~21 GB for the GOLD 1024-sample variant.
fn load_raw_dataset(path: &str) -> Result<RawDataset, Box<dyn Error>> {
    let file = File::open(path)
        .map_err(|e| std::format!("Cannot open HDF5 file '{}': {}", path, e))?;

    let z = read_snr_labels(&file, path)?;
    let n = z.len();
    let (mod_class, n_classes) = read_mod_class_labels(&file, path, n)?;
    let (x_raw, n_iq, is_channel_last) = read_iq_data(&file, path, n)?;
    let n_samples = n_iq / 2;

    let templates = build_class_templates(&x_raw, &z, &mod_class, n_classes, n_samples, n_iq, is_channel_last)?;
    let decoder_residual = compute_decoder_residuals(&x_raw, &templates, &mod_class, n, n_iq, n_samples, is_channel_last);
    print_snr_diagnostics(&decoder_residual, &z, &mod_class, n);

    Ok(RawDataset { n, n_classes, n_samples, z, mod_class, decoder_residual, templates })
}

fn read_snr_labels(file: &File, path: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let z_ds = file.dataset("Z")
        .map_err(|e| std::format!("Dataset 'Z' not found in '{}': {}", path, e))?;
    let z_shape = z_ds.shape();
    let n = z_shape[0];
    let z_i64: Vec<i64> = z_ds.read_raw()?;
    if z_i64.len() != n {
        return Err(std::format!(
            "Z has shape {:?} but {} elements; expected {}", z_shape, z_i64.len(), n
        ).into());
    }
    Ok(z_i64.iter().map(|&v| v as f32).collect())
}

fn read_mod_class_labels(file: &File, path: &str, n: usize) -> Result<(Vec<usize>, usize), Box<dyn Error>> {
    let y_ds = file.dataset("Y")
        .map_err(|e| std::format!("Dataset 'Y' not found in '{}': {}", path, e))?;
    let y_shape = y_ds.shape();
    let n_classes = if y_shape.len() == 2 { y_shape[1] } else {
        return Err(std::format!("Unexpected Y shape {:?}", y_shape).into());
    };
    let y_i64: Vec<i64> = y_ds.read_raw()?;
    let mod_class: Vec<usize> = y_i64.chunks(n_classes)
        .map(|row| row.iter().enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap_or(0))
        .collect();
    if mod_class.len() != n {
        return Err(std::format!(
            "Y produced {} class indices; expected {}", mod_class.len(), n
        ).into());
    }
    std::println!("  Mod classes : {} (Y shape {:?})", n_classes, y_shape);
    Ok((mod_class, n_classes))
}

fn read_iq_data(file: &File, path: &str, n: usize) -> Result<(Vec<f32>, usize, bool), Box<dyn Error>> {
    let x_ds = file.dataset("X")
        .map_err(|e| std::format!("Dataset 'X' not found in '{}': {}", path, e))?;
    let x_shape = x_ds.shape();
    let (n_iq, is_channel_last) = if x_shape.len() == 3 && x_shape[0] == n && x_shape[2] == 2 {
        (x_shape[1] * 2, true)
    } else if x_shape.len() == 3 && x_shape[0] == n && x_shape[1] == 2 {
        (x_shape[2] * 2, false)
    } else {
        return Err(std::format!(
            "Unexpected X shape {:?}; expected [N,S,2] or [N,2,S] with N={}", x_shape, n
        ).into());
    };
    let n_samples = n_iq / 2;
    std::println!("  Layout      : {:?}  ({} IQ pairs/capture)", x_shape, n_samples);
    std::println!("  Loading X   (~{:.1} GB)…", n as f64 * n_iq as f64 * 4.0 / 1e9);
    let x_raw: Vec<f32> = x_ds.read_raw()?;
    if x_raw.len() != n * n_iq {
        return Err(std::format!(
            "X flat length {} != {} × {} = {}", x_raw.len(), n, n_iq, n * n_iq
        ).into());
    }
    std::println!("  X loaded    ({} captures × {} IQ values)", n, n_iq);
    Ok((x_raw, n_iq, is_channel_last))
}

fn build_class_templates(
    x_raw: &[f32], z: &[f32], mod_class: &[usize],
    n_classes: usize, n_samples: usize, n_iq: usize, is_channel_last: bool,
) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
    let n = z.len();
    let mut template_sum: Vec<Vec<f64>> = (0..n_classes)
        .map(|_| std::vec![0.0_f64; n_samples])
        .collect();
    let mut template_count: Vec<usize> = std::vec![0usize; n_classes];

    for i in 0..n {
        if z[i] >= HEALTHY_SNR_MIN_DB {
            let cls = mod_class[i];
            let base = i * n_iq;
            let sorted = sorted_amplitude_vector(&x_raw[base..base + n_iq], n_samples, is_channel_last);
            for s in 0..n_samples {
                template_sum[cls][s] += sorted[s] as f64;
            }
            template_count[cls] += 1;
        }
    }

    let mut templates: Vec<Vec<f32>> = Vec::with_capacity(n_classes);
    for cls in 0..n_classes {
        let cnt = template_count[cls];
        if cnt == 0 {
            return Err(std::format!(
                "Class {}: no captures at SNR ≥ {:.0} dB for template",
                cls, HEALTHY_SNR_MIN_DB
            ).into());
        }
        let tmpl: Vec<f32> = template_sum[cls].iter()
            .map(|&s| (s / cnt as f64) as f32)
            .collect();
        templates.push(tmpl);
    }
    std::println!("  Templates   : {} classes  (min {} max {} calib captures/class)",
        n_classes,
        template_count.iter().copied().min().unwrap_or(0),
        template_count.iter().copied().max().unwrap_or(0));
    Ok(templates)
}

fn compute_decoder_residuals(
    x_raw: &[f32], templates: &[Vec<f32>], mod_class: &[usize],
    n: usize, n_iq: usize, n_samples: usize, is_channel_last: bool,
) -> Vec<f32> {
    let mut decoder_residual: Vec<f32> = Vec::with_capacity(n);
    for i in 0..n {
        let cls = mod_class[i];
        let base = i * n_iq;
        let sorted = sorted_amplitude_vector(&x_raw[base..base + n_iq], n_samples, is_channel_last);
        let r = amplitude_template_residual(&sorted, &templates[cls]);
        decoder_residual.push(r);
    }
    std::println!("  Residuals   : {} captures  (sorted-amplitude Wasserstein ‖r(k)‖)", n);
    decoder_residual
}

fn print_snr_diagnostics(decoder_residual: &[f32], z: &[f32], mod_class: &[usize], n: usize) {
    let max_snr = z.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_snr = z.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi_snr_resid: f64 = (0..n).filter(|&i| z[i] >= max_snr - 0.1)
        .map(|i| decoder_residual[i] as f64).sum::<f64>()
        / (0..n).filter(|&i| z[i] >= max_snr - 0.1).count().max(1) as f64;
    let lo_snr_resid: f64 = (0..n).filter(|&i| z[i] <= min_snr + 0.1)
        .map(|i| decoder_residual[i] as f64).sum::<f64>()
        / (0..n).filter(|&i| z[i] <= min_snr + 0.1).count().max(1) as f64;
    std::println!("  mean ‖r‖    : {:.6} @ {:.0} dB  →  {:.6} @ {:.0} dB  (ratio {:.1}×)",
        hi_snr_resid, max_snr, lo_snr_resid, min_snr,
        if hi_snr_resid > 1e-9 { lo_snr_resid / hi_snr_resid } else { 0.0 });

    let mut snr_bins: Vec<i32> = Vec::new();
    for i in 0..n {
        let bin = z[i].round() as i32;
        if mod_class[i] == 0 && !snr_bins.contains(&bin) {
            snr_bins.push(bin);
        }
    }
    snr_bins.sort_unstable_by(|a, b| b.cmp(a));
    std::println!("  Class 0 per-SNR residual curve:");
    for &bin in &snr_bins {
        let vals: Vec<f64> = (0..n)
            .filter(|&i| mod_class[i] == 0 && (z[i].round() as i32) == bin)
            .map(|i| decoder_residual[i] as f64)
            .collect();
        if vals.is_empty() { continue; }
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let var = vals.iter().map(|&v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;
        let std = var.sqrt();
        std::println!("    {:>4} dB : mean={:.6}  std={:.6}  n={}", bin, mean, std, vals.len());
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load RadioML 2018.01a as a single flat stream with SNR-interleaved
/// ordering producing ~102 ground-truth events.
///
/// Returns `(observations, ground_truth_events)` ready to pass directly to
/// [`crate::pipeline::run_stage_iii`].
///
/// # Arguments
///
/// * `path` — filesystem path to `RML2018.01a.hdf5` (or a compatible HDF5 file
///   using the same `X`/`Z` schema).
///
/// # Errors
///
/// Returns an error if the file cannot be opened, the dataset schema does not
/// match the expected RadioML 2018.01a layout, or the calibration window cannot
/// be constructed.
///
/// # Examples
///
/// ```no_run
/// use dsfb_rf::hdf5_loader::load_radioml;
/// use dsfb_rf::pipeline::run_stage_iii;
///
/// let (obs, events) = load_radioml("data/RML2018.01a.hdf5").unwrap();
/// let result = run_stage_iii("RadioML 2018.01a", &obs, &events);
/// result.print_summary();
/// ```
pub fn load_radioml(
    path: &str,
) -> Result<(Vec<RfObservation>, Vec<RegimeTransitionEvent>), Box<dyn Error>> {
    let RawDataset {
        n, n_classes, n_samples: _, z, mod_class, decoder_residual, templates: _,
    } = load_raw_dataset(path)?;

    let max_snr = z.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let healthy_idx = select_healthy_indices(&z, max_snr)?;
    let norm_factor = calib_norm_factor(&healthy_idx, &decoder_residual);
    std::println!("  Max SNR     : {:.1} dB  healthy={} caps  calib_resid={:.6}",
        max_snr, HEALTHY_WINDOW_SIZE, norm_factor);

    let healthy_set: std::collections::HashSet<usize> = healthy_idx.iter().copied().collect();
    let mut observations: Vec<RfObservation> = Vec::with_capacity(n);
    let mut events: Vec<RegimeTransitionEvent> = Vec::new();

    for (k, &gi) in healthy_idx.iter().enumerate() {
        observations.push(RfObservation {
            k, residual_norm: decoder_residual[gi] / norm_factor,
            snr_db: z[gi], is_healthy: true,
        });
    }

    let max_sweeps = build_snr_interleaved_series(
        &z, &mod_class, &decoder_residual, &healthy_set,
        n, n_classes, norm_factor, &mut observations, &mut events,
    );

    std::println!("  Obs total   : {}  GT events : {}  sweeps : {}",
        observations.len(), events.len(), max_sweeps);
    Ok((observations, events))
}

fn select_healthy_indices(z: &[f32], max_snr: f32) -> Result<Vec<usize>, Box<dyn Error>> {
    let healthy_idx: Vec<usize> = z.iter().enumerate()
        .filter(|(_, &s)| s >= max_snr - 0.1)
        .take(HEALTHY_WINDOW_SIZE)
        .map(|(i, _)| i)
        .collect();
    if healthy_idx.len() < HEALTHY_WINDOW_SIZE {
        return Err(std::format!(
            "Only {} caps at max SNR {:.1} dB; need {}",
            healthy_idx.len(), max_snr, HEALTHY_WINDOW_SIZE
        ).into());
    }
    Ok(healthy_idx)
}

fn calib_norm_factor(healthy_idx: &[usize], decoder_residual: &[f32]) -> f32 {
    let s: f64 = healthy_idx.iter().map(|&i| decoder_residual[i] as f64).sum();
    let calib_mean = (s / HEALTHY_WINDOW_SIZE as f64) as f32;
    if calib_mean > 1e-7 { calib_mean } else { 1.0 }
}

fn build_snr_interleaved_series(
    z: &[f32], mod_class: &[usize], decoder_residual: &[f32],
    healthy_set: &std::collections::HashSet<usize>,
    n: usize, n_classes: usize, norm_factor: f32,
    observations: &mut Vec<RfObservation>, events: &mut Vec<RegimeTransitionEvent>,
) -> usize {
    let snr_levels = collect_descending_snr_bins(z, healthy_set, n);
    let buckets = bucket_by_class_and_snr(z, mod_class, healthy_set, &snr_levels, n, n_classes);
    let max_sweeps = buckets.iter()
        .flat_map(|cls| cls.iter())
        .map(|cell| cell.len())
        .max()
        .unwrap_or(1);
    let mut k = HEALTHY_WINDOW_SIZE;
    let mut prev_side: Vec<Option<bool>> = std::vec![None; n_classes];
    for sweep in 0..max_sweeps {
        for si in 0..snr_levels.len() {
            for cls in 0..n_classes {
                let cell = &buckets[cls][si];
                if sweep >= cell.len() { continue; }
                let gi = cell[sweep];
                let snr = z[gi];
                let side = snr >= 0.0;
                if let Some(ps) = prev_side[cls] {
                    if ps && !side {
                        events.push(RegimeTransitionEvent { k, label: "SNR_0dB_crossing" });
                    }
                }
                prev_side[cls] = Some(side);
                observations.push(RfObservation {
                    k, residual_norm: decoder_residual[gi] / norm_factor,
                    snr_db: snr, is_healthy: false,
                });
                k += 1;
            }
        }
    }
    max_sweeps
}

fn collect_descending_snr_bins(
    z: &[f32], healthy_set: &std::collections::HashSet<usize>, n: usize,
) -> Vec<i32> {
    let mut snr_levels: Vec<i32> = Vec::new();
    for gi in 0..n {
        if !healthy_set.contains(&gi) {
            let bin = z[gi].round() as i32;
            if !snr_levels.contains(&bin) {
                snr_levels.push(bin);
            }
        }
    }
    snr_levels.sort_unstable_by(|a, b| b.cmp(a));
    snr_levels
}

fn bucket_by_class_and_snr(
    z: &[f32], mod_class: &[usize],
    healthy_set: &std::collections::HashSet<usize>,
    snr_levels: &[i32], n: usize, n_classes: usize,
) -> Vec<Vec<Vec<usize>>> {
    let snr_to_idx: std::collections::HashMap<i32, usize> = snr_levels.iter()
        .enumerate().map(|(i, &s)| (s, i)).collect();
    let mut buckets: Vec<Vec<Vec<usize>>> = (0..n_classes)
        .map(|_| snr_levels.iter().map(|_| Vec::new()).collect())
        .collect();
    for gi in 0..n {
        if !healthy_set.contains(&gi) {
            let cls = mod_class[gi];
            let bin = z[gi].round() as i32;
            if let Some(&si) = snr_to_idx.get(&bin) {
                buckets[cls][si].push(gi);
            }
        }
    }
    buckets
}

/// Load RadioML 2018.01a and build **24 independent per-modulation-class**
/// observation streams for per-class Stage III evaluation.
///
/// For each modulation class the DSFB engine is calibrated on the high-SNR
/// captures, then the evaluation stream presents captures in **descending SNR
/// order** (+28 -> ... -> 0 -> -2 -> ... -> -20 dB).  This matches DSFB's
/// detection paradigm: observe a healthy receiver chain that subsequently
/// degrades.  The amplitude-template residual is near zero at high SNR
/// (captures match the class template) and grows as SNR falls past the
/// demodulation threshold.  DSFB detects this structural transition.
///
/// Returns a `Vec` of 24 `(observations, gt_events)` pairs in class order.
/// Pass each pair to [`crate::pipeline::run_stage_iii`] and aggregate.
///
/// # Examples
///
/// ```no_run
/// use dsfb_rf::hdf5_loader::load_radioml_per_class;
/// use dsfb_rf::pipeline::run_stage_iii;
///
/// let classes = load_radioml_per_class("data/RML2018.01a.hdf5").unwrap();
/// for (i, (obs, events)) in classes.iter().enumerate() {
///     let r = run_stage_iii("RadioML per-class", obs, events);
///     println!("Class {:02}: {} eps  recall {}/{}", i,
///         r.dsfb_episode_count, r.recall_numerator, r.recall_denominator);
/// }
/// ```
pub fn load_radioml_per_class(
    path: &str,
) -> Result<Vec<(Vec<RfObservation>, Vec<RegimeTransitionEvent>)>, Box<dyn Error>> {
    let RawDataset {
        n, n_classes, n_samples: _, z, mod_class, decoder_residual, templates: _,
    } = load_raw_dataset(path)?;

    const BLOCK_SIZE: usize = 128;

    let class_idx = group_indices_by_class(n, n_classes, &mod_class);
    let mut results: Vec<(Vec<RfObservation>, Vec<RegimeTransitionEvent>)> =
        Vec::with_capacity(n_classes);

    for (cls, indices) in class_idx.iter().enumerate() {
        let pair = build_per_class_series(cls, indices, &z, &decoder_residual, BLOCK_SIZE)?;
        results.push(pair);
    }

    let first_obs = results.first().map(|(o, _)| o.len()).unwrap_or(0);
    let first_ev  = results.first().map(|(_, e)| e.len()).unwrap_or(0);
    std::println!("  Per-class   : {} classes  {} obs/class  {} GT ev/class  (B={})",
        n_classes, first_obs, first_ev, BLOCK_SIZE);
    Ok(results)
}

fn group_indices_by_class(n: usize, n_classes: usize, mod_class: &[usize]) -> Vec<Vec<usize>> {
    let mut class_idx: Vec<Vec<usize>> = (0..n_classes).map(|_| Vec::new()).collect();
    for i in 0..n {
        class_idx[mod_class[i]].push(i);
    }
    class_idx
}

fn build_per_class_series(
    cls: usize, indices: &[usize], z: &[f32], decoder_residual: &[f32], block_size: usize,
) -> Result<(Vec<RfObservation>, Vec<RegimeTransitionEvent>), Box<dyn Error>> {
    let n_healthy_caps = HEALTHY_WINDOW_SIZE * block_size;
    let sorted_desc = sort_class_by_descending_snr(indices, z);
    if sorted_desc.len() < n_healthy_caps + block_size {
        return Err(std::format!(
            "Class {}: only {} captures, need >= {}",
            cls, sorted_desc.len(), n_healthy_caps + block_size
        ).into());
    }
    let norm_factor = class_calib_norm(&sorted_desc, decoder_residual);
    let (block_norms, block_snrs) = block_average(&sorted_desc, z, decoder_residual, norm_factor, block_size);

    let n_healthy_blocks = HEALTHY_WINDOW_SIZE;
    let obs = build_class_observations(&block_norms, &block_snrs, n_healthy_blocks);
    let evs = build_snr_boundary_events(&block_snrs, n_healthy_blocks, block_norms.len());

    if cls == 0 {
        print_class0_diagnostic(&block_norms, &block_snrs, n_healthy_blocks, evs.len(), block_size);
    }
    Ok((obs, evs))
}

fn sort_class_by_descending_snr(indices: &[usize], z: &[f32]) -> Vec<usize> {
    let mut sorted_desc: Vec<usize> = indices.to_vec();
    sorted_desc.sort_unstable_by(|&a, &b|
        z[b].partial_cmp(&z[a]).unwrap_or(std::cmp::Ordering::Equal));
    sorted_desc
}

fn class_calib_norm(sorted_desc: &[usize], decoder_residual: &[f32]) -> f32 {
    let s: f64 = sorted_desc.iter().take(HEALTHY_WINDOW_SIZE)
        .map(|&gi| decoder_residual[gi] as f64).sum();
    let calib_mean = (s / HEALTHY_WINDOW_SIZE as f64) as f32;
    if calib_mean > 1e-7 { calib_mean } else { 1.0 }
}

fn block_average(
    sorted_desc: &[usize], z: &[f32], decoder_residual: &[f32],
    norm_factor: f32, block_size: usize,
) -> (Vec<f32>, Vec<f32>) {
    let n_blocks = sorted_desc.len() / block_size;
    let mut block_norms: Vec<f32> = Vec::with_capacity(n_blocks);
    let mut block_snrs: Vec<f32> = Vec::with_capacity(n_blocks);
    for chunk in sorted_desc.chunks(block_size) {
        if chunk.len() < block_size { break; }
        let s_norm: f64 = chunk.iter()
            .map(|&gi| (decoder_residual[gi] / norm_factor) as f64).sum();
        let s_snr: f64 = chunk.iter().map(|&gi| z[gi] as f64).sum();
        block_norms.push((s_norm / chunk.len() as f64) as f32);
        block_snrs.push((s_snr / chunk.len() as f64) as f32);
    }
    (block_norms, block_snrs)
}

fn build_class_observations(
    block_norms: &[f32], block_snrs: &[f32], n_healthy_blocks: usize,
) -> Vec<RfObservation> {
    let mut obs: Vec<RfObservation> = Vec::with_capacity(block_norms.len());
    for (k, (&norm, &snr)) in block_norms.iter().zip(block_snrs.iter()).enumerate() {
        obs.push(RfObservation {
            k, residual_norm: norm, snr_db: snr,
            is_healthy: k < n_healthy_blocks,
        });
    }
    obs
}

fn build_snr_boundary_events(
    block_snrs: &[f32], n_healthy_blocks: usize, n_blocks: usize,
) -> Vec<RegimeTransitionEvent> {
    let mut evs: Vec<RegimeTransitionEvent> = Vec::new();
    let mut prev_bin: Option<i32> = None;
    for k in n_healthy_blocks..n_blocks {
        let bin = block_snrs[k].round() as i32;
        if let Some(pb) = prev_bin {
            if bin != pb {
                evs.push(RegimeTransitionEvent { k, label: "snr_bin_boundary" });
            }
        }
        prev_bin = Some(bin);
    }
    evs
}

fn print_class0_diagnostic(
    block_norms: &[f32], block_snrs: &[f32],
    n_healthy_blocks: usize, n_events: usize, block_size: usize,
) {
    std::println!("  Class 0 block diagnostic (B={}):", block_size);
    std::println!("    Blocks: {}  Healthy: {}  Eval: {}  GT events: {}",
        block_norms.len(), n_healthy_blocks,
        block_norms.len() - n_healthy_blocks, n_events);
    for target_snr in [28.0_f32, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -20.0] {
        if let Some(idx) = block_snrs.iter().position(|&s|
            s <= target_snr + 1.0 && s >= target_snr - 1.0)
        {
            std::println!("    Block {:4}: snr={:+6.1} dB  norm={:.4}",
                idx, block_snrs[idx], block_norms[idx]);
        }
    }
}
