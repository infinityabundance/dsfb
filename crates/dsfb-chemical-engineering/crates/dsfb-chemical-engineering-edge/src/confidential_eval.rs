//! Confidential evaluation (Wave-6 confidential-evaluation chain, climax): `ConfidentialEvaluationBundleV1`,
//! `PartnerDataEscrowProtocolV1`, and `ChemicalProcessPlaybookV1`.
//!
//! The commercial unlock: **DSFB does not require plant data to leave the operator's control.** The operator
//! runs the read-only court locally and shares only a redacted, hash-linked evidence bundle for review.
//!
//!   * [`ConfidentialEvaluationBundleV1`] — the artifact an industrial partner returns: a verification-report
//!     hash, an episode summary, a redacted (alias-only) topology, the hash roots, a metrics summary, the
//!     non-claims, and operator annotations — and **no raw time-series** (a sealed `contains_raw_timeseries`
//!     flag that must be false for the bundle to be shareable).
//!   * [`PartnerDataEscrowProtocolV1`] — the documented exchange protocol (operator runs the container
//!     locally → DSFB emits the redacted bundle → hash roots prove reproducibility → operator keeps the raw
//!     data → only the evidence summary is reviewed); valid only if **raw data never egressed**.
//!   * [`ChemicalProcessPlaybookV1`] — a deterministic per-episode "next inspection" playbook keyed on the
//!     event ontology class. **Non-claim: advisory inspection guidance, NOT an operational instruction.**
//!
//! Bounded (non-claims): a confidential bundle conveys *structure + hashes*, not raw data; the escrow protocol
//! removes the data-egress blocker by design but makes no claim about the partner's review quality; the
//! playbook is advisory and requires engineer judgement — it is not a control or safety action. Additive +
//! off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::event_evidence::EventOntologyClass;
use crate::evidence_kind::EvidenceKind;
use crate::hashing::CanonicalHasher;

// ── ConfidentialEvaluationBundleV1 ──────────────────────────────────────────────────────────────────

/// A hash-sealed confidential evaluation bundle (schema v1) — shareable evidence, no raw data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidentialEvaluationBundleV1 {
    pub case_ref: String,
    pub verification_report_hash: String,
    pub n_episodes: usize,
    /// (P93) The dominant `EvidenceKind` tag per episode (`EvidenceKind::tag()`), in episode order — so a reviewer
    /// of the shared bundle sees *what kind* of evidence backs each episode (physical balance vs heuristic vs
    /// precedent, …), not just the count. Classification/triage context only; carries no raw signal or tag name.
    pub evidence_kinds: Vec<String>,
    /// Redacted topology: alias-only edges/nodes (no real tag names).
    pub redacted_topology: Vec<String>,
    pub evidence_root: String,
    pub bundle_root: String,
    /// `(metric_name, value)` summary (counts/rates), not raw signals.
    pub metrics_summary: Vec<(String, f64)>,
    pub non_claims: Vec<String>,
    pub operator_annotations: Vec<String>,
    /// MUST be false for the bundle to be shareable (the whole point — no raw time-series leaves).
    pub contains_raw_timeseries: bool,
    pub bundle_hash: String,
}

impl ConfidentialEvaluationBundleV1 {
    #[allow(clippy::too_many_arguments)]
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"confidential_evaluation_bundle_v1");
        h.field("case_ref", self.case_ref.as_bytes());
        h.field(
            "verification_report_hash",
            self.verification_report_hash.as_bytes(),
        );
        h.u64("n_episodes", self.n_episodes as u64);
        for k in &self.evidence_kinds {
            h.field("evidence_kind", k.as_bytes());
        }
        for t in &self.redacted_topology {
            h.field("topo", t.as_bytes());
        }
        h.field("evidence_root", self.evidence_root.as_bytes());
        h.field("bundle_root", self.bundle_root.as_bytes());
        for (k, v) in &self.metrics_summary {
            h.field("metric", k.as_bytes());
            h.f64q("value", *v);
        }
        for n in &self.non_claims {
            h.field("non_claim", n.as_bytes());
        }
        for a in &self.operator_annotations {
            h.field("annotation", a.as_bytes());
        }
        h.u64(
            "contains_raw_timeseries",
            self.contains_raw_timeseries as u64,
        );
        h.finalize_hex()
    }

    /// Assemble a confidential bundle. `contains_raw_timeseries` is recorded as supplied (a guard the builder
    /// does not silently flip); callers building from redacted inputs pass `false`.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        case_ref: impl Into<String>,
        verification_report_hash: impl Into<String>,
        n_episodes: usize,
        evidence_kinds: Vec<String>,
        redacted_topology: Vec<String>,
        evidence_root: impl Into<String>,
        bundle_root: impl Into<String>,
        metrics_summary: Vec<(String, f64)>,
        non_claims: Vec<String>,
        operator_annotations: Vec<String>,
        contains_raw_timeseries: bool,
    ) -> Self {
        let mut b = ConfidentialEvaluationBundleV1 {
            case_ref: case_ref.into(),
            verification_report_hash: verification_report_hash.into(),
            n_episodes,
            evidence_kinds,
            redacted_topology,
            evidence_root: evidence_root.into(),
            bundle_root: bundle_root.into(),
            metrics_summary,
            non_claims,
            operator_annotations,
            contains_raw_timeseries,
            bundle_hash: String::new(),
        };
        b.bundle_hash = b.seal();
        b
    }

    /// Shareable iff it carries no raw time-series and at least one non-claim (the bounded-claim contract).
    pub fn is_shareable(&self) -> bool {
        !self.contains_raw_timeseries && !self.non_claims.is_empty()
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.bundle_hash
    }
}

