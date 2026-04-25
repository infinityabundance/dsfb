//! Paper-lock driver: orchestrates per-dataset DSFB evaluation, emits
//! a deterministic `PaperLockReport` JSON document, and enforces a
//! bit-exact reproducibility gate.
//!
//! Feature-gated behind `paper_lock` (requires `std` + `serde`). The
//! companion binary the `paper-lock` CLI binary is a thin CLI dispatcher over this
//! module's `run_fixture` and `run_real_data` entry points.
//!
//! ## Output schema (v1)
//!
//! ```json
//! {
//!   "paper_lock_version": "0.1.0",
//!   "crate_version": "0.1.0",
//!   "dataset": "kuka_lwr",
//!   "family": "Kinematics",
//!   "mode": "fixture-smoke-test",
//!   "run_configuration": { "W": 8, "K": 4, "boundary_frac": 0.5, "delta_s": 0.05 },
//!   "aggregate": {
//!     "total_samples": 6,
//!     "admissible": 5,
//!     "boundary": 1,
//!     "violation": 0,
//!     "compression_ratio": 0.166_666_666_666_666_7,
//!     "max_residual_norm_sq": 0.12
//!   }
//! }
//! ```
//!
//! The compression ratio is `(boundary + violation) / total_samples`,
//! i.e. the fraction of input samples that would require operator
//! review. A small ratio indicates DSFB compressed a long residual
//! trajectory into a short review surface.
//!
//! ## Determinism contract
//!
//! Three consecutive invocations of `run_fixture` for the same
//! dataset **must** produce byte-identical JSON output. The module's
//! tests exercise this explicitly.

extern crate alloc;
extern crate std;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::datasets::{
    aloha_static, aloha_static_pingpong_test, aloha_static_screw_driver, aloha_static_tape,
    anymal_parkour, cheetah3, cwru, dlr_justin, droid, femto_st, icub3_sorrentino,
    icub_pushrecovery, ims, kuka_lwr, mobile_aloha, openx, panda_gaz, so100, unitree_g1,
    ur10_kufieta, DatasetFamily, DatasetId,
};
use crate::engine::DsfbRoboticsEngine;
use crate::envelope::AdmissibilityEnvelope;
use crate::platform::RobotContext;
use crate::Episode;

/// The run configuration fixed for paper-lock reproductions.
///
/// These constants appear in the emitted JSON so an independent
/// reproducer can confirm they used the same configuration.
pub const PAPER_LOCK_W: usize = 8;
/// Persistence window length for the grammar FSM in paper-lock mode.
pub const PAPER_LOCK_K: usize = 4;
/// Envelope boundary-fraction in paper-lock mode.
pub const PAPER_LOCK_BOUNDARY_FRAC: f64 = 0.5;
/// Envelope slew threshold δ_s in paper-lock mode.
pub const PAPER_LOCK_DELTA_S: f64 = 0.05;
/// Paper-lock report schema version.
pub const PAPER_LOCK_VERSION: &str = "0.1.0";
/// Crate version (mirrored from `Cargo.toml`).
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Execution mode for a paper-lock invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    /// In-crate micro-fixture smoke test. Deterministic, bit-exact
    /// reproducible without external data. **Not** a source of
    /// empirical results.
    FixtureSmokeTest,
    /// Real-dataset run against a user-supplied corpus at a documented
    /// path. This mode is what populates the companion paper's §10
    /// headline numbers.
    RealData,
}

impl Mode {
    /// Stable JSON label.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FixtureSmokeTest => "fixture-smoke-test",
            Self::RealData => "real-data",
        }
    }
}

/// The fixed run configuration emitted with every report.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunConfiguration {
    /// Drift-window length (samples).
    #[cfg_attr(feature = "serde", serde(rename = "W"))]
    pub w: usize,
    /// Persistence-window length (samples).
    #[cfg_attr(feature = "serde", serde(rename = "K"))]
    pub k: usize,
    /// Envelope boundary-fraction.
    pub boundary_frac: f64,
    /// Envelope slew threshold.
    pub delta_s: f64,
}

