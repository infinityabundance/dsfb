//! Artifact writers: per-run CSV/JSON/Markdown reports, a run manifest, replay-hash records, and a
//! dependency-free (store-only) ZIP bundler so each run produces a single downloadable artifact.
//!
//! ## File types produced
//!
//! | Writer | Output | Contents |
//! |---|---|---|
//! | [`write_detector_outputs`] | `detector_outputs.csv` | Raw score, breach flag, signed margin per (detector, sample) |
//! | [`write_residual_streams`] | `residual_streams.csv` | DSFB residual triple (r, delta, sigma) + grammar state per step |
//! | [`write_episodes`] | `dsfb_episodes.csv` | Fused structural episodes (quorum output) |
//! | [`write_heuristic_labels`] | `heuristic_labels.csv` | Heuristic label or "unknown" per episode |
//! | [`write_result_json`] | `result.json` | Full `AnalysisResult` as pretty-printed JSON |
//! | [`write_replay_hashes`] | `replay_hashes.json` | Dataset → SHA-256 replay certificate |
//! | [`write_summary_md`] | `report.md` | Human-readable summary table + non-claims section |
//! | [`zip_store`] | `artifact_bundle.zip` | Store-only (no compression) ZIP of all above files |
//!
//! ## ZIP format notes
//!
//! The ZIP writer is entirely self-contained (no `zip` crate dependency). It uses store method
//! (compression method 0), a fixed mod-date sentinel (`0x21`, near 1980-01-01) for determinism,
//! and CRC-32 (standard ZIP checksum). Files are added in sorted order so the archive bytes are
//! reproducible given the same artifact content.
//!
//! ## Float formatting
//!
//! All float fields in CSV outputs are formatted to 6 decimal places via the private `fmt` helper.
//! Non-finite values (`NaN`, `±∞`) are written as the string `"nan"` rather than a number, to
//! avoid confusing downstream CSV parsers.

use crate::data::{Baseline, DataMatrix};
use crate::dsfb_core::GrammarState;
use crate::evidence_kind::EvidenceKind;
use crate::fusion::{DetectorTimeline, FusedEpisode, NonAdmission};
use crate::hashing::CanonicalHasher;
use crate::heuristics::HeuristicLabel;
use crate::pipeline::AnalysisResult;
use crate::witness_strength::PhysicalWitnessStrength;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Top-level run manifest written as `manifest.json`.
///
/// Captures the tool identity, corpus SHA-256 (for provenance), and a per-dataset summary
/// including replay hashes. Together with `replay_hashes.json` this provides all the information
/// needed to verify a past run: given the original data slices and the same crate version, the
/// hashes must reproduce exactly.
#[derive(Debug, Serialize)]
pub struct RunManifest {
    /// Crate name; used to distinguish manifests from other DSFB tools.
    pub tool: String,
    /// `CARGO_PKG_VERSION` at compile time; written for provenance.
    pub crate_version: String,
    /// ISO 8601 UTC timestamp of the run start.
    pub timestamp_utc: String,
    /// SHA-256 of the serialised detector-atlas corpus (provenance gate).
    pub corpus_sha256: String,
    /// Total detectors in the atlas (executed + catalogued-only).
    pub corpus_detectors: usize,
    /// Detectors actually executed in this run.
    pub corpus_executed: usize,
    /// Per-dataset summary entries.
    pub datasets: Vec<DatasetEntry>,
}

/// Compact per-dataset entry in the run manifest and `replay_hashes.json`.
#[derive(Debug, Serialize)]
pub struct DatasetEntry {
    /// Dataset name (file stem or synthetic label).
    pub name: String,
    /// Provenance tag ("measured/slice" or "synthetic").
    pub kind: String,
    /// SHA-256 replay certificate for this dataset's grammar + episode stream.
    pub replay_hash: String,
    /// Number of fused structural episodes produced.
    pub fused_episodes: usize,
    /// Fraction of episodes that were deliberately left unlabelled (honesty signal).
    pub unknown_rate: f64,
    /// Compression ratio: raw breach steps / fused episode steps.
    pub episode_compression_ratio: f64,
}

/// Write per-detector raw outputs to CSV (one row per (detector, sample) pair).
///
/// Columns written (in order): dataset, detector_id, family, time_index, variable_scope,
/// raw_score, normalized_score, threshold, signed_margin, breach.
///
/// `signed_margin = raw_score − threshold` (positive → breach, negative → nominal). All floats
/// use 6 decimal places; non-finite values are written as `"nan"`.
pub fn write_detector_outputs(
    path: &Path,
    dataset: &str,
    timelines: &[DetectorTimeline],
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "detector_id",
        "family",
        "time_index",
        "variable_scope",
        "raw_score",
        "normalized_score",
        "threshold",
        "signed_margin",
        "breach",
    ])?;
    for t in timelines {
        for o in &t.outputs {
            w.write_record([
                dataset,
                &o.detector_id,
                o.family.label(),
                &o.time_index.to_string(),
                &o.variable_scope,
                &fmt(o.raw_score),
                &fmt(o.normalized_score),
                &fmt(o.threshold),
                &fmt(o.signed_margin),
                &o.breach.to_string(),
            ])?;
        }
    }
    w.flush()
}

