//! Chemical Court Record v1 — the canonical, versioned, citable evidence bundle.
//!
//! DSFB-Chemical-Engineering **does not emit an alarm. It emits a court record of why an
//! alarm-like structure was or was not admitted.** This module turns the otherwise-scattered
//! per-dataset artifacts into a single named directory, `dsfb_chemical_engineering_casefile_v1/`,
//! with a self-describing manifest and a bundle hash, so the output is a *thing people can cite*
//! rather than a loose pile of CSVs.
//!
//! ## What the bundle contains (exactly these files)
//!
//! | File | Role |
//! |---|---|
//! | `casefile.json` | manifest: format id+version, evidence root, counts, per-episode badges, per-file SHA-256, bundle root |
//! | `admitted_episodes.csv` | the episodes quorum fusion admitted, each with a claim-boundary badge + evidence kind |
//! | `detector_witnesses.csv` | per episode × detector: which witnesses fired vs stayed silent |
//! | `rejected_candidates.csv` | near-episodes fusion examined and refused, with a rejection reason |
//! | `unknown_taxonomy.csv` | each UNKNOWN episode placed in one of five deterministic classes |
//! | `residual_provenance.csv` | the residual-provenance ledger ([`crate::report::write_residual_provenance`]) |
//! | `ne107_status_trace.csv` | per-sample NAMUR NE 107 plant-status trace ([`crate::report::write_ne107_status_trace`]) |
//! | `alarm_rationalization.csv` | ISA-18.2 flood→episode rationalisation ([`crate::report::write_alarm_rationalization`]) |
//! | `operator_report.html` | static operator case file ([`crate::report::write_operator_report_html`]) |
//! | `evidence_root.txt` | one line: the byte-exact replay/evidence root hash |
//! | `non_claims.md` | the bounded non-claims statement + per-run badge summary |
//!
//! ## Determinism
//!
//! Every file is a pure function of the [`AnalysisResult`] + detector timelines (no timestamps, no
//! environment). The manifest hashes the ten content files in sorted-name order and folds them into
//! one `bundle_root`. Two runs over the same analysis therefore produce a byte-identical bundle, and
//! the court-record fields are **not** part of `canonical_replay_hash`, so adding this bundle leaves
//! the sealed Tier-1 replay digest untouched.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;

use crate::dsfb_core::GrammarState;
use crate::evidence_kind::EvidenceKind;
use crate::fusion::{non_admitted_runs, DetectorTimeline, FusedEpisode, FusionConfig};
use crate::heuristics::HeuristicLabel;
use crate::pipeline::AnalysisResult;
use crate::report;

/// Frozen format identifier — also the bundle directory name. Bump only on a breaking schema change.
pub const CASEFILE_FORMAT: &str = "dsfb_chemical_engineering_casefile_v1";
/// Numeric schema version, recorded in the manifest alongside the format id.
pub const CASEFILE_FORMAT_VERSION: u32 = 1;

/// The ten content files (everything except `casefile.json`), in the fixed sorted order the manifest
/// hashes them in. Keeping this list explicit is what lets a test assert the bundle is *exactly* this.
pub const CONTENT_FILES: [&str; 10] = [
    "admitted_episodes.csv",
    "alarm_rationalization.csv",
    "detector_witnesses.csv",
    "evidence_root.txt",
    "ne107_status_trace.csv",
    "non_claims.md",
    "operator_report.html",
    "rejected_candidates.csv",
    "residual_provenance.csv",
    "unknown_taxonomy.csv",
];

/// The one-sentence framing the whole artifact converges toward.
pub const KEY_STATEMENT: &str =
    "It does not emit an alarm. It emits a court record of why an alarm-like structure was or was not admitted.";

// ─────────────────────────────────────────────────────────────────────────────
// Claim-boundary badges — every admitted episode carries exactly one primary badge,
// so the output is self-bounding and overclaiming is mechanically hard.
// ─────────────────────────────────────────────────────────────────────────────

