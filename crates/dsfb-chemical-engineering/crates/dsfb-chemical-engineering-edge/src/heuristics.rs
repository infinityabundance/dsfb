//! Chemical heuristics bank — maps fused residual-episode motifs to operator-readable candidate
//! labels.
//!
//! # Design intent
//! The bank contains 6 heuristics (H1–H6) covering the most common chemical-process episode
//! patterns: sensor drift, valve stiction, thermal excursion, feed disturbance, batch
//! misalignment, and controller masking.  Each heuristic is a *structural rule* over the set of
//! detectors that participated in a fused episode plus scalar episode properties
//! (`peak_drift`, `peak_slew`, `consensus_strength`, `dominant_motif`).
//!
//! # IMPORTANT: candidate labels, not proven root causes
//! Every label produced by this bank is a **candidate** hypothesis for operator review, not a
//! proven diagnosis.  The engineering basis fields document the literature patterns each rule
//! encodes, but matching a rule does not constitute confirmation of the root cause.  The
//! `operator_explanation` text in each heuristic is deliberately hedged ("consistent with",
//! "candidate") to communicate this.
//!
//! # Matching semantics
//! Rules are evaluated in priority order (H1 first, H6 last).  The *first* matching rule wins;
//! ambiguous episodes fall through to the honest "unknown structural episode (evidence preserved)"
//! label.  No probability or confidence score is assigned to the match — the boolean
//! matched/unmatched distinction is intentional.
//!
//! # Determinism
//! Matching is purely functional over its inputs: no RNG, no external state, no I/O.  Given the
//! same `FusedEpisode` and `bank` slice the result is identical across runs.  The bank order
//! (`canonical_bank`) is fixed; reordering entries would change match outcomes.
//!
//! # Severity
//! The `severity` field is an advisory scalar in [0, 1] derived from `consensus_strength`,
//! `peak_drift`, and `peak_slew`.  It is informational only — it does not gate any trip or
//! alarm and carries no safety authority.

use crate::dsfb_core::GrammarState;
use crate::fusion::FusedEpisode;
use serde::{Deserialize, Serialize};

/// A single heuristic entry (the bank schema from the project plan).
///
/// Each field is a string-typed slot in the bank schema; the struct is intentionally transparent
/// so that the full provenance of each rule is visible in serialised reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heuristic {
    /// Short stable identifier, e.g. "H1".  Referenced by `label_episode` rules and downstream
    /// report fields; must not be changed without updating the matching rules.
    pub heuristic_id: String,
    /// Human-readable name for the candidate fault pattern.
    pub name: String,
    /// Process types for which this heuristic is meaningful; purely informational.
    pub applicable_process_type: String,
    /// Detector IDs whose participation in the episode is expected for this pattern.
    /// Used in the matching closure `has(id)` inside `label_episode`.
    pub detector_inputs: Vec<String>,
    /// Natural-language description of the residual pattern that motivates this rule.
    pub residual_pattern: String,
    /// Operator-facing episode label (always hedged — see module doc).
    pub episode_label: String,
    /// Explanation surfaced to the operator in the report; must use hedged language.
    pub operator_explanation: String,
    /// Documented conditions under which this rule may fire on a non-fault episode.
    pub known_false_positive_modes: String,
    /// Documented conditions under which a fault matching this pattern may *not* trigger the rule.
    pub known_false_negative_modes: String,
    /// Advisory action policy; never authorises automatic protective action.
    pub severity_policy: String,
    /// Literature / prior-art basis for the residual pattern.
    pub engineering_basis: String,
}

/// Result of matching a fused episode against the bank.
///
/// Produced by `label_episode` for each `FusedEpisode`; one label per episode.
/// When `matched = false` the `heuristic_id` is "UNKNOWN" and the `episode_label` is the
/// honest "unknown structural episode (evidence preserved)" message — no diagnosis is forced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicLabel {
    /// Zero-based start index of the episode in the original data matrix.
    pub episode_start: usize,
    /// Zero-based end index (inclusive) of the episode in the original data matrix.
    pub episode_end: usize,
    /// `true` if a bank heuristic was matched; `false` if the episode fell through to UNKNOWN.
    pub matched: bool,
    /// ID of the matched heuristic ("H1"–"H6") or "UNKNOWN".
    pub heuristic_id: String,
    /// Operator-facing label string from the matched `Heuristic`, or the UNKNOWN message.
    pub episode_label: String,
    /// Operator-facing explanation; always hedged (see module doc).
    pub operator_explanation: String,
    /// Advisory severity in [0, 1], derived from consensus_strength × f(peak_drift, peak_slew).
    /// This is NOT a probability and carries no engineering authority.  See `label_episode`.
    pub severity: f64,
    /// For UNKNOWN episodes, a computable uncertainty subtype (see `classify_unknown`); `None` for
    /// matched episodes. Limited to subtypes derivable from the episode's own features — context-
    /// dependent classes (phase mismatch, sensor-quality, control-action) are deferred, not guessed.
    pub unknown_subtype: Option<String>,
    /// For UNKNOWN episodes, the per-rule \emph{challenge docket}: the closest bank rule and each of
    /// its conditions with PASS/FAIL (see `challenge_docket`); `None` for matched episodes. Records,
    /// the way an engineer reasons, why the nearest diagnosis was refused.
    pub challenge: Option<String>,
}

