//! Deterministic quorum fusion over per-detector DSFB grammar timelines.
//!
//! Each detector's `signed_margin` stream is run through the DSFB grammar engine, producing a
//! per-step grammar state. Fusion forms *episodes* using explicit, deterministic quorum rules
//! (not black-box voting): an episode opens when at least `min_families` detector families have a
//! breaching detector and stays open while that holds. Detector disagreement is preserved as a
//! first-class output (Shannon entropy of the per-step grammar-state distribution).
//!
//! # Relationship to the paper (Appendix B)
//!
//! This module implements exactly the quorum episode calculus described in Appendix B of the
//! DSFB-Chemical-Engineering paper:
//!
//! - **Detector fires** at step k: `state_k` is non-nominal and not `SensorFault`.
//! - **Quorum met** at step k: the set of families with at least one firing detector has
//!   cardinality ≥ `min_families` (K).
//! - **Fused episode**: a maximal contiguous run of steps where the quorum is met,
//!   retained only if `step_count ≥ min_steps`.
//! - **Consensus strength**: `(1/L) Σ_k (firing_k / N)` where L = episode length, N = total
//!   detectors. This is the mean fraction of all detectors firing per step (not just those that
//!   ever fire during the episode).
//! - **Disagreement entropy**: `(1/L) Σ_k H_k` where `H_k` is the Shannon entropy of the
//!   grammar-state distribution across detectors at step k (nats, not bits).
//! - **Dominant motif**: most frequently occurring non-nominal, non-SensorFault grammar state
//!   across all (step, detector) pairs inside the episode.
//! - **Compression ratio**: `raw_breach_steps / fused_episodes` (Appendix B Table).
//!
//! # Determinism
//!
//! All per-step aggregates iterate over `timelines` in the order they appear in the slice.
//! `BTreeSet<DetectorFamily>` and `BTreeMap` ensure deterministic family counting and motif
//! accumulation independent of HashMap randomisation. `participating_detectors` and `families`
//! in `FusedEpisode` are collected from `BTreeMap` / `BTreeSet` iteration, so their order is
//! also deterministic (lexicographic by detector id / family label).
//!
//! # Novel contribution
//!
//! The underlying detectors (e.g. CUSUM, PCA, MEWMA) are prior art. The novel contribution
//! here is the quorum fusion layer and the episode calculus it defines — translating per-detector
//! grammar timelines into multi-detector structural episodes with quantified consensus and
//! disagreement metrics.

