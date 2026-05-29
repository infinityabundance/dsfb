//! Figure-trace export.
//!
//! The Rust pipeline emits byte-stable trace CSVs; `scripts/gen_figures.py` renders
//! publication-quality PNG/PDF figures from them (kept out of the crate so the core has no
//! plotting dependency). This guarantees the figures in the paper are regenerable from committed
//! numeric traces: anyone who checks out the repository and runs `gen_figures.py` against the
//! committed CSVs will obtain pixel-identical figures without re-running the full pipeline.
//!
//! ## Files emitted
//!
//! | Function | File | Contents |
//! |---|---|---|
//! | [`export_trace`] | `trace_<dataset>.csv` | SPE, T², episode-coverage mask, fault-onset marker |
//! | [`export_grammar_timeline`] | `grammar_<dataset>_<detector_id>.csv` | Grammar-state token + signed margin per step |
//!
//! ## Design choice
//!
//! Float values are formatted to 6 decimal places. This is sufficient for publication figures
//! and ensures the CSV bytes are stable (no platform-dependent `{:?}` or `g`-format surprises).

use crate::data::DataMatrix;
use crate::fusion::DetectorTimeline;
use std::fs;
use std::path::Path;

/// Export the canonical SPE and Hotelling T² monitoring traces for one dataset.
///
/// Writes `<dir>/trace_<dataset>.csv` with columns:
/// - `time_index`: integer sample index starting at 0.
/// - `spe`: SPE (Q statistic) at this step — out-of-plane PCA residual energy.
/// - `t2`: Hotelling T² at this step — in-plane PCA score distance from baseline centre.
/// - `in_episode`: 1 if this sample falls within any fused episode, 0 otherwise.
/// - `is_onset`: 1 exactly at the annotated fault-onset sample (if known), 0 everywhere else.
///
/// `n` is the maximum of the SPE and T² slice lengths; missing values for shorter slices are
/// written as empty strings (the CSV crate's default for `unwrap_or_default()`).
pub fn export_trace(
    dir: &Path,
    dataset: &str,
    spe: &[f64],
    t2: &[f64],
    episode_mask: &[bool],
    onset: Option<usize>,
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("trace_{dataset}.csv"));
    let mut w = csv::Writer::from_path(&path)?;
    w.write_record(["time_index", "spe", "t2", "in_episode", "is_onset"])?;
    // Iterate to the length of the longer slice; shorter slices are accessed safely via `.get`.
    let n = spe.len().max(t2.len());
    for i in 0..n {
        w.write_record([
            i.to_string(),
            spe.get(i).map(|v| format!("{:.6}", v)).unwrap_or_default(),
            t2.get(i).map(|v| format!("{:.6}", v)).unwrap_or_default(),
            // `in_episode` and `is_onset` are integer flags (0/1) for easy pandas boolean ops.
            (*episode_mask.get(i).unwrap_or(&false) as u8).to_string(),
            ((onset == Some(i)) as u8).to_string(),
        ])?;
    }
    w.flush()
}

/// Export a per-detector DSFB grammar-state timeline for one detector.
///
/// Writes `<dir>/grammar_<dataset>_<detector_id>.csv` with columns:
/// - `time_index`: integer sample index.
/// - `state_token`: single-character DSFB grammar state at this step (e.g. "N" nominal,
///   "D" drift, "S" slew, "B" breach, etc. — defined in `dsfb_core::GrammarState`).
/// - `signed_margin`: `raw_score − threshold` from the corresponding detector output
///   (positive → breach side, negative → nominal side). Falls back to 0.0 if the outputs
///   slice is shorter than the steps slice (NOTE: this should not occur in normal operation).
///
/// In `cli::run_demo` this is called only for the `"pca_spe_q"` detector, which provides the
/// most interpretable timeline for publication figures. The same function works for any detector.
pub fn export_grammar_timeline(
    dir: &Path,
    dataset: &str,
    timeline: &DetectorTimeline,
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("grammar_{dataset}_{}.csv", timeline.detector_id));
    let mut w = csv::Writer::from_path(&path)?;
    w.write_record(["time_index", "state_token", "signed_margin"])?;
    for (i, s) in timeline.steps.iter().enumerate() {
        // `outputs` and `steps` are constructed in parallel; index alignment is maintained by
        // `run_detector_dsfb`, which produces one step per output. The fallback to 0.0 is a
        // safety guard.
        let margin = timeline
            .outputs
            .get(i)
            .map(|o| o.signed_margin)
            .unwrap_or(0.0);
        // `signed_margin` is signed (raw − threshold), so a near-zero value can render as "-0.000000";
        // canonicalise that to "0.000000" so the trace bytes are stable across platforms whose last-ULP
        // sign can flip. (SPE/T² above are non-negative, so they need no such guard.)
        let margin_s = {
            let s = format!("{:.6}", margin);
            if s == "-0.000000" {
                "0.000000".to_string()
            } else {
                s
            }
        };
        w.write_record([i.to_string(), s.state.token().to_string(), margin_s])?;
    }
    w.flush()
}

/// Build a boolean episode-coverage mask of length `n`.
///
/// `mask[i] = true` iff sample `i` falls within the closed interval `[start_index, end_index]`
/// of at least one fused episode. Episodes whose `end_index` exceeds `n - 1` are clamped; this
/// handles the edge case where the last episode extends to the last sample exactly.
///
/// The mask is used in `export_trace` as the `in_episode` column and in `pipeline::analyze` as
/// the input for computing `baseline_false_positive_rate`.
pub fn episode_mask(n: usize, episodes: &[crate::fusion::FusedEpisode]) -> Vec<bool> {
    let mut mask = vec![false; n];
    for e in episodes {
        // Paint the (clamped) episode span true. `start_index <= end_index` holds by construction and
        // the upper bound is clamped to `n-1`, so the inclusive slice is always in bounds.
        mask[e.start_index..=e.end_index.min(n.saturating_sub(1))].fill(true);
    }
    mask
}

/// Return `(n_samples, n_vars)` for a dataset matrix.
///
/// Thin convenience accessor. Callers that need both dimensions avoid indexing the struct twice.
pub fn dataset_dims(m: &DataMatrix) -> (usize, usize) {
    (m.n_samples, m.n_vars)
}
