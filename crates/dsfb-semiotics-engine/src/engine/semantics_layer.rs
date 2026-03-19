use std::collections::BTreeSet;

use crate::engine::types::{
    AdmissibilityRequirement, CoordinatedResidualStructure, GrammarState, GrammarStatus,
    HeuristicBankEntry, HeuristicCandidate, HeuristicProvenance, HeuristicScopeConditions,
    SemanticDisposition, SemanticMatchResult, SyntaxCharacterization,
};

#[derive(Clone, Debug)]
struct GrammarEvidence {
    boundary_count: usize,
    violation_count: usize,
    regimes: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct CompatibilityAssessment {
    conflicts: Vec<String>,
    unresolved: Vec<String>,
}

pub fn retrieve_semantics(
    scenario_id: &str,
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
) -> SemanticMatchResult {
    let evidence = grammar_evidence(grammar);
    let mut candidates = heuristic_bank()
        .into_iter()
        .filter_map(|entry| evaluate_entry(&entry, syntax, &evidence, coordinated))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .entry
            .retrieval_priority
            .cmp(&left.entry.retrieval_priority)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.entry.heuristic_id.cmp(&right.entry.heuristic_id))
    });

    let selected_labels = candidates
        .iter()
        .map(|candidate| candidate.entry.motif_label.clone())
        .collect::<Vec<_>>();
    let selected_heuristic_ids = candidates
        .iter()
        .map(|candidate| candidate.entry.heuristic_id.clone())
        .collect::<Vec<_>>();
    let compatibility = compatibility_assessment(&candidates);
    let conflict_notes = compatibility
        .conflicts
        .iter()
        .chain(&compatibility.unresolved)
        .cloned()
        .collect::<Vec<_>>();
    let observation_limited = observation_support_is_limited(syntax, &evidence);
    let (disposition, compatibility_note, note) = if candidates.is_empty() {
        if observation_limited {
            (
                SemanticDisposition::Unknown,
                "No heuristic bank entry matched, and the sampled trajectory provided only limited structural evidence for conservative semantic retrieval.".to_string(),
                "Unknown is returned here because the observation shows weak admissibility interaction and limited radial or curvature structure. The bank is not forced to label low-evidence cases.".to_string(),
            )
        } else {
            (
                SemanticDisposition::Unknown,
                "No heuristic bank entry satisfied the constrained admissibility, scope, and regime checks.".to_string(),
                "Unknown is returned conservatively because the current typed bank does not cover the observed admissibility-qualified syntax under the configured evidence and regime constraints.".to_string(),
            )
        }
    } else if candidates.len() == 1 {
        (
            SemanticDisposition::Match,
            format!(
                "Single heuristic bank entry (`{}`) satisfied the constrained retrieval rules.",
                selected_heuristic_ids[0]
            ),
            "The returned motif remains an illustrative compatibility statement only. It is not a unique-cause diagnosis.".to_string(),
        )
    } else if compatibility.conflicts.is_empty() && compatibility.unresolved.is_empty() {
        (
            SemanticDisposition::CompatibleSet,
            format!(
                "CompatibleSet returned because `{}` matched and every pair is explicitly marked compatible in the typed bank.",
                selected_heuristic_ids.join("`, `")
            ),
            "The engine reports an explicitly compatible motif set only when every matched pair is marked compatible. The result remains non-exclusive and causally conservative.".to_string(),
        )
    } else {
        (
            SemanticDisposition::Ambiguous,
            format!(
                "Ambiguous returned because {} matched entries produced {} explicit conflicts and {} unresolved compatibility pairings.",
                candidates.len(),
                compatibility.conflicts.len(),
                compatibility.unresolved.len()
            ),
            "Ambiguity is explicit rather than silently resolved. The engine does not force a unique semantic label when matched heuristics conflict or when compatibility is not explicitly established.".to_string(),
        )
    };

    SemanticMatchResult {
        scenario_id: scenario_id.to_string(),
        disposition,
        motif_summary: format!(
            "syntax={}, outward={:.3}, inward={:.3}, monotonicity={:.3}, curvature_energy={:.3}, curvature_onset={:.3}, slew_spikes={}, spike_strength={:.3}, boundary_episodes={}, boundary_recoveries={}, violations={}, regimes={}",
            syntax.trajectory_label,
            syntax.outward_drift_fraction,
            syntax.inward_drift_fraction,
            syntax.aggregate_monotonicity,
            syntax.curvature_energy,
            syntax.curvature_onset_score,
            syntax.slew_spike_count,
            syntax.slew_spike_strength,
            syntax.boundary_grazing_episode_count,
            syntax.boundary_recovery_count,
            evidence.violation_count,
            if evidence.regimes.is_empty() {
                "none".to_string()
            } else {
                evidence.regimes.join("|")
            }
        ),
        candidates,
        selected_labels,
        selected_heuristic_ids,
        compatibility_note,
        conflict_notes,
        note,
    }
}

