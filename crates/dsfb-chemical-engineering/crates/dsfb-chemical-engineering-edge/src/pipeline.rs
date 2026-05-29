//! End-to-end DSFB-Chemical-Engineering analysis pipeline for one dataset.
//!
//! ## Data-flow summary
//!
//! ```text
//! DataMatrix (n_samples × n_vars)
//!   │
//!   ├─ Baseline::fit(n_base rows)        → per-variable mean + std (baseline window only)
//!   ├─ standardise every row             → z-scored matrix z[n_samples × n_vars]
//!   ├─ PcaModel::fit(z[..n_base])        → loadings, score_var  (prior-art: PCA)
//!   │   └─ select_components             → retain k PCs covering ≥ variance_target
//!   ├─ pca_statistics(pca, z)            → t2[n_samples], spe[n_samples]
//!   │
//!   ├─ build_bank(m, n_base)             → Vec<Box<dyn Detector>>  (detector atlas)
//!   │   └─ for each detector:
//!   │       det.run(ctx)                 → Vec<DetectorOutput>  (raw scores + breach flags)
//!   │       run_detector_dsfb(outputs)   → DetectorTimeline     (DSFB grammar per step)
//!   │
//!   ├─ fuse(timelines, config)           → Vec<FusedEpisode>    (quorum: ≥K families fire)
//!   ├─ canonical_bank() + label_episode  → Vec<HeuristicLabel>  (honest "unknown" when unmatched)
//!   ├─ compression(timelines, fused)     → CompressionReport    (raw breach steps → episodes)
//!   │
//!   ├─ Metrics computation               → detection delay, baseline FP rate, unknown rate, …
//!   ├─ canonical_replay_hash (×2)        → SHA-256 digest; compared for replay_deterministic flag
//!   └─ AnalysisResult
//! ```
//!
//! ## Novel vs prior-art boundary
//!
//! PCA (T²/SPE), CUSUM, EWMA, and all other estimators in the detector bank are **prior art**.
//! The DSFB grammar (drift/slew vs admissibility envelope → 8 grammar states), quorum fusion
//! (≥K families fire → episode; consensus strength; Shannon disagreement entropy), the heuristics
//! bank, and the deterministic replay hash are the novel contributions of this work.
//!
//! ## Determinism
//!
//! `analyze` is deterministic given fixed `(m, n_base, cfg)`:
//! - `build_bank` and `timelines` use no RNG;
//! - iteration order is determined by the bank slice order (fixed at construction);
//! - `fuse` uses sorted indices, not HashMaps;
//! - `canonical_replay_hash` quantises floats to a 1 ns grid before hashing, making the digest
//!   independent of benign platform float-formatting differences.
//!
//! The `replay_deterministic` flag in [`Metrics`] is set by comparing two independent hash calls
//! on the same data within the same invocation; a mismatch would indicate a non-determinism bug.

use crate::data::{Baseline, DataMatrix};
use crate::detectors::{build_bank, DetectorContext, DetectorOutput};
use crate::fusion::{
    compression, fuse, run_detector_dsfb, run_detector_dsfb_regime, CompressionReport,
    DetectorTimeline, FusedEpisode, FusionConfig,
};
use crate::hashing::CanonicalHasher;
use crate::heuristics::{canonical_bank, label_episode, HeuristicLabel};
use crate::linalg::PcaModel;
use serde::{Deserialize, Serialize};