/// Per-episode claim-boundary badge. The point: a reader can see, from the badge alone, the
/// *strongest* thing the evidence is allowed to say about an episode — never "root cause."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ClaimBoundaryBadge {
    /// Structure is present but no bank rule matched — preserved, not labelled.
    StructureOnly,
    /// A heuristic matched: a *candidate* fault pattern (never asserted as root cause).
    CandidateFault,
    /// Boundary-grazing dominant motif: structured motion that did not fully breach.
    NearMiss,
    /// A non-finite/sensor-fault motif dominates: treat as a sensor-quality question first.
    SensorQuality,
    /// Detectors conflict; resolving needs control/operating context not in the evidence.
    ControlContextRequired,
    /// Evidence falls outside the heuristic bank; a physics/balance witness is needed.
    PhysicsWitnessRequired,
    /// Global: the court never admits a root-cause claim (carried at the bundle level).
    RootCauseNotAdmitted,
    /// Global: the evidence root replayed byte-identically (carried at the bundle level).
    ReplayVerified,
}

impl ClaimBoundaryBadge {
    /// Stable UPPER_SNAKE token used in CSV/JSON output.
    pub fn token(&self) -> &'static str {
        match self {
            ClaimBoundaryBadge::StructureOnly => "STRUCTURE_ONLY",
            ClaimBoundaryBadge::CandidateFault => "CANDIDATE_FAULT",
            ClaimBoundaryBadge::NearMiss => "NEAR_MISS",
            ClaimBoundaryBadge::SensorQuality => "SENSOR_QUALITY",
            ClaimBoundaryBadge::ControlContextRequired => "CONTROL_CONTEXT_REQUIRED",
            ClaimBoundaryBadge::PhysicsWitnessRequired => "PHYSICS_WITNESS_REQUIRED",
            ClaimBoundaryBadge::RootCauseNotAdmitted => "ROOT_CAUSE_NOT_ADMITTED",
            ClaimBoundaryBadge::ReplayVerified => "REPLAY_VERIFIED",
        }
    }
}

