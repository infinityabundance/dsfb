//! `EquipmentSignatureRecordV1` — an equipment-class signature bank with the richer witness-burden schema
//! (panel msg-1 `SignatureRecordV1` + msg-2 equipment-class organisation).
//!
//! The atlas `FaultSignatureRecordV1` bank (F1–F12) is organised by physical *mechanism* and is the frozen
//! authority. This is a complementary **execution-side** view organised by *equipment class* (pump / heat
//! exchanger / reactor / column / …) that adds the fields an elite reviewer asks for: **required**,
//! **forbidden**, and **supporting** witnesses; an explicit **evidence tier** (A physically-grounded · B
//! empirically-grounded · C heuristic-advisory · D unmapped); a **minimum burden of proof**; and per-signature
//! **non-claims**. It does NOT re-curate the atlas (no `atlas_hash_v1` move) — it is a self-sealed reading bank
//! over the existing witnesses, with a burden check so a signature is admitted only when its required witnesses
//! are present and none of its forbidden witnesses fire (else the episode stays an honest unknown).
//!
//! **NON-CLAIM:** these are *candidate* equipment-failure signatures bounded by their tier and witness burden —
//! not a diagnosis, not a root cause, and not a hazard analysis. A `D` (unmapped) structure is preserved for
//! review (the discovery route), never force-labelled. Self-sealed; not part of any frozen authority hash.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The equipment class a signature belongs to (the organising axis of this bank).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EquipmentClass {
    Pump,
    HeatExchanger,
    Reactor,
    DistillationColumn,
    Separator,
    Tank,
    Compressor,
    ControlValve,
    Instrument,
    BatchUnit,
}

impl EquipmentClass {
    pub fn tag(self) -> &'static str {
        match self {
            EquipmentClass::Pump => "pump",
            EquipmentClass::HeatExchanger => "heat_exchanger",
            EquipmentClass::Reactor => "reactor",
            EquipmentClass::DistillationColumn => "distillation_column",
            EquipmentClass::Separator => "separator",
            EquipmentClass::Tank => "tank",
            EquipmentClass::Compressor => "compressor",
            EquipmentClass::ControlValve => "control_valve",
            EquipmentClass::Instrument => "instrument",
            EquipmentClass::BatchUnit => "batch_unit",
        }
    }
}

/// Evidence tier — how strongly a signature is grounded, so breadth never reads as overclaim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SignatureTier {
    /// A — physically grounded: requires a balance / topology / control-loop / first-principles witness.
    PhysicallyGrounded,
    /// B — empirically grounded: derived from labelled datasets, recurrence, or public benchmarks.
    EmpiricallyGrounded,
    /// C — heuristic-advisory: an operator-known pattern not physically proven from the available evidence.
    HeuristicAdvisory,
    /// D — unmapped structural: a stable recurring structure with no admitted interpretation yet (discovery route).
    UnmappedStructural,
}

impl SignatureTier {
    pub fn tag(self) -> &'static str {
        match self {
            SignatureTier::PhysicallyGrounded => "A_physically_grounded",
            SignatureTier::EmpiricallyGrounded => "B_empirically_grounded",
            SignatureTier::HeuristicAdvisory => "C_heuristic_advisory",
            SignatureTier::UnmappedStructural => "D_unmapped_structural",
        }
    }
}

/// One equipment-class signature record (richer schema v1). `&'static` so the bank is `const` (Serialize only —
/// `&'static` fields cannot be deserialized, matching the atlas authority-record convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EquipmentSignatureRecordV1 {
    /// Stable id, prefixed by equipment class (e.g. `"PUMP-CAV-01"`).
    pub signature_id: &'static str,
    /// The equipment class this signature lives under.
    pub equipment_class: EquipmentClass,
    /// The failure / error mode named.
    pub failure_mode: &'static str,
    /// Evidence tier (A–D).
    pub tier: SignatureTier,
    /// Witnesses that MUST be present to admit this signature (burden of proof).
    pub required_witnesses: &'static [&'static str],
    /// Witnesses whose firing RULES OUT this signature (a forbidden co-occurrence).
    pub forbidden_witnesses: &'static [&'static str],
    /// Witnesses that corroborate but are not required.
    pub supporting_witnesses: &'static [&'static str],
    /// Mechanisms / conditions this signature can be mistaken for.
    pub confusers: &'static [&'static str],
    /// The minimum burden, as an inspectable sentence (what must hold before the candidate is admitted).
    pub minimum_burden: &'static str,
    /// Operator-facing one-liner (what to inspect — never an instruction to act).
    pub operator_explanation: &'static str,
    /// Explicit per-signature non-claims.
    pub non_claims: &'static [&'static str],
}