/// Operator-facing metrics for one dataset run.
///
/// All fields are reported honestly:
/// - `unknown_rate` is a deliberate first-class honesty signal — a higher value means the heuristic
///   bank chose not to name a pattern rather than force a speculative label.
/// - `episode_compression_ratio` measures dimensionality reduction from raw breach steps to fused
///   episodes; it does **not** imply improved detection accuracy.
/// - `baseline_false_positive_rate` is a proxy: fraction of baseline-window samples covered by at
///   least one fused episode. A non-zero value in the baseline window is a potential false alarm.
/// - `detection_delay` is `Some` only when the dataset carries a labelled `fault_onset`; negative
///   means the episode started before the annotated onset (early; not necessarily wrong).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total number of samples in the dataset.
    pub n_samples: usize,
    /// Number of process variables (columns).
    pub n_vars: usize,
    /// Number of samples used as the nominal baseline window.
    pub n_baseline: usize,
    /// Number of detectors actually executed (excludes catalogue-only entries).
    pub n_executed_detectors: usize,
    /// Total sample-steps across all detectors where a threshold breach was flagged.
    pub raw_breach_steps: usize,
    /// Number of fused structural episodes produced by quorum fusion.
    pub fused_episodes: usize,
    /// raw_breach_steps / fused episode sample-steps; >1 means compression occurred.
    pub episode_compression_ratio: f64,
    /// Episodes that received at least one heuristic label from the chemical heuristics bank.
    pub labeled_episodes: usize,
    /// Episodes for which no heuristic matched (unknown; evidence is preserved in the episode).
    pub unknown_episodes: usize,
    /// Fraction of fused episodes that received a heuristic label (explanation coverage).
    pub explanation_coverage: f64,
    /// Fraction of fused episodes deliberately left unlabelled (honesty signal).
    pub unknown_rate: f64,
    /// Detection delay in samples from labelled fault onset to first fused episode (if onset known).
    pub detection_delay: Option<i64>,
    /// Fraction of baseline-window samples covered by a fused episode (proxy false-positive rate).
    pub baseline_false_positive_rate: f64,
    /// True iff two independent hash calls over the *same in-memory* result produced the same digest
    /// — i.e. the `CanonicalHasher` is stable. This is a smoke check, NOT a cross-run / cross-machine
    /// pipeline-determinism proof; that guarantee comes from the frozen golden replay-hash gate
    /// (`tests/golden_replay.rs`, which pins each synthetic dataset's `replay_hash`) plus `verify-replay`.
    pub replay_deterministic: bool,
}

/// Full analysis result for one dataset, serialisable to JSON for artifact manifests.
///
/// `fused_episodes` and `heuristic_labels` are parallel slices: `heuristic_labels[i]` describes
/// `fused_episodes[i]`. `replay_hash` is the hex SHA-256 of the canonical grammar+episode stream
/// (see [`canonical_replay_hash`]); it is the machine-verifiable certificate for reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Dataset name (file stem for CSV slices; descriptive label for synthetic sets).
    pub dataset: String,
    /// Provenance tag: "measured/slice" for committed CSVs, "synthetic" for generated data.
    pub data_kind: String,
    pub metrics: Metrics,
    /// Fused structural episodes from quorum fusion (one per detected anomaly window).
    pub fused_episodes: Vec<FusedEpisode>,
    /// Heuristic labels parallel to `fused_episodes`; unmatched episodes carry `matched = false`.
    pub heuristic_labels: Vec<HeuristicLabel>,
    pub compression: CompressionReport,
    /// Hex SHA-256 replay certificate (byte-stable; same inputs → same hash every run).
    pub replay_hash: String,
    /// PCA components retained for T²/SPE (chosen to cover `variance_target` of score variance).
    pub pca_components: usize,
}