/// Assign the single primary badge for an episode from its motif + heuristic outcome.
///
/// Priority is deliberate and deterministic: a sensor-fault motif is a sensor-quality question
/// before anything else; a matched rule is at most a *candidate*; a boundary-grazing motif that
/// nonetheless formed an episode is a near-miss; otherwise the UNKNOWN subtype routes to the kind of
/// context still missing (control vs. physics); the residual case is bare structure.
pub fn primary_badge(e: &FusedEpisode, l: Option<&HeuristicLabel>) -> ClaimBoundaryBadge {
    if e.dominant_motif == GrammarState::SensorFault {
        return ClaimBoundaryBadge::SensorQuality;
    }
    if matches!(l, Some(l) if l.matched) {
        return ClaimBoundaryBadge::CandidateFault;
    }
    if e.dominant_motif == GrammarState::BoundaryGrazing {
        return ClaimBoundaryBadge::NearMiss;
    }
    match l.and_then(|l| l.unknown_subtype.as_deref()) {
        Some("UNKNOWN_DETECTOR_CONFLICT") => ClaimBoundaryBadge::ControlContextRequired,
        Some("UNKNOWN_OUT_OF_BANK_DOMAIN") => ClaimBoundaryBadge::PhysicsWitnessRequired,
        _ => ClaimBoundaryBadge::StructureOnly,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Rejection vocabulary — why a near-episode was NOT admitted.
// ─────────────────────────────────────────────────────────────────────────────

/// Controlled vocabulary for the `rejection_reason` column of `rejected_candidates.csv`.
///
/// The vocabulary is split deliberately into two groups, documented here so the mapping below is not
/// surprising:
///
/// * **Currently emitted.** Only [`RejectionReason::QuorumNotMet`] is produced today. Fusion's two raw
///   non-admission mechanisms — *insufficient co-firing families* and *quorum met but too short* — are
///   both failures of the single combined "≥K families for ≥`min_steps`" quorum rule, so they share
///   this one reason. The distinct raw mechanism is always preserved verbatim in the separate
///   `raw_reason` column, so the merge loses nothing.
/// * **Reserved (schema v1).** The remaining variants are disclosed vocabulary, each tied to a specific
///   context the framework does not yet wire into the *rejection* path. They are listed in
///   [`RejectionReason::RESERVED`] with the context that will emit them, so a reader knows they are
///   forthcoming, not dead. (Several of these contexts already drive *admitted*-episode badges — e.g.
///   sensor-quality and physics-witness — via [`primary_badge`]; reserving them here records the intent
///   to surface the same context as a non-admission reason once it gates fusion.)
///
/// Keeping the full vocabulary in the v1 schema is intentional: it is disclosed prior art for the
/// rejection taxonomy, independent of which reasons the current fusion stage happens to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RejectionReason {
    /// EMITTED: the combined K-families-for-`min_steps` quorum rule was not met.
    QuorumNotMet,
    /// RESERVED — emitted once a batch-phase column gates fusion: evidence fell in an unlabelled phase.
    PhaseContextMissing,
    /// RESERVED — emitted once control-action logs gate fusion: the manipulated variable explains the residual.
    ControlActionExplainsResidual,
    /// RESERVED — emitted once sensor-quality gating runs in the rejection path: the residual is a sensor artefact.
    SensorQualitySuspect,
    /// RESERVED — emitted once a balance witness gates fusion: no closeable mass/energy balance is available.
    MassBalanceNotAvailable,
    /// RESERVED — emitted once disagreement-entropy gating runs: detectors conflict too strongly to admit.
    DetectorConflictTooHigh,
    /// RESERVED — emitted once degeneracy gating runs: the residual stream is constant/degenerate.
    ResidualDegenerate,
    /// RESERVED — emitted once heuristic-FP screening runs: a known false-positive mode is present.
    HeuristicFalsePositiveModePresent,
}

impl RejectionReason {
    /// Stable UPPER_SNAKE token written to the CSV. Every variant has one (the reserved variants are
    /// disclosed in the schema even though fusion does not emit them yet).
    pub fn token(&self) -> &'static str {
        match self {
            RejectionReason::QuorumNotMet => "QUORUM_NOT_MET",
            RejectionReason::PhaseContextMissing => "PHASE_CONTEXT_MISSING",
            RejectionReason::ControlActionExplainsResidual => "CONTROL_ACTION_EXPLAINS_RESIDUAL",
            RejectionReason::SensorQualitySuspect => "SENSOR_QUALITY_SUSPECT",
            RejectionReason::MassBalanceNotAvailable => "MASS_BALANCE_NOT_AVAILABLE",
            RejectionReason::DetectorConflictTooHigh => "DETECTOR_CONFLICT_TOO_HIGH",
            RejectionReason::ResidualDegenerate => "RESIDUAL_DEGENERATE",
            RejectionReason::HeuristicFalsePositiveModePresent => {
                "HEURISTIC_FALSE_POSITIVE_MODE_PRESENT"
            }
        }
    }

    /// The reserved-but-not-yet-emitted vocabulary (schema v1). Listing them here both documents the
    /// forthcoming reasons and references every variant, so the grouping is explicit rather than a set
    /// of silently-unreachable enum arms. `QuorumNotMet` is deliberately absent (it is the emitted one).
    pub const RESERVED: &'static [RejectionReason] = &[
        RejectionReason::PhaseContextMissing,
        RejectionReason::ControlActionExplainsResidual,
        RejectionReason::SensorQualitySuspect,
        RejectionReason::MassBalanceNotAvailable,
        RejectionReason::DetectorConflictTooHigh,
        RejectionReason::ResidualDegenerate,
        RejectionReason::HeuristicFalsePositiveModePresent,
    ];
}

/// Map a raw fusion non-admission token to the controlled vocabulary.
///
/// Fusion (`fusion::non_admitted_runs`) emits exactly two raw tokens today, both of which are quorum
/// failures (see [`RejectionReason`]). The wildcard arm is a *defensive* fallback for an unrecognised
/// token (which the current fusion stage never produces); it is **not** a stand-in for the reserved
/// vocabulary — those reasons are emitted only once their gating context is wired into fusion.
fn map_rejection(raw: &str) -> RejectionReason {
    match raw {
        "insufficient_families" => RejectionReason::QuorumNotMet, // too few co-firing detector families
        "too_short" => RejectionReason::QuorumNotMet, // families met, but run shorter than min_steps
        _ => RejectionReason::QuorumNotMet, // defensive: unknown token from a future fusion change
    }
}

