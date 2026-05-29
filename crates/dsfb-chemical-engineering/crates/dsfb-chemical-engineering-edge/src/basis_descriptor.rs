//! `BasisDescriptorV1` — the *basis* of a reported quantity (panel item #10).
//!
//! Unit-consistency (`UnitConsistencyCourtV1`) checks *dimension* (°C vs K, bar vs Pa). Orthogonal to that —
//! and a classic plant-data trap a generic ML pipeline never models — is the **basis** a quantity is reported
//! on: mass vs mole, wet vs dry, volume, as-received, or at standard / normal conditions. Two tags can be
//! dimensionally consistent yet not additively comparable (mass fraction vs mole fraction; wet- vs dry-basis
//! composition; per-hour vs per-second). This object records the declared basis + reference state so a balance
//! or composition comparison can state whether the conversion is known.
//!
//! **NON-CLAIM:** this records the *declared* basis and whether a conversion is known; it does not perform the
//! conversion or assert the reported value is correct — a basis mismatch is surfaced as a consistency *caveat*,
//! never a silent correction. Self-sealed; not part of any frozen authority hash. Additive, read-only.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The basis a quantity is reported on. Dimensionally-consistent quantities on different bases are NOT
/// additively comparable without a declared conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantityBasis {
    /// Per unit mass (e.g. mass fraction, kg/kg).
    MassBasis,
    /// Per mole (e.g. mole fraction, mol/mol) — converts to mass basis only via molar masses.
    MoleBasis,
    /// Per unit volume (e.g. g/L, vol%) — density-dependent.
    VolumeBasis,
    /// Including moisture (wet basis) — the as-measured stream.
    WetBasis,
    /// Moisture-free (dry basis) — converts to wet basis only via the known moisture content.
    DryBasis,
    /// As-received (as-sampled, before drying/prep) — common for solids / feedstocks.
    AsReceivedBasis,
    /// Referred to standard conditions (e.g. 0 °C, 1 atm) — for gas flows (Nm³, scfm).
    StandardConditions,
    /// Referred to normal conditions (the site's declared reference T,P) — must be stated explicitly.
    NormalConditions,
}

impl QuantityBasis {
    /// Stable snake_case tag (part of the seal).
    pub fn tag(self) -> &'static str {
        match self {
            QuantityBasis::MassBasis => "mass_basis",
            QuantityBasis::MoleBasis => "mole_basis",
            QuantityBasis::VolumeBasis => "volume_basis",
            QuantityBasis::WetBasis => "wet_basis",
            QuantityBasis::DryBasis => "dry_basis",
            QuantityBasis::AsReceivedBasis => "as_received_basis",
            QuantityBasis::StandardConditions => "standard_conditions",
            QuantityBasis::NormalConditions => "normal_conditions",
        }
    }
}

/// The declared basis of one reported quantity (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasisDescriptorV1 {
    /// What the quantity is (e.g. `"composition"`, `"flow"`, `"concentration"`, `"calorific value"`).
    pub quantity_kind: String,
    /// The basis it is reported on.
    pub basis: QuantityBasis,
    /// The reference state the basis is relative to, as an inspectable string (e.g. `"0 °C, 1 atm (STP)"`,
    /// `"dry gas, moisture removed"`, `"as-received solid"`). Empty when the basis needs none.
    pub reference_state: String,
    /// True iff a deterministic conversion to a comparison basis is KNOWN (e.g. molar masses / moisture
    /// content available); false means a comparison across bases must NOT be asserted.
    pub conversion_known: bool,
    /// The assumptions any conversion would rely on (e.g. `["molar masses from the component list", "moisture = 8% w/w"]`).
    pub conversion_assumptions: Vec<String>,
    /// SHA-256 (via [`CanonicalHasher`]) sealing the descriptor.
    pub descriptor_hash: String,
}