// ── PartnerDataEscrowProtocolV1 ─────────────────────────────────────────────────────────────────────

/// One ordered step of the escrow protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowStep {
    pub order: usize,
    pub actor: String,
    pub action: String,
}

/// A hash-sealed partner-data-escrow protocol record (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartnerDataEscrowProtocolV1 {
    pub steps: Vec<EscrowStep>,
    /// The three guarantees a valid run must satisfy.
    pub raw_data_egressed: bool,
    pub only_evidence_reviewed: bool,
    pub reproducible_via_hashes: bool,
    /// Valid iff raw never egressed AND only evidence was reviewed AND it is reproducible via hashes.
    pub valid: bool,
    pub protocol_hash: String,
}

impl PartnerDataEscrowProtocolV1 {
    /// The fixed documented protocol steps.
    fn definition() -> Vec<EscrowStep> {
        [
            ("operator", "run the read-only DSFB container locally on the raw historian export"),
            ("dsfb", "emit a sealed Chemical Court Record + a redacted ConfidentialEvaluationBundle (no raw time-series)"),
            ("operator", "retain the raw data; share ONLY the redacted evidence bundle + hash roots"),
            ("partner", "review the evidence bundle; re-derive the hash roots to confirm reproducibility"),
            ("partner", "return findings against the aliases — never requesting raw data"),
        ]
        .iter()
        .enumerate()
        .map(|(i, (actor, action))| EscrowStep { order: i, actor: (*actor).into(), action: (*action).into() })
        .collect()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"partner_data_escrow_protocol_v1");
        for s in &self.steps {
            h.u64("order", s.order as u64);
            h.field("actor", s.actor.as_bytes());
            h.field("action", s.action.as_bytes());
        }
        h.u64("raw_data_egressed", self.raw_data_egressed as u64);
        h.u64("only_evidence_reviewed", self.only_evidence_reviewed as u64);
        h.u64(
            "reproducible_via_hashes",
            self.reproducible_via_hashes as u64,
        );
        h.u64("valid", self.valid as u64);
        h.finalize_hex()
    }

    /// Record an escrow run with its three guarantee flags; the protocol is valid only if raw data never
    /// egressed, only the evidence was reviewed, and the result is reproducible via hashes.
    pub fn record(
        raw_data_egressed: bool,
        only_evidence_reviewed: bool,
        reproducible_via_hashes: bool,
    ) -> Self {
        let valid = !raw_data_egressed && only_evidence_reviewed && reproducible_via_hashes;
        let mut p = PartnerDataEscrowProtocolV1 {
            steps: Self::definition(),
            raw_data_egressed,
            only_evidence_reviewed,
            reproducible_via_hashes,
            valid,
            protocol_hash: String::new(),
        };
        p.protocol_hash = p.seal();
        p
    }

    pub fn verify(&self) -> bool {
        let valid =
            !self.raw_data_egressed && self.only_evidence_reviewed && self.reproducible_via_hashes;
        valid == self.valid && self.seal() == self.protocol_hash
    }
}

// ── ChemicalProcessPlaybookV1 ───────────────────────────────────────────────────────────────────────