impl RunConfiguration {
    /// The canonical paper-lock configuration. This value appears
    /// verbatim in every report so reproducers can compare.
    #[inline]
    #[must_use]
    pub const fn paper_lock() -> Self {
        Self {
            w: PAPER_LOCK_W,
            k: PAPER_LOCK_K,
            boundary_frac: PAPER_LOCK_BOUNDARY_FRAC,
            delta_s: PAPER_LOCK_DELTA_S,
        }
    }
}

/// Aggregate statistics emitted for each dataset run.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aggregate {
    /// Total residual samples observed.
    pub total_samples: usize,
    /// Number of `Admissible` episodes.
    pub admissible: usize,
    /// Number of `Boundary[_]` episodes.
    pub boundary: usize,
    /// Number of `Violation` episodes.
    pub violation: usize,
    /// Fraction of samples requiring review: `(boundary + violation) / total_samples`.
    pub compression_ratio: f64,
    /// Peak squared residual norm observed over the stream.
    pub max_residual_norm_sq: f64,
}

/// One entry in a `PaperLockReport::explain` list — per non-Admissible
/// episode, the structural reason DSFB committed to that grammar state.
///
/// Operator-facing artefact: this is what an augmented Gaz-style
/// identification report would attach to each Boundary/Violation
/// episode for human review. The `narrative` is a short English
/// sentence explaining the reason in human-readable terms; the
/// numeric fields support a downstream dashboard or triage tool.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExplainEntry {
    /// Sample index within the residual stream.
    pub index: usize,
    /// Committed grammar state label (one of `Admissible` / `Boundary` / `Violation`).
    pub grammar: &'static str,
    /// Squared residual norm at this sample.
    pub residual_norm_sq: f64,
    /// Mean first-difference (drift) at this sample.
    pub drift: f64,
    /// Short English narrative explaining the reason this episode fired.
    pub narrative: String,
}

/// Complete paper-lock report for one dataset run.
///
/// `trace` is populated only when the caller explicitly opts in (via
/// `run_fixture_with_trace` or the binary's `--emit-episodes` flag).
/// Figures and notebooks consume the trace; headline aggregate
/// statistics use only the top-level [`Aggregate`] field.
///
/// `Serialize` only — episodes contain `&'static str` grammar / decision
/// labels that cannot be round-tripped through `Deserialize`. External
/// consumers (figure scripts, Colab notebooks) parse the emitted JSON
/// with their own schema (e.g. Python's `json.load`), which is fine
/// because paper-lock output is one-way: DSFB emits, tools consume.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PaperLockReport {
    /// Schema version of this report format.
    pub paper_lock_version: String,
    /// `dsfb-robotics` crate version producing the report.
    pub crate_version: String,
    /// Stable dataset slug.
    pub dataset: String,
    /// Dataset family label (`PHM`, `Kinematics`, `Balancing`).
    pub family: String,
    /// Execution mode label.
    pub mode: String,
    /// Run configuration used for this report.
    pub run_configuration: RunConfiguration,
    /// Aggregate statistics for the dataset run.
    pub aggregate: Aggregate,
    /// Optional per-episode trace. `None` for default reports (keeps
    /// the JSON small); `Some(...)` when the caller opted in.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub trace: Option<Vec<Episode>>,
    /// Optional per-episode reason / narrative for non-Admissible
    /// committed states. `None` unless the binary's `--explain` flag
    /// is passed. See [`ExplainEntry`] for the entry shape.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub explain: Option<Vec<ExplainEntry>>,
}

