//! P84 — `confidential-demo`: wire the Wave-6 confidential-evaluation objects into ONE command that emits a
//! shareable, redacted evidence bundle from a synthetic historian fixture — making the headline adoption
//! claim concrete in a single run: *the operator runs the read-only court locally and shares only a
//! redacted, hash-linked evidence bundle — raw plant data never leaves their control.*
//!
//! The chain's objects already exist + are individually tested ([`crate::confidential_eval`],
//! [`crate::redaction`]); what was missing was the wiring. This module is the assembly layer: [`assemble`]
//! takes the minimal facts of a local court run (episode briefs, the operator's tag roles, the casefile's
//! per-file hashes, the evidence/bundle roots, a verification-report hash, a metrics summary) and returns the
//! complete set of sealed shareable objects — redaction map, tamper seal, audit trail, confidential bundle,
//! escrow protocol, per-episode playbooks. It is pure (no I/O), so it is host-tested directly; the thin CLI
//! `cli::run_confidential_demo` extracts these facts from a real `AnalysisResult` and writes the JSONs.
//!
//! Bounded (non-claims): this is an *illustration of the protocol on a synthetic fixture*, not a certified
//! data-handling guarantee; the shareable bundle conveys structure + hashes, never raw time-series.

use serde::Serialize;

use crate::confidential_eval::{
    ChemicalProcessPlaybookV1, ConfidentialEvaluationBundleV1, PartnerDataEscrowProtocolV1,
};
use crate::event_evidence::EventOntologyClass;
use crate::redaction::{
    AuditExport, AuditTrailExportV1, CaseFileRedactionMapV1, TamperEvidenceSealV1,
};

/// The minimal brief of one fused episode the shareable bundle needs (label drives the playbook class;
/// bounds become an opaque episode ref — no raw signal).
pub struct EpisodeBrief {
    pub label: String,
    pub start: usize,
    pub end: usize,
}

/// The complete set of sealed objects an operator shares for confidential evaluation — **no raw time-series**.
#[derive(Debug, Clone, Serialize)]
pub struct ConfidentialDemoBundleV1 {
    pub redaction_map: CaseFileRedactionMapV1,
    pub tamper_seal: TamperEvidenceSealV1,
    pub audit_trail: AuditTrailExportV1,
    pub confidential_bundle: ConfidentialEvaluationBundleV1,
    pub escrow_protocol: PartnerDataEscrowProtocolV1,
    pub playbooks: Vec<ChemicalProcessPlaybookV1>,
}

impl ConfidentialDemoBundleV1 {
    /// The bundle is sound + safe to share iff every sealed object self-verifies, the redaction map leaks no
    /// real names, the confidential bundle carries no raw time-series, and the escrow protocol's
    /// raw-never-egressed guarantee holds. This is the machine gate the CLI asserts before declaring success.
    pub fn is_shareable(&self) -> bool {
        self.redaction_map.verify()
            && self.redaction_map.is_safe_to_share()
            && self.tamper_seal.verify()
            && self.audit_trail.verify()
            && self.confidential_bundle.verify()
            && self.confidential_bundle.is_shareable()
            && self.escrow_protocol.verify()
            && self.escrow_protocol.valid
            && self.playbooks.iter().all(|p| p.verify())
    }
}

/// Coarse unit class from a free-text engineering unit — shareable *structure*, not the exact unit/value.
pub fn unit_class(unit: &str) -> String {
    let u = unit.to_lowercase();
    let m = |k: &str| u.contains(k);
    if m("°c") || m("deg") || m("kelvin") || m("temp") {
        "temperature"
    } else if m("bar") || m("pa") || m("psi") || m("pressure") {
        "pressure"
    } else if m("m3") || m("l/") || m("gpm") || m("flow") || m("kg/") || m("/s") || m("/h") {
        "flow"
    } else if m("cm") || m("level") || m("%") {
        "level"
    } else if m("amp") || m("current") {
        "current"
    } else {
        "other"
    }
    .to_string()
}

/// Map a casefile content-file name to an audit-trail format id.
fn export_format(file_name: &str) -> String {
    match file_name.rsplit('.').next().unwrap_or("") {
        "html" => "html",
        "csv" => "csv",
        "json" => "jsonl",
        "txt" => "worm_manifest",
        "md" => "markdown",
        _ => "other",
    }
    .to_string()
}