/// Tunable parameters for a pipeline run.
///
/// All defaults are deliberately conservative to avoid over-detection on small datasets.
/// `Copy` is intentional: configs are passed by value into closures without clone noise.
#[derive(Debug, Clone, Copy)]
pub struct PipelineConfig {
    /// Look-back window (in samples) for the DSFB drift estimator in each detector timeline.
    /// Larger windows smooth transient noise at the cost of detection latency.
    pub drift_window: usize,
    /// Quorum-fusion parameters (minimum firing families, etc.); see [`FusionConfig`].
    pub fusion: FusionConfig,
    /// Hard upper bound on PCA components to fit; further capped by `n_vars-1` and `n_base-1`
    /// to avoid rank-deficiency. The actual retained count is chosen by `variance_target`.
    pub max_pca_components: usize,
    /// Cumulative score variance fraction to cover when selecting the number of PCs.
    /// 0.90 means retain enough components to explain ≥90 % of the baseline score variance.
    pub variance_target: f64,
    /// Opt-in regime-conditioned admissibility envelopes. When true **and** the dataset carries
    /// per-sample regime/phase labels (`DataMatrix::phase`), each detector is run through
    /// `fusion::run_detector_dsfb_regime` (per-regime calibrated envelope) instead of the single
    /// global envelope. Default `false` reproduces the single-envelope pipeline byte-for-byte;
    /// datasets without a phase column are unaffected even when this is `true`.
    pub regime_conditioned: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        // drift_window = 15: roughly one minute at 1-sample/4 s (TEP convention).
        // max_pca_components = 15: generous ceiling; actual count is driven by variance_target.
        // variance_target = 0.90: standard choice in process-chemometrics literature.
        PipelineConfig {
            drift_window: 15,
            fusion: FusionConfig::default(),
            max_pca_components: 15,
            variance_target: 0.90,
            regime_conditioned: false,
        }
    }
}

/// Compute per-sample Hotelling T² and SPE (Q statistic) for every row of the z-scored matrix.
///
/// T² measures how far the sample's PCA scores lie from the multivariate baseline centre (in-plane
/// distance). SPE measures how much of the sample is unexplained by the retained PCs (out-of-plane
/// residual energy). Together they form the standard two-statistic monitoring chart of process PCA.
///
/// `pca` here is the *truncated* model (k retained components), so SPE is non-trivial (it captures
/// variance in the discarded components). If k == n_vars the SPE would be identically zero.
fn pca_statistics(pca: &PcaModel, z: &[Vec<f64>]) -> (Vec<f64>, Vec<f64>) {
    let t2: Vec<f64> = z.iter().map(|row| pca.t2(row)).collect();
    let spe: Vec<f64> = z.iter().map(|row| pca.spe(row)).collect();
    (t2, spe)
}

/// Choose how many PCA components to retain so that cumulative score variance / total variance ≥ `target`.
///
/// Because the input matrix is z-scored (unit variance per variable), the total variance equals
/// `n_vars` and serves as the normalising denominator. The loop walks components in decreasing
/// eigenvalue order (as supplied by `PcaModel::fit`) and stops at the first component where the
/// cumulative fraction reaches the target. Returns at least 1 component regardless of target.
/// Falls back to `full.k` (all fitted components) if the target is never reached — this can happen
/// when `max_pca_components` < number needed to reach the target.
fn select_components(full: &PcaModel, n_vars: usize, target: f64) -> usize {
    let total = n_vars as f64; // standardised: each variable contributes unit variance
    let mut cum = 0.0;
    for (i, v) in full.score_var.iter().enumerate() {
        cum += v;
        if cum / total >= target {
            return (i + 1).max(1);
        }
    }
    full.k.max(1)
}

