//! `HierarchicalMultiScaleFusionV1` (Wave-7 semiotics) — roll episodes up the plant hierarchy
//! **sensor → unit → plant-wide**, with a cross-scale consistency invariant.
//!
//! A single anomalous sensor is weak evidence; the same anomaly corroborated across a unit's sensors, and
//! across multiple units plant-wide, is strong. This object aggregates per-sensor firings into per-unit
//! rollups (a unit fires when enough of its sensors do) and into a plant-wide verdict (the plant fires when
//! enough units do), and records the **cross-scale consistency** invariant: a higher-scale conclusion is
//! always supported by the lower scales beneath it (no spurious top-level firing without bottom-up support).
//!
//! Bounded (non-claims): the rollup aggregates *evidence by scale*, it does not assert causation or that a
//! plant-wide firing has a single cause — it says the anomaly is corroborated across scales. Additive + off
//! the replay path; deterministic, hash-sealed, self-verifying.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One unit's rollup of its sensors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitRollup {
    pub unit_id: String,
    pub n_sensors: usize,
    pub n_fired: usize,
    pub fired: bool,
}

/// A hash-sealed hierarchical multi-scale fusion (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalMultiScaleFusionV1 {
    pub min_sensors_per_unit: usize,
    pub min_units_for_plant: usize,
    pub n_sensors_fired: usize,
    /// Per-unit rollups, sorted by `unit_id` for determinism.
    pub units: Vec<UnitRollup>,
    pub n_units_fired: usize,
    pub plant_fired: bool,
    /// True iff the rollup is monotone-supported: `plant_fired` ⇒ `n_units_fired ≥ min_units_for_plant` and
    /// every fired unit had `≥ min_sensors_per_unit` fired sensors (a verifiable cross-scale invariant).
    pub cross_scale_consistent: bool,
    pub fusion_hash: String,
}

impl HierarchicalMultiScaleFusionV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"hierarchical_multi_scale_fusion_v1");
        h.u64("min_sensors_per_unit", self.min_sensors_per_unit as u64);
        h.u64("min_units_for_plant", self.min_units_for_plant as u64);
        h.u64("n_sensors_fired", self.n_sensors_fired as u64);
        for u in &self.units {
            h.field("unit_id", u.unit_id.as_bytes());
            h.u64("n_sensors", u.n_sensors as u64);
            h.u64("n_fired", u.n_fired as u64);
            h.u64("fired", u.fired as u64);
        }
        h.u64("n_units_fired", self.n_units_fired as u64);
        h.u64("plant_fired", self.plant_fired as u64);
        h.u64("cross_scale_consistent", self.cross_scale_consistent as u64);
        h.finalize_hex()
    }

    /// Compute the rollup from `(unit_id, sensor_fired)` rows. A unit fires when `≥ min_sensors_per_unit` of
    /// its sensors fired; the plant fires when `≥ min_units_for_plant` units fired.
    pub fn build(
        sensor_firings: &[(String, bool)],
        min_sensors_per_unit: usize,
        min_units_for_plant: usize,
    ) -> Self {
        // Group by unit (BTreeMap → deterministic, sorted iteration).
        let mut by_unit: BTreeMap<String, (usize, usize)> = BTreeMap::new(); // unit -> (n_sensors, n_fired)
        for (unit, fired) in sensor_firings {
            let e = by_unit.entry(unit.clone()).or_insert((0, 0));
            e.0 += 1;
            if *fired {
                e.1 += 1;
            }
        }
        let units: Vec<UnitRollup> = by_unit
            .into_iter()
            .map(|(unit_id, (n_sensors, n_fired))| UnitRollup {
                unit_id,
                n_sensors,
                n_fired,
                fired: n_fired >= min_sensors_per_unit.max(1),
            })
            .collect();
        let n_sensors_fired = sensor_firings.iter().filter(|(_, f)| *f).count();
        let n_units_fired = units.iter().filter(|u| u.fired).count();
        let plant_fired = n_units_fired >= min_units_for_plant.max(1);
        // Cross-scale invariant: a plant firing is supported by enough fired units, each itself supported.
        let cross_scale_consistent = !plant_fired
            || (n_units_fired >= min_units_for_plant.max(1)
                && units
                    .iter()
                    .filter(|u| u.fired)
                    .all(|u| u.n_fired >= min_sensors_per_unit.max(1)));
        let mut f = HierarchicalMultiScaleFusionV1 {
            min_sensors_per_unit,
            min_units_for_plant,
            n_sensors_fired,
            units,
            n_units_fired,
            plant_fired,
            cross_scale_consistent,
            fusion_hash: String::new(),
        };
        f.fusion_hash = f.seal();
        f
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.fusion_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn rolls_up_sensor_to_unit_to_plant() {
        // Unit A: 2 of 3 sensors fired; Unit B: 2 of 2 fired; Unit C: 0 of 2 fired.
        // min_sensors_per_unit=2 → A and B fire, C does not; min_units_for_plant=2 → plant fires.
        let firings = vec![
            (s("A"), true),
            (s("A"), true),
            (s("A"), false),
            (s("B"), true),
            (s("B"), true),
            (s("C"), false),
            (s("C"), false),
        ];
        let f = HierarchicalMultiScaleFusionV1::build(&firings, 2, 2);
        assert_eq!(f.n_sensors_fired, 4);
        assert_eq!(f.units.len(), 3); // A, B, C (sorted)
        assert!(f.units[0].fired && f.units[1].fired && !f.units[2].fired);
        assert_eq!(f.n_units_fired, 2);
        assert!(f.plant_fired && f.cross_scale_consistent);
        assert!(f.fusion_hash.len() == 64 && f.verify());
    }

    #[test]
    fn plant_does_not_fire_without_enough_units() {
        // Only one unit fires; min_units_for_plant=2 → plant does not fire.
        let firings = vec![
            (s("A"), true),
            (s("A"), true),
            (s("B"), false),
            (s("B"), false),
        ];
        let f = HierarchicalMultiScaleFusionV1::build(&firings, 2, 2);
        assert_eq!(f.n_units_fired, 1);
        assert!(!f.plant_fired && f.cross_scale_consistent); // !plant ⇒ trivially consistent
        assert!(f.verify());
    }

    #[test]
    fn tampering_a_rollup_breaks_the_seal() {
        let firings = vec![(s("A"), true), (s("A"), true)];
        let mut f = HierarchicalMultiScaleFusionV1::build(&firings, 2, 1);
        assert!(f.verify());
        f.plant_fired = false; // forge away the plant firing
        assert!(!f.verify());
    }
}
