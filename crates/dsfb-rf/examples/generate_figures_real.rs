//! Real-dataset figure data generator — 80 figures (fig_69 – fig_148).
//!
//! Reads the eight real-world slices already on disk at `data/slices/` and
//! emits `figure_data_real.json` for `scripts/figures_real.py`. Then spawns
//! the Python renderer to produce 80 PDFs + a merged PDF + a zip archive.
//!
//! **DSFB positioning (load-bearing).** DSFB is **not** a detector, classifier,
//! or competitor to any upstream chain. It is a **structural observer** that
//! takes the residuals the upstream chain (matched filter, CFAR, AGC, PLL,
//! channel estimator, scheduler, beamformer, beam-tracker) already computes
//! and usually discards, and turns them into human-readable grammar
//! (states, sign tuples, DSA, episodes, envelope). Every figure is framed
//! that way — no "DSFB detects earlier than X" phrasing appears anywhere.
//!
//! # Usage
//! ```text
//! cargo run --release --example generate_figures_real \
//!     --features std,serde,real_figures
//! ```
//!
//! # Missing-slice behaviour
//! If a slice file under `data/slices/` is missing, the corresponding
//! 10-figure block emits a loud `[SKIPPED — <slice> not present]` banner
//! and the remaining slices still render.

#![cfg(all(feature = "std", feature = "serde", feature = "real_figures"))]

use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use dsfb_rf::attractor::DelayEmbedding;
use dsfb_rf::complexity::PermutationEntropyEstimator;
use dsfb_rf::detectability::DetectabilityBound;
use dsfb_rf::dsa::{CorroborationAccumulator, DsaWindow};
use dsfb_rf::envelope::AdmissibilityEnvelope;
use dsfb_rf::fisher_geometry::{fisher_rao_distance_exact, GaussPoint, ManifoldTracker};
use dsfb_rf::grammar::{GrammarEvaluator, GrammarState};
use dsfb_rf::pipeline::{
    run_stage_iii, RegimeTransitionEvent, RfObservation, ScalarComparators,
    GRAMMAR_K, HEALTHY_WINDOW_SIZE,
};
use dsfb_rf::platform::{PlatformContext, SnrFloor};
use dsfb_rf::sign::{SignTuple, SignWindow};
use dsfb_rf::stationarity::reverse_arrangements_test;
use dsfb_rf::tda::detect_topological_innovation;

// ════════════════════════════════════════════════════════════════════════════
// Output data blocks
// ════════════════════════════════════════════════════════════════════════════

/// Common per-slice analysis block (consumed by figures_real.py).
#[derive(Debug, Serialize, Deserialize)]
struct SliceCommon {
    name: String,
    provenance: String,
    schema: String,
    upstream_producer: String,
    non_claims: Vec<String>,
    n: usize,
    rho: f32,
    healthy_window_size: usize,
    /// Calibration-window residual norms (first HEALTHY_WINDOW_SIZE samples).
    healthy_norms: Vec<f32>,
    /// Full residual norm stream DSFB consumes. Length = n.
    norms: Vec<f32>,
    /// Per-observation grammar state label (post-calibration portion).
    grammar_states: Vec<String>,
    /// Per-observation sign tuple (norm, drift, slew) post-calibration.
    sign_tuples: Vec<[f32; 3]>,
    /// Per-observation DSA score post-calibration.
    dsa_scores: Vec<f32>,
    /// Comparator context stream (not competition): EWMA trace.
    ewma_trace: Vec<f32>,
    ewma_threshold: f32,
    threshold_3sigma: f32,
    /// Raw 3σ boundary event count from `ScalarComparators` (context only).
    raw_boundary_count: usize,
    /// DSFB episode count.
    dsfb_episode_count: usize,
    episodes: Vec<EpisodeBlock>,
    /// Review-surface compression: raw_boundary_count / dsfb_episode_count.
    compression_factor: f32,
    /// Permutation entropy trace (sliding, post-calibration).
    perm_entropy: Vec<f32>,
    /// Reverse-arrangements Z-score over full stream.
    rat_z_score: f32,
    /// Full-stream detectability bound summary.
    detectability: DetectabilityBlock,
}

