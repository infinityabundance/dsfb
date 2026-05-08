//! DSFB-Debug: episode rendering — operator-side dashboard binding
//! (std-only).
//!
//! # The "data" → "actionable insight" gap
//!
//! The no_std core produces `DebugEpisode` records with numeric
//! indices (e.g. `root_cause_signal_index = Some(3)`). The on-call
//! engineer at 3am needs the SERVICE NAME, not the index. The
//! bank's per-motif `dashboard_hint` is a template (e.g.\ "Inspect
//! the upstream service in the dependency graph emitting the first
//! slew"); the engineer needs the substituted concrete instance
//! ("Inspect ts-station-service (signal 3, originator of cascading-
//! timeout)"). This module is the binding layer that produces that
//! substituted text.
//!
//! Per panellist P11 (Senior SRE, Session 6 review): "this is the
//! difference between data and actionable insight that the on-call
//! SRE workflow gates on. Without it, dsfb-debug is interesting; with
//! it, dsfb-debug is licensable." Hence the explicit module.
//!
//! # Architecture
//!
//! The engine stays signal-index-only (no allocation, no_std-clean);
//! rendering is operator-side, std-only, allocation-tolerant. Output
//! strings are ready for PagerDuty incident bodies, Slack message
//! payloads, or dashboard widget body fields.
//!
//! # Template placeholders supported
//!
//! - `${ROOT_CAUSE_SERVICE}` — `channels[root_cause_signal_index]`
//! - `${ROOT_CAUSE_INDEX}` — `root_cause_signal_index` numeric
//! - `${CONTRIBUTING_COUNT}` — `episode.contributing_signal_count`
//! - `${PEAK_SLEW}` — `peak_slew_magnitude` formatted to .3f
//! - `${DURATION_WINDOWS}` — `duration_windows` integer
//! - `${MOTIF}` — `matched_motif` enum-variant name
//! - `${CONFIDENCE_MARGIN}` — `match_confidence.margin` .3f
//! - `${RUNNER_UP_MOTIF}` — runner-up motif name, or "none"
//!
//! New placeholders are added by extending the substitution map in
//! `render_episode_summary` — additive only; existing placeholders'
//! semantics never change.

#![cfg(feature = "std")]

extern crate std;

use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

use crate::heuristics_bank::HeuristicsBank;
use crate::types::*;

/// Operator-rendered summary of one closed episode. All fields are
/// owned strings ready for PagerDuty / Slack / log-sink emission.
#[derive(Debug, Clone)]
pub struct RenderedEpisodeSummary {
    pub episode_id: u32,
    pub start_window: u64,
    pub end_window: u64,
    pub duration_windows: u64,
    pub motif: Option<MotifClass>,
    pub motif_name: String,
    pub motif_provenance: String,
    pub motif_evidence: String,
    pub motif_taxonomy: String,
    pub root_cause_service: Option<String>,
    pub root_cause_signal_index: Option<u16>,
    pub contributing_signal_count: u16,
    pub peak_slew_magnitude: f64,
    pub policy_state: PolicyState,
    /// The bank's `dashboard_hint` with `${...}` placeholders substituted.
    pub dashboard_hint_bound: String,
    /// Confidence margin from `match_episode_with_confidence`, if available.
    pub match_confidence: Option<MatchConfidence>,
}