/// Run the full pipeline on one dataset and return an [`AnalysisResult`].
///
/// # Arguments
/// - `name`   – human-readable label used in CSV/JSON outputs.
/// - `kind`   – provenance tag ("measured/slice" or "synthetic").
/// - `m`      – the full `n_samples × n_vars` process matrix.
/// - `n_base` – number of leading rows to treat as the nominal baseline window.
///   Clamped to `[2, n_samples]` so arithmetic below is always valid.
/// - `cfg`    – pipeline tuning parameters.
///
/// # Side-effects
/// None. The function is read-only with respect to `m` and produces a fresh [`AnalysisResult`].
///
/// # Examples
/// ```
/// use dsfb_chemical_engineering_edge::{analyze, DataMatrix, PipelineConfig};
/// // A tiny two-variable matrix: 16 baseline rows + a sustained step on var 0.
/// let rows: Vec<Vec<f64>> = (0..32)
///     .map(|i| vec![if i >= 16 { 5.0 } else { 0.0 }, (i % 3) as f64])
///     .collect();
/// let m = DataMatrix::new(vec!["temp".into(), "flow".into()], rows);
/// let res = analyze("demo", "synthetic", &m, 16, PipelineConfig::default());
/// // The result is sealed by a deterministic 64-hex replay hash, read-only over `m`.
/// assert_eq!(res.replay_hash.len(), 64);
/// assert!(res.metrics.replay_deterministic);
/// // Re-running identical inputs reproduces the identical seal.
/// let again = analyze("demo", "synthetic", &m, 16, PipelineConfig::default());
/// assert_eq!(res.replay_hash, again.replay_hash);
/// ```
pub fn analyze(
    name: &str,
    kind: &str,
    m: &DataMatrix,
    n_base: usize,
    cfg: PipelineConfig,
) -> AnalysisResult {
    // Degenerate guard (library-surface hole, since `analyze` is `pub`): the baseline + the
    // difference/lag-based detectors need at least two samples. With `m.n_samples < 2` the
    // `n_base.clamp(2, m.n_samples)` below would itself panic (`clamp` requires min <= max, and
    // `2 > 1`), and `m.n_samples - 1` later would underflow. Return a well-formed *empty* result
    // instead so a truncated/empty CSV (or a caller passing a 0/1-row matrix) degrades gracefully
    // rather than aborting the whole demo. The empty result is replay-stable: `canonical_replay_hash`
    // over empty inputs is well-defined, and this path is never taken by the synthetic/golden suites
    // (all of which have many rows), so frozen replay hashes are unaffected.
    if m.n_samples < 2 {
        return AnalysisResult {
            dataset: name.to_string(),
            data_kind: kind.to_string(),
            metrics: Metrics {
                n_samples: m.n_samples,
                n_vars: m.n_vars,
                n_baseline: m.n_samples,
                n_executed_detectors: 0,
                raw_breach_steps: 0,
                fused_episodes: 0,
                episode_compression_ratio: 0.0,
                labeled_episodes: 0,
                unknown_episodes: 0,
                explanation_coverage: 0.0,
                unknown_rate: 0.0,
                // An annotated onset that can never be reached by an episode is a miss, matching the
                // `find(...).None => i64::MAX` semantics of the normal path below.
                detection_delay: m.fault_onset.map(|_| i64::MAX),
                baseline_false_positive_rate: 0.0,
                replay_deterministic: true,
            },
            fused_episodes: Vec::new(),
            heuristic_labels: Vec::new(),
            compression: CompressionReport {
                raw_breach_steps: 0,
                fused_episodes: 0,
                compression_ratio: 0.0,
            },
            replay_hash: canonical_replay_hash(&[], &[]),
            pca_components: 0,
        };
    }

    // Guard: ensure n_base is at least 2 (for meaningful statistics) and at most n_samples.
    let n_base = n_base.clamp(2, m.n_samples).min(m.n_samples);

    // Step 1: Fit baseline statistics (mean, std) on the first n_base rows only.
    let baseline = Baseline::fit(m, n_base);

    // Step 2: Z-score every sample (all n_samples rows) against the baseline statistics.
    // Downstream detectors and PCA work on the standardised matrix z, not raw process values.
    let z: Vec<Vec<f64>> = (0..m.n_samples)
        .map(|i| baseline.standardise(m.row(i)))
        .collect();
    let zbase: Vec<Vec<f64>> = z[..n_base].to_vec();

    // Step 3: Fit PCA (k_max), then truncate to the variance target so SPE is meaningful.
    // k_max is capped at n_vars-1 (to keep at least one residual component for SPE) and at
    // n_base-1 (can't have more PCs than baseline samples minus one).
    let k_max = cfg
        .max_pca_components
        .min(m.n_vars.saturating_sub(1).max(1))
        .min(n_base.saturating_sub(1).max(1));
    let full = PcaModel::fit(&zbase, m.n_vars, k_max, 60);
    let k = select_components(&full, m.n_vars, cfg.variance_target).min(full.k);
    // Build the truncated model: slice the first k loadings and score variances from the full fit.
    let pca = PcaModel {
        n_vars: full.n_vars,
        k,
        loadings: full.loadings[..k].to_vec(),
        score_var: full.score_var[..k].to_vec(),
    };
    let (t2, spe) = pca_statistics(&pca, &z);

    // Bundle references into a shared context struct so each detector can access what it needs
    // without re-computing standardisation or PCA scores.
    let ctx = DetectorContext {
        m,
        n_base,
        baseline: &baseline,
        pca: &pca,
        z: &z,
        t2: &t2,
        spe: &spe,
    };

    // Step 4: Run the detector atlas.
    // Each detector receives the shared context, emits one DetectorOutput per sample, and those
    // outputs are immediately passed through the DSFB grammar to produce a DetectorTimeline (the
    // per-sample grammar-state sequence for that detector). Iteration is over a deterministic
    // slice, so timeline ordering is stable across runs.
    let bank = build_bank(m, n_base);
    let n_exec = bank.len();
    // Regime-conditioned envelopes (opt-in): enabled only when the flag is set AND the dataset
    // carries per-sample regime/phase labels. Datasets without a phase column take the global
    // single-envelope path even when the flag is on, so their sealed replay hashes are preserved.
    let regimes: Option<&[u32]> = if cfg.regime_conditioned {
        m.phase.as_deref()
    } else {
        None
    };
    let timelines: Vec<DetectorTimeline> = bank
        .iter()
        .map(|det| {
            let outputs: Vec<DetectorOutput> = det.run(&ctx);
            match regimes {
                Some(rg) => run_detector_dsfb_regime(
                    outputs,
                    det.family(),
                    det.id(),
                    cfg.drift_window,
                    rg,
                    n_base,
                ),
                None => run_detector_dsfb(outputs, det.family(), det.id(), cfg.drift_window),
            }
        })
        .collect();

    // Step 5: Quorum fusion → heuristics bank.
    // `fuse` groups timelines by family, applies the quorum threshold (≥K families must fire), and
    // merges overlapping breach windows into FusedEpisodes with consensus strength and Shannon
    // disagreement entropy.
    // `canonical_bank` returns the read-only chemical heuristics bank; `label_episode` attempts to
    // match each episode to a named chemical fault pattern and returns "unknown" (matched=false)
    // when no rule fires — this is the intended failure mode rather than a forced guess.
    let fused = fuse(&timelines, cfg.fusion);
    let hbank = canonical_bank();
    let labels: Vec<HeuristicLabel> = fused.iter().map(|e| label_episode(e, &hbank)).collect();
    let comp = compression(&timelines, &fused);

    // ── Step 6: Metrics ───────────────────────────────────────────────────────
    let labeled = labels.iter().filter(|l| l.matched).count();
    let unknown = labels.len() - labeled;

    // Detection delay: signed offset (samples) from fault onset to the start of the first episode
    // that ends at or after the onset. Negative → early detection; i64::MAX → missed detection.
    let detection_delay =
        m.fault_onset
            .map(|onset| match fused.iter().find(|e| e.end_index >= onset) {
                Some(e) => e.start_index as i64 - onset as i64,
                None => i64::MAX,
            });

    // Baseline false-positive proxy: count baseline samples covered by any fused episode.
    // This is a proxy, not a calibrated FPR; it is reported as-is, without statistical
    // significance claims, because the baseline window may be small.
    let mut covered = vec![false; m.n_samples];
    for e in &fused {
        // `start_index <= end_index` by construction; the upper bound is clamped to `n_samples-1`, so
        // the inclusive slice is always in bounds.
        covered[e.start_index..=e.end_index.min(m.n_samples - 1)].fill(true);
    }
    let base_fp = covered[..n_base].iter().filter(|&&c| c).count() as f64 / n_base as f64;

    // Step 7: Deterministic replay verification.
    // Hash the same data twice in the same call; if the crate is deterministic the two hex strings
    // must be identical. A mismatch here would be a correctness bug in the pipeline.
    let replay_hash = canonical_replay_hash(&timelines, &fused);
    let replay_hash2 = canonical_replay_hash(&timelines, &fused);

    let metrics = Metrics {
        n_samples: m.n_samples,
        n_vars: m.n_vars,
        n_baseline: n_base,
        n_executed_detectors: n_exec,
        raw_breach_steps: comp.raw_breach_steps,
        fused_episodes: fused.len(),
        episode_compression_ratio: comp.compression_ratio,
        labeled_episodes: labeled,
        unknown_episodes: unknown,
        explanation_coverage: if !labels.is_empty() {
            labeled as f64 / labels.len() as f64
        } else {
            0.0
        },
        unknown_rate: if !labels.is_empty() {
            unknown as f64 / labels.len() as f64
        } else {
            0.0
        },
        detection_delay,
        baseline_false_positive_rate: base_fp,
        replay_deterministic: replay_hash == replay_hash2,
    };

    AnalysisResult {
        dataset: name.to_string(),
        data_kind: kind.to_string(),
        metrics,
        fused_episodes: fused,
        heuristic_labels: labels,
        compression: comp,
        replay_hash,
        pca_components: k,
    }
}