/// Write the per-detector DSFB residual triple and grammar state stream to CSV.
///
/// Each row corresponds to one time step for one detector. Columns:
/// - `r`: instantaneous normalised residual (signed distance from admissibility centre).
/// - `delta`: drift component (slow-moving trend estimate over the drift window).
/// - `sigma`: slew component (rate of change of the residual).
/// - `grammar_state`: single-character token for the DSFB grammar state at this step.
/// - `reason`: short human-readable code explaining the grammar-state transition.
pub fn write_residual_streams(
    path: &Path,
    dataset: &str,
    timelines: &[DetectorTimeline],
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "detector_id",
        "time_index",
        "r",
        "delta",
        "sigma",
        "grammar_state",
        "reason",
    ])?;
    for t in timelines {
        for s in &t.steps {
            w.write_record([
                dataset,
                &t.detector_id,
                // `timestamp` is stored as f64 but represents an integer sample index.
                &(s.triple.timestamp as i64).to_string(),
                &fmt(s.triple.r),
                &fmt(s.triple.delta),
                &fmt(s.triple.sigma),
                s.state.token(),
                s.reason.as_str(),
            ])?;
        }
    }
    w.flush()
}

/// Write fused DSFB structural episodes to CSV.
///
/// Each row is one episode from quorum fusion. Key columns:
/// - `dominant_motif`: the grammar-state token that occurred most frequently across participating
///   detectors during this episode (e.g. "D" for drift, "S" for slew).
/// - `consensus_strength`: fraction of participating detectors that agreed on the dominant motif
///   (∈ [0, 1]); higher → tighter agreement.
/// - `disagreement_entropy`: Shannon entropy over motif votes (bits); lower → more consensus.
/// - `families`: pipe-separated list of detector families that contributed to the quorum.
/// - `n_detectors`: count of participating detector instances (finer-grained than families).
pub fn write_episodes(path: &Path, dataset: &str, res: &AnalysisResult) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "start_index",
        "end_index",
        "step_count",
        "dominant_motif",
        "consensus_strength",
        "disagreement_entropy",
        "peak_drift",
        "peak_slew",
        "families",
        "n_detectors",
    ])?;
    for e in &res.fused_episodes {
        w.write_record([
            dataset,
            &e.start_index.to_string(),
            &e.end_index.to_string(),
            &e.step_count.to_string(),
            e.dominant_motif.token(),
            &fmt(e.consensus_strength),
            &fmt(e.disagreement_entropy),
            &fmt(e.peak_drift),
            &fmt(e.peak_slew),
            &e.families.join("|"),
            &e.participating_detectors.len().to_string(),
        ])?;
    }
    w.flush()
}

/// Write heuristic labels for each fused episode to CSV.
///
/// Rows are parallel to `fused_episodes`: row i describes episode i. When `matched = false`,
/// the heuristic bank did not match any rule — this is the deliberate "unknown" outcome that
/// is counted in `Metrics::unknown_rate`. The `heuristic_id` and `episode_label` fields are
/// empty strings in the unmatched case. `severity` is 0.0 when unmatched.
pub fn write_heuristic_labels(
    path: &Path,
    dataset: &str,
    res: &AnalysisResult,
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_start",
        "episode_end",
        "matched",
        "heuristic_id",
        "episode_label",
        "severity",
        "unknown_subtype",
        "challenge_docket",
    ])?;
    for l in &res.heuristic_labels {
        w.write_record([
            dataset,
            &l.episode_start.to_string(),
            &l.episode_end.to_string(),
            &l.matched.to_string(),
            &l.heuristic_id,
            &l.episode_label,
            &fmt(l.severity),
            // Empty for matched episodes; the uncertainty subtype + per-rule near-miss for UNKNOWN ones.
            l.unknown_subtype.as_deref().unwrap_or(""),
            l.challenge.as_deref().unwrap_or(""),
        ])?;
    }
    w.flush()
}

/// Court-language evidence-quality grade for a fused episode, derived only from features the episode
/// already carries — never a probability or confidence. Deterministic; first match wins:
/// - `EVIDENCE_DETECTOR_CONFLICT_HIGH` — `disagreement_entropy > 1.0` nats (detectors disagree).
/// - `EVIDENCE_THIN_SUPPORT`           — `consensus_strength < 0.15` (quorum met, few fire per step).
/// - `EVIDENCE_SHORT_WINDOW`           — `step_count <= 3` (brief; little temporal corroboration).
/// - `EVIDENCE_COMPLETE`               — sustained, agreed, well-supported episode.
///
/// Run-level replay verification is reported separately via the `replay_deterministic` flag, not per
/// episode. Context-dependent grades (missing control context, phase uncertain, sensor-quality,
/// baseline-unstable) require context not yet plumbed and are deliberately NOT emitted here.
pub fn evidence_grade(ep: &FusedEpisode) -> &'static str {
    if ep.disagreement_entropy > 1.0 {
        "EVIDENCE_DETECTOR_CONFLICT_HIGH"
    } else if ep.consensus_strength < 0.15 {
        "EVIDENCE_THIN_SUPPORT"
    } else if ep.step_count <= 3 {
        "EVIDENCE_SHORT_WINDOW"
    } else {
        "EVIDENCE_COMPLETE"
    }
}

/// The [`PhysicalWitnessStrength`] rung an episode sits at, derived *only* from what this generic detector
/// pipeline can see (P83). A fused episode meets the family quorum, so its honest ceiling is
/// `DetectorFamilyQuorum` (statistical agreement of independent detector families — explicitly NOT physics);
/// an episode that fell back to a single supporting family is `HeuristicPatternOnly`. The first-principles
/// rungs (`BalanceClosure` / `TopologyResidenceAligned`) require physical witnesses that the dedicated
/// balance-witness court attaches when a dataset carries roles — they are never asserted from detectors alone.
/// This keeps the operator report from ever dressing statistical agreement up as physical proof.
pub fn episode_witness_strength(ep: &FusedEpisode, matched: bool) -> PhysicalWitnessStrength {
    if ep.families.len() >= 2 {
        PhysicalWitnessStrength::DetectorFamilyQuorum
    } else if matched {
        PhysicalWitnessStrength::HeuristicPatternOnly
    } else {
        PhysicalWitnessStrength::PrecedentSimilarityOnly
    }
}