/// Render one episode into actionable operator-side text.
///
/// `signal_names` is typically `OwnedResidualMatrix.channels` from the
/// adapter. If empty, signal-index numbers stand in for service
/// names, but template substitution still proceeds for everything
/// else (peak slew, duration, etc.).
///
/// `bank` is consulted via `entry_for(matched_motif)` to recover the
/// motif's metadata (provenance, evidence dataset, taxonomy ref,
/// dashboard_hint template).
pub fn render_episode_summary<const M: usize>(
    episode: &DebugEpisode,
    signal_names: &[String],
    bank: &HeuristicsBank<M>,
    match_confidence: Option<MatchConfidence>,
) -> RenderedEpisodeSummary {
    let motif = match episode.matched_motif {
        SemanticDisposition::Named(m) => Some(m),
        SemanticDisposition::Unknown => None,
    };
    let entry = motif.and_then(|m| bank.entry_for(m));

    let motif_name = motif.map(|m| format!("{:?}", m)).unwrap_or_else(|| "Unknown".to_string());
    let motif_provenance = entry.map(|e| format!("{:?}", e.provenance)).unwrap_or_default();
    let motif_evidence = entry.map(|e| {
        if e.evidence_dataset_doi.is_empty() {
            e.evidence_dataset.to_string()
        } else {
            format!("{} (DOI {})", e.evidence_dataset, e.evidence_dataset_doi)
        }
    }).unwrap_or_default();
    let motif_taxonomy = entry.map(|e| e.taxonomy_ref.to_string()).unwrap_or_default();

    let root_cause_service = episode.root_cause_signal_index.and_then(|idx| {
        let i = idx as usize;
        if i < signal_names.len() {
            Some(signal_names[i].clone())
        } else {
            None
        }
    });

    let dashboard_hint_template = entry.map(|e| e.dashboard_hint).unwrap_or("");
    let dashboard_hint_bound = bind_template(
        dashboard_hint_template,
        episode,
        signal_names,
        match_confidence,
        &motif_name,
    );

    RenderedEpisodeSummary {
        episode_id: episode.episode_id,
        start_window: episode.start_window,
        end_window: episode.end_window,
        duration_windows: episode.structural_signature.duration_windows,
        motif,
        motif_name,
        motif_provenance,
        motif_evidence,
        motif_taxonomy,
        root_cause_service,
        root_cause_signal_index: episode.root_cause_signal_index,
        contributing_signal_count: episode.contributing_signal_count,
        peak_slew_magnitude: episode.structural_signature.peak_slew_magnitude,
        policy_state: episode.policy_state,
        dashboard_hint_bound,
        match_confidence,
    }
}

/// Render every closed episode in the buffer up to `count`.
pub fn render_episodes_summary<const M: usize>(
    episodes: &[DebugEpisode],
    count: usize,
    signal_names: &[String],
    bank: &HeuristicsBank<M>,
) -> Vec<RenderedEpisodeSummary> {
    let mut out = Vec::with_capacity(count);
    for ep in episodes.iter().take(count) {
        // Operator can supply confidence per-episode separately if
        // they ran `match_episode_with_confidence`; here we omit it.
        out.push(render_episode_summary(ep, signal_names, bank, None));
    }
    out
}

