//! `BalanceWitnessV1` — a hash-sealed balance-closure witness record, with reaction-chemistry residual
//! kinds (stoichiometric / yield / selectivity, P61) and the P75 conservation-balance pack
//! (tank-inventory / splitter-mixer / heat-exchanger-energy / element / separator-component / utility-loop).
//!
//! `balance.rs` already recomputes a balance-closure residual from raw columns for the documented
//! physical-balance types (three-tank mass, CSTR energy, quad-tank mass, CSTH energy, tank-volume). A
//! non-zero closure is admitted as a residual stream — "imbalance becomes court-admissible evidence".
//! This module turns that into a first-class **witness record**: it names the balance *kind*, the
//! documented balance equation, the closure units, and seals the residual stream + its summary under a
//! `witness_hash`. It also adds three **reaction-chemistry residual kinds** the physical balances did
//! not cover — `Stoichiometric`, `Yield`, `Selectivity` — each with a pure helper that computes the
//! closure residual, so a reaction's stoichiometry/yield/selectivity imbalance is read the same way.
//!
//! Additive + off the replay path; it records a witness for a residual the pipeline already admits.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The kind of balance the witness closes. The first five mirror the physical balances `balance.rs`
/// already computes; the last three are the reaction-chemistry residual kinds added in P61.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceWitnessKind {
    MassThreeTank,
    EnergyCstr,
    MassQuadTank,
    EnergyCsth,
    MassTankVolume,
    /// Product formed vs reactant consumed under the reaction stoichiometry (closure ≈ 0 if it holds).
    Stoichiometric,
    /// Actual product vs theoretical maximum (yield gap).
    Yield,
    /// Desired product vs total converted (selectivity gap).
    Selectivity,
    // ── P75 balance-pack: six more documented conservation closures (each a residual stream the grammar
    //    admits exactly like the physical balances above; a non-zero closure is court-admissible evidence). ──
    /// Tank/vessel inventory: accumulation rate `dV/dt` vs net flow `Σin − Σout` (unaccounted in/outflow).
    TankInventory,
    /// Splitter / mixer node: total mass in vs total mass out (a node that should conserve total flow).
    SplitterMixer,
    /// Heat-exchanger energy: hot-side duty `(ṁcₚΔT)_hot` vs cold-side duty `(ṁcₚΔT)_cold` (loss/fouling).
    HeatExchangerEnergy,
    /// Element/atom balance across a reaction: atoms of an element in vs out (conserved through reaction).
    ElementBalance,
    /// Separator component balance: feed of a component vs (top + bottoms) of that component.
    SeparatorComponent,
    /// Closed utility loop (steam / cooling-water / refrigerant): supplied vs returned (loop conservation).
    UtilityLoop,
}

impl BalanceWitnessKind {
    fn tag(self) -> &'static str {
        match self {
            BalanceWitnessKind::MassThreeTank => "mass_three_tank",
            BalanceWitnessKind::EnergyCstr => "energy_cstr",
            BalanceWitnessKind::MassQuadTank => "mass_quad_tank",
            BalanceWitnessKind::EnergyCsth => "energy_csth",
            BalanceWitnessKind::MassTankVolume => "mass_tank_volume",
            BalanceWitnessKind::Stoichiometric => "stoichiometric",
            BalanceWitnessKind::Yield => "yield",
            BalanceWitnessKind::Selectivity => "selectivity",
            BalanceWitnessKind::TankInventory => "tank_inventory",
            BalanceWitnessKind::SplitterMixer => "splitter_mixer",
            BalanceWitnessKind::HeatExchangerEnergy => "heat_exchanger_energy",
            BalanceWitnessKind::ElementBalance => "element_balance",
            BalanceWitnessKind::SeparatorComponent => "separator_component",
            BalanceWitnessKind::UtilityLoop => "utility_loop",
        }
    }
}

/// Stoichiometric closure residual: `product_formed − stoich_ratio · reactant_consumed`, per sample.
/// Zero when the observed product matches what the stoichiometry predicts from the consumed reactant.
/// Lengths must match; the shorter length governs (extra tail is ignored).
pub fn stoichiometric_residual(
    reactant_consumed: &[f64],
    product_formed: &[f64],
    stoich_ratio: f64,
) -> Vec<f64> {
    reactant_consumed
        .iter()
        .zip(product_formed)
        .map(|(&rc, &pf)| pf - stoich_ratio * rc)
        .collect()
}