use crate::detectors::{DetectorFamily, DetectorOutput};
use crate::dsfb_core::{
    aggregate_episodes, calibrate_regime_envelope, AdmissibilityEnvelope, AnnotatedStep,
    DeterministicDsfb, GrammarState, ResidualProcessor, ResidualSample, ResidualTriple,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Complete per-detector DSFB annotation: both the raw detector outputs and their grammar
/// interpretation.
///
/// Bundles the original `DetectorOutput` sequence alongside the per-step `AnnotatedStep`
/// sequence so that both the numeric signal (needed for `compression`) and the grammar
/// timeline (needed for `fuse`) are available without re-running the pipeline.
///
/// One `DetectorTimeline` corresponds to one detector instance on one channel.
pub struct DetectorTimeline {
    /// Unique identifier for this detector instance (e.g. `"CUSUM-reactor_T"`).
    pub detector_id: String,
    /// Detector family (used for quorum counting by family, not by individual detector).
    pub family: DetectorFamily,
    /// DSFB grammar annotation of the `signed_margin` exceedance stream, in time order.
    pub steps: Vec<AnnotatedStep>,
    /// Raw detector outputs, parallel to `steps` (same length, same time ordering).
    pub outputs: Vec<DetectorOutput>,
}

/// Run one detector's margin stream through the DSFB grammar engine, producing a
/// `DetectorTimeline`.
///
/// # One-sided exceedance transform
///
/// Detector breach is one-sided: a `signed_margin ≤ 0` means the statistic is at or below its
/// control limit, which is the nominal state. We transform to:
///
/// ```text
/// exceedance = max(0.0, signed_margin)
/// ```
///
/// so that the residual fed to DSFB is zero during normal operation and grows positive only
/// when the detector breaches. This prevents the envelope's lower bound (`r_min = −0.25`)
/// from being spuriously tripped by well-behaved detectors running below their limit.
///
/// The asymmetric envelope `(-0.25, 1.0, -0.15, 0.6, -0.5, 2.0, grazing=0.15)` is calibrated
/// for this exceedance space:
/// - `r_max = 1.0`: one full control-limit width of exceedance → `EnvViolation`.
/// - `delta_max = 0.6`: sustained partial exceedance → `DriftAccum`.
/// - `sigma_max = 2.0`: fast onset (slew) can exceed r transiently → `SlewSpike`.
/// - Negative bounds are narrow (−0.25, −0.15, −0.5) to allow small numerical fluctuations
///   around zero without misclassification.
///
/// NOTE: these envelope constants are chosen for detector-margin streams, not engineering-unit
/// residuals. Callers feeding different signal types must construct their own envelope.
pub fn run_detector_dsfb(
    outputs: Vec<DetectorOutput>,
    family: DetectorFamily,
    detector_id: &str,
    drift_window: usize,
) -> DetectorTimeline {
    // Asymmetric envelope tuned for exceedance ∈ [0, ∞); r_max = 1 = one limit-width breach.
    let env = AdmissibilityEnvelope::new(-0.25, 1.0, -0.15, 0.6, -0.5, 2.0, 0.15);
    let mut eng = DeterministicDsfb::with_window(env, drift_window, detector_id);
    let mut steps = Vec::with_capacity(outputs.len());
    for o in &outputs {
        // Clamp to zero: negative margin means below-limit (nominal); not an anomaly.
        let exceedance = o.signed_margin.max(0.0);
        // expected = 0.0 so that residual r = exceedance − 0 = exceedance (identity).
        let s = ResidualSample::new(o.time_index as f64, exceedance, 0.0, detector_id);
        steps.push(eng.ingest_sample(&s));
    }
    DetectorTimeline {
        detector_id: detector_id.to_string(),
        family,
        steps,
        outputs,
    }
}

/// Regime-conditioned variant of [`run_detector_dsfb`]: classify each sample against an
/// admissibility envelope calibrated for that sample's operating regime, instead of one global
/// envelope.
///
/// `regimes[i]` is the regime / batch-phase label of sample `i` (from `DataMatrix::phase`); the
/// first `n_base` samples are the baseline window used for calibration. For each regime present in
/// the baseline, the global default envelope is *relaxed* to cover that regime's own baseline
/// variability (`dsfb_core::calibrate_regime_envelope`); regimes with fewer than
/// `MIN_REGIME_BASELINE` baseline samples (or that appear only after the baseline) fall back to the
/// default envelope. This lowers baseline false positives on heterogeneous / non-stationary
/// processes without tightening any regime.
///
/// Determinism: a throwaway processor recomputes the triples for calibration, then a fresh engine
/// reproduces the identical triples (the drift/slew estimators are pure functions of the input
/// order) and classifies each against its regime envelope. Fixed quantile, `BTreeMap` iteration,
/// no RNG. If every `regimes[i]` is the same label this reduces to the single-envelope behaviour of
/// `run_detector_dsfb` (modulo the per-regime relaxation of that one regime's envelope).
pub fn run_detector_dsfb_regime(
    outputs: Vec<DetectorOutput>,
    family: DetectorFamily,
    detector_id: &str,
    drift_window: usize,
    regimes: &[u32],
    n_base: usize,
) -> DetectorTimeline {
    use std::collections::BTreeMap;
    // Same global default as run_detector_dsfb; per-regime envelopes only ever widen it.
    let default_env = AdmissibilityEnvelope::new(-0.25, 1.0, -0.15, 0.6, -0.5, 2.0, 0.15);
    const MIN_REGIME_BASELINE: usize = 8;
    const Q_HIGH: f64 = 0.99;

    // Pre-pass: recompute drift/slew triples on a throwaway processor (no classification yet) and
    // bucket the baseline triples by regime, so each regime's envelope is fixed before any sample
    // is classified. The exceedance transform matches run_detector_dsfb: r = max(0, signed_margin).
    let mut calib_proc = ResidualProcessor::new(drift_window);
    let n_base = n_base.min(outputs.len());
    let mut by_regime: BTreeMap<u32, Vec<ResidualTriple>> = BTreeMap::new();
    for (i, o) in outputs.iter().enumerate() {
        let s = ResidualSample::new(
            o.time_index as f64,
            o.signed_margin.max(0.0),
            0.0,
            detector_id,
        );
        let tr = calib_proc.process(&s);
        if i < n_base {
            by_regime
                .entry(regimes.get(i).copied().unwrap_or(0))
                .or_default()
                .push(tr);
        }
    }
    let envelopes: BTreeMap<u32, AdmissibilityEnvelope> = by_regime
        .iter()
        .map(|(g, tr)| {
            (
                *g,
                calibrate_regime_envelope(tr, default_env, MIN_REGIME_BASELINE, Q_HIGH),
            )
        })
        .collect();

    // Classification pass: a fresh engine reproduces the identical triples and classifies each
    // against its regime envelope (default for uncalibrated / unseen regimes).
    let mut eng = DeterministicDsfb::with_window(default_env, drift_window, detector_id);
    let mut steps = Vec::with_capacity(outputs.len());
    for (i, o) in outputs.iter().enumerate() {
        let s = ResidualSample::new(
            o.time_index as f64,
            o.signed_margin.max(0.0),
            0.0,
            detector_id,
        );
        let env = envelopes
            .get(&regimes.get(i).copied().unwrap_or(0))
            .copied()
            .unwrap_or(default_env);
        steps.push(eng.ingest_sample_env(&s, &env));
    }
    DetectorTimeline {
        detector_id: detector_id.to_string(),
        family,
        steps,
        outputs,
    }
}

/// A fused, cross-detector structural episode: a maximal contiguous time span during which
/// the quorum condition (≥ `min_families` distinct detector families firing) is met.
///
/// This is the primary output unit of the DSFB fusion layer and the main subject of the
/// paper's Appendix B episode calculus. Each field quantifies a different facet of the
/// multi-detector agreement structure within the episode window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedEpisode {
    /// Start time-step index (into the original timeline arrays, 0-based).
    pub start_index: usize,
    /// End time-step index (inclusive).
    pub end_index: usize,
    /// Number of time steps in this episode: `end_index − start_index + 1`.
    pub step_count: usize,
    /// Detector ids that were firing (non-nominal, non-SensorFault) at least once within
    /// `[start_index, end_index]`. Sorted lexicographically (BTreeMap iteration order).
    pub participating_detectors: Vec<String>,
    /// String labels of the detector families represented among `participating_detectors`.
    /// Sorted lexicographically (BTreeSet iteration order).
    pub families: Vec<String>,
    /// Most frequently occurring non-nominal grammar state across all (step, detector) pairs
    /// inside the episode. Falls back to `Nominal` if no non-nominal states are recorded
    /// (should not occur given the quorum condition, but handled gracefully).
    pub dominant_motif: GrammarState,
    /// Mean fraction of all detectors firing per step within the episode.
    ///
    /// Computed as `(Σ_k firing_k/N) / L` where L = `step_count`, N = total detector count,
    /// and `firing_k` = number of detectors firing at step k (non-nominal, non-SensorFault).
    /// A value of 1.0 means every detector fires at every step of the episode.
    pub consensus_strength: f64,
    /// Mean Shannon entropy (nats) of the per-step grammar-state distribution.
    ///
    /// Computed as `(Σ_k H_k) / L` where `H_k = −Σ_s p_s ln(p_s)` over grammar states s
    /// with `p_s = count(state=s at step k) / N`. High entropy indicates detectors disagree
    /// on which grammar state to assign; low entropy indicates consensus on a single state.
    pub disagreement_entropy: f64,
    /// Maximum absolute drift δ observed across any firing detector within the episode.
    pub peak_drift: f64,
    /// Maximum absolute slew σ observed across any firing detector within the episode.
    pub peak_slew: f64,
}