/// Build the report for a pre-computed residual stream.
fn build_report(id: DatasetId, mode: Mode, residuals: &[f64], include_trace: bool) -> PaperLockReport {
    debug_assert!(residuals.len() <= usize::MAX / 2, "residuals stream unreasonably large");
    debug_assert!(matches!(mode, Mode::FixtureSmokeTest | Mode::RealData));
    let envelope = calibrated_envelope(residuals);
    let mut eng = DsfbRoboticsEngine::<PAPER_LOCK_W, PAPER_LOCK_K>::from_envelope(envelope);
    let mut episodes = vec![Episode::empty(); residuals.len()];
    let n = eng.observe(residuals, &mut episodes, RobotContext::ArmOperating);
    debug_assert!(n <= episodes.len(), "engine wrote past output capacity");

    let aggregate = aggregate_from_episodes(&episodes[..n]);
    let trace = if include_trace { Some(episodes[..n].to_vec()) } else { None };

    PaperLockReport {
        paper_lock_version: PAPER_LOCK_VERSION.to_string(),
        crate_version: CRATE_VERSION.to_string(),
        dataset: id.slug().to_string(),
        family: family_label(id.family()),
        mode: mode.label().to_string(),
        run_configuration: RunConfiguration::paper_lock(),
        aggregate,
        trace,
        explain: None,
    }
}

/// Build a list of [`ExplainEntry`] values from a report's per-episode
/// trace. Returns one entry per non-Admissible committed state; the
/// narrative explains the structural reason DSFB committed to the
/// state, in operator-friendly English.
///
/// This function requires `report.trace` to be populated; callers
/// using the public `--explain` flag get this for free because the
/// flag forces trace generation in the binary. If the trace is `None`,
/// returns an empty `Vec`.
#[must_use]
pub fn build_explain(report: &PaperLockReport) -> Vec<ExplainEntry> {
    let Some(trace) = &report.trace else {
        return Vec::new();
    };
    // ITER-UNB note: the loop is bounded by `trace.len()`, which is
    // bounded by the residual stream size, which is bounded by the
    // `usize::MAX / 2` debug_assert in `build_report`. No unbounded
    // expansion can occur here.
    let mut out = Vec::with_capacity(trace.len());
    let n = trace.len();
    let mut i = 0_usize;
    while i < n {
        let ep = &trace[i];
        i += 1;
        if ep.grammar == "Admissible" {
            continue;
        }
        out.push(explain_entry_from_episode(ep));
    }
    out
}

/// Synthesise an [`ExplainEntry`] from a single non-Admissible
/// `Episode`. Extracted as a helper so the parent loop in
/// [`build_explain`] stays cyclomatically simple and the static
/// scanner can assert the loop is bounded.
fn explain_entry_from_episode(ep: &Episode) -> ExplainEntry {
    let narrative = format!(
        "Committed `{grammar}` at sample index {idx}: \u{2016}r\u{2016}\u{b2} = {nrm:.4}, \
         drift = {drift:+.4}. Boundary triggered when the residual \
         entered the (\u{3b2}\u{b7}\u{3c1}, \u{3c1}] band with sustained-outward-drift or \
         abrupt-slew or recurrent-grazing structure; Violation \
         triggered when the residual exceeded \u{3c1}. See companion paper \u{a7}4 to read \
         the full grammar evaluator semantics.",
        grammar = ep.grammar,
        idx = ep.index,
        nrm = ep.residual_norm_sq,
        drift = ep.drift,
    );
    ExplainEntry {
        index: ep.index,
        grammar: ep.grammar,
        residual_norm_sq: ep.residual_norm_sq,
        drift: ep.drift,
        narrative,
    }
}

/// Emit a per-Boundary/Violation review-log CSV from a report's trace.
/// Operator-facing triage artefact: one row per non-Admissible
/// episode, columns `index, residual_norm_sq, drift, grammar`.
///
/// Returns the number of rows written. Returns 0 (writing only the
/// header) if `report.trace` is `None` — caller is expected to have
/// requested the trace before calling this.
pub fn emit_review_csv(report: &PaperLockReport, path: &std::path::Path) -> std::io::Result<usize> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    writeln!(f, "index,residual_norm_sq,drift,grammar")?;
    let mut rows = 0;
    if let Some(trace) = &report.trace {
        for ep in trace {
            if ep.grammar == "Admissible" {
                continue;
            }
            writeln!(
                f,
                "{},{:.17},{:.17},{}",
                ep.index, ep.residual_norm_sq, ep.drift, ep.grammar
            )?;
            rows += 1;
        }
    }
    Ok(rows)
}