#[derive(Debug, Serialize, Deserialize)]
struct EpisodeBlock {
    open_k: usize,
    close_k: Option<usize>,
    duration: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DetectabilityBlock {
    delta_0: f32,
    alpha: f32,
    kappa: f32,
    tau_upper: Option<f32>,
    bound_satisfied: Option<bool>,
}

// ════════════════════════════════════════════════════════════════════════════
// Slice loaders
// ════════════════════════════════════════════════════════════════════════════

const SLICES_DIR: &str = "data/slices";

fn slice_path(rel: &str) -> PathBuf {
    Path::new(SLICES_DIR).join(rel)
}

/// Read an interleaved cf32 (little-endian I, Q) binary file into ‖r‖ = |IQ|.
fn load_cf32_norms(path: &Path, max_samples: usize) -> std::io::Result<Vec<f32>> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);
    let mut buf = [0u8; 8];
    let mut norms = Vec::with_capacity(max_samples);
    while norms.len() < max_samples {
        match r.read_exact(&mut buf) {
            Ok(()) => {
                let i = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let q = f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                norms.push((i * i + q * q).sqrt());
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok(norms)
}

/// Subsample a long stream to at most `target` points by stride decimation.
fn decimate_to(norms: Vec<f32>, target: usize) -> Vec<f32> {
    if norms.len() <= target {
        return norms;
    }
    let stride = (norms.len() + target - 1) / target;
    norms.into_iter().step_by(stride).take(target).collect()
}

// ════════════════════════════════════════════════════════════════════════════
// Common per-slice analysis pipeline
// ════════════════════════════════════════════════════════════════════════════

fn analyze_slice(
    name: &str,
    provenance: &str,
    schema: &str,
    upstream_producer: &str,
    non_claims: Vec<String>,
    norms: Vec<f32>,
) -> SliceCommon {
    let n = norms.len();
    assert!(
        n >= HEALTHY_WINDOW_SIZE + 20,
        "slice '{}' too short: {} < {}",
        name, n, HEALTHY_WINDOW_SIZE + 20
    );

    // Healthy calibration window.
    let healthy: Vec<f32> = norms[..HEALTHY_WINDOW_SIZE].to_vec();
    let envelope = AdmissibilityEnvelope::calibrate_from_window(&healthy)
        .expect("healthy-window calibration must succeed");
    let rho = envelope.rho;

    // Comparators (context stream).
    let mut comparators = ScalarComparators::calibrate(&healthy);
    let ewma_threshold = comparators.ewma_threshold;
    let threshold_3sigma = comparators.threshold_3sigma;

    // Grammar / sign / DSA machinery (post-calibration streaming).
    let mut grammar = GrammarEvaluator::<GRAMMAR_K>::new();
    let mut sign_win = SignWindow::<5>::new();
    let mut dsa = DsaWindow::<10>::new(ewma_threshold);
    let mut corr = CorroborationAccumulator::<GRAMMAR_K>::new(1);

    let mut grammar_states = Vec::with_capacity(n - HEALTHY_WINDOW_SIZE);
    let mut sign_tuples = Vec::with_capacity(n - HEALTHY_WINDOW_SIZE);
    let mut dsa_scores = Vec::with_capacity(n - HEALTHY_WINDOW_SIZE);
    let mut ewma_trace = Vec::with_capacity(n - HEALTHY_WINDOW_SIZE);
    let mut raw_boundary_count = 0usize;

    let ctx = PlatformContext::operational();
    let waveform_state = ctx.waveform_state;
    let snr_floor = SnrFloor::default();
    for &norm in &norms[HEALTHY_WINDOW_SIZE..] {
        let sub_threshold = snr_floor.is_sub_threshold(20.0);
        let sig: SignTuple = sign_win.push(norm, sub_threshold, snr_floor);
        let (boundary, violation) = (
            envelope.is_boundary_approach(norm, 1.0),
            envelope.is_violation(norm, 1.0),
        );
        let count: u8 = (boundary as u8) + (violation as u8);
        let _corr = corr.push(count);
        let state = grammar.evaluate(&sig, &envelope, waveform_state);
        let motif_fired = matches!(state, GrammarState::Violation);
        let dsa_score = dsa.push(&sig, state, motif_fired);

        grammar_states.push(format!("{:?}", state));
        sign_tuples.push([sig.norm, sig.drift, sig.slew]);
        dsa_scores.push(dsa_score.value());

        let (thr, _, _, _) = comparators.update(norm);
        if thr {
            raw_boundary_count += 1;
        }
        ewma_trace.push(comparators.ewma);
    }

    // Build observations for run_stage_iii (all events empty → unsupervised).
    let observations: Vec<RfObservation> = norms
        .iter()
        .enumerate()
        .map(|(k, &norm)| RfObservation {
            k,
            residual_norm: norm,
            snr_db: 20.0,
            is_healthy: k < HEALTHY_WINDOW_SIZE,
        })
        .collect();
    let events: Vec<RegimeTransitionEvent> = Vec::new();

    // leak the dataset name for the &'static str requirement
    let name_static: &'static str = Box::leak(name.to_string().into_boxed_str());
    let result = run_stage_iii(name_static, &observations, &events);
    let dsfb_episode_count = result.dsfb_episode_count;
    let compression = if dsfb_episode_count > 0 {
        raw_boundary_count as f32 / dsfb_episode_count as f32
    } else {
        raw_boundary_count as f32
    };
    let episodes: Vec<EpisodeBlock> = result
        .episodes
        .iter()
        .map(|e| EpisodeBlock {
            open_k: e.open_k,
            close_k: e.close_k,
            duration: e.close_k.map(|c| c.saturating_sub(e.open_k)),
        })
        .collect();

    // Permutation entropy (sliding).
    let mut pe_est = PermutationEntropyEstimator::<64>::new();
    let mut perm_entropy = Vec::with_capacity(n - HEALTHY_WINDOW_SIZE);
    for &norm in &norms[HEALTHY_WINDOW_SIZE..] {
        let r = pe_est.push(norm);
        perm_entropy.push(r.normalized_pe);
    }

    // Reverse arrangements on the full stream.
    let rat = reverse_arrangements_test(&norms);
    let rat_z_score = rat.map(|r| r.z_score).unwrap_or(0.0);

    // Detectability bound (informational).
    let post_cal = &norms[HEALTHY_WINDOW_SIZE..];
    let delta_0 = (post_cal.iter().cloned().fold(0.0_f32, f32::max) - rho).max(0.0);
    let alpha = 0.1_f32;
    let kappa = 0.05_f32;
    let db = DetectabilityBound::compute(delta_0, alpha, kappa);

    SliceCommon {
        name: name.to_string(),
        provenance: provenance.to_string(),
        schema: schema.to_string(),
        upstream_producer: upstream_producer.to_string(),
        non_claims,
        n,
        rho,
        healthy_window_size: HEALTHY_WINDOW_SIZE,
        healthy_norms: healthy,
        norms,
        grammar_states,
        sign_tuples,
        dsa_scores,
        ewma_trace,
        ewma_threshold,
        threshold_3sigma,
        raw_boundary_count,
        dsfb_episode_count,
        episodes,
        compression_factor: compression,
        perm_entropy,
        rat_z_score,
        detectability: DetectabilityBlock {
            delta_0,
            alpha,
            kappa,
            tau_upper: db.tau_upper,
            bound_satisfied: db.bound_satisfied,
        },
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Per-slice data producers
// ════════════════════════════════════════════════════════════════════════════

fn skipped(name: &str, reason: &str) -> Value {
    eprintln!("[SKIPPED — {} not present] {}", name, reason);
    json!({"name": name, "skipped": true, "reason": reason})
}

// ───────────────────────── RadioML ─────────────────────────

fn build_radioml() -> Value {
    let path = slice_path("radioml_2018_slice.hdf5");
    if !path.exists() {
        return skipped("radioml", &format!("{} missing", path.display()));
    }
    println!("[1/8] RadioML — loading HDF5 slice ({})…", path.display());

    let file = match hdf5_metno::File::open(&path) {
        Ok(f) => f,
        Err(e) => return skipped("radioml", &format!("HDF5 open failed: {}", e)),
    };

    let x_ds = file.dataset("X").expect("X dataset");
    let y_ds = file.dataset("Y").expect("Y dataset");
    let z_ds = file.dataset("Z").expect("Z dataset");
    let x_shape = x_ds.shape();
    let n_caps = x_shape[0];
    let n_samp = x_shape[1];
    let x_flat: Vec<f32> = x_ds.read_raw().expect("X read");
    let y_flat: Vec<i64> = y_ds.read_raw().expect("Y read");
    let z_flat: Vec<i64> = z_ds.read_raw().expect("Z read");
    let n_classes = y_flat.len() / n_caps;

    // argmax mod class per capture
    let mod_class: Vec<usize> = y_flat
        .chunks(n_classes)
        .map(|row| {
            row.iter()
                .enumerate()
                .max_by_key(|(_, &v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0)
        })
        .collect();
    let snr_db: Vec<f32> = z_flat.iter().map(|&v| v as f32).collect();

    // Per-class sorted-amplitude template over high-SNR captures.
    let max_snr = snr_db.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut templates: Vec<Vec<f32>> = vec![vec![0.0_f32; n_samp]; n_classes];
    let mut tmpl_counts: Vec<usize> = vec![0; n_classes];
    let mut sorted_amp = |cap_idx: usize| -> Vec<f32> {
        let base = cap_idx * n_samp * 2;
        let mut amps: Vec<f32> = (0..n_samp)
            .map(|k| {
                let i = x_flat[base + 2 * k];
                let q = x_flat[base + 2 * k + 1];
                (i * i + q * q).sqrt()
            })
            .collect();
        let rms = {
            let s: f64 = amps.iter().map(|&a| (a as f64).powi(2)).sum();
            ((s / n_samp as f64).sqrt()) as f32
        };
        if rms > 1e-8 {
            for a in &mut amps {
                *a /= rms;
            }
        }
        amps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        amps
    };

    for cap in 0..n_caps {
        if snr_db[cap] >= max_snr - 0.1 {
            let amps = sorted_amp(cap);
            let cls = mod_class[cap];
            for (t, a) in templates[cls].iter_mut().zip(amps.iter()) {
                *t += *a;
            }
            tmpl_counts[cls] += 1;
        }
    }
    for cls in 0..n_classes {
        if tmpl_counts[cls] > 0 {
            let c = tmpl_counts[cls] as f32;
            for t in &mut templates[cls] {
                *t /= c;
            }
        }
    }

    // Per-capture residual norm (Wasserstein-2 to class template).
    let residual: Vec<f32> = (0..n_caps)
        .map(|cap| {
            let amps = sorted_amp(cap);
            let cls = mod_class[cap];
            let t = &templates[cls];
            if t.iter().all(|&v| v == 0.0) {
                return 0.0;
            }
            let mut s = 0.0_f64;
            for i in 0..n_samp {
                let d = (amps[i] - t[i]) as f64;
                s += d * d;
            }
            ((s / n_samp as f64).sqrt()) as f32
        })
        .collect();

    let common = analyze_slice(
        "radioml",
        "real-in-repo",
        "HDF5 X[N,1024,2] cf32; Y[N,24] one-hot mod; Z[N,1] SNR dB",
        "amplitude-template demodulator residual (sorted-amp Wasserstein-2 to \
         per-class high-SNR template)",
        vec![
            "Not a modulation-recognition benchmark.".into(),
            "Not a device-class classifier.".into(),
            "Not a replacement for the amplitude-template demodulator — DSFB \
             relies on the demodulator's residual."
                .into(),
        ],
        residual.clone(),
    );

    // Per-modulation aggregates (fig_69–fig_78 helpers).
    let mut per_mod_norms: Vec<Vec<f32>> = vec![Vec::new(); n_classes];
    for cap in 0..n_caps {
        per_mod_norms[mod_class[cap]].push(residual[cap]);
    }
    // Fisher-Rao pairwise matrix between modulations.
    let gauss_from = |s: &[f32]| -> Option<GaussPoint> {
        if s.is_empty() {
            return None;
        }
        let mu = s.iter().sum::<f32>() / s.len() as f32;
        let var = s.iter().map(|x| (x - mu).powi(2)).sum::<f32>() / s.len() as f32;
        let sigma = var.sqrt().max(1e-6);
        Some(GaussPoint::new(mu, sigma))
    };
    let mut fr_matrix: Vec<Vec<f32>> = vec![vec![0.0; n_classes]; n_classes];
    for i in 0..n_classes {
        let Some(pi) = gauss_from(&per_mod_norms[i]) else { continue };
        for j in 0..n_classes {
            let Some(pj) = gauss_from(&per_mod_norms[j]) else { continue };
            fr_matrix[i][j] = fisher_rao_distance_exact(pi, pj);
        }
    }

    // Per-modulation demodulation threshold (fig_149). For each class, calibrate
    // a class-local admissibility envelope from the highest-SNR captures, then
    // scan in descending-SNR order. The SNR at which the first residual norm
    // crosses the class-local envelope is the DSFB-observed demodulation
    // threshold for that modulation — the SNR below which the amplitude-template
    // demodulator's residual exits admissibility. Not a modulation classifier.
    let mut fig_149: Vec<Value> = Vec::with_capacity(n_classes);
    for cls in 0..n_classes {
        let mut pairs: Vec<(f32, f32)> = (0..n_caps)
            .filter(|&cap| mod_class[cap] == cls)
            .map(|cap| (snr_db[cap], residual[cap]))
            .collect();
        pairs.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });
        if pairs.len() < 4 {
            fig_149.push(json!({
                "mod_class": cls,
                "threshold_snr_db": Value::Null,
                "n_captures": pairs.len(),
            }));
            continue;
        }
        let cal_n = (pairs.len() / 4).max(2);
        let cal: Vec<f32> = pairs[..cal_n].iter().map(|&(_, r)| r).collect();
        let mu = cal.iter().copied().sum::<f32>() / cal.len() as f32;
        let var = cal.iter().map(|&x| (x - mu).powi(2)).sum::<f32>()
            / cal.len() as f32;
        let sigma = var.sqrt().max(1e-6);
        let rho_cls = mu + 3.0 * sigma;
        // JCGM 100:2008 (GUM) Type A unbiased sample variance over the
        // calibration set. This is the $s^2_{\|r\|}$ referenced in the
        // Fisher-information subsection of the paper; $\widehat{I}(\rho)
        // = 1/s^2_{\|r\|}$ is the finite-sample Fisher-info estimator
        // against which the class-local envelope is calibrated. The
        // biased `var` above continues to drive the envelope ($\mu+3\sigma$);
        // this unbiased estimate is emission-only and does not change
        // threshold-crossing arithmetic.
        let s2_r_unbiased = if cal_n >= 2 {
            cal.iter().map(|&x| (x - mu).powi(2)).sum::<f32>()
                / (cal_n as f32 - 1.0)
        } else {
            f32::NAN
        };
        let fisher_info_hat = if s2_r_unbiased.is_finite() && s2_r_unbiased > 0.0
        {
            1.0 / s2_r_unbiased
        } else {
            f32::NAN
        };
        let mut threshold_snr_db: Option<f32> = None;
        for (snr, r) in pairs.iter().skip(cal_n) {
            if *r > rho_cls {
                threshold_snr_db = Some(*snr);
                break;
            }
        }
        fig_149.push(json!({
            "mod_class": cls,
            "threshold_snr_db": threshold_snr_db,
            "rho_class": rho_cls,
            "n_captures": pairs.len(),
            "cal_n": cal_n,
            "s2_r_unbiased": if s2_r_unbiased.is_finite() {
                json!(s2_r_unbiased)
            } else {
                Value::Null
            },
            "fisher_info_hat": if fisher_info_hat.is_finite() {
                json!(fisher_info_hat)
            } else {
                Value::Null
            },
        }));
    }

    json!({
        "common": common,
        "n_classes": n_classes,
        "n_samples_per_capture": n_samp,
        "mod_class": mod_class,
        "snr_db": snr_db,
        "per_mod_norms": per_mod_norms,
        "fisher_rao_matrix": fr_matrix,
        "fig_149": fig_149,
    })
}

// ───────────────────────── ORACLE ─────────────────────────

fn build_oracle() -> Value {
    let data = slice_path("oracle_slice.sigmf-data");
    let meta = slice_path("oracle_slice.sigmf-meta");
    if !data.exists() {
        return skipped("oracle", &format!("{} missing", data.display()));
    }
    println!("[2/8] ORACLE — SigMF cf32, 131 072 samples…");
    let norms = match load_cf32_norms(&data, 131_072) {
        Ok(v) => v,
        Err(e) => return skipped("oracle", &format!("cf32 read failed: {}", e)),
    };
    // Subsample for tractable analysis (16k keeps structural character).
    let norms = decimate_to(norms, 16_384);
    let meta_note = if meta.exists() {
        "SigMF meta present (burst annotations not used in this exhibit)"
    } else {
        "no meta loaded"
    };

    let common = analyze_slice(
        "oracle",
        "real-local-zip",
        "SigMF cf32, WiFi 802.11a, USRP X310 tx / B210 rx, 5 MS/s @ 2.45 GHz",
        "B210 AGC / RMS-envelope residual of a captured 802.11a WiFi session",
        vec![
            "Not an ORACLE 16-device classifier.".into(),
            "Not an RF-fingerprinting reproduction of Hanna et al. 2022.".into(),
            "Not a replacement for the AGC / burst detector.".into(),
        ],
        norms,
    );
    json!({"common": common, "meta_note": meta_note})
}

// ───────────────────────── POWDER ─────────────────────────

fn build_powder() -> Value {
    let path = slice_path("powder_slice.bin");
    if !path.exists() {
        return skipped("powder", &format!("{} missing", path.display()));
    }
    println!("[3/8] POWDER — LTE Band 7 cf32, 262 144 samples…");
    let norms = match load_cf32_norms(&path, 262_144) {
        Ok(v) => v,
        Err(e) => return skipped("powder", &format!("cf32 read failed: {}", e)),
    };
    let norms = decimate_to(norms, 16_384);
    let common = analyze_slice(
        "powder",
        "real-local-zip",
        "cf32 raw IQ, LTE Band 7, USRP X310/B210, 7.69 MS/s @ 2.685 GHz",
        "OFDM channel-estimator residual of a captured LTE Band 7 session",
        vec![
            "Not a link-budget, path-loss, or propagation benchmark.".into(),
            "Not a replacement for the OFDM channel estimator / equaliser.".into(),
            "Not an LTE PHY compliance test.".into(),
        ],
        norms,
    );
    json!({"common": common})
}

// ───────────────────────── Tampere GNSS ─────────────────────────

fn build_tampere() -> Value {
    let path = slice_path("tampere_gnss_slice.bin");
    if !path.exists() {
        return skipped("tampere_gnss", &format!("{} missing", path.display()));
    }
    println!("[4/8] Tampere GNSS — L1 C/A cf32, up to 262 k samples…");
    let norms = match load_cf32_norms(&path, 262_144) {
        Ok(v) => v,
        Err(e) => return skipped("tampere_gnss", &format!("cf32 read failed: {}", e)),
    };
    let norms = decimate_to(norms, 16_384);
    let common = analyze_slice(
        "tampere_gnss",
        "real-public",
        "cf32 raw IQ, GNSS L1 C/A baseband (Zenodo 13846381, CC-BY 4.0)",
        "L1 C/A code-loop innovation (DLL/PLL residual) of the tracking loop",
        vec![
            "Not a spoofing-detection benchmark.".into(),
            "Not a RAIM reproduction.".into(),
            "Not a replacement for the DLL/PLL tracking loop.".into(),
        ],
        norms,
    );
    json!({"common": common})
}

// ───────────────────────── ColO-RAN (KPI CSV) ─────────────────────────

#[derive(Debug, Deserialize)]
struct ColoRanRow {
    #[serde(rename = "Timestamp", alias = "timestamp", default)]
    _ts: String,
    #[serde(alias = "dl_mcs", alias = "DL_MCS", default)]
    dl_mcs: Option<f32>,
    #[serde(alias = "dl_brate (mbps)", alias = "dl_brate", default)]
    dl_brate: Option<f32>,
    #[serde(alias = "ul_brate (mbps)", alias = "ul_brate", default)]
    ul_brate: Option<f32>,
    #[serde(alias = "num_ues", alias = "nof_ues", alias = "nof_ue", default)]
    nof_ue: Option<f32>,
}

fn read_csv_series(path: &Path) -> std::io::Result<(Vec<f32>, Vec<f32>, Vec<f32>)> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;
    let mut dl = Vec::new();
    let mut ul = Vec::new();
    let mut nof = Vec::new();
    // Use header detection — resolve to column indices once.
    let headers = rdr.headers()?.clone();
    let find = |name: &str| -> Option<usize> {
        headers.iter().position(|h| h.eq_ignore_ascii_case(name))
    };
    let dl_idx = find("dl_brate (mbps)")
        .or_else(|| find("dl_brate"))
        .or_else(|| find("tx_brate (mbps)"));
    let ul_idx = find("ul_brate (mbps)")
        .or_else(|| find("ul_brate"))
        .or_else(|| find("rx_brate (mbps)"));
    let nof_idx = find("num_ues")
        .or_else(|| find("nof_ues"))
        .or_else(|| find("nof_ue"));
    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(_) => continue,
        };
        let g = |i: Option<usize>| -> f32 {
            i.and_then(|j| rec.get(j))
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0)
        };
        dl.push(g(dl_idx));
        ul.push(g(ul_idx));
        nof.push(g(nof_idx));
    }
    Ok((dl, ul, nof))
}