/// Quorum thresholds that govern episode formation in `fuse`.
///
/// Both fields must be ≥ 1 in meaningful deployments. The defaults (K=2, min_steps=2)
/// require at least two distinct detector families to co-breach for at least two consecutive
/// steps, filtering out single-family or single-step coincidences.
#[derive(Debug, Clone, Copy)]
pub struct FusionConfig {
    /// Minimum number of distinct detector *families* (not individual detectors) that must
    /// simultaneously be firing for the quorum to be met at a step. Using families rather
    /// than raw detector counts avoids giving disproportionate weight to families that deploy
    /// many detector instances.
    pub min_families: usize,
    /// Minimum episode length in steps. Episodes shorter than this are silently discarded
    /// after the maximal-run scan. Guards against spurious single-step quorum events.
    pub min_steps: usize,
}

impl Default for FusionConfig {
    fn default() -> Self {
        FusionConfig {
            min_families: 2,
            min_steps: 2,
        }
    }
}

/// Compute Shannon entropy H = −Σ p_s ln(p_s) (nats) from a state-count histogram.
///
/// `counts` maps each grammar state to the number of detectors in that state at one step.
/// The total N = Σ counts is used to normalise to probabilities p_s = count_s / N.
///
/// Returns 0.0 for an empty map (no detectors, or all counts zero). States with count 0 are
/// skipped to avoid `0 * ln(0)`, which is defined as 0 by convention (the limit is 0).
///
/// Iterates over `counts.values()` in BTreeMap key order — deterministic across all calls.
fn shannon_entropy(counts: &BTreeMap<GrammarState, usize>) -> f64 {
    let total: usize = counts.values().sum();
    if total == 0 {
        return 0.0;
    }
    let mut h = 0.0;
    for &c in counts.values() {
        if c > 0 {
            let p = c as f64 / total as f64;
            h -= p * p.ln(); // Natural log → result in nats.
        }
    }
    h
}