/// A hash-sealed advisory "next inspection" playbook (schema v1) for one episode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalProcessPlaybookV1 {
    pub episode_ref: String,
    pub ontology_class: String,
    /// The `EvidenceKind` the playbook's inspection is designed to rest on / gather for this ontology class
    /// (e.g. a Thermal class resolves on a physical energy balance). Display/triage context only — it does not
    /// strengthen the advisory; the playbook stays advisory regardless of the rung behind the kind.
    pub evidence_kind: String,
    pub steps: Vec<String>,
    pub non_claim: String,
    pub playbook_hash: String,
}

impl ChemicalProcessPlaybookV1 {
    const NON_CLAIM: &'static str =
        "advisory inspection guidance only; NOT an operational instruction, control action, or safety step — engineer judgement required";

    /// Deterministic per-ontology-class inspection steps. Conservative: an unknown-structural episode gets a
    /// "gather more evidence" step, never a fabricated specific instruction.
    fn steps_for(class: EventOntologyClass) -> Vec<String> {
        let s: &[&str] = match class {
            EventOntologyClass::Controller => &[
                "Trend the loop's PV / MV / SP for stiction, saturation, or oscillation",
                "Review recent controller-mode changes and tuning edits",
                "Inspect the final control element (valve/actuator)",
            ],
            EventOntologyClass::Thermal => &[
                "Verify coolant flow and inlet/outlet temperatures",
                "Check the heat-transfer surface for fouling",
                "Review the reaction-rate / duty trend over the window",
            ],
            EventOntologyClass::Hydraulic => &[
                "Check pump suction conditions and NPSH for cavitation",
                "Trend flow vs differential pressure for blockage/restriction",
                "Inspect strainers/filters upstream",
            ],
            EventOntologyClass::Reaction => &[
                "Review feed composition and conversion trend",
                "Check catalyst/reagent age and recent lot change",
                "Confirm the energy balance closes around the reactor",
            ],
            EventOntologyClass::Separation => &[
                "Trend column differential pressure for flooding/weeping",
                "Review reflux ratio and reboiler duty",
                "Confirm the component balance across the separator",
            ],
            EventOntologyClass::Sensor => &[
                "Compare the suspect tag against a redundant/soft-sensor estimate",
                "Check for a recent calibration event or a frozen/flatlined signal",
                "Review the sensor-trust ledger entry for this tag",
            ],
            EventOntologyClass::Material => &[
                "Cross-reference the active feedstock lot and its certificate of analysis",
                "Check whether the episode onset aligns with a lot change",
            ],
            EventOntologyClass::Cleaning => &[
                "Review the most recent clean-in-place cycle vs its phase targets",
                "Confirm the episode is not a residual-from-cleaning transient",
            ],
            EventOntologyClass::Maintenance => &[
                "Overlay the maintenance log; check for an open work order on the unit",
                "Review any calibration event in the window",
            ],
            EventOntologyClass::ExternalUtility => &[
                "Check the shared utility header (steam/cooling water) for an upstream disturbance",
                "Correlate with other units on the same utility loop",
            ],
            EventOntologyClass::UnknownStructural => &[
                "Insufficient structure to recommend a specific inspection; gather more witnesses before acting",
            ],
        };
        s.iter().map(|x| x.to_string()).collect()
    }

    /// The `EvidenceKind` an inspection of this ontology class is designed to rest on — chosen by what kind of
    /// witness the `steps_for` checks gather, not arbitrarily: thermal/hydraulic/separation resolve on a physical
    /// balance, reaction on a first-principles equation, controller on controller context, sensor on
    /// instrumentation health, external-utility on process topology (cross-unit coupling), and the
    /// material/cleaning/maintenance context classes on operator-supplied annotation. An unknown-structural
    /// episode matched no signature, so it rests only on a heuristic pattern (the weakest, candidate-only kind) —
    /// deliberately never a physical-witness kind it has not earned.
    fn evidence_kind_for(class: EventOntologyClass) -> EvidenceKind {
        match class {
            EventOntologyClass::Controller => EvidenceKind::ControllerContext,
            EventOntologyClass::Thermal
            | EventOntologyClass::Hydraulic
            | EventOntologyClass::Separation => EvidenceKind::PhysicalBalance,
            EventOntologyClass::Reaction => EvidenceKind::FirstPrinciplesEquation,
            EventOntologyClass::Sensor => EvidenceKind::InstrumentationHealth,
            EventOntologyClass::ExternalUtility => EvidenceKind::ProcessTopology,
            EventOntologyClass::Material
            | EventOntologyClass::Cleaning
            | EventOntologyClass::Maintenance => EvidenceKind::OperatorAnnotation,
            EventOntologyClass::UnknownStructural => EvidenceKind::HeuristicPattern,
        }
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"chemical_process_playbook_v1");
        h.field("episode_ref", self.episode_ref.as_bytes());
        h.field("ontology_class", self.ontology_class.as_bytes());
        h.field("evidence_kind", self.evidence_kind.as_bytes());
        for step in &self.steps {
            h.field("step", step.as_bytes());
        }
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Build the advisory playbook for an episode from its event ontology class.
    pub fn build(episode_ref: impl Into<String>, class: EventOntologyClass) -> Self {
        let mut p = ChemicalProcessPlaybookV1 {
            episode_ref: episode_ref.into(),
            ontology_class: class.tag().into(),
            evidence_kind: Self::evidence_kind_for(class).tag().into(),
            steps: Self::steps_for(class),
            non_claim: Self::NON_CLAIM.into(),
            playbook_hash: String::new(),
        };
        p.playbook_hash = p.seal();
        p
    }