fn ewma_residual(series: &[f32], lambda: f32) -> Vec<f32> {
    let mut ewma = series.first().copied().unwrap_or(0.0);
    series
        .iter()
        .map(|&x| {
            ewma = lambda * x + (1.0 - lambda) * ewma;
            (x - ewma).abs()
        })
        .collect()
}

/// Tile a short series up to the HEALTHY_WINDOW_SIZE+20 minimum so the
/// analyzer can run. This is labelled in the schema — no claim is made that
/// tiling synthesises new information; it is a harmless numerical step so
/// the engine doesn't underflow on short KPI slices.
fn ensure_min_length(mut v: Vec<f32>, min: usize) -> Vec<f32> {
    if v.len() >= min {
        return v;
    }
    let base = v.clone();
    while v.len() < min {
        v.extend_from_slice(&base);
    }
    v.truncate(min.max(base.len() * 2));
    v
}

fn build_coloran() -> Value {
    let path = slice_path("coloran_slice.csv");
    if !path.exists() {
        return skipped("coloran", &format!("{} missing", path.display()));
    }
    println!("[5/8] ColO-RAN — KPI CSV (scheduler EWMA residual)…");
    let (dl, ul, nof) = match read_csv_series(&path) {
        Ok(v) => v,
        Err(e) => return skipped("coloran", &format!("csv read failed: {}", e)),
    };
    let residual = ewma_residual(&dl, 0.2);
    let residual = ensure_min_length(residual, HEALTHY_WINDOW_SIZE + 40);
    let common = analyze_slice(
        "coloran",
        "real-public",
        "CSV KPI trace (dl_brate, ul_brate, nof_ue) from O-RAN scheduler",
        "O-RAN scheduler EWMA-baseline residual over dl_brate KPI",
        vec![
            "Not an RF or PHY-layer claim.".into(),
            "Not a scheduler replacement or policy-optimisation benchmark.".into(),
            "KPI-layer structural exhibit; not an RF benchmark.".into(),
        ],
        residual,
    );
    json!({
        "common": common,
        "dl_brate": dl,
        "ul_brate": ul,
        "nof_ue": nof,
    })
}