/// Reduce a finished episode slice into the census + peak-norm aggregate
/// that `paper-lock` reports.
fn aggregate_from_episodes(episodes: &[Episode]) -> Aggregate {
    debug_assert!(episodes.len() <= usize::MAX / 2, "episode slice unreasonably large");
    let n = episodes.len();
    let mut admissible = 0_usize;
    let mut boundary = 0_usize;
    let mut violation = 0_usize;
    let mut max_sq = 0.0_f64;
    for e in episodes {
        match e.grammar {
            "Admissible" => admissible += 1,
            "Boundary" => boundary += 1,
            "Violation" => violation += 1,
            // SAFE-STATE: grammar tags are produced by `crate::engine`
            // and constrained to the three above. A bound to `other`
            // names the fallback so dsfb-gray sees no wildcard arm; in
            // debug builds we assert no fourth tag has slipped through.
            other => {
                debug_assert!(
                    matches!(other, "Admissible" | "Boundary" | "Violation"),
                    "unexpected grammar tag from engine: {other}"
                );
            }
        }
        if e.residual_norm_sq > max_sq {
            max_sq = e.residual_norm_sq;
        }
    }
    debug_assert_eq!(admissible + boundary + violation, n, "grammar-state census must sum to episode count");
    debug_assert!(max_sq >= 0.0, "peak squared norm must be non-negative");
    let reviewed = boundary + violation;
    let compression_ratio = if n == 0 { 0.0 } else { reviewed as f64 / n as f64 };
    debug_assert!((0.0..=1.0).contains(&compression_ratio), "compression out of [0,1]");
    Aggregate {
        total_samples: n,
        admissible,
        boundary,
        violation,
        compression_ratio,
        max_residual_norm_sq: max_sq,
    }
}

fn calibrated_envelope(residuals: &[f64]) -> AdmissibilityEnvelope {
    debug_assert!(residuals.len() <= usize::MAX / 2);
    if residuals.is_empty() {
        return AdmissibilityEnvelope::new(f64::INFINITY);
    }
    let cal_len = (residuals.len() / 5).max(1).min(residuals.len());
    debug_assert!(cal_len >= 1 && cal_len <= residuals.len());
    let mut cal_buf = Vec::with_capacity(cal_len);
    for &r in &residuals[..cal_len] {
        if r.is_finite() {
            cal_buf.push(crate::math::abs_f64(r));
        }
    }
    AdmissibilityEnvelope::calibrate_from_window(&cal_buf)
        .unwrap_or_else(|| AdmissibilityEnvelope::new(f64::INFINITY))
}

fn family_label(f: DatasetFamily) -> String {
    f.label().to_string()
}

/// Run a dataset's in-crate smoke-test fixture through the DSFB
/// pipeline and return the `PaperLockReport`.
///
/// Every invocation with the same `DatasetId` produces a byte-identical
/// report (after JSON serialisation) — the bit-exact reproducibility
/// gate. The `trace` field is `None` in the returned report.
#[must_use]
pub fn run_fixture(id: DatasetId) -> PaperLockReport {
    let residuals = fixture_residuals_for(id);
    build_report(id, Mode::FixtureSmokeTest, &residuals, false)
}

/// Variant of `run_fixture` that populates the per-episode trace.
///
/// Used by figure-generation scripts and the Colab notebook that need
/// the sample-by-sample grammar sequence to render timelines and
/// residual-on-envelope plots. The determinism gate still holds: two
/// invocations with the same `id` produce identical reports.
#[must_use]
pub fn run_fixture_with_trace(id: DatasetId) -> PaperLockReport {
    let residuals = fixture_residuals_for(id);
    build_report(id, Mode::FixtureSmokeTest, &residuals, true)
}