fn grammar_evidence(grammar: &[GrammarStatus]) -> GrammarEvidence {
    let boundary_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Boundary))
        .count();
    let violation_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Violation))
        .count();
    let regimes = grammar
        .iter()
        .map(|status| status.regime.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    GrammarEvidence {
        boundary_count,
        violation_count,
        regimes,
    }
}

fn heuristic_bank() -> Vec<HeuristicBankEntry> {
    vec![
        HeuristicBankEntry {
            heuristic_id: "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
            motif_label: "gradual degradation candidate".to_string(),
            short_label: "persistent_outward".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: Some(0.60),
                max_outward_drift_fraction: None,
                min_inward_drift_fraction: None,
                max_curvature_energy: Some(3.0e-9),
                min_curvature_energy: None,
                max_curvature_onset_score: Some(0.25),
                min_curvature_onset_score: None,
                min_directional_persistence: Some(0.65),
                min_sign_consistency: Some(0.60),
                min_channel_coherence: Some(0.55),
                min_aggregate_monotonicity: Some(0.72),
                min_slew_spike_count: None,
                min_slew_spike_strength: None,
                min_boundary_grazing_episodes: None,
                min_boundary_recovery_count: None,
                require_group_breach: false,
            },
            admissibility_requirements: AdmissibilityRequirement::Any,
            regime_tags: vec![
                "fixed".to_string(),
                "widening".to_string(),
                "regime_shifted".to_string(),
            ],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping from persistent outward drift syntax to a conservative degradation-style motif.".to_string(),
            },
            applicability_note: "Use only as an admissibility-qualified drift motif. It does not determine underlying physical cause.".to_string(),
            retrieval_priority: 90,
            compatible_with: vec![
                "H-BOUNDARY-GRAZING".to_string(),
                "H-COORDINATED-RISE".to_string(),
            ],
            incompatible_with: vec![
                "H-DISCRETE-EVENT".to_string(),
                "H-CURVATURE-RICH-TRANSITION".to_string(),
                "H-INWARD-CONTAINMENT".to_string(),
            ],
        },
        HeuristicBankEntry {
            heuristic_id: "H-DISCRETE-EVENT".to_string(),
            motif_label: "discrete event candidate".to_string(),
            short_label: "discrete_event".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: None,
                max_outward_drift_fraction: None,
                min_inward_drift_fraction: None,
                max_curvature_energy: None,
                min_curvature_energy: Some(2.0e-6),
                max_curvature_onset_score: None,
                min_curvature_onset_score: Some(0.20),
                min_directional_persistence: None,
                min_sign_consistency: None,
                min_channel_coherence: None,
                min_aggregate_monotonicity: None,
                min_slew_spike_count: Some(1),
                min_slew_spike_strength: Some(0.05),
                min_boundary_grazing_episodes: None,
                min_boundary_recovery_count: None,
                require_group_breach: false,
            },
            admissibility_requirements: AdmissibilityRequirement::Any,
            regime_tags: vec![
                "fixed".to_string(),
                "widening".to_string(),
                "regime_shifted".to_string(),
            ],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping from localized high-slew activity to an abrupt-event-like motif.".to_string(),
            },
            applicability_note: "Treat as an event-compatible motif only. Multiple abrupt or switching mechanisms may produce similar signatures.".to_string(),
            retrieval_priority: 85,
            compatible_with: vec!["H-CURVATURE-RICH-TRANSITION".to_string()],
            incompatible_with: vec![
                "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
                "H-INWARD-CONTAINMENT".to_string(),
            ],
        },
        HeuristicBankEntry {
            heuristic_id: "H-CURVATURE-RICH-TRANSITION".to_string(),
            motif_label: "curvature-rich transition candidate".to_string(),
            short_label: "curvature_transition".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: None,
                max_outward_drift_fraction: None,
                min_inward_drift_fraction: None,
                max_curvature_energy: None,
                min_curvature_energy: Some(4.0e-9),
                max_curvature_onset_score: None,
                min_curvature_onset_score: Some(0.15),
                min_directional_persistence: None,
                min_sign_consistency: None,
                min_channel_coherence: Some(0.30),
                min_aggregate_monotonicity: None,
                min_slew_spike_count: Some(1),
                min_slew_spike_strength: Some(0.01),
                min_boundary_grazing_episodes: None,
                min_boundary_recovery_count: None,
                require_group_breach: false,
            },
            admissibility_requirements: AdmissibilityRequirement::Any,
            regime_tags: vec![
                "fixed".to_string(),
                "widening".to_string(),
                "regime_shifted".to_string(),
            ],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping for residual trajectories whose interpretation is governed more by curvature than by monotone migration.".to_string(),
            },
            applicability_note: "Use when slew-rich structure is material. This remains a motif statement, not a validated mechanism classifier.".to_string(),
            retrieval_priority: 80,
            compatible_with: vec!["H-DISCRETE-EVENT".to_string()],
            incompatible_with: vec![
                "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
                "H-INWARD-CONTAINMENT".to_string(),
            ],
        },
        HeuristicBankEntry {
            heuristic_id: "H-BOUNDARY-GRAZING".to_string(),
            motif_label: "near-boundary operation candidate".to_string(),
            short_label: "boundary_grazing".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: None,
                max_outward_drift_fraction: Some(0.70),
                min_inward_drift_fraction: None,
                max_curvature_energy: Some(0.050),
                min_curvature_energy: None,
                max_curvature_onset_score: Some(0.45),
                min_curvature_onset_score: None,
                min_directional_persistence: None,
                min_sign_consistency: None,
                min_channel_coherence: None,
                min_aggregate_monotonicity: None,
                min_slew_spike_count: None,
                min_slew_spike_strength: None,
                min_boundary_grazing_episodes: Some(2),
                min_boundary_recovery_count: Some(1),
                require_group_breach: false,
            },
            admissibility_requirements: AdmissibilityRequirement::NoViolation,
            regime_tags: vec![
                "fixed".to_string(),
                "widening".to_string(),
                "tightening".to_string(),
                "regime_nominal".to_string(),
                "regime_shifted".to_string(),
            ],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping from repeated admissibility grazing without decisive breach to a near-boundary operating motif.".to_string(),
            },
            applicability_note: "Boundary grazing reflects operation relative to the configured envelope only. It is not a diagnosis of unsafe or failed hardware.".to_string(),
            retrieval_priority: 70,
            compatible_with: vec![
                "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
                "H-COORDINATED-RISE".to_string(),
            ],
            incompatible_with: vec!["H-INWARD-CONTAINMENT".to_string()],
        },
        HeuristicBankEntry {
            heuristic_id: "H-COORDINATED-RISE".to_string(),
            motif_label: "correlated degradation or common-mode disturbance candidate".to_string(),
            short_label: "coordinated_rise".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: Some(0.45),
                max_outward_drift_fraction: None,
                min_inward_drift_fraction: None,
                max_curvature_energy: None,
                min_curvature_energy: None,
                max_curvature_onset_score: Some(0.40),
                min_curvature_onset_score: None,
                min_directional_persistence: Some(0.45),
                min_sign_consistency: Some(0.45),
                min_channel_coherence: Some(0.55),
                min_aggregate_monotonicity: Some(0.45),
                min_slew_spike_count: None,
                min_slew_spike_strength: None,
                min_boundary_grazing_episodes: None,
                min_boundary_recovery_count: None,
                require_group_breach: true,
            },
            admissibility_requirements: AdmissibilityRequirement::Any,
            regime_tags: vec!["aggregate".to_string()],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping from coordinated envelope-relative rise across grouped residuals to a common-mode motif.".to_string(),
            },
            applicability_note: "Use only for grouped residual structures with explicit aggregate envelopes. It does not identify the shared latent cause uniquely.".to_string(),
            retrieval_priority: 88,
            compatible_with: vec![
                "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
                "H-BOUNDARY-GRAZING".to_string(),
            ],
            incompatible_with: vec!["H-INWARD-CONTAINMENT".to_string()],
        },
        HeuristicBankEntry {
            heuristic_id: "H-INWARD-CONTAINMENT".to_string(),
            motif_label: "inward-compatible containment candidate".to_string(),
            short_label: "inward_containment".to_string(),
            scope_conditions: HeuristicScopeConditions {
                min_outward_drift_fraction: None,
                max_outward_drift_fraction: Some(0.35),
                min_inward_drift_fraction: Some(0.55),
                max_curvature_energy: Some(0.020),
                min_curvature_energy: None,
                max_curvature_onset_score: Some(0.25),
                min_curvature_onset_score: None,
                min_directional_persistence: Some(0.55),
                min_sign_consistency: Some(0.55),
                min_channel_coherence: Some(0.45),
                min_aggregate_monotonicity: None,
                min_slew_spike_count: None,
                min_slew_spike_strength: None,
                min_boundary_grazing_episodes: None,
                min_boundary_recovery_count: None,
                require_group_breach: false,
            },
            admissibility_requirements: AdmissibilityRequirement::NoViolation,
            regime_tags: vec![
                "fixed".to_string(),
                "tightening".to_string(),
                "regime_nominal".to_string(),
            ],
            provenance: HeuristicProvenance {
                source: "typed heuristic bank".to_string(),
                note: "Illustrative mapping from persistent inward-compatible evolution to a containment or recovery-style motif.".to_string(),
            },
            applicability_note: "This motif is admissibility-relative and does not assert underlying recovery physics.".to_string(),
            retrieval_priority: 75,
            compatible_with: Vec::new(),
            incompatible_with: vec![
                "H-PERSISTENT-OUTWARD-DRIFT".to_string(),
                "H-DISCRETE-EVENT".to_string(),
                "H-CURVATURE-RICH-TRANSITION".to_string(),
                "H-BOUNDARY-GRAZING".to_string(),
                "H-COORDINATED-RISE".to_string(),
            ],
        },
    ]
}