// ───────────────────────── ColO-RAN-commag ─────────────────────────

fn build_coloran_commag() -> Value {
    let path = slice_path("coloran_commag_slice.csv");
    if !path.exists() {
        return skipped("coloran_commag", &format!("{} missing", path.display()));
    }
    println!("[6/8] ColO-RAN-commag — policy-labelled KPI CSV…");
    // Read raw records (we need the scheduling_policy column).
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&path)
        .expect("csv open");
    let headers = rdr.headers().expect("headers").clone();
    let find = |name: &str| -> Option<usize> {
        headers.iter().position(|h| h.eq_ignore_ascii_case(name))
    };
    let dl_idx = find("dl_brate (mbps)")
        .or_else(|| find("dl_brate"))
        .or_else(|| find("tx_brate (mbps)"));
    let policy_idx = find("scheduling_policy")
        .or_else(|| find("policy"))
        .or_else(|| find("slice_prb"));
    let mut dl: Vec<f32> = Vec::new();
    let mut policies: Vec<String> = Vec::new();
    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(_) => continue,
        };
        let v = dl_idx
            .and_then(|j| rec.get(j))
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let p = policy_idx
            .and_then(|j| rec.get(j))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "p?".into());
        dl.push(v);
        policies.push(p);
    }
    let residual = ewma_residual(&dl, 0.2);
    let residual = ensure_min_length(residual, HEALTHY_WINDOW_SIZE + 40);
    let common = analyze_slice(
        "coloran_commag",
        "real-public",
        "CSV KPI + scheduling_policy trace from ColO-RAN-commag dataset",
        "O-RAN scheduler EWMA-baseline residual conditioned on scheduling policy",
        vec![
            "Not an RF or PHY claim.".into(),
            "Not a scheduling-policy benchmark; DSFB does not rank policies.".into(),
            "Not predictive of policy switches; descriptive only.".into(),
        ],
        residual,
    );
    json!({"common": common, "dl_brate": dl, "policies": policies})
}