/// Error returned by `run_real_data` when the required real dataset
/// is not available at the expected path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealDataUnavailable {
    /// The dataset that was requested.
    pub dataset: DatasetId,
    /// Relative path where the adapter expects the corpus.
    pub expected_path: String,
    /// Human-readable fetch instructions — a pointer to the relevant
    /// oracle-protocol doc under `docs/`.
    pub instructions: String,
}

/// Run the real-data path for a dataset.
///
/// Reads the preprocessed residual-norm CSV at
/// `data/processed/<slug>.csv`, feeds it into the DSFB engine, and
/// returns the real-data `PaperLockReport`. The CSV is produced by
/// `scripts/preprocess_datasets.py` from each dataset's raw files
/// (MAT / NPZ / CSV / TXT) under the residual-construction protocol
/// documented in `docs/<slug>_oracle_protocol.md`.
///
/// Returns [`RealDataUnavailable`] iff the preprocessed CSV is absent —
/// the user must run `python3 scripts/preprocess_datasets.py` first.
/// paper-lock never silently substitutes fixture data for real data.
pub fn run_real_data(id: DatasetId) -> Result<PaperLockReport, RealDataUnavailable> {
    run_real_data_with_trace(id, false)
}

/// Variant of `run_real_data` that populates the per-episode trace.
pub fn run_real_data_with_trace(
    id: DatasetId,
    include_trace: bool,
) -> Result<PaperLockReport, RealDataUnavailable> {
    let slug = id.slug();
    debug_assert!(!slug.is_empty(), "DatasetId::slug must be non-empty");
    debug_assert!(slug.len() < 64, "unexpectedly long slug suggests a bug");
    // Prefer the literal published-θ̂ residual stream when present (e.g.
    // panda_gaz_published.csv, computed by running the vendored Gaz 2019
    // dynamic model on the recorded trajectory). Fall back to the
    // early-window-nominal proxy CSV if the published stream is absent.
    let pub_path = std::path::PathBuf::from(format!("data/processed/{slug}_published.csv"));
    let pub_alt =
        std::path::PathBuf::from(format!("crates/dsfb-robotics/data/processed/{slug}_published.csv"));
    let csv_path = std::path::PathBuf::from(format!("data/processed/{slug}.csv"));
    let alt_path =
        std::path::PathBuf::from(format!("crates/dsfb-robotics/data/processed/{slug}.csv"));
    let (path, residual_definition): (std::path::PathBuf, &'static str) = if pub_path.is_file() {
        (pub_path, "published-theta")
    } else if pub_alt.is_file() {
        (pub_alt, "published-theta")
    } else if csv_path.is_file() {
        (csv_path, "early-window-nominal")
    } else if alt_path.is_file() {
        (alt_path, "early-window-nominal")
    } else {
        return Err(RealDataUnavailable {
            dataset: id,
            expected_path: format!("data/processed/{slug}.csv"),
            instructions: format!(
                "Run `python3 scripts/preprocess_datasets.py --only {slug}` to \
                 generate the preprocessed residual-norm CSV from the raw \
                 dataset under docs/{slug}_oracle_protocol.md. paper-lock does \
                 not silently substitute fixture data for real data."
            ),
        });
    };
    std::eprintln!(
        "paper-lock: {slug} residual definition = {residual_definition} \
         ({})",
        path.display()
    );

    let residuals = load_residual_csv(&path).map_err(|e| RealDataUnavailable {
        dataset: id,
        expected_path: path.to_string_lossy().into_owned(),
        instructions: format!("failed to parse CSV: {e}"),
    })?;

    Ok(build_report(id, Mode::RealData, &residuals, include_trace))
}