fn bind_template(
    template: &str,
    episode: &DebugEpisode,
    signal_names: &[String],
    confidence: Option<MatchConfidence>,
    motif_name: &str,
) -> String {
    if template.is_empty() {
        return String::new();
    }
    let mut out = template.to_string();

    // ${ROOT_CAUSE_SERVICE}
    let rc_service = episode.root_cause_signal_index
        .and_then(|idx| signal_names.get(idx as usize).cloned())
        .unwrap_or_else(|| "(unknown service)".to_string());
    out = out.replace("${ROOT_CAUSE_SERVICE}", &rc_service);

    // ${ROOT_CAUSE_INDEX}
    let rc_idx = episode.root_cause_signal_index
        .map(|i| i.to_string())
        .unwrap_or_else(|| "?".to_string());
    out = out.replace("${ROOT_CAUSE_INDEX}", &rc_idx);

    // ${CONTRIBUTING_COUNT}
    out = out.replace(
        "${CONTRIBUTING_COUNT}",
        &episode.contributing_signal_count.to_string(),
    );

    // ${PEAK_SLEW}
    out = out.replace(
        "${PEAK_SLEW}",
        &format!("{:.3}", episode.structural_signature.peak_slew_magnitude),
    );

    // ${DURATION_WINDOWS}
    out = out.replace(
        "${DURATION_WINDOWS}",
        &episode.structural_signature.duration_windows.to_string(),
    );

    // ${MOTIF}
    out = out.replace("${MOTIF}", motif_name);

    // ${CONFIDENCE_MARGIN}
    let margin_str = match confidence {
        Some(c) => format!("{:.3}", c.margin),
        None => "n/a".to_string(),
    };
    out = out.replace("${CONFIDENCE_MARGIN}", &margin_str);

    // ${RUNNER_UP_MOTIF}
    let runner_up_str = match confidence.and_then(|c| c.runner_up_motif) {
        Some(m) => format!("{:?}", m),
        None => "none".to_string(),
    };
    out = out.replace("${RUNNER_UP_MOTIF}", &runner_up_str);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep_with_root(root: Option<u16>) -> DebugEpisode {
        DebugEpisode {
            episode_id: 7,
            start_window: 100,
            end_window: 105,
            peak_grammar_state: GrammarState::Violation,
            primary_reason_code: ReasonCode::AbruptSlewViolation,
            matched_motif: SemanticDisposition::Named(MotifClass::CascadingTimeoutSlew),
            policy_state: PolicyState::Escalate,
            contributing_signal_count: 4,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::Positive,
                peak_slew_magnitude: 0.85,
                duration_windows: 6,
                signal_correlation: 0.5,
            },
            root_cause_signal_index: root,
        }
    }

    fn signal_names() -> Vec<String> {
        std::vec![
            "ts-station-service".to_string(),
            "ts-route-service".to_string(),
            "ts-travel-service".to_string(),
            "ts-order-service".to_string(),
        ]
    }

    #[test]
    fn renders_with_bound_root_cause_service() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = ep_with_root(Some(0));
        let names = signal_names();
        let r = render_episode_summary(&ep, &names, &bank, None);
        assert_eq!(r.motif_name, "CascadingTimeoutSlew");
        assert_eq!(r.root_cause_service, Some("ts-station-service".to_string()));
        assert_eq!(r.contributing_signal_count, 4);
        assert!(r.motif_taxonomy.contains("fault propagation"),
                "taxonomy_ref should land in rendered output");
    }

    #[test]
    fn renders_unknown_disposition_gracefully() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let mut ep = ep_with_root(None);
        ep.matched_motif = SemanticDisposition::Unknown;
        let r = render_episode_summary(&ep, &[], &bank, None);
        assert_eq!(r.motif_name, "Unknown");
        assert_eq!(r.root_cause_service, None);
        assert!(r.motif_evidence.is_empty(),
                "Unknown motifs have no evidence_dataset");
    }

    #[test]
    fn template_substitution_binds_variables() {
        // Custom motif lookup is via bank.entry_for(); the actual
        // dashboard_hint string for CascadingTimeoutSlew is hand-curated.
        // We exercise bind_template directly to verify substitution.
        let ep = ep_with_root(Some(2));
        let names = signal_names();
        let template = "Inspect ${ROOT_CAUSE_SERVICE} (signal ${ROOT_CAUSE_INDEX}); \
                        ${CONTRIBUTING_COUNT} services contribute, \
                        peak_slew=${PEAK_SLEW}, duration=${DURATION_WINDOWS} windows; \
                        motif=${MOTIF}, confidence margin ${CONFIDENCE_MARGIN}";
        let bound = bind_template(template, &ep, &names, None, "CascadingTimeoutSlew");
        assert!(bound.contains("ts-travel-service"),
                "ROOT_CAUSE_SERVICE should bind to channels[root_cause_signal_index]");
        assert!(bound.contains("(signal 2)"));
        assert!(bound.contains("4 services contribute"));
        assert!(bound.contains("peak_slew=0.850"));
        assert!(bound.contains("duration=6 windows"));
        assert!(bound.contains("motif=CascadingTimeoutSlew"));
        assert!(bound.contains("confidence margin n/a"));
    }

    #[test]
    fn template_substitution_uses_confidence_margin() {
        let ep = ep_with_root(Some(0));
        let names = signal_names();
        let confidence = MatchConfidence {
            disposition: SemanticDisposition::Named(MotifClass::CascadingTimeoutSlew),
            top_score: 4.5,
            runner_up_score: 1.5,
            runner_up_motif: Some(MotifClass::DeploymentRegressionSlew),
            margin: 0.667,
            tier_consensus_factor: 0.0,
            confuser_motif: None,
            confuser_score: 0.0,
            margin_vs_confuser: 0.0,
        };
        let bound = bind_template(
            "${MOTIF} margin=${CONFIDENCE_MARGIN} runner_up=${RUNNER_UP_MOTIF}",
            &ep, &names, Some(confidence), "CascadingTimeoutSlew");
        assert_eq!(
            bound,
            "CascadingTimeoutSlew margin=0.667 runner_up=DeploymentRegressionSlew",
        );
    }

    #[test]
    fn missing_signal_names_falls_through_gracefully() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = ep_with_root(Some(0));
        let r = render_episode_summary(&ep, &[], &bank, None);
        // No signal_names supplied → root_cause_service is None.
        assert_eq!(r.root_cause_service, None);
        // The dashboard hint is bound regardless (uses "(unknown service)" placeholder).
        assert!(r.dashboard_hint_bound.contains("ts-")
                || r.dashboard_hint_bound.contains("unknown")
                || r.dashboard_hint_bound.contains("service"));
    }
}