/// Deterministic operator action for each UNKNOWN taxonomy class (advisory; no authority).
fn unknown_action(subtype: &str) -> &'static str {
    match subtype {
        "UNKNOWN_SHORT_TRANSIENT" => "monitor: brief transient, no action unless it recurs",
        "UNKNOWN_OUT_OF_BANK_DOMAIN" => "extend the heuristic bank / consult a domain expert; evidence is outside the catalogued vocabulary",
        "UNKNOWN_DETECTOR_CONFLICT" => "investigate the detector disagreement; check operating-regime/control context",
        "UNKNOWN_WEAK_QUORUM" => "watch: quorum met but thin support",
        "UNKNOWN_STRUCTURAL_UNMAPPED" => "escalate for diagnosis: sustained, consistent structure with no matching rule",
        _ => "review",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Manifest types (serialised to casefile.json).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct EpisodeBadge {
    episode_index: usize,
    start_index: usize,
    end_index: usize,
    badge: &'static str,
    /// (P93) The typed `EvidenceKind` tag for this episode — *what kind* of evidence backs it (physical balance vs
    /// chemometric detector vs heuristic vs precedent), derived from the episode's physical-witness rung via the
    /// single-source-of-truth mapping (same as the operator report). A display/classification field for legibility;
    /// it is NOT lane evidence, so it does not enter `evidence_root` (only the manifest → `bundle_root`).
    evidence_kind: &'static str,
}

#[derive(Serialize)]
struct FileDigest {
    name: &'static str,
    sha256: String,
}

#[derive(Serialize)]
struct Counts {
    admitted_episodes: usize,
    rejected_candidates: usize,
    unknown_episodes: usize,
    near_miss: usize,
    candidate_fault: usize,
}

#[derive(Serialize)]
struct CaseFileManifest {
    format: &'static str,
    format_version: u32,
    dataset: String,
    data_kind: String,
    software_version: &'static str,
    backend: &'static str,
    /// The byte-exact replay/evidence root (edge `canonical_replay_hash`).
    evidence_root: String,
    replay_deterministic: bool,
    counts: Counts,
    /// Bundle-level claim boundaries that hold for every episode.
    global_badges: Vec<&'static str>,
    episode_badges: Vec<EpisodeBadge>,
    files: Vec<FileDigest>,
    /// SHA-256 over the sorted `name:sha256` lines of the ten content files — one hash for the bundle.
    bundle_root: String,
    key_statement: &'static str,
}

// ─────────────────────────────────────────────────────────────────────────────
// Bundle writer.
// ─────────────────────────────────────────────────────────────────────────────

/// Write the full `dsfb_chemical_engineering_casefile_v1/` bundle under `out_dir`.
///
/// `out_dir` is the per-dataset directory; the bundle is created as a `CASEFILE_FORMAT`-named
/// subdirectory inside it. Returns the `bundle_root` hash on success.
///
/// # Examples
/// ```no_run
/// use std::path::Path;
/// use dsfb_chemical_engineering_edge::{analyze, DataMatrix, PipelineConfig};
/// use dsfb_chemical_engineering_edge::pipeline::timelines_for;
/// use dsfb_chemical_engineering_edge::court_record::write_court_record;
/// let m = DataMatrix::new(vec!["temp".into(), "flow".into()], vec![vec![0.0, 0.0]; 32]);
/// let res = analyze("demo", "synthetic", &m, 16, PipelineConfig::default());
/// let timelines = timelines_for("demo", &m, 16, PipelineConfig::default());
/// // Writes the dsfb_chemical_engineering_casefile_v1/ bundle and returns its 64-hex bundle_root.
/// let bundle_root = write_court_record(
///     Path::new("/tmp/demo_case"), &res, &timelines, 16, PipelineConfig::default().fusion,
/// )?;
/// assert_eq!(bundle_root.len(), 64);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn write_court_record(
    out_dir: &Path,
    res: &AnalysisResult,
    timelines: &[DetectorTimeline],
    n_base: usize,
    fusion_cfg: FusionConfig,
) -> io::Result<String> {
    let bundle = out_dir.join(CASEFILE_FORMAT);
    fs::create_dir_all(&bundle)?;
    let dataset = res.dataset.as_str();

    // 1. Reuse the existing, audited writers for the four artifacts that already exist verbatim.
    report::write_residual_provenance(
        &bundle.join("residual_provenance.csv"),
        dataset,
        timelines,
        n_base,
    )?;
    report::write_alarm_rationalization(&bundle.join("alarm_rationalization.csv"), dataset, res)?;
    report::write_operator_report_html(&bundle.join("operator_report.html"), dataset, res)?;
    report::write_ne107_status_trace(&bundle.join("ne107_status_trace.csv"), dataset, timelines)?;

    // 2. New court-record artifacts.
    write_admitted_episodes(&bundle.join("admitted_episodes.csv"), res)?;
    write_detector_witnesses(&bundle.join("detector_witnesses.csv"), res, timelines)?;
    let rejected = non_admitted_runs(timelines, fusion_cfg);
    write_rejected_candidates(&bundle.join("rejected_candidates.csv"), dataset, &rejected)?;
    write_unknown_taxonomy(&bundle.join("unknown_taxonomy.csv"), res)?;
    fs::write(
        bundle.join("evidence_root.txt"),
        format!("{}\n", res.replay_hash),
    )?;
    write_non_claims(&bundle.join("non_claims.md"), res)?;

    // 3. Manifest last: hash the ten content files (sorted-name order) and fold into bundle_root.
    let mut files = Vec::with_capacity(CONTENT_FILES.len());
    let mut root_input = String::new();
    for name in CONTENT_FILES {
        let bytes = fs::read(bundle.join(name))?;
        let sha = crate::hashing::sha256_hex(&bytes);
        root_input.push_str(name);
        root_input.push(':');
        root_input.push_str(&sha);
        root_input.push('\n');
        files.push(FileDigest { name, sha256: sha });
    }
    let bundle_root = crate::hashing::sha256_hex(root_input.as_bytes());

    let episode_badges: Vec<EpisodeBadge> = res
        .fused_episodes
        .iter()
        .enumerate()
        .map(|(i, e)| {
            // The episode's physical-witness rung → its EvidenceKind, mirroring the operator report exactly
            // (`matched` = a heuristic label actually matched; otherwise it sits at a weaker, generic-detector rung).
            let matched = res
                .heuristic_labels
                .get(i)
                .map(|l| l.matched)
                .unwrap_or(false);
            let witness = report::episode_witness_strength(e, matched);
            EpisodeBadge {
                episode_index: i,
                start_index: e.start_index,
                end_index: e.end_index,
                badge: primary_badge(e, res.heuristic_labels.get(i)).token(),
                evidence_kind: EvidenceKind::from_witness_strength(witness).tag(),
            }
        })
        .collect();
    let near_miss = episode_badges
        .iter()
        .filter(|b| b.badge == "NEAR_MISS")
        .count();
    let candidate_fault = episode_badges
        .iter()
        .filter(|b| b.badge == "CANDIDATE_FAULT")
        .count();

    let mut global_badges = vec![ClaimBoundaryBadge::RootCauseNotAdmitted.token()];
    if res.metrics.replay_deterministic {
        global_badges.push(ClaimBoundaryBadge::ReplayVerified.token());
    }

    let manifest = CaseFileManifest {
        format: CASEFILE_FORMAT,
        format_version: CASEFILE_FORMAT_VERSION,
        dataset: res.dataset.clone(),
        data_kind: res.data_kind.clone(),
        software_version: crate::VERSION,
        backend: "cpu-edge",
        evidence_root: res.replay_hash.clone(),
        replay_deterministic: res.metrics.replay_deterministic,
        counts: Counts {
            admitted_episodes: res.fused_episodes.len(),
            rejected_candidates: rejected.len(),
            unknown_episodes: res.metrics.unknown_episodes,
            near_miss,
            candidate_fault,
        },
        global_badges,
        episode_badges,
        files,
        bundle_root: bundle_root.clone(),
        key_statement: KEY_STATEMENT,
    };
    fs::write(
        bundle.join("casefile.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(bundle_root)
}

/// `admitted_episodes.csv` — the episodes fusion admitted, each carrying its claim-boundary badge.
fn write_admitted_episodes(path: &Path, res: &AnalysisResult) -> io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_index",
        "start_index",
        "end_index",
        "step_count",
        "dominant_motif",
        "ne107_status",
        "evidence_grade",
        "consensus_strength",
        "disagreement_entropy",
        "peak_drift",
        "peak_slew",
        "families",
        "participating_detectors",
        "matched",
        "heuristic_id",
        "label",
        "claim_boundary_badge",
        "evidence_kind",
    ])?;
    for (i, e) in res.fused_episodes.iter().enumerate() {
        let l = res.heuristic_labels.get(i);
        let (matched, hid, label) = match l {
            Some(l) if l.matched => (true, l.heuristic_id.clone(), l.episode_label.clone()),
            Some(l) => (
                false,
                "UNKNOWN".to_string(),
                l.unknown_subtype
                    .clone()
                    .unwrap_or_else(|| "UNKNOWN".into()),
            ),
            None => (false, "UNKNOWN".to_string(), "UNKNOWN".to_string()),
        };
        // evidence_kind: the episode's physical-witness rung → EvidenceKind, derived identically to the
        // casefile.json EpisodeBadge so the claim-audit CSV and the manifest never disagree.
        let evidence_kind =
            EvidenceKind::from_witness_strength(report::episode_witness_strength(e, matched)).tag();
        w.write_record([
            res.dataset.as_str(),
            &i.to_string(),
            &e.start_index.to_string(),
            &e.end_index.to_string(),
            &e.step_count.to_string(),
            e.dominant_motif.token(),
            report::ne107_status(e.dominant_motif),
            report::evidence_grade(e),
            &fmt6(e.consensus_strength),
            &fmt6(e.disagreement_entropy),
            &fmt6(e.peak_drift),
            &fmt6(e.peak_slew),
            &e.families.join("|"),
            &e.participating_detectors.join("|"),
            &matched.to_string(),
            &hid,
            &label,
            primary_badge(e, l).token(),
            evidence_kind,
        ])?;
    }
    w.flush()
}