/// Form fused structural episodes from per-detector grammar timelines.
///
/// Runs in two conceptual phases:
///
/// ## Phase 1 — Per-step quorum evaluation (lines: "Per-step: …")
///
/// For each time step `i` in `[0, n)` (where `n` is the length of the longest timeline),
/// iterate over all timelines and classify each detector as "firing" (non-nominal,
/// non-SensorFault) or not. Count distinct firing families (`BTreeSet<DetectorFamily>` for
/// deterministic cardinality) and record the fraction of all detectors that are firing
/// (`firing / n_det`). Also accumulate the step's grammar-state distribution into
/// `step_states[i]` (used for entropy in Phase 2). Set `quorum_met[i]` to true if the
/// family count meets `cfg.min_families`.
///
/// Detectors with timelines shorter than `n` steps are simply absent at those steps —
/// the `i < t.steps.len()` guard treats missing steps as non-firing.
///
/// ## Phase 2 — Maximal-run extraction and feature aggregation
///
/// Scan `quorum_met` left-to-right for maximal contiguous true-runs. For each run
/// `[start, end]` with `step_count = end − start + 1 ≥ cfg.min_steps`:
///
/// - Accumulate `frac_firing[s]` into `consensus_acc` and `shannon_entropy(step_states[s])`
///   into `entropy_acc` (summed over steps, divided by `step_count` at the end).
/// - Collect participating detector ids and family labels from firing detectors only (to
///   avoid crediting detectors that were silent during this episode).
/// - Accumulate grammar state counts in `motif_counts` from firing (step, detector) pairs
///   for dominant-motif selection.
/// - Track peak |δ| and |σ| across firing detectors (non-finite values excluded).
///
/// After the aggregation, `dominant_motif` is selected by max count in `motif_counts`.
/// A tie is broken deterministically by `max_by_key` over the ascending-iterating `BTreeMap`:
/// Rust's `max_by_key` returns the *last* maximum, so the tied state with the **largest** `Ord`
/// value wins (consistent with the call site below). NOTE: the tie-break is deterministic but not
/// semantically meaningful.
pub fn fuse(timelines: &[DetectorTimeline], cfg: FusionConfig) -> Vec<FusedEpisode> {
    if timelines.is_empty() {
        return Vec::new();
    }
    // n = number of time steps; align to the longest timeline.
    let n = timelines.iter().map(|t| t.steps.len()).max().unwrap_or(0);
    // n_det used as denominator for per-step firing fraction; never zero here.
    let n_det = timelines.len() as f64;

    // Phase 1: per-step quorum evaluation.
    let mut quorum_met = vec![false; n];
    let mut frac_firing = vec![0.0f64; n];
    let mut step_states: Vec<BTreeMap<GrammarState, usize>> = vec![BTreeMap::new(); n];

    for i in 0..n {
        // BTreeSet ensures family cardinality is counted without duplicate families from
        // multiple detectors of the same family type.
        let mut fams: std::collections::BTreeSet<DetectorFamily> = Default::default();
        let mut firing = 0usize;
        for t in timelines {
            if i < t.steps.len() {
                let st = t.steps[i].state;
                // Record state in the distribution histogram regardless of firing/nominal.
                *step_states[i].entry(st).or_insert(0) += 1;
                // SensorFault is excluded from the quorum: a dead sensor should not count
                // as evidence of a process event.
                if st.is_non_nominal() && st != GrammarState::SensorFault {
                    fams.insert(t.family);
                    firing += 1;
                }
            }
        }
        quorum_met[i] = fams.len() >= cfg.min_families;
        frac_firing[i] = firing as f64 / n_det;
    }

    // Phase 2: maximal-run scan and feature aggregation.
    let mut episodes = Vec::new();
    let mut i = 0;
    while i < n {
        if !quorum_met[i] {
            i += 1;
            continue;
        }
        // Found the start of a quorum-met run. Advance i to find the end.
        let start = i;
        while i < n && quorum_met[i] {
            i += 1;
        }
        let end = i - 1; // i now points one past the run.
        let step_count = end - start + 1;
        // Discard episodes shorter than min_steps (transient coincidences).
        if step_count < cfg.min_steps {
            continue;
        }

        // Aggregate participant features over [start, end].
        // BTreeMap<id, ()> used as an ordered set of detector ids (no duplicate check needed
        // since insert on an existing key is a no-op, and we only care about membership).
        let mut participants: BTreeMap<String, ()> = BTreeMap::new();
        let mut fams: std::collections::BTreeSet<String> = Default::default();
        let mut motif_counts: BTreeMap<GrammarState, usize> = BTreeMap::new();
        let mut consensus_acc = 0.0;
        let mut entropy_acc = 0.0;
        let mut peak_drift = 0.0f64;
        let mut peak_slew = 0.0f64;
        for s in start..=end {
            // Accumulate step-level consensus and entropy contributions.
            consensus_acc += frac_firing[s];
            entropy_acc += shannon_entropy(&step_states[s]);
            for t in timelines {
                if s < t.steps.len() {
                    let st = t.steps[s].state;
                    // Only firing detectors contribute to participant lists and motif counts.
                    if st.is_non_nominal() && st != GrammarState::SensorFault {
                        participants.insert(t.detector_id.clone(), ());
                        fams.insert(t.family.label().to_string());
                        *motif_counts.entry(st).or_insert(0) += 1;
                        let tr = &t.steps[s].triple;
                        // Update peaks from the grammar triple; skip NaN (SensorFault guard
                        // above means NaN should not appear here, but is_finite is defensive).
                        if tr.delta.is_finite() {
                            peak_drift = peak_drift.max(tr.delta.abs());
                        }
                        if tr.sigma.is_finite() {
                            peak_slew = peak_slew.max(tr.sigma.abs());
                        }
                    }
                }
            }
        }
        // Select the most frequently occurring non-nominal state as the dominant motif.
        // `max_by_key` on a BTreeMap iterator visits keys in ascending order; when counts
        // are tied, the state encountered later (higher Ord value) is selected by Rust's
        // max_by_key semantics (last maximum wins). This is deterministic but arbitrary.
        let dominant_motif = motif_counts
            .iter()
            .max_by_key(|(_, &c)| c)
            .map(|(s, _)| *s)
            .unwrap_or(GrammarState::Nominal); // Fallback should not occur post-quorum.
        episodes.push(FusedEpisode {
            start_index: start,
            end_index: end,
            step_count,
            // into_keys() on BTreeMap returns keys in ascending order → deterministic.
            participating_detectors: participants.into_keys().collect(),
            families: fams.into_iter().collect(), // BTreeSet iterator → ascending order.
            dominant_motif,
            // Divide accumulated sums by step_count to get per-step means.
            consensus_strength: consensus_acc / step_count as f64,
            disagreement_entropy: entropy_acc / step_count as f64,
            peak_drift,
            peak_slew,
        });
    }
    episodes
}