/// Yield closure residual: `theoretical_max − actual_product`, per sample (the yield *gap*; 0 = ideal).
pub fn yield_residual(actual_product: &[f64], theoretical_max: &[f64]) -> Vec<f64> {
    actual_product
        .iter()
        .zip(theoretical_max)
        .map(|(&ap, &tm)| tm - ap)
        .collect()
}

/// Selectivity closure residual: `target_selectivity − desired/total`, per sample (0 = at target).
/// A `total` of 0 yields a 0 residual for that sample (no conversion ⇒ selectivity undefined, not a fault).
pub fn selectivity_residual(
    desired_product: &[f64],
    total_converted: &[f64],
    target_selectivity: f64,
) -> Vec<f64> {
    desired_product
        .iter()
        .zip(total_converted)
        .map(|(&d, &t)| {
            if t == 0.0 {
                0.0
            } else {
                target_selectivity - d / t
            }
        })
        .collect()
}

// ── P75 balance-pack closure helpers ────────────────────────────────────────────────────────────────
//
// All six conservation laws below reduce to "what goes in must come out (or accumulate)". The canonical
// `in − out` form is shared by four of them ([`inout_closure_residual`]) — that the math is identical is
// honest: conservation *is* conservation; the [`BalanceWitnessKind`] + equation string + units carry the
// distinct physical meaning. The two genuinely-different shapes (a difference of `ṁcₚΔT` products for the
// heat-exchanger duty; a three-term feed = top + bottoms split for the separator) get their own helpers.

/// Canonical conservation closure `lhs[k] − rhs[k]`, per sample (the shorter length governs). Zero when the
/// law closes; a sustained non-zero is the imbalance evidence. Serves the conservation closures whose two
/// sides are already aggregated to one stream each:
///   * **TankInventory** — `lhs` = accumulation rate `dV/dt`, `rhs` = net flow `Σin − Σout`.
///   * **SplitterMixer** — `lhs` = total mass in, `rhs` = total mass out.
///   * **ElementBalance** — `lhs` = atoms of an element entering, `rhs` = atoms leaving.
///   * **UtilityLoop**   — `lhs` = utility supplied, `rhs` = utility returned.
pub fn inout_closure_residual(lhs: &[f64], rhs: &[f64]) -> Vec<f64> {
    lhs.iter().zip(rhs).map(|(&a, &b)| a - b).collect()
}

/// Heat-exchanger energy closure `(ṁcₚ·ΔT)_hot − (ṁcₚ·ΔT)_cold`, per sample. `mcp_*` is the heat-capacity
/// rate `ṁ·cₚ` of each side and `dt_*` its temperature change (hot side: `T_in − T_out`; cold side:
/// `T_out − T_in`, both positive in normal counter-current duty). Zero when the duty transferred out of the
/// hot stream equals the duty into the cold stream (no loss / no fouling-driven mismatch).
pub fn hx_energy_residual(
    mcp_hot: &[f64],
    dt_hot: &[f64],
    mcp_cold: &[f64],
    dt_cold: &[f64],
) -> Vec<f64> {
    let n = mcp_hot
        .len()
        .min(dt_hot.len())
        .min(mcp_cold.len())
        .min(dt_cold.len());
    (0..n)
        .map(|k| mcp_hot[k] * dt_hot[k] - mcp_cold[k] * dt_cold[k])
        .collect()
}

/// Separator component closure `feed_component − (top_component + bottom_component)`, per sample (the shorter
/// length governs). Zero when the component fed splits exactly into the two product streams (a column /
/// flash / decanter component balance); a non-zero closure flags an unaccounted component path or a
/// composition-measurement inconsistency.
pub fn separator_component_residual(
    feed_comp: &[f64],
    top_comp: &[f64],
    bottom_comp: &[f64],
) -> Vec<f64> {
    let n = feed_comp.len().min(top_comp.len()).min(bottom_comp.len());
    (0..n)
        .map(|k| feed_comp[k] - (top_comp[k] + bottom_comp[k]))
        .collect()
}