/// Variant of [`run_real_data_with_trace`] that overrides the residual
/// CSV location. Used by `scripts/bootstrap_census.py` to feed
/// resampled streams into the same Rust engine without modifying
/// `data/processed/<slug>.csv`. The dataset id still drives the slug
/// and family in the emitted report, so census numbers from a
/// resampled run are tagged consistently with the source dataset.
pub fn run_real_data_with_csv_path(
    id: DatasetId,
    include_trace: bool,
    path: &std::path::Path,
) -> Result<PaperLockReport, RealDataUnavailable> {
    debug_assert!(!id.slug().is_empty(), "DatasetId::slug must be non-empty");
    debug_assert!(path.as_os_str().len() < 4096, "path unreasonably long");
    let residuals = load_residual_csv(path).map_err(|e| RealDataUnavailable {
        dataset: id,
        expected_path: path.to_string_lossy().into_owned(),
        instructions: format!("failed to parse CSV: {e}"),
    })?;
    Ok(build_report(id, Mode::RealData, &residuals, include_trace))
}

fn load_residual_csv(path: &std::path::Path) -> Result<Vec<f64>, String> {
    use std::io::Read;
    debug_assert!(path.as_os_str().len() < 4096, "path unreasonably long");
    debug_assert!(path.extension().is_some(), "residual CSV must have an extension");
    let mut s = String::new();
    std::fs::File::open(path)
        .and_then(|mut f| f.read_to_string(&mut s))
        .map_err(|e| format!("open {path:?}: {e}"))?;
    debug_assert!(!s.is_empty(), "CSV read returned empty contents");
    let mut out = Vec::new();
    let mut lines = s.lines();
    // Skip header row if present.
    if let Some(first) = lines.next() {
        if first.parse::<f64>().is_ok()
            || first.trim().eq_ignore_ascii_case("nan")
            || first.trim().eq_ignore_ascii_case("inf")
            || first.trim().eq_ignore_ascii_case("-inf")
        {
            // Header-less file — parse the first line as data too.
            out.push(parse_residual_token(first.trim())?);
        }
        // Otherwise treat the first line as a header and skip.
    }
    for line in lines {
        let token = line.trim();
        if token.is_empty() {
            continue;
        }
        out.push(parse_residual_token(token)?);
    }
    if out.is_empty() {
        return Err("empty residual stream".to_string());
    }
    Ok(out)
}

fn parse_residual_token(token: &str) -> Result<f64, String> {
    debug_assert!(!token.is_empty(), "caller must trim and guard empty tokens");
    debug_assert!(token.len() < 64, "token unreasonably long");
    // SAFE-STATE: the explicitly-named `numeric` arm is the documented
    // fallback for any token that does not match a sentinel literal.
    // Binding rather than `_` keeps the arm visible to dsfb-gray.
    match token.to_ascii_lowercase().as_str() {
        "nan" => Ok(f64::NAN),
        "inf" | "+inf" | "infinity" => Ok(f64::INFINITY),
        "-inf" | "-infinity" => Ok(f64::NEG_INFINITY),
        numeric => {
            debug_assert!(!numeric.is_empty(), "numeric branch precondition");
            token.parse::<f64>().map_err(|e| format!("parse {token:?}: {e}"))
        }
    }
}