/// Compression report comparing raw detector alarm volume to fused episode count.
///
/// This is the primary quantitative justification for the fusion layer: many individual
/// detector breach steps (possibly from the same underlying process event) collapse into
/// a small number of fused structural episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionReport {
    /// Sum of `DetectorOutput::breach == true` steps across all detectors and all timelines.
    /// This counts raw individual-detector alarm steps, not unique process events.
    pub raw_breach_steps: usize,
    /// Number of fused structural episodes produced by `fuse`.
    pub fused_episodes: usize,
    /// `raw_breach_steps / fused_episodes`: the average number of raw breach steps per
    /// fused episode. A value > 1 indicates alarm compression; 0.0 when `fused_episodes == 0`
    /// (guard against division by zero).
    pub compression_ratio: f64,
}

/// Compute the compression report for a set of timelines and their fused episodes.
///
/// `raw_breach_steps` is derived from `DetectorOutput::breach` (the raw detector flag),
/// not from the grammar classification. This keeps the two counts independent: grammar states
/// and breach flags can disagree on exact boundaries, and the comparison is the paper's
/// Appendix B Table.
pub fn compression(timelines: &[DetectorTimeline], episodes: &[FusedEpisode]) -> CompressionReport {
    let raw: usize = timelines
        .iter()
        .flat_map(|t| t.outputs.iter())
        .filter(|o| o.breach)
        .count();
    let fe = episodes.len();
    CompressionReport {
        raw_breach_steps: raw,
        fused_episodes: fe,
        compression_ratio: if fe > 0 { raw as f64 / fe as f64 } else { 0.0 },
    }
}