/// `detector_witnesses.csv` — per episode × detector, who testified (fired) vs stayed silent.
///
/// "Firing" matches fusion's definition: a non-nominal, non-`SensorFault` grammar state. The silent
/// witnesses are as informative as the firing ones (e.g. SPE firing while T² is silent points to a
/// residual-subspace rather than score-space shift), which is why every executed detector gets a row.
fn write_detector_witnesses(
    path: &Path,
    res: &AnalysisResult,
    timelines: &[DetectorTimeline],
) -> io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_index",
        "episode_start",
        "episode_end",
        "detector_id",
        "family",
        "status",
        "steps_firing",
        "dominant_state",
        "peak_drift",
        "peak_slew",
    ])?;
    for (ei, e) in res.fused_episodes.iter().enumerate() {
        for t in timelines {
            let mut steps_firing = 0usize;
            let mut state_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
            let mut peak_drift = 0.0f64;
            let mut peak_slew = 0.0f64;
            for idx in e.start_index..=e.end_index {
                if let Some(s) = t.steps.get(idx) {
                    let firing = s.state.is_non_nominal() && s.state != GrammarState::SensorFault;
                    if firing {
                        steps_firing += 1;
                        *state_counts.entry(s.state.token()).or_insert(0) += 1;
                        peak_drift = peak_drift.max(s.triple.delta.abs());
                        peak_slew = peak_slew.max(s.triple.sigma.abs());
                    }
                }
            }
            // Dominant firing state: highest count, ties broken by token order (BTreeMap is sorted).
            let dominant = state_counts
                .iter()
                .max_by_key(|(_, c)| **c)
                .map(|(k, _)| *k)
                .unwrap_or("NOM");
            let status = if steps_firing > 0 { "firing" } else { "silent" };
            w.write_record([
                res.dataset.as_str(),
                &ei.to_string(),
                &e.start_index.to_string(),
                &e.end_index.to_string(),
                t.detector_id.as_str(),
                t.family.label(),
                status,
                &steps_firing.to_string(),
                dominant,
                &fmt6(peak_drift),
                &fmt6(peak_slew),
            ])?;
        }
    }
    w.flush()
}