/// Compute a byte-stable SHA-256 replay certificate over the complete grammar-state stream and
/// fused episode list.
///
/// ## What is hashed (in order)
/// 1. A version tag (`"dsfb-chemical-engineering-edge" / "v1"`) so future format changes produce
///    a distinct digest without requiring explicit version negotiation.
/// 2. For every `DetectorTimeline` (in the order they appear in the slice — must match `build_bank`
///    order):
///    - The `detector_id` string.
///    - For every step: grammar state (as `u64`), then r, delta, sigma quantised to a 1 ns grid.
/// 3. For every `FusedEpisode`: start/end indices, dominant motif, consensus strength, Shannon
///    disagreement entropy — all quantised the same way.
///
/// ## Float quantisation
/// Floats are rounded to `round(v * 1e9)` as `i64` before hashing. This means differences
/// smaller than 1e-9 are collapsed, which is intentional: they are within platform float noise
/// and should not cause replay mismatches across compilers or CPU microarchitectures.
///
/// ## Ordering contract
/// `timelines` must be in the same insertion order produced by `build_bank`. Inserting or
/// re-ordering detectors will change the hash. This is correct behaviour — it makes the hash
/// sensitive to changes in the detector bank configuration.
pub fn canonical_replay_hash(timelines: &[DetectorTimeline], fused: &[FusedEpisode]) -> String {
    let mut h = CanonicalHasher::new();
    // Version sentinel: changing the pipeline format bumps this tag, invalidating old hashes.
    h.field("dsfb-chemical-engineering-edge", b"v1");
    for t in timelines {
        h.field("detector", t.detector_id.as_bytes());
        for s in &t.steps {
            h.u64("state", s.state as u64);
            h.f64q("r", s.triple.r);
            h.f64q("delta", s.triple.delta);
            h.f64q("sigma", s.triple.sigma);
        }
    }
    for e in fused {
        h.u64("ep_start", e.start_index as u64);
        h.u64("ep_end", e.end_index as u64);
        h.u64("motif", e.dominant_motif as u64);
        h.f64q("consensus", e.consensus_strength);
        h.f64q("entropy", e.disagreement_entropy);
    }
    h.finalize_hex()
}

