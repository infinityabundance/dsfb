//! Material & QA context witnesses (Wave-4 historian layer): `MaterialLotWitnessV1`,
//! `CertificateOfAnalysisDensorV1`, and `CleanInPlaceWitnessV1`.
//!
//! Process episodes rarely occur in a vacuum: a deviation often coincides with a new feedstock lot, an
//! off-spec certificate of analysis, or a cleaning cycle. These three sealed records attach that operational
//! context to the Court Record so an engineer can *see* the coincidence — without DSFB ever claiming the
//! context caused the episode.
//!
//!   * [`MaterialLotWitnessV1`] — which material lot was active over a window (id / material / supplier).
//!   * [`CertificateOfAnalysisDensorV1`] — a lot's declared CoA assay values against their spec band.
//!   * [`CleanInPlaceWitnessV1`] — a CIP cycle's phases and whether each met its target.
//!
//! Bounded (non-claims): a lot correlation is a **candidate context link, not proof of causation**; a CoA is
//! the **supplier's declared values, not independent lab truth**; a CIP record attests the **cycle as
//! executed, not a validated-clean certificate**. Additive + off the replay path; deterministic, hash-sealed,
//! self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── MaterialLotWitnessV1 ────────────────────────────────────────────────────────────────────────────

/// A hash-sealed material-lot witness (schema v1): a feedstock lot active over a sample window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialLotWitnessV1 {
    pub lot_id: String,
    pub material: String,
    pub supplier: String,
    /// Inclusive sample window `[start, end]` over which this lot was the active feed.
    pub window_start: usize,
    pub window_end: usize,
    pub witness_hash: String,
}

impl MaterialLotWitnessV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"material_lot_witness_v1");
        h.field("lot_id", self.lot_id.as_bytes());
        h.field("material", self.material.as_bytes());
        h.field("supplier", self.supplier.as_bytes());
        h.u64("window_start", self.window_start as u64);
        h.u64("window_end", self.window_end as u64);
        h.finalize_hex()
    }

    pub fn build(
        lot_id: impl Into<String>,
        material: impl Into<String>,
        supplier: impl Into<String>,
        window_start: usize,
        window_end: usize,
    ) -> Self {
        let mut w = MaterialLotWitnessV1 {
            lot_id: lot_id.into(),
            material: material.into(),
            supplier: supplier.into(),
            window_start,
            window_end,
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// True iff `episode` (a sample index) falls inside this lot's active window — a candidate context link.
    pub fn covers(&self, episode_index: usize) -> bool {
        episode_index >= self.window_start && episode_index <= self.window_end
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.witness_hash
    }
}

// ── CertificateOfAnalysisDensorV1 ───────────────────────────────────────────────────────────────────

/// One declared CoA property and its spec band.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoAProperty {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub spec_min: Option<f64>,
    pub spec_max: Option<f64>,
}

impl CoAProperty {
    /// True iff the declared value lies within the declared spec band (open on a missing side).
    pub fn in_spec(&self) -> bool {
        self.spec_min.map(|lo| self.value >= lo).unwrap_or(true)
            && self.spec_max.map(|hi| self.value <= hi).unwrap_or(true)
    }
}

/// A hash-sealed certificate-of-analysis record (schema v1) for a material lot — the supplier's declared
/// assay values, *recorded as-is*. ("Densor" = a sealed, sensor-like declared record.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateOfAnalysisDensorV1 {
    pub lot_id: String,
    pub properties: Vec<CoAProperty>,
    /// Count of declared properties outside their spec band.
    pub n_out_of_spec: usize,
    /// The fixed non-claim, sealed so it cannot be silently dropped.
    pub non_claim: String,
    pub coa_hash: String,
}

impl CertificateOfAnalysisDensorV1 {
    const NON_CLAIM: &'static str =
        "supplier-declared certificate of analysis, recorded as-is; NOT independent lab truth and NOT verified by DSFB";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"certificate_of_analysis_densor_v1");
        h.field("lot_id", self.lot_id.as_bytes());
        for p in &self.properties {
            h.field("name", p.name.as_bytes());
            h.f64q("value", p.value);
            h.field("unit", p.unit.as_bytes());
            h.u64("has_min", p.spec_min.is_some() as u64);
            h.f64q("spec_min", p.spec_min.unwrap_or(0.0));
            h.u64("has_max", p.spec_max.is_some() as u64);
            h.f64q("spec_max", p.spec_max.unwrap_or(0.0));
        }
        h.u64("n_out_of_spec", self.n_out_of_spec as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    pub fn build(lot_id: impl Into<String>, properties: Vec<CoAProperty>) -> Self {
        let n_out_of_spec = properties.iter().filter(|p| !p.in_spec()).count();
        let mut c = CertificateOfAnalysisDensorV1 {
            lot_id: lot_id.into(),
            properties,
            n_out_of_spec,
            non_claim: Self::NON_CLAIM.to_string(),
            coa_hash: String::new(),
        };
        c.coa_hash = c.seal();
        c
    }

    pub fn all_in_spec(&self) -> bool {
        self.n_out_of_spec == 0
    }

    pub fn verify(&self) -> bool {
        let n_out_of_spec = self.properties.iter().filter(|p| !p.in_spec()).count();
        n_out_of_spec == self.n_out_of_spec
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.coa_hash
    }
}