    pub fn verify(&self) -> bool {
        self.non_claim == Self::NON_CLAIM && self.seal() == self.playbook_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn confidential_bundle_is_shareable_without_raw_data() {
        let b = ConfidentialEvaluationBundleV1::build(
            "case-001",
            "ab".repeat(32),
            7,
            vec![s("physical_balance"), s("heuristic_pattern")],
            vec![s("TAG_0001->TAG_0002")],
            "cd".repeat(32),
            "ef".repeat(32),
            vec![(s("fused_episodes"), 7.0), (s("baseline_fp_rate"), 0.02)],
            vec![s("no root cause; advisory; read-only")],
            vec![s("operator: matches the Aug turnaround")],
            false, // no raw time-series
        );
        assert!(b.is_shareable());
        assert!(b.bundle_hash.len() == 64 && b.verify());
        // A bundle that admits raw time-series is NOT shareable.
        let leaky = ConfidentialEvaluationBundleV1::build(
            "c",
            "ab".repeat(32),
            1,
            vec![],
            vec![],
            "cd".repeat(32),
            "ef".repeat(32),
            vec![],
            vec![s("nc")],
            vec![],
            true,
        );
        assert!(!leaky.is_shareable());
        // Tamper the raw-data flag → seal breaks.
        let mut t = b.clone();
        t.contains_raw_timeseries = true;
        assert!(!t.verify());
    }

    #[test]
    fn escrow_protocol_valid_only_when_raw_never_egresses() {
        let good = PartnerDataEscrowProtocolV1::record(false, true, true);
        assert!(good.valid && good.steps.len() == 5 && good.verify());
        // Raw data egressed → invalid, regardless of the other flags.
        let bad = PartnerDataEscrowProtocolV1::record(true, true, true);
        assert!(!bad.valid && bad.verify());
        // Tamper: forge valid=true while raw egressed.
        let mut t = bad.clone();
        t.valid = true;
        assert!(!t.verify());
    }

    #[test]
    fn playbook_is_class_specific_and_advisory() {
        let thermal = ChemicalProcessPlaybookV1::build("e1", EventOntologyClass::Thermal);
        assert!(thermal.steps.iter().any(|s| s.contains("coolant")));
        assert!(thermal.non_claim.contains("NOT an operational instruction"));
        assert!(thermal.playbook_hash.len() == 64 && thermal.verify());
        // (P88) The evidence kind is the witness the inspection rests on: thermal -> physical balance,
        // controller -> controller context, and an unknown-structural episode rests only on a heuristic pattern
        // (the weakest, candidate-only kind) -- never a physical-witness kind it has not earned.
        assert_eq!(thermal.evidence_kind, EvidenceKind::PhysicalBalance.tag());
        assert_eq!(
            ChemicalProcessPlaybookV1::build("e1c", EventOntologyClass::Controller).evidence_kind,
            EvidenceKind::ControllerContext.tag()
        );
        // Unknown-structural gets the conservative "gather more" step, not a fabricated instruction.
        let unknown = ChemicalProcessPlaybookV1::build("e2", EventOntologyClass::UnknownStructural);
        assert_eq!(unknown.steps.len(), 1);
        assert!(unknown.steps[0].contains("gather more witnesses"));
        assert_eq!(unknown.evidence_kind, EvidenceKind::HeuristicPattern.tag());
        assert!(unknown.verify());
        // Tamper away the non-claim → seal breaks.
        let mut t = thermal.clone();
        t.non_claim = "do this now".into();
        assert!(!t.verify());
    }
}