/// Classify an UNKNOWN (unmatched) episode into an actionable uncertainty subtype, using only
/// features computable from the episode itself and the bank vocabulary — no context is invented.
/// Deterministic; first matching rule wins. Returns a stable token. Subtypes:
/// - `UNKNOWN_SHORT_TRANSIENT`    — `step_count <= 3`; a brief transient.
/// - `UNKNOWN_OUT_OF_BANK_DOMAIN` — no participating detector appears in any heuristic's
///   `detector_inputs`; the evidence is outside the bank vocabulary, so no rule could apply.
/// - `UNKNOWN_DETECTOR_CONFLICT`  — `disagreement_entropy > 1.0` nats; detectors disagree on form.
/// - `UNKNOWN_WEAK_QUORUM`        — `consensus_strength < 0.15`; quorum met but thin support.
/// - `UNKNOWN_STRUCTURAL_UNMAPPED`— default: a sustained, consistent, bank-relevant episode that
///   matches no rule (the genuine "new structure" bucket).
///
/// Thresholds are conservative documented defaults and are tunable; they do not affect replay
/// (heuristic labels are not part of the evidence digest).
fn classify_unknown(ep: &FusedEpisode, bank: &[Heuristic]) -> &'static str {
    if ep.step_count <= 3 {
        return "UNKNOWN_SHORT_TRANSIENT";
    }
    let in_bank = ep.participating_detectors.iter().any(|d| {
        bank.iter()
            .any(|h| h.detector_inputs.iter().any(|i| i == d))
    });
    if !in_bank {
        return "UNKNOWN_OUT_OF_BANK_DOMAIN";
    }
    if ep.disagreement_entropy > 1.0 {
        return "UNKNOWN_DETECTOR_CONFLICT";
    }
    if ep.consensus_strength < 0.15 {
        return "UNKNOWN_WEAK_QUORUM";
    }
    "UNKNOWN_STRUCTURAL_UNMAPPED"
}