// ── CleanInPlaceWitnessV1 ───────────────────────────────────────────────────────────────────────────

/// One CIP phase as executed against its target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CipPhase {
    /// e.g. `"pre_rinse"`, `"caustic"`, `"intermediate_rinse"`, `"acid"`, `"final_rinse"`.
    pub name: String,
    pub duration_s: f64,
    pub target_duration_s: f64,
    pub temperature_c: f64,
    pub target_temperature_c: f64,
}

impl CipPhase {
    /// True iff the phase met both its duration and temperature targets (within a small tolerance).
    pub fn met_target(&self) -> bool {
        self.duration_s + 1e-9 >= self.target_duration_s
            && self.temperature_c + 1e-9 >= self.target_temperature_c
    }
}

/// A hash-sealed clean-in-place witness (schema v1): the CIP cycle as executed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CleanInPlaceWitnessV1 {
    pub equipment: String,
    pub phases: Vec<CipPhase>,
    /// Count of phases that did not meet their target.
    pub n_phases_off_target: usize,
    pub non_claim: String,
    pub cip_hash: String,
}

impl CleanInPlaceWitnessV1 {
    const NON_CLAIM: &'static str =
        "records the CIP cycle as executed; NOT a cleaning-validation / verified-clean certificate";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"clean_in_place_witness_v1");
        h.field("equipment", self.equipment.as_bytes());
        for p in &self.phases {
            h.field("name", p.name.as_bytes());
            h.f64q("duration_s", p.duration_s);
            h.f64q("target_duration_s", p.target_duration_s);
            h.f64q("temperature_c", p.temperature_c);
            h.f64q("target_temperature_c", p.target_temperature_c);
        }
        h.u64("n_phases_off_target", self.n_phases_off_target as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    pub fn build(equipment: impl Into<String>, phases: Vec<CipPhase>) -> Self {
        let n_phases_off_target = phases.iter().filter(|p| !p.met_target()).count();
        let mut w = CleanInPlaceWitnessV1 {
            equipment: equipment.into(),
            phases,
            n_phases_off_target,
            non_claim: Self::NON_CLAIM.to_string(),
            cip_hash: String::new(),
        };
        w.cip_hash = w.seal();
        w
    }

    pub fn cycle_complete_to_target(&self) -> bool {
        self.n_phases_off_target == 0
    }

    pub fn verify(&self) -> bool {
        let n_off = self.phases.iter().filter(|p| !p.met_target()).count();
        n_off == self.n_phases_off_target
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.cip_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_lot_covers_window_and_self_verifies() {
        let w = MaterialLotWitnessV1::build("LOT-B-2207", "ethylene_feed", "AcmeChem", 120, 168);
        assert!(w.covers(140) && !w.covers(200) && !w.covers(100));
        assert!(w.witness_hash.len() == 64 && w.verify());
        let mut t = w.clone();
        t.window_end = 999;
        assert!(!t.verify());
    }

    #[test]
    fn coa_counts_out_of_spec_and_seals_the_nonclaim() {
        let props = vec![
            CoAProperty {
                name: "purity".into(),
                value: 99.2,
                unit: "wt%".into(),
                spec_min: Some(99.0),
                spec_max: None,
            },
            CoAProperty {
                name: "moisture".into(),
                value: 0.30,
                unit: "wt%".into(),
                spec_min: None,
                spec_max: Some(0.20),
            }, // OOS
        ];
        let c = CertificateOfAnalysisDensorV1::build("LOT-B-2207", props);
        assert_eq!(c.n_out_of_spec, 1);
        assert!(!c.all_in_spec());
        assert!(c.non_claim.contains("NOT independent lab truth"));
        assert!(c.verify());
        let mut t = c.clone();
        t.n_out_of_spec = 0; // forge an all-in-spec verdict
        assert!(!t.verify());
    }

    #[test]
    fn cip_flags_a_short_phase() {
        let phases = vec![
            CipPhase {
                name: "caustic".into(),
                duration_s: 600.0,
                target_duration_s: 600.0,
                temperature_c: 80.0,
                target_temperature_c: 80.0,
            },
            CipPhase {
                name: "final_rinse".into(),
                duration_s: 120.0,
                target_duration_s: 300.0,
                temperature_c: 20.0,
                target_temperature_c: 20.0,
            }, // short
        ];
        let w = CleanInPlaceWitnessV1::build("reactor_R1", phases);
        assert_eq!(w.n_phases_off_target, 1);
        assert!(!w.cycle_complete_to_target());
        assert!(w.non_claim.contains("NOT a cleaning-validation"));
        assert!(w.verify());
    }
}