/// The equipment-class signature bank. Each `*_witnesses` entry names an object that already exists in the
/// workspace, so the bank is a reading view over real evidence, not aspiration. Illustrative breadth across
/// equipment classes + tiers; extend it freely (it re-seals, it never touches `atlas_hash_v1`).
pub const EQUIPMENT_SIGNATURES: &[EquipmentSignatureRecordV1] = &[
    EquipmentSignatureRecordV1 {
        signature_id: "PUMP-CAV-01",
        equipment_class: EquipmentClass::Pump,
        failure_mode: "cavitation / suction starvation",
        tier: SignatureTier::PhysicallyGrounded,
        required_witnesses: &["BalanceWitnessV1 (suction/flow inconsistency)", "SlewSpike (head oscillation)"],
        forbidden_witnesses: &["GrammarState::SensorFault (a dead flow channel explains it instead)"],
        supporting_witnesses: &["spectral motif (vane-pass band)", "ControlLoopInteractionMapV1 (speed/throughput)"],
        confusers: &["entrained gas / two-phase feed", "downstream blockage", "flow-meter fault"],
        minimum_burden: "a flow/head balance inconsistency AND a transient head motif, with the flow channel proven live",
        operator_explanation: "inspect suction pressure / NPSH margin and recent throughput changes",
        non_claims: &["not a proven mechanical-damage assessment", "no remaining-useful-life claim"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "HX-FOUL-01",
        equipment_class: EquipmentClass::HeatExchanger,
        failure_mode: "heat-transfer fouling / reduced duty",
        tier: SignatureTier::PhysicallyGrounded,
        required_witnesses: &["BalanceWitnessV1 (energy-balance closure deficit)", "DriftAccum (slow duty decline)"],
        forbidden_witnesses: &["GrammarState::SensorFault (temperature channel)"],
        supporting_witnesses: &["EquationResidualPassportV1 (heat-transfer residual)", "MaterialLotWitnessV1 (feed change)"],
        confusers: &["thermocouple drift / bias", "feed-composition step", "reduced coolant flow (valve)"],
        minimum_burden: "a sustained energy-balance closure deficit with both temperature legs proven live",
        operator_explanation: "compare clean-vs-current duty; review last cleaning + coolant flow",
        non_claims: &["does not localise the fouled side", "not a cleaning-schedule directive"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "RXR-THERM-01",
        equipment_class: EquipmentClass::Reactor,
        failure_mode: "thermal-excursion precursor",
        tier: SignatureTier::HeuristicAdvisory,
        required_witnesses: &["DriftAccum (temperature group)", "co-drift of cooling/heating variables"],
        forbidden_witnesses: &["GrammarState::SensorFault"],
        supporting_witnesses: &["accelerating slew", "ControlLoopInteractionMapV1 (coolant saturation)"],
        confusers: &["setpoint change", "ambient/jacket disturbance", "sensor drift"],
        minimum_burden: "temperature-group drift WITH corroborating cooling-effort residual (advisory only — no balance witness)",
        operator_explanation: "watch reactor T trend + coolant valve headroom; verify setpoint history",
        non_claims: &["NOT a runaway prediction", "no safety-instrumented-function role", "advisory precursor only"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "VLV-STIC-01",
        equipment_class: EquipmentClass::ControlValve,
        failure_mode: "stiction / stick-slip",
        tier: SignatureTier::EmpiricallyGrounded,
        required_witnesses: &["SlewSpike (snap after delay)", "ControlLoopInteractionMapV1 (oscillatory MV)"],
        forbidden_witnesses: &["GrammarState::Nominal (no control activity)"],
        supporting_witnesses: &["spectral content shift", "SetpointResidualSeparationV1"],
        confusers: &["aggressive controller tuning", "external load oscillation", "process limit cycle"],
        minimum_burden: "a delayed-snap slew motif coincident with oscillatory controller effort (labelled-dataset grounded)",
        operator_explanation: "review valve travel/position vs MV; check recent maintenance",
        non_claims: &["does not quantify stiction band", "not a calibration certificate"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "COL-FLOOD-01",
        equipment_class: EquipmentClass::DistillationColumn,
        failure_mode: "flooding / loading precursor",
        tier: SignatureTier::HeuristicAdvisory,
        required_witnesses: &["DriftAccum (column ΔP)", "reflux/boilup residual drift"],
        forbidden_witnesses: &["GrammarState::SensorFault (ΔP cell)"],
        supporting_witnesses: &["BalanceWitnessV1 (component balance deficit)", "ProcessTopologyGraphV1"],
        confusers: &["feed-rate ramp", "pressure-control upset", "ΔP-cell plugging"],
        minimum_burden: "sustained ΔP drift with corroborating reflux/boilup residual (advisory — ΔP cell proven live)",
        operator_explanation: "inspect column ΔP trend vs throughput; review reflux ratio",
        non_claims: &["does not locate the flooding stage", "not a capacity-limit certification"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "TANK-BAL-01",
        equipment_class: EquipmentClass::Tank,
        failure_mode: "level / mass-balance inconsistency (leak or unmetered draw)",
        tier: SignatureTier::PhysicallyGrounded,
        required_witnesses: &["BalanceWitnessV1 (tank-volume mass balance)"],
        forbidden_witnesses: &["GrammarState::SensorFault (level transmitter)"],
        supporting_witnesses: &["EnvViolation (level)", "ControlLoopInteractionMapV1 (inflow/outflow)"],
        confusers: &["level-sensor spoof/bias", "unmetered legitimate draw", "flow-meter calibration"],
        minimum_burden: "a closed-tank mass-balance residual breach with the level + flow legs proven live",
        operator_explanation: "reconcile level rate-of-change vs metered in/out flows; check for unmetered draws",
        non_claims: &["does not assert a leak location", "leak vs unmetered-draw not disambiguated without more witnesses"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "INST-CAL-01",
        equipment_class: EquipmentClass::Instrument,
        failure_mode: "sensor calibration drift / bias",
        tier: SignatureTier::EmpiricallyGrounded,
        required_witnesses: &["SensorTrustDegradationLedgerV1 (single-tag drift)", "DriftAccum (one variable)"],
        forbidden_witnesses: &["correlated co-drift of physically-coupled neighbours (then it is a process change)"],
        supporting_witnesses: &["BalanceWitnessV1 (balance stays closed → it is the sensor, not the process)"],
        confusers: &["genuine slow process drift", "ambient effect on the transmitter", "soft-sensor model drift"],
        minimum_burden: "one variable drifts while its physically-coupled neighbours and the balance do NOT",
        operator_explanation: "compare against a redundant tag or a recent lab assay; review calibration date",
        non_claims: &["does not certify the instrument", "no recalibration directive"],
    },
    EquipmentSignatureRecordV1 {
        signature_id: "INST-PAT-DRIFT-01",
        equipment_class: EquipmentClass::Instrument,
        failure_mode: "PAT / soft-sensor model drift (e.g. NIR standardization)",
        tier: SignatureTier::EmpiricallyGrounded,
        required_witnesses: &["SoftSensorWitnessV1 (prediction residual drift)", "CalibrationModelPassportV1 (out-of-validation / Q-residual)"],
        forbidden_witnesses: &["lab assay confirms the predicted shift (then it is a real process change)"],
        supporting_witnesses: &["ManualSampleBridgeV1 (sparse lab disagreement)"],
        confusers: &["a genuine process/composition change", "instrument standardization shift", "sample-presentation change"],
        minimum_burden: "soft-sensor residual drift WITH spectra flagged out-of-model OR predictions out of the validated range",
        operator_explanation: "compare soft-sensor prediction to the latest lab sample; check instrument standardization",
        non_claims: &["does not re-validate the model", "advisory model-health flag, not a prediction correction"],
    },
];

/// A sealed snapshot of the equipment-signature bank (schema v1) + its non-claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSignatureBankV1 {
    pub n_signatures: usize,
    pub non_claim: String,
    pub bank_hash: String,
}

impl EquipmentSignatureBankV1 {
    pub const NON_CLAIM: &'static str = "Candidate equipment-failure signatures bounded by their tier and witness \
        burden — NOT a diagnosis, root cause, or hazard analysis; an unmapped structure is preserved for review, \
        never force-labelled.";

    fn seal() -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"equipment_signature_bank_v1");
        for s in EQUIPMENT_SIGNATURES {
            h.field("signature_id", s.signature_id.as_bytes());
            h.field("equipment_class", s.equipment_class.tag().as_bytes());
            h.field("failure_mode", s.failure_mode.as_bytes());
            h.field("tier", s.tier.tag().as_bytes());
            for w in s.required_witnesses {
                h.field("required", w.as_bytes());
            }
            for w in s.forbidden_witnesses {
                h.field("forbidden", w.as_bytes());
            }
            for w in s.supporting_witnesses {
                h.field("supporting", w.as_bytes());
            }
            for c in s.confusers {
                h.field("confuser", c.as_bytes());
            }
            h.field("minimum_burden", s.minimum_burden.as_bytes());
            for n in s.non_claims {
                h.field("non_claim_item", n.as_bytes());
            }
        }
        h.field("non_claim", Self::NON_CLAIM.as_bytes());
        h.finalize_hex()
    }

    pub fn build() -> Self {
        EquipmentSignatureBankV1 {
            n_signatures: EQUIPMENT_SIGNATURES.len(),
            non_claim: Self::NON_CLAIM.to_string(),
            bank_hash: Self::seal(),
        }
    }

    pub fn verify(&self) -> bool {
        self.n_signatures == EQUIPMENT_SIGNATURES.len()
            && self.non_claim == Self::NON_CLAIM
            && self.bank_hash == Self::seal()
    }
}

impl EquipmentSignatureRecordV1 {
    /// Burden of proof: this signature may be admitted as a *candidate* iff every required witness is present and
    /// no forbidden witness fired. Otherwise the episode stays an honest unknown (never force-labelled). `present`
    /// is the set of witness tags observed for the episode.
    pub fn meets_burden(&self, present: &[&str]) -> bool {
        let all_required = self.required_witnesses.iter().all(|r| present.contains(r));
        let no_forbidden = !self.forbidden_witnesses.iter().any(|f| present.contains(f));
        all_required && no_forbidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_self_verifies_and_is_deterministic() {
        let b = EquipmentSignatureBankV1::build();
        assert!(
            b.n_signatures >= 8,
            "illustrative breadth across equipment classes"
        );
        assert!(b.verify());
        assert_eq!(b.bank_hash, EquipmentSignatureBankV1::build().bank_hash);
    }

    #[test]
    fn every_signature_is_grounded_and_bounded() {
        // No signature may be witness-less or non-claim-less; physically-grounded (tier A) ones must require a
        // balance/topology witness (so a tier can never overclaim its grounding).
        for s in EQUIPMENT_SIGNATURES {
            assert!(
                !s.required_witnesses.is_empty(),
                "{}: needs ≥1 required witness",
                s.signature_id
            );
            assert!(
                !s.non_claims.is_empty(),
                "{}: needs explicit non-claims",
                s.signature_id
            );
            if s.tier == SignatureTier::PhysicallyGrounded {
                let has_phys = s
                    .required_witnesses
                    .iter()
                    .chain(s.supporting_witnesses)
                    .any(|w| w.contains("BalanceWitness") || w.contains("Topology"));
                assert!(
                    has_phys,
                    "{}: tier A must cite a balance/topology witness",
                    s.signature_id
                );
            }
        }
    }

    #[test]
    fn burden_requires_required_and_excludes_forbidden() {
        let pump = EQUIPMENT_SIGNATURES
            .iter()
            .find(|s| s.signature_id == "PUMP-CAV-01")
            .unwrap();
        // required present, no forbidden → candidate admitted
        assert!(pump.meets_burden(pump.required_witnesses));
        // a forbidden witness present → burden NOT met (stays unknown)
        let mut present: Vec<&str> = pump.required_witnesses.to_vec();
        present.push(pump.forbidden_witnesses[0]);
        assert!(
            !pump.meets_burden(&present),
            "a forbidden witness must block admission"
        );
        // missing a required witness → burden NOT met
        assert!(!pump.meets_burden(&[]));
    }
}