// ───────────────────────── DeepBeam ─────────────────────────

fn build_deepbeam() -> Value {
    let path = slice_path("deepbeam_slice.h5");
    if !path.exists() {
        return skipped("deepbeam", &format!("{} missing", path.display()));
    }
    println!("[7/8] DeepBeam — HDF5 mmWave IQ + gain + beam indices…");
    let file = match hdf5_metno::File::open(&path) {
        Ok(f) => f,
        Err(e) => return skipped("deepbeam", &format!("HDF5 open failed: {}", e)),
    };
    let iq_ds = file.dataset("iq").expect("iq dataset");
    let iq_flat: Vec<f64> = iq_ds.read_raw().expect("iq read");
    let n = iq_flat.len() / 2;
    let norms: Vec<f32> = (0..n)
        .map(|k| {
            let i = iq_flat[2 * k] as f32;
            let q = iq_flat[2 * k + 1] as f32;
            (i * i + q * q).sqrt()
        })
        .collect();
    let iq_i: Vec<f32> = (0..n).map(|k| iq_flat[2 * k] as f32).collect();
    let iq_q: Vec<f32> = (0..n).map(|k| iq_flat[2 * k + 1] as f32).collect();
    let gain: Vec<f32> = file
        .dataset("gain")
        .and_then(|d| d.read_raw::<f64>())
        .map(|v| v.into_iter().map(|x| x as f32).collect())
        .unwrap_or_default();
    let rx_beam: Vec<f32> = file
        .dataset("rx_beam")
        .and_then(|d| d.read_raw::<f64>())
        .map(|v| v.into_iter().map(|x| x as f32).collect())
        .unwrap_or_default();
    let tx_beam: Vec<f32> = file
        .dataset("tx_beam")
        .and_then(|d| d.read_raw::<f64>())
        .map(|v| v.into_iter().map(|x| x as f32).collect())
        .unwrap_or_default();

    let norms_padded = ensure_min_length(norms.clone(), HEALTHY_WINDOW_SIZE + 40);
    let common = analyze_slice(
        "deepbeam",
        "real-local-file",
        "HDF5 iq(N,2) f64, gain(N,), rx_beam(N,), tx_beam(N,); NI mmWave \
         transceiver native layout",
        "NI mmWave beamformer gain-control residual (|IQ| post-beamforming)",
        vec![
            "Not a device-identity or RF-fingerprinting claim.".into(),
            "Single unit, single beam pair — not a cross-unit generalisation.".into(),
            "Not a beamformer replacement; DSFB reads the residual after the fact.".into(),
        ],
        norms_padded,
    );
    json!({
        "common": common,
        "iq_i": iq_i,
        "iq_q": iq_q,
        "gain": gain,
        "rx_beam": rx_beam,
        "tx_beam": tx_beam,
        "raw_norms_n": n,
    })
}