/// Assemble the full shareable confidential-evaluation bundle from the facts of one local court run. Pure +
/// deterministic; every returned object is hash-sealed and `contains_raw_timeseries = false` by construction.
#[allow(clippy::too_many_arguments)]
pub fn assemble(
    case_ref: &str,
    episodes: &[EpisodeBrief],
    roles_rows: &[(String, String, String)],
    file_hashes: &[(String, String)],
    evidence_root: &str,
    bundle_root: &str,
    verification_report_hash: &str,
    metrics_summary: Vec<(String, f64)>,
) -> ConfidentialDemoBundleV1 {
    let redaction_map = CaseFileRedactionMapV1::build(roles_rows);
    // Deterministic seal (no timestamp): the first link in the operator's append-only chain.
    let tamper_seal = TamperEvidenceSealV1::build(file_hashes.to_vec(), None, None, None);
    let exports = file_hashes
        .iter()
        .map(|(n, h)| AuditExport {
            format: export_format(n),
            file_name: n.clone(),
            content_hash: h.clone(),
        })
        .collect();
    let audit_trail = AuditTrailExportV1::build(case_ref, exports);

    // Redacted topology = alias + coarse structure only (no real tag names).
    let redacted_topology = redaction_map
        .entries
        .iter()
        .map(|e| format!("{} [{}/{}]", e.alias, e.unit_class, e.variable_group))
        .collect();
    let non_claims = vec![
        "advisory, read-only — no root cause, no causality, no control or safety-instrumented-function authority".to_string(),
        "no raw time-series leaves the operator; only redacted, hash-linked evidence is shared".to_string(),
    ];
    let operator_annotations = vec![format!(
        "local court run on the synthetic fixture '{case_ref}' (no real plant data)"
    )];
    // One advisory playbook per episode, keyed on the episode's ontology class (from its label). Built BEFORE the
    // bundle so the bundle's per-episode evidence-kind summary is derived from the same source (one derivation,
    // no drift between the playbooks and the bundle).
    let playbooks: Vec<ChemicalProcessPlaybookV1> = episodes
        .iter()
        .enumerate()
        .map(|(i, e)| {
            ChemicalProcessPlaybookV1::build(
                format!("ep{i:03}@{}-{}", e.start, e.end),
                EventOntologyClass::from_label(&e.label),
            )
        })
        .collect();
    // (P93) Per-episode dominant evidence-kind tags for the bundle — exactly the kinds the playbooks carry.
    let evidence_kinds: Vec<String> = playbooks.iter().map(|p| p.evidence_kind.clone()).collect();
    let confidential_bundle = ConfidentialEvaluationBundleV1::build(
        case_ref,
        verification_report_hash,
        episodes.len(),
        evidence_kinds,
        redacted_topology,
        evidence_root,
        bundle_root,
        metrics_summary,
        non_claims,
        operator_annotations,
        false, // contains_raw_timeseries — the whole point
    );
    // The escrow protocol run: raw never egressed, only evidence reviewed, reproducible via hashes.
    let escrow_protocol = PartnerDataEscrowProtocolV1::record(false, true, true);

    ConfidentialDemoBundleV1 {
        redaction_map,
        tamper_seal,
        audit_trail,
        confidential_bundle,
        escrow_protocol,
        playbooks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    /// The assembled bundle is shareable, carries no raw time-series, serialises to JSON, and routes each
    /// episode to its ontology-class playbook (a cavitation label → Hydraulic).
    #[test]
    fn assembled_bundle_is_shareable_with_no_raw_data() {
        let episodes = vec![
            EpisodeBrief {
                label: s("pump cavitation"),
                start: 360,
                end: 600,
            },
            EpisodeBrief {
                label: s("unknown structural episode"),
                start: 50,
                end: 60,
            },
        ];
        let roles = vec![
            (s("TIC-101.PV"), s("temperature"), s("reactor")),
            (s("FIC-204.PV"), s("flow"), s("utility")),
        ];
        let files = vec![
            (s("casefile.json"), "aa".repeat(32)),
            (s("operator_report.html"), "bb".repeat(32)),
        ];
        let b = assemble(
            "synthetic_tank_historian",
            &episodes,
            &roles,
            &files,
            &"cd".repeat(32),
            &"ef".repeat(32),
            &"12".repeat(32),
            vec![(s("fused_episodes"), 2.0), (s("unknown_rate"), 0.5)],
        );
        assert!(
            b.is_shareable(),
            "the assembled bundle must be safe to share"
        );
        assert!(
            !b.confidential_bundle.contains_raw_timeseries,
            "no raw time-series may be present"
        );
        assert!(
            b.redaction_map.is_safe_to_share(),
            "the redaction map must leak no real names"
        );
        assert_eq!(b.playbooks.len(), 2);
        assert_eq!(
            b.playbooks[0].ontology_class, "hydraulic",
            "a cavitation label routes to the hydraulic playbook"
        );
        // (P93) The bundle's per-episode evidence-kind summary is present and consistent with the playbooks
        // (one derivation, no drift): a cavitation (hydraulic) episode rests on a physical balance.
        assert_eq!(b.confidential_bundle.evidence_kinds.len(), 2);
        assert_eq!(
            b.confidential_bundle.evidence_kinds,
            b.playbooks
                .iter()
                .map(|p| p.evidence_kind.clone())
                .collect::<Vec<_>>(),
            "bundle evidence_kinds must equal the playbooks' kinds"
        );
        assert_eq!(b.confidential_bundle.evidence_kinds[0], "physical_balance");
        assert!(b.escrow_protocol.valid && !b.escrow_protocol.raw_data_egressed);
        // The whole bundle serialises to JSON (the shareable artifact) and contains no raw-residual fields.
        let json = serde_json::to_string(&b).expect("serialises");
        assert!(
            json.contains("TAG_0000") && !json.contains("TIC-101.PV"),
            "aliases shared, real names redacted"
        );
    }
}