/// `rejected_candidates.csv` — near-episodes fusion examined and refused, with the rejection reason.
fn write_rejected_candidates(
    path: &Path,
    dataset: &str,
    rejected: &[crate::fusion::NonAdmission],
) -> io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "candidate_index",
        "start_index",
        "end_index",
        "step_count",
        "peak_families",
        "required_families",
        "min_steps",
        "raw_reason",
        "rejection_reason",
        "missing_context",
        "evidence_hash",
    ])?;
    for (i, r) in rejected.iter().enumerate() {
        let reason = map_rejection(r.reason);
        let missing = match reason {
            RejectionReason::QuorumNotMet => {
                "needs an additional co-firing detector family or a longer sustained run"
            }
            _ => "context not available in this run",
        };
        // Deterministic per-candidate evidence hash over its canonical fields.
        let canon = format!(
            "{dataset}|{}|{}|{}|{}|{}",
            r.start_index, r.end_index, r.step_count, r.peak_families, r.reason
        );
        let evidence_hash = crate::hashing::sha256_hex(canon.as_bytes());
        w.write_record([
            dataset,
            &i.to_string(),
            &r.start_index.to_string(),
            &r.end_index.to_string(),
            &r.step_count.to_string(),
            &r.peak_families.to_string(),
            &r.required_families.to_string(),
            &r.min_steps.to_string(),
            r.reason,
            reason.token(),
            missing,
            &evidence_hash,
        ])?;
    }
    w.flush()
}