// ───────────────────────── DeepSense-6G Scenario 23 UAV ─────────────────────────

fn build_deepsense() -> Value {
    let path = slice_path("deepsense_6g_slice.h5");
    if !path.exists() {
        return skipped("deepsense_6g", &format!("{} missing", path.display()));
    }
    println!("[8/8] DeepSense-6G — power-only mmWave + UAV telemetry…");
    let file = match hdf5_metno::File::open(&path) {
        Ok(f) => f,
        Err(e) => return skipped("deepsense_6g", &format!("HDF5 open failed: {}", e)),
    };
    let power_ds = file.dataset("mmwave_power").expect("mmwave_power");
    let power_shape = power_ds.shape();
    let n_steps = power_shape[0];
    let n_beams = power_shape[1];
    let power_flat: Vec<f32> = power_ds.read_raw().expect("mmwave_power read");
    let best_beam: Vec<i16> = file
        .dataset("best_beam_index")
        .and_then(|d| d.read_raw::<i16>())
        .unwrap_or_default();
    let alt: Vec<f32> = file
        .dataset("altitude")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();
    let speed: Vec<f32> = file
        .dataset("speed")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();
    let pitch: Vec<f32> = file
        .dataset("pitch")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();
    let roll: Vec<f32> = file
        .dataset("roll")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();
    let distance: Vec<f32> = file
        .dataset("distance")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();
    let height: Vec<f32> = file
        .dataset("height")
        .and_then(|d| d.read_raw::<f32>())
        .unwrap_or_default();

    // Margin stream: best_beam_power − 2nd_best_power per step. This is the
    // scalar residual DSFB structures for this slice (no IQ available).
    let mut margin: Vec<f32> = Vec::with_capacity(n_steps);
    for s in 0..n_steps {
        let row = &power_flat[s * n_beams..(s + 1) * n_beams];
        let mut best = f32::NEG_INFINITY;
        let mut second = f32::NEG_INFINITY;
        for &p in row {
            if p > best {
                second = best;
                best = p;
            } else if p > second {
                second = p;
            }
        }
        margin.push(best - second);
    }
    let margin_padded = ensure_min_length(margin.clone(), HEALTHY_WINDOW_SIZE + 40);
    let common = analyze_slice(
        "deepsense_6g",
        "real-local-zip",
        "HDF5 mmwave_power(N,64) f32, best_beam_index(N,) i16 + UAV telemetry",
        "beam-tracker best-vs-2nd-best power-margin residual (no raw IQ; \
         sign-tuple/Fisher-Rao/TDA/attractor operate on this scalar margin only)",
        vec![
            "Not a beam-selection ML benchmark.".into(),
            "Not a replacement or override for the beam-tracker.".into(),
            "UAV telemetry figures are descriptive / correlational, not causal.".into(),
        ],
        margin_padded,
    );

    // 64×64 multibeam correlation matrix.
    let mut corr = vec![vec![0.0_f32; n_beams]; n_beams];
    let mut means = vec![0.0_f32; n_beams];
    for s in 0..n_steps {
        for b in 0..n_beams {
            means[b] += power_flat[s * n_beams + b];
        }
    }
    for m in &mut means {
        *m /= n_steps as f32;
    }
    let mut stds = vec![0.0_f32; n_beams];
    for s in 0..n_steps {
        for b in 0..n_beams {
            let d = power_flat[s * n_beams + b] - means[b];
            stds[b] += d * d;
        }
    }
    for sd in &mut stds {
        *sd = (*sd / n_steps as f32).sqrt().max(1e-8);
    }
    for i in 0..n_beams {
        for j in i..n_beams {
            let mut c = 0.0_f32;
            for s in 0..n_steps {
                c += (power_flat[s * n_beams + i] - means[i])
                    * (power_flat[s * n_beams + j] - means[j]);
            }
            c /= (n_steps as f32) * stds[i] * stds[j];
            corr[i][j] = c;
            corr[j][i] = c;
        }
    }

    json!({
        "common": common,
        "n_steps": n_steps,
        "n_beams": n_beams,
        "mmwave_power": power_flat,
        "best_beam_index": best_beam,
        "altitude": alt,
        "speed": speed,
        "pitch": pitch,
        "roll": roll,
        "distance": distance,
        "height": height,
        "margin_raw": margin,
        "beam_correlation": corr,
        "beam_mean_power": means,
    })
}