/// Flatten all per-detector outputs across all timelines into a single borrowed slice.
///
/// Useful for CSV export callers that iterate every (detector, step) pair without caring about
/// which timeline each output came from. The order is: all outputs for timeline[0], then
/// timeline[1], etc. — matching the order produced by `build_bank`.
pub fn flatten_detector_outputs(timelines: &[DetectorTimeline]) -> Vec<&DetectorOutput> {
    timelines.iter().flat_map(|t| t.outputs.iter()).collect()
}

/// Re-run the baseline + PCA + detector stages and return the raw per-detector timelines.
///
/// This is a convenience entry point used in `cli::run_demo` to obtain the timelines separately
/// from the `AnalysisResult` (which does not expose them to keep the serialisable surface clean).
/// The computation is identical to the first half of [`analyze`]; the `name` parameter is unused
/// here but kept for call-site symmetry.
///
/// NOTE: This re-runs the full pipeline up to (but not including) fusion, so calling both
/// `analyze` and `timelines_for` on the same dataset does double the computation. This is an
/// accepted cost to keep the two public surfaces independent and simple.
pub fn timelines_for(
    name: &str,
    m: &DataMatrix,
    n_base: usize,
    cfg: PipelineConfig,
) -> Vec<DetectorTimeline> {
    let _ = name; // parameter kept for call-site symmetry; not used in computation
                  // Same degenerate guard as `analyze`: with < 2 samples the `clamp(2, ..)` below would panic and
                  // there is no baseline to fit, so there are no timelines to produce.
    if m.n_samples < 2 {
        return Vec::new();
    }
    let n_base = n_base.clamp(2, m.n_samples).min(m.n_samples);
    let baseline = Baseline::fit(m, n_base);
    let z: Vec<Vec<f64>> = (0..m.n_samples)
        .map(|i| baseline.standardise(m.row(i)))
        .collect();
    let zbase: Vec<Vec<f64>> = z[..n_base].to_vec();
    let k_max = cfg
        .max_pca_components
        .min(m.n_vars.saturating_sub(1).max(1))
        .min(n_base.saturating_sub(1).max(1));
    let full = PcaModel::fit(&zbase, m.n_vars, k_max, 60);
    let k = select_components(&full, m.n_vars, cfg.variance_target).min(full.k);
    let pca = PcaModel {
        n_vars: full.n_vars,
        k,
        loadings: full.loadings[..k].to_vec(),
        score_var: full.score_var[..k].to_vec(),
    };
    let (t2, spe) = pca_statistics(&pca, &z);
    let ctx = DetectorContext {
        m,
        n_base,
        baseline: &baseline,
        pca: &pca,
        z: &z,
        t2: &t2,
        spe: &spe,
    };
    let regimes: Option<&[u32]> = if cfg.regime_conditioned {
        m.phase.as_deref()
    } else {
        None
    };
    build_bank(m, n_base)
        .iter()
        .map(|det| match regimes {
            Some(rg) => run_detector_dsfb_regime(
                det.run(&ctx),
                det.family(),
                det.id(),
                cfg.drift_window,
                rg,
                n_base,
            ),
            None => run_detector_dsfb(det.run(&ctx), det.family(), det.id(), cfg.drift_window),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataMatrix;

    /// Regression guard for the degenerate-matrix hole: `analyze` and `timelines_for` are `pub`, so a
    /// caller (or a truncated/empty CSV) can hand them a 0- or 1-row matrix. Before P38 this panicked
    /// in `n_base.clamp(2, n_samples)` (min > max). Both must now return a well-formed empty result.
    #[test]
    fn degenerate_matrices_do_not_panic() {
        let cfg = PipelineConfig::default();
        for rows in [Vec::<Vec<f64>>::new(), vec![vec![1.0, 2.0, 3.0]]] {
            let n = rows.len();
            let m = DataMatrix::new(vec!["a".into(), "b".into(), "c".into()], rows);
            let r = analyze("degenerate", "synthetic", &m, 8, cfg);
            assert_eq!(r.metrics.n_samples, n);
            assert_eq!(r.metrics.fused_episodes, 0);
            assert_eq!(r.metrics.n_executed_detectors, 0);
            assert!(r.fused_episodes.is_empty());
            assert!(r.metrics.replay_deterministic);
            assert!(
                !r.replay_hash.is_empty(),
                "empty inputs still produce a well-formed replay hash"
            );
            assert!(timelines_for("degenerate", &m, 8, cfg).is_empty());
        }
    }
}