/// The per-episode **evidence anchor**: a content digest over an episode's sealed fields, so each episode is
/// independently citable. The operator report shows its short form (one per row); the narration-context document
/// lists the full set as the citable-anchor vocabulary a downstream constrained narrator may reference — both call
/// THIS one function, so a sentence citing either is checkable against the same anchor by the hallucination gate.
///
/// Display/citation only: it is **not** folded into the dataset `evidence_root` (which seals the grammar+episode
/// stream), so computing it moves no frozen hash. The field order below is the canonical anchor contract — changing
/// it re-mints every operator-report `bundle_root`, so it is deliberately frozen and shared rather than duplicated.
pub fn episode_evidence_anchor(
    dataset: &str,
    e: &FusedEpisode,
    witness: PhysicalWitnessStrength,
    evidence_kind: &str,
    label: &str,
) -> String {
    let mut h = CanonicalHasher::new();
    h.field("episode_anchor_v1", dataset.as_bytes());
    h.u64("start", e.start_index as u64);
    h.u64("end", e.end_index as u64);
    h.field("motif", e.dominant_motif.token().as_bytes());
    h.field("families", e.families.join(",").as_bytes());
    h.f64q("consensus", e.consensus_strength);
    h.f64q("entropy", e.disagreement_entropy);
    h.field("witness", witness.tag().as_bytes());
    h.field("evidence_kind", evidence_kind.as_bytes());
    h.field("label", label.as_bytes());
    h.finalize_hex()
}

/// Write `episode_evidence.csv`: a per-episode detector-disagreement fingerprint plus the
/// court-language evidence grade.
///
/// For each fused episode this records which detector families and detectors *fired* versus stayed
/// *silent* (the complement against the executed atlas), the Shannon disagreement entropy and
/// consensus strength, the dominant grammar motif, the matched heuristic id (or UNKNOWN), and the
/// evidence grade (see `evidence_grade`). The silent sets are the discriminating part of a
/// disagreement fingerprint — e.g. an SPE detector firing while the T² detector stays silent points
/// to residual-subspace rather than score-space structure. Pure function of the committed episode
/// and timeline data; it adds nothing to the evidence digest.
pub fn write_episode_evidence(
    path: &Path,
    dataset: &str,
    res: &AnalysisResult,
    timelines: &[DetectorTimeline],
) -> std::io::Result<()> {
    use std::collections::BTreeSet;
    // Full executed-atlas vocabulary (sorted ⇒ deterministic) for computing the silent complements.
    let all_detectors: BTreeSet<&str> = timelines.iter().map(|t| t.detector_id.as_str()).collect();
    let all_families: BTreeSet<String> = timelines
        .iter()
        .map(|t| t.family.label().to_string())
        .collect();
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_start",
        "episode_end",
        "step_count",
        "matched",
        "heuristic_id",
        "evidence_grade",
        "consensus_strength",
        "disagreement_entropy",
        "dominant_motif",
        "families_fired",
        "families_silent",
        "detectors_fired",
        "detectors_silent",
    ])?;
    for (i, e) in res.fused_episodes.iter().enumerate() {
        let fired_fam: BTreeSet<&str> = e.families.iter().map(|s| s.as_str()).collect();
        let silent_fam: Vec<&str> = all_families
            .iter()
            .map(|s| s.as_str())
            .filter(|f| !fired_fam.contains(f))
            .collect();
        let fired_det: BTreeSet<&str> = e
            .participating_detectors
            .iter()
            .map(|s| s.as_str())
            .collect();
        let silent_det: Vec<&str> = all_detectors
            .iter()
            .copied()
            .filter(|d| !fired_det.contains(d))
            .collect();
        let (matched, hid) = res
            .heuristic_labels
            .get(i)
            .map(|l| (l.matched, l.heuristic_id.as_str()))
            .unwrap_or((false, "UNKNOWN"));
        w.write_record([
            dataset,
            &e.start_index.to_string(),
            &e.end_index.to_string(),
            &e.step_count.to_string(),
            &matched.to_string(),
            hid,
            evidence_grade(e),
            &fmt(e.consensus_strength),
            &fmt(e.disagreement_entropy),
            e.dominant_motif.token(),
            &e.families.join("|"),
            &silent_fam.join("|"),
            &e.participating_detectors.join("|"),
            &silent_det.join("|"),
        ])?;
    }
    w.flush()
}

// ── NAMUR NE 107 device-status mapping (operator legibility; not a compliance claim) ──────────
/// Map a DSFB grammar state to a NAMUR NE 107 condensed status category. Follows the paper's
/// standards-alignment mapping; SlewSpike is treated as Out-of-specification (a sharp transient
/// excursion) and Recovery as Maintenance-required (a watch state returning to nominal). A
/// presentation mapping only.
pub fn ne107_status(state: GrammarState) -> &'static str {
    match state {
        GrammarState::Nominal => "OK",
        GrammarState::DriftAccum | GrammarState::BoundaryGrazing | GrammarState::Recovery => {
            "Maintenance required"
        }
        GrammarState::SlewSpike => "Out of specification",
        GrammarState::EnvViolation | GrammarState::Compound => "Failure",
        GrammarState::SensorFault => "Function check",
    }
}

/// Severity rank to pick the plant-wide worst status at a sample (a reporting convention, not an
/// NE 107-defined ordering).
fn ne107_severity(status: &str) -> u8 {
    match status {
        "Failure" => 4,
        "Function check" => 3,
        "Out of specification" => 2,
        "Maintenance required" => 1,
        _ => 0, // OK
    }
}