/// Return the canonical six-heuristic chemical bank (H1–H6) as a fresh `Vec`.
///
/// The six heuristics, in priority order used by `label_episode`:
/// 1. **H1** Sensor bias / calibration drift candidate.
/// 2. **H2** Actuator stiction / valve stick-slip candidate.
/// 3. **H3** Reactor thermal-excursion candidate.
/// 4. **H4** Feed-composition disturbance candidate (propagating).
/// 5. **H5** Batch-phase misalignment candidate.
/// 6. **H6** Controller-compensation masking candidate.
///
/// Ordering matters: `label_episode` tries H1 first and returns on the first match.
/// Heuristics lower in the list only fire when the higher-priority structural conditions are
/// not met.  To change priority, reorder the rules in `label_episode` (not here).
pub fn canonical_bank() -> Vec<Heuristic> {
    vec![
        Heuristic {
            heuristic_id: "H1".into(),
            name: "Sensor bias / calibration drift candidate".into(),
            applicable_process_type: "continuous, batch, spectroscopic".into(),
            detector_inputs: vec!["sensor_bias".into(), "shewhart_max_z".into(), "robust_z_mad".into()],
            residual_pattern: "one variable drifts; correlated variables do not follow; PCA contribution dominated by one variable; balance residuals stable".into(),
            episode_label: "likely sensor bias / calibration drift".into(),
            operator_explanation: "A single channel is deviating while structurally-related channels stay nominal — consistent with instrument bias rather than a process upset.".into(),
            known_false_positive_modes: "a genuinely local process change affecting one measurement point; intermittent comms dropouts".into(),
            known_false_negative_modes: "simultaneous multi-sensor calibration drift presents as co-drift, not isolation".into(),
            severity_policy: "advisory; recommend cross-check against redundant instrument".into(),
            engineering_basis: "single-variable contribution isolation in MSPC contribution analysis".into(),
        },
        Heuristic {
            heuristic_id: "H2".into(),
            name: "Actuator stiction / valve stick-slip candidate".into(),
            applicable_process_type: "control loops, actuated valves".into(),
            detector_inputs: vec!["page_hinkley_spe".into(), "cusum_spe".into(), "spectral_entropy_spe".into()],
            residual_pattern: "residual slew snaps after a delay; oscillatory correction follows; spectral content shifts".into(),
            episode_label: "candidate actuator stiction / valve stick-slip".into(),
            operator_explanation: "An abrupt residual slew followed by oscillatory recovery and a spectral-entropy shift is consistent with valve stiction releasing under controller pressure.".into(),
            known_false_positive_modes: "legitimate setpoint changes; recipe transitions; aggressive controller tuning".into(),
            known_false_negative_modes: "slow stiction without limit-cycle oscillation may not produce a slew snap".into(),
            severity_policy: "advisory; correlate with valve position vs controller output".into(),
            engineering_basis: "stick-slip limit cycles in DAMADICS-class actuator FDI literature".into(),
        },
        Heuristic {
            heuristic_id: "H3".into(),
            name: "Reactor thermal-excursion candidate".into(),
            applicable_process_type: "reactors, exothermic processes".into(),
            detector_inputs: vec!["co_drift".into(), "ewma_spe".into(), "pca_t2".into()],
            residual_pattern: "temperature-group residual drift with co-drift of cooling/heating variables and rising slew".into(),
            episode_label: "candidate reactor thermal excursion".into(),
            operator_explanation: "Co-drifting thermal variables with accelerating slew are consistent with an emerging thermal excursion; cross-check conversion/quality proxies.".into(),
            known_false_positive_modes: "ambient/utility transients; planned ramp; feed preheat changes".into(),
            known_false_negative_modes: "fast runaway can outpace the drift window; very slow excursions look nominal".into(),
            severity_policy: "advisory; this carries NO trip authority and must not gate safety interlocks".into(),
            engineering_basis: "co-monotone thermal residual signatures in CSTR/TEP-class benchmarks".into(),
        },
        Heuristic {
            heuristic_id: "H4".into(),
            name: "Feed-composition disturbance candidate".into(),
            applicable_process_type: "continuous plantwide".into(),
            detector_inputs: vec!["co_drift".into(), "pca_spe_q".into(), "mann_kendall_spe".into()],
            residual_pattern: "upstream composition/flow residual breach; downstream contribution migration; detector agreement grows over propagation time".into(),
            episode_label: "candidate feed-composition disturbance (propagating)".into(),
            operator_explanation: "A residual that emerges upstream and recruits downstream detectors over time is consistent with a propagating feed-composition disturbance.".into(),
            known_false_positive_modes: "coordinated multi-unit setpoint moves; shared-utility disturbances".into(),
            known_false_negative_modes: "fast-mixing processes may not show a clear propagation delay".into(),
            severity_policy: "advisory; inspect upstream analyser and feed lineup".into(),
            engineering_basis: "fault-propagation contribution migration in plantwide MSPC".into(),
        },
        Heuristic {
            heuristic_id: "H5".into(),
            name: "Batch-phase misalignment candidate".into(),
            applicable_process_type: "batch, fed-batch, fermentation".into(),
            detector_inputs: vec!["pca_spe_q".into(), "psi_spe".into(), "ks_spe".into()],
            residual_pattern: "large residuals only at phase boundaries; phase-aligned envelope reduces anomaly".into(),
            episode_label: "candidate batch-phase misalignment".into(),
            operator_explanation: "Residuals concentrated at phase transitions, distributionally distinct from steady phases, are consistent with batch alignment/recipe-stage timing rather than a true fault.".into(),
            known_false_positive_modes: "genuine phase-localised faults; recipe edits".into(),
            known_false_negative_modes: "misalignment uniformly across a phase may not localise to boundaries".into(),
            severity_policy: "advisory; recommend phase-aligned re-evaluation before escalation".into(),
            engineering_basis: "multiway/phase-aligned PCA in batch process monitoring (e.g. IndPenSim)".into(),
        },
        Heuristic {
            heuristic_id: "H6".into(),
            name: "Controller-compensation masking candidate".into(),
            applicable_process_type: "closed-loop controlled processes".into(),
            detector_inputs: vec!["pca_t2".into(), "ewma_spe".into(), "co_drift".into()],
            residual_pattern: "controlled variable appears stable; manipulated-variable residual grows; latent residual changes before any raw limit breach".into(),
            episode_label: "candidate controller-compensation masking".into(),
            operator_explanation: "Latent (score-space) residual growth while the controlled variable stays in-band can indicate the control system is absorbing growing process stress — visible structurally before a raw alarm.".into(),
            known_false_positive_modes: "normal load-following; disturbance rejection within design envelope".into(),
            known_false_negative_modes: "open-loop or saturated actuators remove the masking signature".into(),
            severity_policy: "advisory; surface manipulated-variable headroom to operators".into(),
            engineering_basis: "score-space vs raw-space residual divergence under closed-loop control".into(),
        },
    ]
}