fn evaluate_entry(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> Option<HeuristicCandidate> {
    if !admissibility_satisfied(entry, evidence) {
        return None;
    }
    if !regime_satisfied(entry, evidence, coordinated) {
        return None;
    }
    if !scope_satisfied(entry, syntax, coordinated) {
        return None;
    }

    let available_regimes = available_regimes(evidence, coordinated);
    let matched_regimes = if entry.regime_tags.is_empty() {
        available_regimes.clone()
    } else {
        available_regimes
            .iter()
            .filter(|regime| entry.regime_tags.contains(*regime))
            .cloned()
            .collect::<Vec<_>>()
    };

    Some(HeuristicCandidate {
        entry: entry.clone(),
        score: score_candidate(entry, syntax, evidence, coordinated),
        rationale: rationale(entry, syntax, evidence, coordinated),
        matched_regimes,
    })
}

fn admissibility_satisfied(entry: &HeuristicBankEntry, evidence: &GrammarEvidence) -> bool {
    match entry.admissibility_requirements {
        AdmissibilityRequirement::Any => true,
        AdmissibilityRequirement::BoundaryInteraction => evidence.boundary_count > 0,
        AdmissibilityRequirement::ViolationRequired => evidence.violation_count > 0,
        AdmissibilityRequirement::NoViolation => evidence.violation_count == 0,
    }
}

fn regime_satisfied(
    entry: &HeuristicBankEntry,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> bool {
    let available = available_regimes(evidence, coordinated);
    entry.regime_tags.is_empty() || entry.regime_tags.iter().any(|tag| available.contains(tag))
}

fn scope_satisfied(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> bool {
    let scope = &entry.scope_conditions;
    if !min_ok(
        syntax.outward_drift_fraction,
        scope.min_outward_drift_fraction,
    ) {
        return false;
    }
    if !max_ok(
        syntax.outward_drift_fraction,
        scope.max_outward_drift_fraction,
    ) {
        return false;
    }
    if !min_ok(
        syntax.inward_drift_fraction,
        scope.min_inward_drift_fraction,
    ) {
        return false;
    }
    if !max_ok(syntax.curvature_energy, scope.max_curvature_energy) {
        return false;
    }
    if !min_ok(syntax.curvature_energy, scope.min_curvature_energy) {
        return false;
    }
    if !max_ok(
        syntax.curvature_onset_score,
        scope.max_curvature_onset_score,
    ) {
        return false;
    }
    if !min_ok(
        syntax.curvature_onset_score,
        scope.min_curvature_onset_score,
    ) {
        return false;
    }
    if !min_ok(
        syntax.directional_persistence,
        scope.min_directional_persistence,
    ) {
        return false;
    }
    if !min_ok(syntax.sign_consistency, scope.min_sign_consistency) {
        return false;
    }
    if !min_ok(syntax.channel_coherence, scope.min_channel_coherence) {
        return false;
    }
    if !min_ok(
        syntax.aggregate_monotonicity,
        scope.min_aggregate_monotonicity,
    ) {
        return false;
    }
    if !min_usize_ok(syntax.slew_spike_count, scope.min_slew_spike_count) {
        return false;
    }
    if !min_ok(syntax.slew_spike_strength, scope.min_slew_spike_strength) {
        return false;
    }
    if !min_usize_ok(
        syntax.boundary_grazing_episode_count,
        scope.min_boundary_grazing_episodes,
    ) {
        return false;
    }
    if !min_usize_ok(
        syntax.boundary_recovery_count,
        scope.min_boundary_recovery_count,
    ) {
        return false;
    }
    if scope.require_group_breach && !has_group_breach(coordinated) {
        return false;
    }
    true
}

fn score_candidate(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    _evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> f64 {
    let group_breach_ratio = coordinated_group_breach_ratio(coordinated);
    let score = match entry.heuristic_id.as_str() {
        "H-PERSISTENT-OUTWARD-DRIFT" => {
            0.28 * syntax.outward_drift_fraction
                + 0.24 * syntax.directional_persistence
                + 0.24 * syntax.aggregate_monotonicity
                + 0.12 * syntax.sign_consistency
                + 0.06 * (1.0 / (1.0 + 20.0 * syntax.curvature_energy))
                + 0.06 * (1.0 - syntax.curvature_onset_score)
        }
        "H-DISCRETE-EVENT" => {
            0.28 * (syntax.max_slew_norm / (syntax.max_slew_norm + 0.15))
                + 0.22 * (syntax.slew_spike_count.min(3) as f64 / 3.0)
                + 0.22 * (syntax.curvature_energy / (syntax.curvature_energy + 0.03))
                + 0.18 * (syntax.curvature_onset_score / (syntax.curvature_onset_score + 0.2))
                + 0.10 * (syntax.slew_spike_strength / (syntax.slew_spike_strength + 0.2))
        }
        "H-CURVATURE-RICH-TRANSITION" => {
            0.30 * (syntax.curvature_energy / (syntax.curvature_energy + 0.03))
                + 0.25 * syntax.curvature_onset_score
                + 0.15 * (syntax.slew_spike_count.min(3) as f64 / 3.0)
                + 0.10 * (syntax.slew_spike_strength / (syntax.slew_spike_strength + 0.2))
                + 0.10 * syntax.channel_coherence
                + 0.10 * (1.0 - syntax.aggregate_monotonicity)
        }
        "H-BOUNDARY-GRAZING" => {
            0.35 * (syntax.boundary_grazing_episode_count.min(4) as f64 / 4.0)
                + 0.20 * (syntax.boundary_recovery_count.min(4) as f64 / 4.0)
                + 0.20 * (1.0 / (1.0 + syntax.min_margin.abs() * 15.0))
                + 0.15 * (1.0 - syntax.outward_drift_fraction.clamp(0.0, 1.0))
                + 0.10 * (1.0 / (1.0 + 20.0 * syntax.curvature_energy))
        }
        "H-COORDINATED-RISE" => {
            0.38 * group_breach_ratio
                + 0.22 * syntax.outward_drift_fraction
                + 0.18 * syntax.channel_coherence
                + 0.22 * syntax.directional_persistence
        }
        "H-INWARD-CONTAINMENT" => {
            0.35 * syntax.inward_drift_fraction
                + 0.20 * syntax.directional_persistence
                + 0.20 * syntax.sign_consistency
                + 0.15 * (syntax.min_margin / (syntax.min_margin + 0.1)).clamp(0.0, 1.0)
                + 0.05 * (1.0 - syntax.outward_drift_fraction.clamp(0.0, 1.0))
                + 0.05 * (1.0 - syntax.curvature_onset_score)
        }
        _ => 0.0,
    };
    score.clamp(0.0, 1.0)
}

fn rationale(
    entry: &HeuristicBankEntry,
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> String {
    let group_breach = if has_group_breach(coordinated) {
        "group aggregate breach present"
    } else {
        "no aggregate group breach"
    };
    let matched_regimes = available_regimes(evidence, coordinated)
        .into_iter()
        .filter(|regime| entry.regime_tags.is_empty() || entry.regime_tags.contains(regime))
        .collect::<Vec<_>>();
    format!(
        "Applicable because admissibility=`{}`, matched_regimes=`{}`, and scope conditions were satisfied. {}. outward={:.3}, inward={:.3}, persistence={:.3}, residual_norm_monotonicity={:.3}, curvature_energy={:.3}, curvature_onset={:.3}, slew_spikes={}, spike_strength={:.3}, boundary_episodes={}, boundary_recoveries={}, violations={}, {}",
        admissibility_label(&entry.admissibility_requirements),
        if matched_regimes.is_empty() {
            "none".to_string()
        } else {
            matched_regimes.join("|")
        },
        entry.applicability_note,
        syntax.outward_drift_fraction,
        syntax.inward_drift_fraction,
        syntax.directional_persistence,
        syntax.aggregate_monotonicity,
        syntax.curvature_energy,
        syntax.curvature_onset_score,
        syntax.slew_spike_count,
        syntax.slew_spike_strength,
        syntax.boundary_grazing_episode_count,
        syntax.boundary_recovery_count,
        evidence.violation_count,
        group_breach,
    )
}

fn observation_support_is_limited(
    syntax: &SyntaxCharacterization,
    evidence: &GrammarEvidence,
) -> bool {
    syntax
        .outward_drift_fraction
        .max(syntax.inward_drift_fraction)
        < 0.35
        && syntax.directional_persistence < 0.35
        && syntax.sign_consistency < 0.35
        && syntax.curvature_onset_score < 0.15
        && syntax.slew_spike_count == 0
        && syntax.boundary_grazing_episode_count == 0
        && evidence.boundary_count == 0
        && evidence.violation_count == 0
}

fn admissibility_label(requirement: &AdmissibilityRequirement) -> &'static str {
    match requirement {
        AdmissibilityRequirement::Any => "any",
        AdmissibilityRequirement::BoundaryInteraction => "boundary-interaction",
        AdmissibilityRequirement::ViolationRequired => "violation-required",
        AdmissibilityRequirement::NoViolation => "no-violation",
    }
}

fn compatibility_assessment(candidates: &[HeuristicCandidate]) -> CompatibilityAssessment {
    let mut conflicts = Vec::new();
    let mut unresolved = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let left = &candidates[i].entry;
            let right = &candidates[j].entry;
            if left.incompatible_with.contains(&right.heuristic_id)
                || right.incompatible_with.contains(&left.heuristic_id)
            {
                conflicts.push(format!(
                    "{} conflicts with {} under the bank compatibility rules.",
                    left.motif_label, right.motif_label
                ));
            } else if !left.compatible_with.contains(&right.heuristic_id)
                || !right.compatible_with.contains(&left.heuristic_id)
            {
                unresolved.push(format!(
                    "{} and {} both matched, but the bank does not mark the pair as explicitly compatible.",
                    left.motif_label, right.motif_label
                ));
            }
        }
    }
    CompatibilityAssessment {
        conflicts,
        unresolved,
    }
}

fn available_regimes(
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> Vec<String> {
    let mut regimes = evidence.regimes.iter().cloned().collect::<BTreeSet<_>>();
    if coordinated.is_some() {
        regimes.insert("aggregate".to_string());
    }
    regimes.into_iter().collect()
}

fn has_group_breach(coordinated: Option<&CoordinatedResidualStructure>) -> bool {
    coordinated
        .map(|structure| {
            structure
                .points
                .iter()
                .any(|point| point.aggregate_margin < 0.0)
        })
        .unwrap_or(false)
}

fn coordinated_group_breach_ratio(coordinated: Option<&CoordinatedResidualStructure>) -> f64 {
    match coordinated {
        Some(structure) if !structure.points.is_empty() => {
            structure
                .points
                .iter()
                .filter(|point| point.aggregate_margin < 0.0)
                .count() as f64
                / structure.points.len() as f64
        }
        _ => 0.0,
    }
}

fn min_ok(value: f64, minimum: Option<f64>) -> bool {
    minimum
        .map(|minimum| value + 1.0e-9 >= minimum)
        .unwrap_or(true)
}

fn max_ok(value: f64, maximum: Option<f64>) -> bool {
    maximum
        .map(|maximum| value <= maximum + 1.0e-9)
        .unwrap_or(true)
}

fn min_usize_ok(value: usize, minimum: Option<usize>) -> bool {
    minimum.map(|minimum| value >= minimum).unwrap_or(true)
}