/// Aggregate one detector's grammar steps into per-detector episodes.
///
/// Thin wrapper over `dsfb_core::aggregate_episodes` for use in per-detector report generation,
/// separate from the cross-detector fusion path.
pub fn per_detector_episodes(t: &DetectorTimeline) -> Vec<crate::dsfb_core::Episode> {
    aggregate_episodes(&t.steps)
}

/// A near-episode that fusion examined but did NOT admit, with the reason. Recording refused claims
/// (not only admitted ones) is what makes the court auditable: a reviewer can see where evidence was
/// present yet the quorum/duration thresholds declined to form an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAdmission {
    pub start_index: usize,
    pub end_index: usize,
    pub step_count: usize,
    /// Peak count of distinct firing families observed within the run.
    pub peak_families: usize,
    pub required_families: usize,
    pub min_steps: usize,
    /// Why the run was not admitted: "insufficient_families" or "too_short".
    pub reason: &'static str,
}

/// Recompute fusion's per-step quorum and return the near-episodes it did NOT admit:
/// (a) contiguous runs with ≥1 firing family but fewer than `min_families` (`insufficient_families`);
/// (b) contiguous quorum-met runs shorter than `min_steps` (`too_short` — the runs `fuse` discards by
/// the length filter). Deterministic; mirrors `fuse`'s firing definition (non-nominal, non-SensorFault).
/// This is the counterfactual record: the evidence DSFB saw and declined to promote to an episode.
pub fn non_admitted_runs(timelines: &[DetectorTimeline], cfg: FusionConfig) -> Vec<NonAdmission> {
    use std::collections::BTreeSet;
    if timelines.is_empty() {
        return Vec::new();
    }
    let n = timelines.iter().map(|t| t.steps.len()).max().unwrap_or(0);
    let mut fam_count = vec![0usize; n]; // distinct firing families at each step
    let mut quorum = vec![false; n];
    for i in 0..n {
        let mut fams: BTreeSet<DetectorFamily> = BTreeSet::new();
        for t in timelines {
            if let Some(s) = t.steps.get(i) {
                if s.state.is_non_nominal() && s.state != GrammarState::SensorFault {
                    fams.insert(t.family);
                }
            }
        }
        fam_count[i] = fams.len();
        quorum[i] = fams.len() >= cfg.min_families;
    }
    let mut out: Vec<NonAdmission> = Vec::new();
    // (a) sub-quorum active runs: 1 ≤ families < min_families throughout.
    let mut i = 0;
    while i < n {
        if fam_count[i] >= 1 && !quorum[i] {
            let start = i;
            let mut peak = 0;
            while i < n && fam_count[i] >= 1 && !quorum[i] {
                peak = peak.max(fam_count[i]);
                i += 1;
            }
            out.push(NonAdmission {
                start_index: start,
                end_index: i - 1,
                step_count: i - start,
                peak_families: peak,
                required_families: cfg.min_families,
                min_steps: cfg.min_steps,
                reason: "insufficient_families",
            });
        } else {
            i += 1;
        }
    }
    // (b) quorum-met runs that are too short to be admitted.
    let mut i = 0;
    while i < n {
        if quorum[i] {
            let start = i;
            let mut peak = 0;
            while i < n && quorum[i] {
                peak = peak.max(fam_count[i]);
                i += 1;
            }
            if i - start < cfg.min_steps {
                out.push(NonAdmission {
                    start_index: start,
                    end_index: i - 1,
                    step_count: i - start,
                    peak_families: peak,
                    required_families: cfg.min_families,
                    min_steps: cfg.min_steps,
                    reason: "too_short",
                });
            }
        } else {
            i += 1;
        }
    }
    out.sort_by_key(|r| (r.start_index, r.end_index));
    out
}