/// Deterministically match a fused episode to the first applicable heuristic (priority order).
///
/// # Algorithm
/// 1. Compute an advisory `severity` scalar from episode properties (see formula below).
/// 2. Test structural rules H1–H6 in order; return on the first match.
/// 3. If no rule matches, return the UNKNOWN label — evidence is preserved, no diagnosis forced.
///
/// # Severity formula
/// `severity = clamp(consensus_strength · (1 + min(peak_drift,5) + min(peak_slew,5)) / 11, 0, 1)`
/// - The denominator 11 normalises the range: maximum numerator is `1.0 · (1+5+5) = 11`.
/// - `peak_drift` and `peak_slew` are capped at 5 to prevent a single extreme sample from
///   dominating the score.
/// - This formula is *heuristic*; it has no statistical derivation — treat the result as
///   a relative ordering aid, not a calibrated probability.
///
/// # Matching rules (in priority order)
/// - **H1**: `sensor_bias` fires AND `co_drift` does NOT fire — single-variable isolation pattern.
/// - **H2**: `page_hinkley` + `spectral_entropy` both fire AND `peak_slew > 1.0` — slew + spectral shift.
/// - **H3**: `co_drift` + `ewma_spe` AND dominant motif is `DriftAccum` — sustained rising drift.
/// - **H4**: `co_drift` + `mann_kendall` AND at least 3 detector families represented — propagating trend.
/// - **H5**: `psi_spe` + `ks_spe` + `pca_spe_q` — distributional fingerprint consistent with phase misalignment.
/// - **H6**: `pca_t2` + `ewma_spe` + `co_drift` AND dominant motif is `BoundaryGrazing` — latent headroom loss under control.
///
/// Rules are deliberately conservative: requiring multiple detector signals reduces false positives
/// at the cost of some false negatives.  Ambiguous episodes fall through to UNKNOWN.
pub fn label_episode(ep: &FusedEpisode, bank: &[Heuristic]) -> HeuristicLabel {
    // Advisory severity: normalised product of consensus strength and episode intensity proxies.
    // The .min(5.0) caps on drift/slew prevent outlier samples from inflating the score.
    // Result is clamped to [0, 1]; treat as ordinal advisory only.
    let severity = (ep.consensus_strength * (1.0 + ep.peak_drift.min(5.0) + ep.peak_slew.min(5.0))
        / 11.0)
        .clamp(0.0, 1.0);

    // Look up a heuristic by ID in the bank slice (returns None if not found).
    let pick = |id: &str| bank.iter().find(|h| h.heuristic_id == id).cloned();

    // Evaluate every rule's conditions (priority order). A heuristic matches iff ALL its conditions
    // pass, so the first fully-passing rule wins --- identical to the prior priority-ordered if/else
    // chain, but now each condition is individually inspectable for the challenge docket below.
    let rules = evaluate_rules(ep);
    let matched = rules
        .iter()
        .find(|(_, conds)| conds.iter().all(|c| c.pass))
        .and_then(|(id, _)| pick(id));

    match matched {
        Some(h) => HeuristicLabel {
            episode_start: ep.start_index,
            episode_end: ep.end_index,
            matched: true,
            heuristic_id: h.heuristic_id,
            episode_label: h.episode_label,
            operator_explanation: h.operator_explanation,
            severity,
            unknown_subtype: None,
            challenge: None,
        },
        None => HeuristicLabel {
            episode_start: ep.start_index,
            episode_end: ep.end_index,
            matched: false,
            heuristic_id: "UNKNOWN".into(),
            episode_label: "unknown structural episode (evidence preserved)".into(),
            // The explanation is explicit that no diagnosis has been forced; the evidence
            // (fused episode + detector outputs) remains available for manual review.
            operator_explanation: "A fused structural episode was detected but did not match any bank heuristic. The underlying detector evidence is preserved for review; no diagnosis is forced.".into(),
            severity,
            unknown_subtype: Some(classify_unknown(ep, bank).to_string()),
            // Per-rule counterfactual: which bank rule came closest and which conditions failed.
            challenge: Some(challenge_docket(&rules)),
        },
    }
}