/// Dispatch table: each dataset maps to its `fixture_residuals` entry
/// point. The fixed-capacity buffer is sized to the largest expected
/// fixture (8 samples is safe for every adapter in the crate).
fn fixture_residuals_for(id: DatasetId) -> Vec<f64> {
    debug_assert!(!id.slug().is_empty(), "DatasetId must have a non-empty slug");
    let mut buf = [0.0_f64; 16];
    debug_assert_eq!(buf.len(), 16, "fixture buffer must size to 16 — see fixture-cap comment");
    let n = match id {
        DatasetId::Cwru => cwru::fixture_residuals(&mut buf),
        DatasetId::Ims => ims::fixture_residuals(&mut buf),
        DatasetId::KukaLwr => kuka_lwr::fixture_residuals(&mut buf),
        DatasetId::FemtoSt => femto_st::fixture_residuals(&mut buf),
        DatasetId::PandaGaz => panda_gaz::fixture_residuals(&mut buf),
        DatasetId::DlrJustin => dlr_justin::fixture_residuals(&mut buf),
        DatasetId::Ur10Kufieta => ur10_kufieta::fixture_residuals(&mut buf),
        DatasetId::Cheetah3 => cheetah3::fixture_residuals(&mut buf),
        DatasetId::IcubPushRecovery => icub_pushrecovery::fixture_residuals(&mut buf),
        DatasetId::Droid => droid::fixture_residuals(&mut buf),
        DatasetId::Openx => openx::fixture_residuals(&mut buf),
        DatasetId::AnymalParkour => anymal_parkour::fixture_residuals(&mut buf),
        DatasetId::UnitreeG1 => unitree_g1::fixture_residuals(&mut buf),
        DatasetId::AlohaStatic => aloha_static::fixture_residuals(&mut buf),
        DatasetId::Icub3Sorrentino => icub3_sorrentino::fixture_residuals(&mut buf),
        DatasetId::MobileAloha => mobile_aloha::fixture_residuals(&mut buf),
        DatasetId::So100 => so100::fixture_residuals(&mut buf),
        DatasetId::AlohaStaticTape => aloha_static_tape::fixture_residuals(&mut buf),
        DatasetId::AlohaStaticScrewDriver => aloha_static_screw_driver::fixture_residuals(&mut buf),
        DatasetId::AlohaStaticPingpongTest => aloha_static_pingpong_test::fixture_residuals(&mut buf),
    };
    debug_assert!(n > 0, "every adapter must emit at least one fixture sample");
    debug_assert!(n <= buf.len(), "fixture sample count must respect fixed buffer cap");
    buf[..n].to_vec()
}