impl BasisDescriptorV1 {
    fn seal(
        quantity_kind: &str,
        basis: QuantityBasis,
        reference_state: &str,
        conversion_known: bool,
        assumptions: &[String],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"basis_descriptor_v1");
        h.field("quantity_kind", quantity_kind.as_bytes());
        h.field("basis", basis.tag().as_bytes());
        h.field("reference_state", reference_state.as_bytes());
        h.u64("conversion_known", conversion_known as u64);
        for a in assumptions {
            h.field("assumption", a.as_bytes());
        }
        h.finalize_hex()
    }

    /// Build a sealed basis descriptor.
    pub fn build(
        quantity_kind: impl Into<String>,
        basis: QuantityBasis,
        reference_state: impl Into<String>,
        conversion_known: bool,
        conversion_assumptions: Vec<String>,
    ) -> Self {
        let quantity_kind = quantity_kind.into();
        let reference_state = reference_state.into();
        let descriptor_hash = Self::seal(
            &quantity_kind,
            basis,
            &reference_state,
            conversion_known,
            &conversion_assumptions,
        );
        BasisDescriptorV1 {
            quantity_kind,
            basis,
            reference_state,
            conversion_known,
            conversion_assumptions,
            descriptor_hash,
        }
    }

    /// True iff two descriptors are additively comparable as-is: same basis, OR a known conversion on both.
    /// A `false` here is exactly the case a balance/composition comparison must flag, not silently combine.
    pub fn comparable_with(&self, other: &BasisDescriptorV1) -> bool {
        self.basis == other.basis || (self.conversion_known && other.conversion_known)
    }

    /// Re-derive the seal and confirm it matches (tamper-evident).
    pub fn verify(&self) -> bool {
        self.descriptor_hash
            == Self::seal(
                &self.quantity_kind,
                self.basis,
                &self.reference_state,
                self.conversion_known,
                &self.conversion_assumptions,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_self_verifies_and_is_deterministic() {
        let d = BasisDescriptorV1::build(
            "composition",
            QuantityBasis::DryBasis,
            "dry gas",
            true,
            vec!["moisture = 8% w/w".into()],
        );
        assert!(d.verify());
        let d2 = BasisDescriptorV1::build(
            "composition",
            QuantityBasis::DryBasis,
            "dry gas",
            true,
            vec!["moisture = 8% w/w".into()],
        );
        assert_eq!(d.descriptor_hash, d2.descriptor_hash);
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut d = BasisDescriptorV1::build(
            "flow",
            QuantityBasis::StandardConditions,
            "0 C, 1 atm",
            false,
            vec![],
        );
        d.reference_state = "25 C, 1 atm".into();
        assert!(!d.verify(), "a changed reference state must break the seal");
    }

    #[test]
    fn cross_basis_comparability_requires_a_known_conversion() {
        // mass fraction vs mole fraction: NOT comparable unless both declare a known conversion.
        let mass =
            BasisDescriptorV1::build("composition", QuantityBasis::MassBasis, "", false, vec![]);
        let mole =
            BasisDescriptorV1::build("composition", QuantityBasis::MoleBasis, "", false, vec![]);
        assert!(
            !mass.comparable_with(&mole),
            "differing bases without a conversion must NOT be comparable"
        );
        // same basis is always comparable
        let mass2 =
            BasisDescriptorV1::build("composition", QuantityBasis::MassBasis, "", false, vec![]);
        assert!(mass.comparable_with(&mass2));
        // both with a known conversion → comparable
        let mole_ok = BasisDescriptorV1::build(
            "composition",
            QuantityBasis::MoleBasis,
            "",
            true,
            vec!["molar masses known".into()],
        );
        let mass_ok = BasisDescriptorV1::build(
            "composition",
            QuantityBasis::MassBasis,
            "",
            true,
            vec!["molar masses known".into()],
        );
        assert!(mass_ok.comparable_with(&mole_ok));
    }
}