/// Write only the per-sample plant-wide NE 107 status trace to an explicit `path` (the worst device
/// status across all detectors at each sample). Extracted from [`write_ne107`] so the Chemical Court
/// Record bundle can emit exactly `ne107_status_trace.csv` without the companion episode-summary JSON.
pub fn write_ne107_status_trace(
    path: &Path,
    dataset: &str,
    timelines: &[DetectorTimeline],
) -> std::io::Result<()> {
    let n = timelines.iter().map(|t| t.steps.len()).max().unwrap_or(0);
    let mut tw = csv::Writer::from_path(path)?;
    tw.write_record([
        "dataset",
        "time_index",
        "ne107_status",
        "n_detectors_non_ok",
    ])?;
    for i in 0..n {
        let mut worst = "OK";
        let mut non_ok = 0usize;
        for t in timelines {
            if let Some(s) = t.steps.get(i) {
                let st = ne107_status(s.state);
                if st != "OK" {
                    non_ok += 1;
                }
                if ne107_severity(st) > ne107_severity(worst) {
                    worst = st;
                }
            }
        }
        tw.write_record([dataset, &i.to_string(), worst, &non_ok.to_string()])?;
    }
    tw.flush()
}

/// Write the NAMUR NE 107 views: a per-sample plant-wide status trace (`ne107_status_trace.csv` —
/// the worst device-status category across all detectors at each sample) and a per-episode summary
/// (`ne107_episode_summary.json` — the status implied by each fused episode's dominant motif).
/// Presentation mapping only (see `ne107_status`); no compliance is claimed.
pub fn write_ne107(
    dir: &Path,
    dataset: &str,
    res: &AnalysisResult,
    timelines: &[DetectorTimeline],
) -> std::io::Result<()> {
    write_ne107_status_trace(&dir.join("ne107_status_trace.csv"), dataset, timelines)?;

    #[derive(Serialize)]
    struct Ne107Episode<'a> {
        start_index: usize,
        end_index: usize,
        dominant_motif: &'a str,
        ne107_status: &'a str,
        families: &'a [String],
        n_detectors: usize,
    }
    #[derive(Serialize)]
    struct Ne107Summary<'a> {
        dataset: &'a str,
        episodes: Vec<Ne107Episode<'a>>,
    }
    let episodes = res
        .fused_episodes
        .iter()
        .map(|e| Ne107Episode {
            start_index: e.start_index,
            end_index: e.end_index,
            dominant_motif: e.dominant_motif.token(),
            ne107_status: ne107_status(e.dominant_motif),
            families: &e.families,
            n_detectors: e.participating_detectors.len(),
        })
        .collect();
    let summary = Ne107Summary { dataset, episodes };
    fs::write(
        dir.join("ne107_episode_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )
}

/// Deterministic, advisory-only suggested operator disposition for an episode. Carries no control
/// authority; it is a triage hint derived from the match result and unknown subtype.
fn suggested_disposition(l: &HeuristicLabel) -> String {
    if l.matched {
        return format!("investigate (candidate: {})", l.episode_label);
    }
    match l.unknown_subtype.as_deref() {
        Some("UNKNOWN_STRUCTURAL_UNMAPPED") => "review: candidate new pattern".into(),
        Some("UNKNOWN_DETECTOR_CONFLICT") => "review: detector disagreement".into(),
        Some("UNKNOWN_OUT_OF_BANK_DOMAIN") => "review: evidence outside heuristic bank".into(),
        Some("UNKNOWN_WEAK_QUORUM") => "monitor: thin support".into(),
        Some("UNKNOWN_SHORT_TRANSIENT") => "monitor: brief transient".into(),
        _ => "review".into(),
    }
}

/// Write `alarm_rationalization.csv`: the ISA-18.2 / IEC 62682 flood-reduction artifact. Raw detector
/// breaches (alarms) collapse into fused episodes (rationalised alarms); each row is one rationalised
/// episode with its dominant motif, NE 107 status, participating detectors, label or unknown subtype,
/// evidence grade, a deterministic advisory disposition, and the evidence root (replay hash) sealing
/// the dataset. The dataset-level raw→episode compression is repeated per row for spreadsheet ease.
/// Operator-facing; the disposition carries no control authority.
pub fn write_alarm_rationalization(
    path: &Path,
    dataset: &str,
    res: &AnalysisResult,
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_start",
        "episode_end",
        "raw_breach_steps_total",
        "fused_episodes_total",
        "compression_ratio",
        "dominant_motif",
        "ne107_status",
        "participating_detectors",
        "label_or_unknown_subtype",
        "evidence_grade",
        "suggested_disposition",
        "evidence_root",
    ])?;
    let raw = res.compression.raw_breach_steps.to_string();
    let nfe = res.metrics.fused_episodes.to_string();
    let cr = fmt(res.metrics.episode_compression_ratio);
    for (i, e) in res.fused_episodes.iter().enumerate() {
        let l = res.heuristic_labels.get(i);
        let label_or_sub = match l {
            Some(l) if l.matched => l.heuristic_id.clone(),
            Some(l) => l
                .unknown_subtype
                .clone()
                .unwrap_or_else(|| "UNKNOWN".into()),
            None => "UNKNOWN".into(),
        };
        let disp = l
            .map(suggested_disposition)
            .unwrap_or_else(|| "review".into());
        w.write_record([
            dataset,
            &e.start_index.to_string(),
            &e.end_index.to_string(),
            &raw,
            &nfe,
            &cr,
            e.dominant_motif.token(),
            ne107_status(e.dominant_motif),
            &e.participating_detectors.join("|"),
            &label_or_sub,
            evidence_grade(e),
            &disp,
            &res.replay_hash,
        ])?;
    }
    w.flush()
}