/// A hash-sealed balance-closure witness (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceWitnessV1 {
    pub dataset: String,
    pub kind: BalanceWitnessKind,
    /// The documented balance/closure equation (disclosed, e.g. `"dV/dt = q_in − q_out"`).
    pub balance_equation: String,
    /// Engineering units of the closure residual (e.g. `"cm^3/s"`, `"J/min"`, `"mol/min"`, `"dimensionless"`).
    pub closure_units: String,
    pub n_samples: usize,
    /// Maximum `|residual|` across the stream (a non-zero peak is the imbalance evidence).
    pub peak_abs_residual: f64,
    /// SHA-256 of the residual stream (quantized f64), so the exact stream is sealed.
    pub residual_hash: String,
    /// SHA-256 sealing the whole record.
    pub witness_hash: String,
}

impl BalanceWitnessV1 {
    fn hash_residual(residual: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"balance_residual_stream_v1");
        for &v in residual {
            h.f64q("r", v);
        }
        h.finalize_hex()
    }

    fn seal(
        dataset: &str,
        kind: BalanceWitnessKind,
        balance_equation: &str,
        closure_units: &str,
        n_samples: usize,
        peak_abs_residual: f64,
        residual_hash: &str,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"balance_witness_v1");
        h.field("dataset", dataset.as_bytes());
        h.field("kind", kind.tag().as_bytes());
        h.field("balance_equation", balance_equation.as_bytes());
        h.field("closure_units", closure_units.as_bytes());
        h.u64("n_samples", n_samples as u64);
        h.f64q("peak_abs_residual", peak_abs_residual);
        h.field("residual_hash", residual_hash.as_bytes());
        h.finalize_hex()
    }

    /// Build + seal a balance witness from a (precomputed) closure-residual stream.
    pub fn build(
        dataset: impl Into<String>,
        kind: BalanceWitnessKind,
        balance_equation: impl Into<String>,
        closure_units: impl Into<String>,
        residual: &[f64],
    ) -> Self {
        let dataset = dataset.into();
        let balance_equation = balance_equation.into();
        let closure_units = closure_units.into();
        let peak_abs_residual = residual
            .iter()
            .filter(|x| x.is_finite())
            .fold(0.0f64, |a, &x| a.max(x.abs()));
        let residual_hash = Self::hash_residual(residual);
        let witness_hash = Self::seal(
            &dataset,
            kind,
            &balance_equation,
            &closure_units,
            residual.len(),
            peak_abs_residual,
            &residual_hash,
        );
        BalanceWitnessV1 {
            dataset,
            kind,
            balance_equation,
            closure_units,
            n_samples: residual.len(),
            peak_abs_residual,
            residual_hash,
            witness_hash,
        }
    }

    /// Re-derive the seal (including the residual hash, supplied again) and check it matches.
    pub fn verify(&self, residual: &[f64]) -> bool {
        let residual_hash = Self::hash_residual(residual);
        residual_hash == self.residual_hash
            && Self::seal(
                &self.dataset,
                self.kind,
                &self.balance_equation,
                &self.closure_units,
                self.n_samples,
                self.peak_abs_residual,
                &residual_hash,
            ) == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stoichiometric_residual_zero_when_stoichiometry_holds() {
        // 2 mol reactant → 1 mol product (ratio 0.5): product = 0.5·reactant ⇒ residual 0.
        let rc = vec![2.0, 4.0, 6.0];
        let pf = vec![1.0, 2.0, 3.0];
        let r = stoichiometric_residual(&rc, &pf, 0.5);
        assert!(r.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn selectivity_handles_zero_conversion() {
        let r = selectivity_residual(&[0.0, 5.0], &[0.0, 10.0], 0.8);
        assert_eq!(r[0], 0.0); // no conversion ⇒ no residual
        assert!((r[1] - (0.8 - 0.5)).abs() < 1e-12);
    }

    #[test]
    fn inout_closure_serves_the_canonical_conservation_laws() {
        // A tank whose accumulation (dV/dt = +2) matches its net inflow (10 − 8 = 2) closes to ~0; a leak
        // of 1 (net inflow 2 but accumulation only 1) leaves a sustained −1 closure.
        let acc = vec![2.0, 2.0, 2.0];
        let net_in = vec![2.0, 2.0, 2.0];
        assert!(inout_closure_residual(&acc, &net_in)
            .iter()
            .all(|&x| x.abs() < 1e-12));
        let leak = inout_closure_residual(&[1.0, 1.0, 1.0], &net_in);
        assert!(leak.iter().all(|&x| (x - (-1.0)).abs() < 1e-12));
    }

    #[test]
    fn hx_energy_closes_when_duties_match_and_flags_a_fouling_mismatch() {
        // Balanced: hot ṁcₚ=10, ΔT=20 ⇒ 200; cold ṁcₚ=20, ΔT=10 ⇒ 200 ⇒ closure 0.
        let r = hx_energy_residual(&[10.0], &[20.0], &[20.0], &[10.0]);
        assert!(r[0].abs() < 1e-12);
        // Fouling cuts the cold-side pickup to ΔT=8 ⇒ cold duty 160 ⇒ closure +40 (hot gave up more than
        // the cold stream received — the unaccounted loss across the fouled surface).
        let r2 = hx_energy_residual(&[10.0], &[20.0], &[20.0], &[8.0]);
        assert!((r2[0] - 40.0).abs() < 1e-12);
    }

    #[test]
    fn separator_component_closes_on_an_exact_split() {
        // Feed 100 of a component splits 70 top + 30 bottoms ⇒ closure 0; a 5-unit unaccounted path ⇒ +5.
        let r = separator_component_residual(&[100.0], &[70.0], &[30.0]);
        assert!(r[0].abs() < 1e-12);
        let r2 = separator_component_residual(&[100.0], &[70.0], &[25.0]);
        assert!((r2[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn new_balance_kinds_seal_and_self_verify() {
        // Each new kind wraps its closure stream in the same sealed, self-verifying witness record.
        let r = hx_energy_residual(&[10.0, 10.0], &[20.0, 20.0], &[20.0, 20.0], &[10.0, 8.0]);
        let w = BalanceWitnessV1::build(
            "hx_demo",
            BalanceWitnessKind::HeatExchangerEnergy,
            "(mcp*dT)_hot - (mcp*dT)_cold",
            "W",
            &r,
        );
        assert_eq!(w.kind, BalanceWitnessKind::HeatExchangerEnergy);
        assert!((w.peak_abs_residual - 40.0).abs() < 1e-9);
        assert!(w.verify(&r) && w.witness_hash.len() == 64);
        // Every new kind has a distinct stable tag (no accidental aliasing in the hash protocol).
        let kinds = [
            BalanceWitnessKind::TankInventory,
            BalanceWitnessKind::SplitterMixer,
            BalanceWitnessKind::HeatExchangerEnergy,
            BalanceWitnessKind::ElementBalance,
            BalanceWitnessKind::SeparatorComponent,
            BalanceWitnessKind::UtilityLoop,
        ];
        let mut tags: Vec<&str> = kinds.iter().map(|k| k.tag()).collect();
        tags.sort();
        tags.dedup();
        assert_eq!(tags.len(), kinds.len());
    }

    #[test]
    fn witness_is_deterministic_self_verifying_and_tamper_evident() {
        let residual = yield_residual(&[8.0, 9.0, 9.5], &[10.0, 10.0, 10.0]); // gaps 2,1,0.5
        let a = BalanceWitnessV1::build(
            "cstr_reactor",
            BalanceWitnessKind::Yield,
            "Y = P_actual/P_max",
            "mol/min",
            &residual,
        );
        let b = BalanceWitnessV1::build(
            "cstr_reactor",
            BalanceWitnessKind::Yield,
            "Y = P_actual/P_max",
            "mol/min",
            &residual,
        );
        assert_eq!(a, b);
        assert_eq!(a.witness_hash.len(), 64);
        assert!((a.peak_abs_residual - 2.0).abs() < 1e-12);
        assert!(a.verify(&residual));
        // A different residual stream fails verification against the sealed record.
        assert!(!a.verify(&[0.0, 0.0, 0.0]));
    }
}