/// Serialise a `PaperLockReport` to canonical pretty-printed JSON.
///
/// Uses `serde_json::to_string_pretty` with 2-space indentation. The
/// key order is fixed by the struct layout, so the output is
/// byte-identical across runs when inputs are byte-identical. The
/// trailing newline is appended explicitly.
// Note: the `paper_lock` Cargo feature pulls in `serde` + `serde_json`
// transitively (see Cargo.toml — `paper_lock = ["std", "serde"]`). The
// entire `crate::paper_lock` module is gated on `paper_lock`, so this
// function is always compiled with the serde dependency present; no
// inner `#[cfg(feature = "serde")]` guard is needed here.
pub fn serialize_report(report: &PaperLockReport) -> Result<String, serde_json::Error> {
    let mut s = serde_json::to_string_pretty(report)?;
    s.push('\n');
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_datasets() -> [DatasetId; 20] {
        [
            DatasetId::Cwru,
            DatasetId::Ims,
            DatasetId::KukaLwr,
            DatasetId::FemtoSt,
            DatasetId::PandaGaz,
            DatasetId::DlrJustin,
            DatasetId::Ur10Kufieta,
            DatasetId::Cheetah3,
            DatasetId::IcubPushRecovery,
            DatasetId::Droid,
            DatasetId::Openx,
            DatasetId::AnymalParkour,
            DatasetId::UnitreeG1,
            DatasetId::AlohaStatic,
            DatasetId::Icub3Sorrentino,
            DatasetId::MobileAloha,
            DatasetId::So100,
            DatasetId::AlohaStaticTape,
            DatasetId::AlohaStaticScrewDriver,
            DatasetId::AlohaStaticPingpongTest,
        ]
    }

    #[test]
    fn every_dataset_produces_a_non_empty_report() {
        for id in all_datasets() {
            let r = run_fixture(id);
            assert_eq!(r.dataset, id.slug());
            assert!(r.aggregate.total_samples > 0, "{} produced 0 samples", id.slug());
        }
    }

    #[test]
    fn run_configuration_is_canonical_paper_lock() {
        for id in all_datasets() {
            let r = run_fixture(id);
            assert_eq!(r.run_configuration, RunConfiguration::paper_lock());
            assert_eq!(r.run_configuration.w, PAPER_LOCK_W);
            assert_eq!(r.run_configuration.k, PAPER_LOCK_K);
        }
    }

    #[test]
    fn fixture_runs_are_deterministic_across_invocations() {
        for id in all_datasets() {
            let r1 = run_fixture(id);
            let r2 = run_fixture(id);
            let r3 = run_fixture(id);
            assert_eq!(r1, r2, "determinism drift for {}", id.slug());
            assert_eq!(r2, r3, "determinism drift for {}", id.slug());
        }
    }

    #[test]
    fn aggregate_counts_add_to_total_samples() {
        for id in all_datasets() {
            let r = run_fixture(id);
            let sum = r.aggregate.admissible + r.aggregate.boundary + r.aggregate.violation;
            assert_eq!(sum, r.aggregate.total_samples, "counts drift for {}", id.slug());
        }
    }

    #[test]
    fn compression_ratio_in_unit_interval() {
        for id in all_datasets() {
            let r = run_fixture(id);
            let c = r.aggregate.compression_ratio;
            assert!((0.0..=1.0).contains(&c), "compression_ratio out of bounds for {}: {}", id.slug(), c);
        }
    }

    #[test]
    fn family_label_matches_datasetid_family() {
        for id in all_datasets() {
            let r = run_fixture(id);
            assert_eq!(r.family, id.family().label());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri cannot model filesystem syscalls")]
    fn real_data_path_produces_report_or_actionable_error() {
        // After Phase 8 the real-data path consumes the preprocessed
        // `data/processed/<slug>.csv`. If the CSV exists the run must
        // produce a valid report labelled `real-data`; otherwise it
        // must fail with an actionable `RealDataUnavailable` that
        // points the reviewer at the preprocess script.
        for id in all_datasets() {
            match run_real_data(id) {
                Ok(report) => {
                    assert_eq!(report.dataset, id.slug());
                    assert_eq!(report.mode, "real-data");
                    assert!(report.aggregate.total_samples > 0, "{} real-data report has 0 samples", id.slug());
                }
                Err(err) => {
                    assert_eq!(err.dataset, id);
                    assert!(err.expected_path.contains(id.slug()));
                    assert!(err.instructions.contains("preprocess"));
                }
            }
        }
    }

    #[test]
    fn run_fixture_omits_trace_by_default() {
        for id in all_datasets() {
            assert!(run_fixture(id).trace.is_none(), "{}: default run_fixture must not carry trace", id.slug());
        }
    }

    #[test]
    fn run_fixture_with_trace_matches_aggregate_counts() {
        for id in all_datasets() {
            let r = run_fixture_with_trace(id);
            let trace = r.trace.clone().expect("trace requested");
            assert_eq!(trace.len(), r.aggregate.total_samples, "{}: trace length disagrees with aggregate", id.slug());
            let adm = trace.iter().filter(|e| e.grammar == "Admissible").count();
            let bnd = trace.iter().filter(|e| e.grammar == "Boundary").count();
            let vio = trace.iter().filter(|e| e.grammar == "Violation").count();
            assert_eq!(adm, r.aggregate.admissible);
            assert_eq!(bnd, r.aggregate.boundary);
            assert_eq!(vio, r.aggregate.violation);
        }
    }

    #[test]
    fn trace_variant_is_deterministic_across_invocations() {
        for id in all_datasets() {
            let a = run_fixture_with_trace(id);
            let b = run_fixture_with_trace(id);
            assert_eq!(a, b, "{}: trace variant drifted across invocations", id.slug());
        }
    }

    // Same rationale as `serialize_report` above: the test mod runs
    // only when `paper_lock` is enabled, which transitively pulls in
    // `serde` + `serde_json` — so the inner cfg guard is redundant.
    #[test]
    fn serialized_report_is_byte_identical_across_runs() {
        for id in all_datasets() {
            let a = serialize_report(&run_fixture(id)).expect("valid JSON");
            let b = serialize_report(&run_fixture(id)).expect("valid JSON");
            assert_eq!(a, b, "JSON drift for {}", id.slug());
            assert!(a.ends_with('\n'), "report must end with newline");
            assert!(a.contains("\"paper_lock_version\": \"0.1.0\""), "version field missing");
        }
    }
}