/// One condition within a heuristic rule, paired with whether the episode satisfies it.
struct RuleCheck {
    desc: &'static str,
    pass: bool,
}

/// Evaluate every bank rule's conditions against an episode, in priority order. A heuristic matches
/// iff all its conditions pass; this is the single authoritative encoding of the six rules, consumed
/// by both `label_episode` (the matcher) and `challenge_docket` (the explanation), so the two cannot
/// drift. The conditions reproduce the previous hand-written if/else chain exactly.
fn evaluate_rules(ep: &FusedEpisode) -> Vec<(&'static str, Vec<RuleCheck>)> {
    let has = |id: &str| ep.participating_detectors.iter().any(|d| d == id);
    let n_fams = ep.families.len();
    vec![
        (
            "H1",
            vec![
                RuleCheck {
                    desc: "sensor_bias fired",
                    pass: has("sensor_bias"),
                },
                RuleCheck {
                    desc: "co_drift absent",
                    pass: !has("co_drift"),
                },
            ],
        ),
        (
            "H2",
            vec![
                RuleCheck {
                    desc: "page_hinkley_spe fired",
                    pass: has("page_hinkley_spe"),
                },
                RuleCheck {
                    desc: "spectral_entropy_spe fired",
                    pass: has("spectral_entropy_spe"),
                },
                RuleCheck {
                    desc: "peak_slew > 1.0",
                    pass: ep.peak_slew > 1.0,
                },
            ],
        ),
        (
            "H3",
            vec![
                RuleCheck {
                    desc: "co_drift fired",
                    pass: has("co_drift"),
                },
                RuleCheck {
                    desc: "ewma_spe fired",
                    pass: has("ewma_spe"),
                },
                RuleCheck {
                    desc: "motif = DriftAccum",
                    pass: ep.dominant_motif == GrammarState::DriftAccum,
                },
            ],
        ),
        (
            "H4",
            vec![
                RuleCheck {
                    desc: "co_drift fired",
                    pass: has("co_drift"),
                },
                RuleCheck {
                    desc: "mann_kendall_spe fired",
                    pass: has("mann_kendall_spe"),
                },
                RuleCheck {
                    desc: ">=3 families",
                    pass: n_fams >= 3,
                },
            ],
        ),
        (
            "H5",
            vec![
                RuleCheck {
                    desc: "psi_spe fired",
                    pass: has("psi_spe"),
                },
                RuleCheck {
                    desc: "ks_spe fired",
                    pass: has("ks_spe"),
                },
                RuleCheck {
                    desc: "pca_spe_q fired",
                    pass: has("pca_spe_q"),
                },
            ],
        ),
        (
            "H6",
            vec![
                RuleCheck {
                    desc: "pca_t2 fired",
                    pass: has("pca_t2"),
                },
                RuleCheck {
                    desc: "ewma_spe fired",
                    pass: has("ewma_spe"),
                },
                RuleCheck {
                    desc: "co_drift fired",
                    pass: has("co_drift"),
                },
                RuleCheck {
                    desc: "motif = BoundaryGrazing",
                    pass: ep.dominant_motif == GrammarState::BoundaryGrazing,
                },
            ],
        ),
    ]
}

/// Build the per-rule challenge docket for an unmatched episode: the closest bank rule (most
/// conditions satisfied; ties broken by priority order) with each of its conditions marked PASS/FAIL.
/// This is the heuristic-level counterfactual --- it records *why* the nearest diagnosis was refused,
/// the way an experienced engineer reasons ("looked like H3, but the motif was wrong").
fn challenge_docket(rules: &[(&'static str, Vec<RuleCheck>)]) -> String {
    let mut best: Option<&(&'static str, Vec<RuleCheck>)> = None;
    let mut best_passes = 0usize;
    for r in rules {
        let passes = r.1.iter().filter(|c| c.pass).count();
        // Strict `>` keeps the first (highest-priority) rule on ties.
        if best.is_none() || passes > best_passes {
            best = Some(r);
            best_passes = passes;
        }
    }
    match best {
        Some((id, conds)) => {
            let detail: Vec<String> = conds
                .iter()
                .map(|c| format!("{} {}", c.desc, if c.pass { "PASS" } else { "FAIL" }))
                .collect();
            format!(
                "closest {id} ({best_passes}/{}): {}",
                conds.len(),
                detail.join("; ")
            )
        }
        None => String::new(),
    }
}