/// `unknown_taxonomy.csv` — each UNKNOWN episode placed in its deterministic class with an action.
fn write_unknown_taxonomy(path: &Path, res: &AnalysisResult) -> io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "dataset",
        "episode_index",
        "start_index",
        "end_index",
        "unknown_subtype",
        "operator_action",
        "challenge_docket",
    ])?;
    for (i, l) in res.heuristic_labels.iter().enumerate() {
        if l.matched {
            continue;
        }
        let subtype = l
            .unknown_subtype
            .clone()
            .unwrap_or_else(|| "UNKNOWN".into());
        let challenge = l.challenge.clone().unwrap_or_default();
        w.write_record([
            res.dataset.as_str(),
            &i.to_string(),
            &l.episode_start.to_string(),
            &l.episode_end.to_string(),
            &subtype,
            unknown_action(&subtype),
            &challenge,
        ])?;
    }
    w.flush()
}

/// `non_claims.md` — the bounded non-claims statement plus this run's badge summary.
fn write_non_claims(path: &Path, res: &AnalysisResult) -> io::Result<()> {
    // Deterministic badge tally (sorted token order).
    let mut tally: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (i, e) in res.fused_episodes.iter().enumerate() {
        *tally
            .entry(primary_badge(e, res.heuristic_labels.get(i)).token())
            .or_insert(0) += 1;
    }
    let mut summary = String::new();
    for (badge, n) in &tally {
        summary.push_str(&format!("- `{badge}`: {n}\n"));
    }
    if summary.is_empty() {
        summary.push_str("- (no admitted episodes)\n");
    }

    let md = format!(
        "# DSFB-Chemical-Engineering — non-claims for this case file\n\
\n\
**{key}**\n\
\n\
This case file (`{fmt_id}`) is **read-only evidence over residuals**. It does NOT:\n\
\n\
- assert a physical **root cause** — every label is a *candidate*, bounded by its claim-boundary badge;\n\
- replace PCA / PLS / MSPC / ML detectors, a controller, an estimator, a historian, or an alarm system;\n\
- write to any setpoint, alarm limit, historian tag, or control variable;\n\
- claim higher accuracy or faster detection than the incumbent detectors it reads.\n\
\n\
Admitted episodes carry a claim-boundary badge; UNKNOWN episodes are preserved with a taxonomy class\n\
(`unknown_taxonomy.csv`); refused candidates are recorded with a reason (`rejected_candidates.csv`).\n\
`ROOT_CAUSE_NOT_ADMITTED` applies to the entire case file.\n\
\n\
## Evidence root\n\
\n\
- replay/evidence root: `{root}`\n\
- replay deterministic: **{det}**\n\
\n\
## Claim-boundary badge summary (this run)\n\
\n\
{summary}",
        key = KEY_STATEMENT,
        fmt_id = CASEFILE_FORMAT,
        root = res.replay_hash,
        det = res.metrics.replay_deterministic,
        summary = summary,
    );
    fs::write(path, md)
}

/// Fixed 6-decimal formatting so numeric CSV columns are byte-stable across runs/platforms.
///
/// Canonicalises signed zero: a value that renders as `"-0.000000"` (negative zero, or a tiny negative
/// that rounds to zero at six decimals) is emitted as `"0.000000"`. Without this, a near-zero `signed_margin`
/// whose last-ULP sign flips across compilers would change the CSV bytes — and hence the `bundle_root`
/// that SHA-256s these files — even though the sealed `evidence_root`/`replay_hash` (which use the
/// quantised `f64q` path) stay identical. Every non-zero value is formatted exactly as before.
fn fmt6(v: f64) -> String {
    let s = format!("{v:.6}");
    if s == "-0.000000" {
        "0.000000".to_string()
    } else {
        s
    }
}