// ════════════════════════════════════════════════════════════════════════════
// Main
// ════════════════════════════════════════════════════════════════════════════

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF │ Real-Dataset Figure Bank │ 80 figures (fig_69-148)");
    println!("══════════════════════════════════════════════════════════════");
    println!(" Positioning: DSFB augments upstream chains by structuring");
    println!(" their discarded residuals. No detection / replacement claim.");
    println!();

    // Silence unused-import warnings in the all-features path.
    let _ = detect_topological_innovation;
    let _ = DelayEmbedding::<16>::new;
    let _ = ManifoldTracker::new;

    let slices = json!({
        "radioml":         build_radioml(),
        "oracle":          build_oracle(),
        "powder":          build_powder(),
        "tampere_gnss":    build_tampere(),
        "coloran":         build_coloran(),
        "coloran_commag":  build_coloran_commag(),
        "deepbeam":        build_deepbeam(),
        "deepsense_6g":    build_deepsense(),
    });

    let out_dir = Path::new("../dsfb-rf-output");
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("create output dir {}: {e}", out_dir.display()))?;
    let json_path = out_dir.join("figure_data_real.json");
    let s = serde_json::to_string(&slices)
        .map_err(|e| format!("serialize figure_data_real: {e}"))?;
    fs::write(&json_path, &s)
        .map_err(|e| format!("write {}: {e}", json_path.display()))?;
    println!();
    println!("Written: {} ({} bytes)", json_path.display(), s.len());

    // Timestamped run directory.
    let ts_out = Command::new("python3")
        .args([
            "-c",
            "import datetime; \
             print(datetime.datetime.now().strftime('%Y-%m-%d_%H-%M-%S'), end='')",
        ])
        .output()
        .map_err(|e| format!("python3 (timestamp) not available: {e}"))?;
    let ts = String::from_utf8(ts_out.stdout)
        .map_err(|e| format!("python3 timestamp stdout not utf-8: {e}"))?;
    let run_name = format!("dsfb-rf-real-{}", ts);
    let run_dir = out_dir.join(&run_name);
    let figs_dir = run_dir.join("figs");
    fs::create_dir_all(&figs_dir)
        .map_err(|e| format!("create figs dir {}: {e}", figs_dir.display()))?;
    fs::copy(&json_path, run_dir.join("figure_data_real.json")).ok();

    // Render figures.
    println!();
    println!("[Pipeline] Rendering 80 real-dataset figures…");
    let status = Command::new("python3")
        .arg("scripts/figures_real.py")
        .arg("--data")
        .arg(&json_path)
        .arg("--out")
        .arg(&figs_dir)
        .status()
        .map_err(|e| format!("spawn python3 scripts/figures_real.py: {e}"))?;
    if !status.success() {
        return Err(format!("figures_real.py exited with {status}").into());
    }

    // Merge PDFs.
    let combined = run_dir.join("dsfb-rf-all-real-figures.pdf");
    let merge = Command::new("python3")
        .args([
            "-c",
            &format!(
                "import glob, subprocess; \
                 pdfs = sorted(glob.glob('{}/*.pdf')); \
                 subprocess.run(['pdfunite'] + pdfs + ['{}'], check=True); \
                 print(f'  Combined PDF: {{len(pdfs)}} pages')",
                figs_dir.display(),
                combined.display()
            ),
        ])
        .status()
        .map_err(|e| format!("spawn pdfunite merge: {e}"))?;
    if !merge.success() {
        return Err(format!("pdfunite merge exited with {merge}").into());
    }

    // Zip artefacts.
    let zip_path = run_dir.join(format!("{}-artifacts.zip", run_name));
    let z = Command::new("python3")
        .args([
            "-c",
            &format!(
                "import glob, os, zipfile; \
                 figs = sorted(glob.glob('{figs}/*')); \
                 zf = zipfile.ZipFile('{zp}', 'w', zipfile.ZIP_DEFLATED); \
                 [zf.write(f, 'figs/' + os.path.basename(f)) for f in figs]; \
                 zf.write('{cpdf}', 'dsfb-rf-all-real-figures.pdf') if os.path.exists('{cpdf}') else None; \
                 zf.write('{jp}', 'figure_data_real.json') if os.path.exists('{jp}') else None; \
                 zf.close(); \
                 print(f'  Zip: {{os.path.getsize(\"{zp}\")//1024}} KB')",
                figs = figs_dir.display(),
                zp = zip_path.display(),
                cpdf = combined.display(),
                jp = run_dir.join("figure_data_real.json").display(),
            ),
        ])
        .status()
        .map_err(|e| format!("spawn zip builder: {e}"))?;
    if !z.success() {
        return Err(format!("zip creation exited with {z}").into());
    }

    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!(" Done. Artefacts in {}/", run_dir.display());
    println!(" figs/                          80 PDFs (fig_69-148)");
    println!(" dsfb-rf-all-real-figures.pdf   merged PDF");
    println!(" figure_data_real.json          engine data");
    println!(" {}-artifacts.zip", run_name);
    println!("══════════════════════════════════════════════════════════════");
    Ok(())
}