/// Minimal HTML escape for the few free-text fields embedded in the operator report. The strings
/// come from the committed corpus (no external input), but escaping `&<>` keeps the output valid.
fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Write a static, operator-readable HTML case file for one dataset: headline metrics, the fused
/// episodes with NE 107 status / evidence grade / label, and an explicit "what DSFB does not claim"
/// footer. Generated deterministically from the `AnalysisResult`.
pub fn write_operator_report_html(
    path: &Path,
    dataset: &str,
    res: &AnalysisResult,
) -> std::io::Result<()> {
    let m = &res.metrics;
    let delay = m
        .detection_delay
        .map(|d| d.to_string())
        .unwrap_or_else(|| "n/a (unlabelled)".into());
    let mut rows = String::new();
    for (i, e) in res.fused_episodes.iter().enumerate() {
        let l = res.heuristic_labels.get(i);
        let matched = l.map(|l| l.matched).unwrap_or(false);
        let label = l
            .map(|l| {
                if l.matched {
                    l.episode_label.clone()
                } else {
                    l.unknown_subtype
                        .clone()
                        .unwrap_or_else(|| "UNKNOWN".into())
                }
            })
            .unwrap_or_else(|| "UNKNOWN".into());
        // P83: each row carries its physical-witness rung so a reader sees statistical-vs-physics grounding.
        let witness = episode_witness_strength(e, matched);
        let witness_cell = format!(
            "{} ({})",
            witness.tag(),
            if witness.is_physical() {
                "physics"
            } else {
                "statistical"
            }
        );
        // P85: the typed EvidenceKind for this episode (what the evidence *is*), derived from its witness rung
        // via the single-source-of-truth mapping — so the legend's "evidence kind" axis now has a per-row value.
        let kind = EvidenceKind::from_witness_strength(witness);
        let evidence_kind = kind.tag();
        // P102: the per-row CLAIM STRENGTH (the tier this evidence kind is allowed to assert — derived from the
        // EvidenceKind via the single-source-of-truth map, never hand-picked), and a per-episode EVIDENCE ANCHOR:
        // a content digest over this episode's sealed fields so each row is independently citable. The anchor is
        // a *display/citation* value computed here at render time — it is NOT folded into the dataset `evidence_root`
        // (that seal is over the grammar+episode stream), so adding it does not move any frozen evidence hash.
        let claim = kind.claim_strength().tag();
        // Shared anchor fn (also used by `narration_context`) — byte-identical to the prior inline computation,
        // so the operator-report bytes (and every bundle_root) are unchanged.
        let anchor = episode_evidence_anchor(dataset, e, witness, evidence_kind, &label);
        rows.push_str(&format!(
            "<tr><td>{}–{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td><td class=anchor>{}…</td></tr>\n",
            e.start_index, e.end_index, e.dominant_motif.token(), ne107_status(e.dominant_motif),
            esc_html(&e.families.join(", ")), evidence_grade(e), esc_html(&witness_cell), evidence_kind, claim, esc_html(&label), &anchor[..12],
        ));
    }
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<title>DSFB-Chemical-Engineering operator report — {ds}</title>\
<style>body{{font-family:system-ui,sans-serif;max-width:900px;margin:2rem auto;color:#1d3557;line-height:1.5}}\
h1{{font-size:1.4rem}}table{{border-collapse:collapse;width:100%;font-size:.9rem}}\
th,td{{border:1px solid #ccc;padding:4px 8px;text-align:left}}th{{background:#1d3557;color:#fff}}\
.kv{{margin:.2rem 0}}.note{{background:#fff8e1;border:1px solid #e6a700;padding:.8rem;border-radius:4px;margin-top:1.5rem}}\
.banner{{background:#fdecea;border:2px solid #d55e00;color:#7a2e00;padding:.7rem 1rem;border-radius:5px;margin:.5rem 0 1rem;font-size:.92rem}}\
.banner b{{color:#7a2e00}}.legend{{font-size:.82rem;color:#555;margin:.3rem 0 0}}\
.anchor{{font-family:monospace;font-size:.8rem;color:#5a6b80}}\
code{{background:#eef;padding:1px 4px;border-radius:3px}}</style></head><body>\
<h1>DSFB-Chemical-Engineering — operator case file</h1>\
<div class=banner><b>ADVISORY · READ-ONLY · NO CONTROL OR SAFETY-INSTRUMENTED-FUNCTION AUTHORITY.</b> \
Every label below is a <b>CANDIDATE</b> — a structural-evidence hint, <b>not a proven root cause</b>; \
unmatched episodes are preserved as “unknown structural episode”. Removing DSFB restores the \
pre-deployment baseline exactly. \
<p class=legend><b>Reading the evidence — four axes, each row honest about its own strength:</b><br>\
<b>Claim strength:</b> <code>SealedFact</code> (Tier 1, byte-exact replay-verified — the evidence root, \
episode counts) · <code>EvidenceInterpretation</code> (Tier 2, advisory candidate labels &amp; motifs) · \
<code>SpeculativeImplication</code> (Tier 3, bounded newly-shown effects) · <code>NonClaim</code> (root \
cause / causality / accuracy superiority / regulatory compliance — never asserted).<br>\
<b>Witness strength</b> (how physically grounded, weakest→strongest): PrecedentSimilarityOnly &lt; \
HeuristicPatternOnly &lt; DetectorFamilyQuorum &lt; ControlActionConsistent &lt; \
<b>TopologyResidenceAligned</b> &lt; <b>BalanceClosure</b> — the last two (bold) are first-principles \
physics, the rest statistical/structural. Episodes here sit at <b>DetectorFamilyQuorum</b> unless a \
balance/topology witness corroborates, so a heuristic motif can never masquerade as physical proof.<br>\
<b>Evidence kind:</b> what the evidence <i>is</i> — physical balance, first-principles equation, chemometric \
detector, heuristic pattern, precedent, controller/topology/data-quality context.<br>\
<b>Unknown route:</b> an episode matching no heuristic is never forced into a label — it is preserved and \
dispositioned into one of seven unknown classes (short-transient, detector-conflict, out-of-regime, \
residual-degeneracy, missing-context, non-stationary-baseline, insufficient-witness-diversity).<br>\
<b>Claim strength &amp; evidence anchor</b> (per row): each episode now shows the <i>claim-strength tier</i> it is \
allowed to assert — derived from its evidence kind via the single source of truth, never hand-picked — and an \
<i>evidence anchor</i>, a per-episode content digest (first 12 hex) so each row is independently citable. The \
dataset-level evidence root below remains the cryptographic seal; the anchor is a citation handle, not a new seal.</p></div>\
<p class=kv><b>Dataset:</b> {ds} &nbsp; <b>Provenance:</b> {kind}</p>\
<p class=kv><b>Samples:</b> {ns} ({nb} baseline) &nbsp; <b>Detectors executed:</b> {nd}</p>\
<h2>What happened</h2>\
<p class=kv>Raw detector breaches: <b>{raw}</b> &rarr; rationalised into <b>{fe}</b> fused episodes \
(compression {cr}&times;). Detection delay vs. labelled onset: <b>{delay}</b> samples \
(negative = at/before onset). Unknown rate: <b>{ur:.0}%</b> (episodes preserved, not forced).</p>\
<h2>Episodes</h2>\
<table><tr><th>Samples</th><th>Dominant motif</th><th>NE 107</th><th>Families firing</th>\
<th>Evidence grade</th><th>Witness strength</th><th>Evidence kind</th><th>Claim strength</th>\
<th>Label / unknown subtype</th><th>Evidence anchor</th></tr>\n{rows}</table>\
<h2>Evidence root (replay-verified)</h2>\
<p class=kv><code>{hash}</code> &nbsp; replay deterministic: <b>{det}</b></p>\
<div class=note><b>What DSFB does not claim.</b> This is an advisory, read-only evidence record. \
It does not assert proven root cause, superior detection accuracy, or regulatory compliance; every \
label is a candidate, and unmatched episodes are preserved as “unknown structural episode \
(evidence preserved)”. It emits no control signal and has no safety authority.</div>\
</body></html>",
        ds = esc_html(dataset),
        kind = esc_html(&res.data_kind),
        ns = m.n_samples, nb = m.n_baseline, nd = m.n_executed_detectors,
        raw = res.compression.raw_breach_steps, fe = m.fused_episodes,
        cr = fmt(m.episode_compression_ratio), delay = delay, ur = m.unknown_rate * 100.0,
        rows = rows, hash = res.replay_hash, det = m.replay_deterministic,
    );
    fs::write(path, html)
}

/// Write `residual_provenance_ledger.csv`: a provenance record for each executed detector's residual
/// stream. Per detector: the family/model class, the variable scope observed, the control-limit
/// threshold, the baseline window used to calibrate it, and a SHA-256 `source_hash` over the
/// detector's own (quantised) signed-margin stream — so each residual is traceable to the exact
/// stream that produced it. Pipeline-wide policies (z-score standardisation, percentile limits) are
/// constants documented in the paper, not per-detector, so they are not repeated per row.
pub fn write_residual_provenance(
    path: &Path,
    dataset: &str,
    timelines: &[DetectorTimeline],
    n_base: usize,
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "detector_id",
        "family_class",
        "variable_scope",
        "threshold",
        "baseline_start",
        "baseline_end",
        "n_samples",
        "source_hash",
    ])?;
    for t in timelines {
        let scope = t
            .outputs
            .first()
            .map(|o| o.variable_scope.as_str())
            .unwrap_or("");
        let threshold = t.outputs.first().map(|o| o.threshold).unwrap_or(f64::NAN);
        let mut h = CanonicalHasher::new();
        h.field("detector", t.detector_id.as_bytes());
        for o in &t.outputs {
            h.f64q("margin", o.signed_margin);
        }
        let source_hash = h.finalize_hex();
        w.write_record([
            dataset,
            &t.detector_id,
            t.family.label(),
            scope,
            &fmt(threshold),
            "0",
            &n_base.to_string(),
            &t.outputs.len().to_string(),
            &source_hash,
        ])?;
    }
    w.flush()
}

/// Write `non_admission.csv`: the counterfactual record of near-episodes fusion declined to admit
/// (see `fusion::non_admitted_runs`). Each row is evidence that was present but did not clear the
/// quorum/duration thresholds, with the reason — proving the court records refused claims, not only
/// admitted ones.
pub fn write_non_admission(
    path: &Path,
    dataset: &str,
    runs: &[NonAdmission],
) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "start_index",
        "end_index",
        "step_count",
        "peak_families",
        "required_families",
        "min_steps",
        "reason",
    ])?;
    for r in runs {
        w.write_record([
            dataset,
            &r.start_index.to_string(),
            &r.end_index.to_string(),
            &r.step_count.to_string(),
            &r.peak_families.to_string(),
            &r.required_families.to_string(),
            &r.min_steps.to_string(),
            r.reason,
        ])?;
    }
    w.flush()
}

/// Write `contribution_traces.csv`: per fused episode, the `top_k` variables ranked by mean absolute
/// standardised deviation (|z|) over the episode window, against the same baseline mean/std the
/// pipeline uses. A per-variable contribution *witness* — which variables moved most during the
/// episode — admitted as evidence, NOT as causal proof. This is a standardised-deviation contribution,
/// not a PCA-decomposed SPE/T² contribution; richer decompositions are left to downstream review.
pub fn write_contribution_traces(
    path: &Path,
    dataset: &str,
    res: &AnalysisResult,
    m: &DataMatrix,
    n_base: usize,
    top_k: usize,
) -> std::io::Result<()> {
    let baseline = Baseline::fit(m, n_base);
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_start",
        "episode_end",
        "rank",
        "variable",
        "mean_abs_z",
    ])?;
    if m.n_samples == 0 {
        return w.flush();
    }
    for e in &res.fused_episodes {
        let lo = e.start_index.min(m.n_samples - 1);
        let hi = e.end_index.min(m.n_samples - 1);
        let len = (hi - lo + 1) as f64;
        let mut acc = vec![0.0f64; m.n_vars];
        for i in lo..=hi {
            let z = baseline.standardise(m.row(i));
            for (v, zv) in z.iter().enumerate() {
                if zv.is_finite() {
                    acc[v] += zv.abs();
                }
            }
        }
        // Rank variables by mean |z| (descending); ties broken by variable index for determinism.
        let mut ranked: Vec<(usize, f64)> = acc.iter().map(|a| a / len).enumerate().collect();
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        for (rank, (v, mz)) in ranked.iter().take(top_k).enumerate() {
            let name = m.var_names.get(*v).map(|s| s.as_str()).unwrap_or("?");
            w.write_record([
                dataset,
                &e.start_index.to_string(),
                &e.end_index.to_string(),
                &(rank + 1).to_string(),
                name,
                &fmt(*mz),
            ])?;
        }
    }
    w.flush()
}

/// Write the per-dataset replay-hash records as `replay_hashes.json`.
///
/// Each entry contains the dataset name, the SHA-256 hex digest of the grammar+episode stream,
/// and a boolean confirming that two independent hash calls agreed within the same run
/// (`replay_deterministic`). Readers can verify a later re-run by comparing the `replay_hash`
/// field against the re-computed hash.
pub fn write_replay_hashes(path: &Path, results: &[AnalysisResult]) -> std::io::Result<()> {
    #[derive(Serialize)]
    struct Rec<'a> {
        dataset: &'a str,
        replay_hash: &'a str,
        replay_deterministic: bool,
    }
    let recs: Vec<Rec> = results
        .iter()
        .map(|r| Rec {
            dataset: &r.dataset,
            replay_hash: &r.replay_hash,
            replay_deterministic: r.metrics.replay_deterministic,
        })
        .collect();
    fs::write(path, serde_json::to_string_pretty(&recs).unwrap())
}

/// Serialise a full `AnalysisResult` to pretty-printed JSON.
///
/// This is the most detailed artifact: it includes every metric field, all fused episodes, all
/// heuristic labels, and the replay hash. Suitable for programmatic consumption by downstream
/// analysis scripts.
pub fn write_result_json(path: &Path, res: &AnalysisResult) -> std::io::Result<()> {
    fs::write(path, serde_json::to_string_pretty(res).unwrap())
}

/// Write a human-readable Markdown run summary (`report.md`).
///
/// Contains a summary table (one row per dataset) plus a "Non-claims" section that explicitly
/// states what this tool does not assert. The non-claims section is reproduced in the paper to
/// avoid over-claiming: the grammar and fusion layers are augmentation layers over established
/// estimators, not replacements or improvements in detection accuracy.
pub fn write_summary_md(path: &Path, results: &[AnalysisResult]) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str("# DSFB-Chemical-Engineering — Edge Run Summary\n\n");
    s.push_str(
        "Read-only residual semiotics over the chemometric detector atlas. All numbers below\n",
    );
    s.push_str("are produced by this run and are reproducible via the recorded replay hashes.\n\n");
    s.push_str("| Dataset | Kind | Samples | Vars | PCs | Detectors | Fused episodes | Compression | Unknown rate | Replay |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for r in results {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {:.2}x | {:.0}% | {} |\n",
            r.dataset,
            r.data_kind,
            r.metrics.n_samples,
            r.metrics.n_vars,
            r.pca_components,
            r.metrics.n_executed_detectors,
            r.metrics.fused_episodes,
            r.metrics.episode_compression_ratio,
            r.metrics.unknown_rate * 100.0,
            if r.metrics.replay_deterministic {
                "OK"
            } else {
                "FAIL"
            },
        ));
    }
    // The non-claims section is part of the honest reporting contract: this tool does not claim
    // to outperform established chemometrics, and does not force a diagnosis when uncertain.
    s.push_str("\n## Non-claims\n\n");
    s.push_str("- This is augmentation, not competition: no claim of higher accuracy or faster detection than established chemometrics.\n");
    s.push_str(
        "- Residual motifs suggest structural candidates; they do not prove physical root cause.\n",
    );
    s.push_str("- The correct failure mode is to emit \"unknown structural episode (evidence preserved)\" rather than force a diagnosis.\n");
    fs::write(path, s)
}

/// Format a float for CSV: 6 decimal places for finite values, `"nan"` otherwise.
///
/// Non-finite values (including `±∞`) are rendered as the string `"nan"` to avoid producing
/// invalid CSV that confuses pandas/numpy readers which accept `"nan"` but not `"inf"`.
/// Signed zero is canonicalised (`"-0.000000"` → `"0.000000"`) so a near-zero value whose sign flips
/// across platforms cannot change the emitted bytes; every non-zero value is unchanged.
fn fmt(v: f64) -> String {
    if v.is_finite() {
        let s = format!("{:.6}", v);
        if s == "-0.000000" {
            "0.000000".to_string()
        } else {
            s
        }
    } else {
        "nan".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Minimal dependency-free store-only ZIP writer (with CRC-32) for artifact bundles.
//
// Implements only what is needed: PKZIP local-file-header + data + central-directory
// entry + end-of-central-directory record. No compression (method = 0, store).
// A fixed mod-date sentinel (0x21, near DOS date 1980-01-01) is used instead of
// the real timestamp so that re-bundling the same artifacts produces byte-for-byte
// identical ZIPs (important for deterministic artifact hashing).
// ─────────────────────────────────────────────────────────────────────────────

/// CRC-32 (ISO 3309 / ITU-T V.42 polynomial 0xEDB88320, reflected).
/// Standard checksum required by the ZIP format in every local and central entry.
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            // Bit-by-bit table-less CRC via the reflected polynomial.
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Encode a `u16` as two little-endian bytes (ZIP spec requires LE throughout).
fn le16(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}
/// Encode a `u32` as four little-endian bytes.
fn le32(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

/// Bundle a list of `(archive_name, file_path)` pairs into a store-only ZIP at `zip_path`.
///
/// Each entry is written as:
/// 1. A PKZIP local file header (signature `0x04034b50`).
/// 2. The raw file bytes (no compression; method = 0).
/// 3. A corresponding central-directory entry accumulated in `central`.
///
/// After all entries, the central directory and end-of-central-directory record are appended.
/// `offset` tracks the running byte position of each local header within the file, which the
/// central directory records as a relative offset from the start of the archive.
///
/// The mod-date field is set to the fixed sentinel `0x21` (≈ DOS 1980-01-01) rather than the
/// real date, so ZIP bytes are deterministic across runs. File content bytes are unchanged.
pub fn zip_store(zip_path: &Path, entries: &[(String, PathBuf)]) -> std::io::Result<()> {
    let mut out = fs::File::create(zip_path)?;
    let mut central = Vec::new(); // accumulates central-directory entries
    let mut offset: u32 = 0; // byte offset of the current local file header
    for (name, path) in entries {
        let data = fs::read(path)?;
        let crc = crc32(&data);
        let nb = name.as_bytes();
        // ── Local file header (PKZIP spec §4.3.7) ────────────────────────────
        let mut lfh = Vec::new();
        lfh.extend_from_slice(&le32(0x0403_4b50)); // local file header signature
        lfh.extend_from_slice(&le16(20)); // version needed to extract (2.0)
        lfh.extend_from_slice(&le16(0)); // general purpose bit flag
        lfh.extend_from_slice(&le16(0)); // compression method: 0 = store
        lfh.extend_from_slice(&le16(0)); // last mod file time (zeroed)
        lfh.extend_from_slice(&le16(0x21)); // last mod file date (fixed sentinel)
        lfh.extend_from_slice(&le32(crc)); // CRC-32
        lfh.extend_from_slice(&le32(data.len() as u32)); // compressed size (= uncompressed)
        lfh.extend_from_slice(&le32(data.len() as u32)); // uncompressed size
        lfh.extend_from_slice(&le16(nb.len() as u16)); // file name length
        lfh.extend_from_slice(&le16(0)); // extra field length
        lfh.extend_from_slice(nb); // file name bytes
        out.write_all(&lfh)?;
        out.write_all(&data)?;
        // ── Central directory entry (PKZIP spec §4.3.12) ─────────────────────
        central.extend_from_slice(&le32(0x0201_4b50)); // central dir signature
        central.extend_from_slice(&le16(20)); // version made by
        central.extend_from_slice(&le16(20)); // version needed
        central.extend_from_slice(&le16(0)); // bit flag
        central.extend_from_slice(&le16(0)); // method: store
        central.extend_from_slice(&le16(0)); // mod time
        central.extend_from_slice(&le16(0x21)); // mod date (same sentinel)
        central.extend_from_slice(&le32(crc));
        central.extend_from_slice(&le32(data.len() as u32)); // compressed
        central.extend_from_slice(&le32(data.len() as u32)); // uncompressed
        central.extend_from_slice(&le16(nb.len() as u16));
        central.extend_from_slice(&le16(0)); // extra field length
        central.extend_from_slice(&le16(0)); // file comment length
        central.extend_from_slice(&le16(0)); // disk number start
        central.extend_from_slice(&le16(0)); // internal file attributes
        central.extend_from_slice(&le32(0)); // external file attributes
        central.extend_from_slice(&le32(offset)); // relative offset of local header
        central.extend_from_slice(nb);
        offset += lfh.len() as u32 + data.len() as u32;
    }
    let cd_offset = offset;
    let cd_size = central.len() as u32;
    out.write_all(&central)?;
    // ── End of central directory record (PKZIP spec §4.3.16) ─────────────────
    let mut eocd = Vec::new();
    eocd.extend_from_slice(&le32(0x0605_4b50)); // EOCD signature
    eocd.extend_from_slice(&le16(0)); // disk number
    eocd.extend_from_slice(&le16(0)); // disk with central dir
    eocd.extend_from_slice(&le16(entries.len() as u16)); // entries on this disk
    eocd.extend_from_slice(&le16(entries.len() as u16)); // total entries
    eocd.extend_from_slice(&le32(cd_size)); // size of central dir
    eocd.extend_from_slice(&le32(cd_offset)); // offset of central dir
    eocd.extend_from_slice(&le16(0)); // comment length
    out.write_all(&eocd)?;
    Ok(())
}

/// Recursively collect all files under `dir` as `(archive_name, absolute_path)` pairs.
///
/// `archive_name` is the path of the file relative to `base`, with backslashes replaced by
/// forward slashes for cross-platform ZIP compatibility. Entries are sorted lexicographically
/// at each directory level so the returned Vec is in a deterministic order regardless of
/// filesystem readdir order. Directories themselves are not included — only files.
pub fn collect_files(dir: &Path, base: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        // Sort for deterministic ZIP entry order across filesystems.
        entries.sort();
        for p in entries {
            if p.is_dir() {
                out.extend(collect_files(&p, base));
            } else if let Ok(rel) = p.strip_prefix(base) {
                // ZIP paths must use forward slashes (PKZIP spec §4.4.17).
                out.push((rel.to_string_lossy().replace('\\', "/"), p.clone()));
            }
        }
    }
    out
}
