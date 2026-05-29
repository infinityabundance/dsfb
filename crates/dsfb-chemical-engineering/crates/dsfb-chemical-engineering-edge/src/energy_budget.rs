//! `ResidualEnergyBudgetV1` (Wave-3 physics) — decompose an episode's total residual "energy" into an
//! interpretable budget, so an operator can see **where the anomaly's weight lives**.
//!
//! DSFB fuses many residual channels into one episode. A single fused score answers "how anomalous?" but not
//! "anomalous *in what*?". This object attributes the episode's total residual energy (sum of squares — a
//! variance-like magnitude) across named, interpretable components, the way a heat/energy balance is read as
//! a budget: e.g. *"62 % of this episode's residual energy is in the mass-balance closure, 24 % spectral, 14 %
//! controller effort"*. The dominant share points the engineer at the channel carrying the anomaly, and the
//! full split travels with the case file as sealed evidence.
//!
//! Canonical component categories (callers may use any label; these are the documented set):
//! `score` · `residual` · `spectral` · `variable_group:<name>` · `balance_closure` · `controller_effort`.
//!
//! Bounded (non-claims): a budget is an **interpretability decomposition of residual magnitude, not a causal
//! attribution**. A large share in a component means that channel carries most of the residual weight in the
//! window — *not* that it is the cause, and not a sensitivity or contribution-to-detection guarantee.
//! Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Documented canonical component labels (callers may also use `variable_group:<name>` or any custom label).
pub mod category {
    pub const SCORE: &str = "score";
    pub const RESIDUAL: &str = "residual";
    pub const SPECTRAL: &str = "spectral";
    pub const BALANCE_CLOSURE: &str = "balance_closure";
    pub const CONTROLLER_EFFORT: &str = "controller_effort";
}

/// Sum-of-squares "energy" of a residual stream (a variance-like magnitude; non-finite samples are skipped).
pub fn stream_energy(stream: &[f64]) -> f64 {
    stream.iter().filter(|x| x.is_finite()).map(|x| x * x).sum()
}

/// One component of the budget: its label, absolute energy, and share of the total.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetComponent {
    pub label: String,
    pub energy: f64,
    /// Share of the total energy in `[0, 1]` (0 when the total is 0).
    pub fraction: f64,
}

/// A hash-sealed residual-energy budget (schema v1) for one episode/window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidualEnergyBudgetV1 {
    pub episode_ref: String,
    /// Components in the caller-supplied order (deterministic; not re-sorted).
    pub components: Vec<BudgetComponent>,
    pub total_energy: f64,
    /// Label of the largest-energy component (ties broken by first occurrence); `"none"` if total is 0.
    pub dominant_component: String,
    /// Share of the dominant component (0 if total is 0).
    pub dominant_fraction: f64,
    pub budget_hash: String,
}

impl ResidualEnergyBudgetV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"residual_energy_budget_v1");
        h.field("episode_ref", self.episode_ref.as_bytes());
        for c in &self.components {
            h.field("label", c.label.as_bytes());
            h.f64q("energy", c.energy);
            h.f64q("fraction", c.fraction);
        }
        h.f64q("total_energy", self.total_energy);
        h.field("dominant_component", self.dominant_component.as_bytes());
        h.f64q("dominant_fraction", self.dominant_fraction);
        h.finalize_hex()
    }

    /// Build + seal a budget from `(label, energy)` pairs. Negative energies are clamped to 0 (energy is a
    /// sum of squares; a negative input is a caller error, treated as no contribution). The fractions, total,
    /// and dominant component are derived deterministically.
    pub fn build(episode_ref: impl Into<String>, raw: &[(String, f64)]) -> Self {
        let energies: Vec<f64> = raw
            .iter()
            .map(|(_, e)| if e.is_finite() && *e > 0.0 { *e } else { 0.0 })
            .collect();
        let total_energy: f64 = energies.iter().sum();
        let components: Vec<BudgetComponent> = raw
            .iter()
            .zip(&energies)
            .map(|((label, _), &energy)| BudgetComponent {
                label: label.clone(),
                energy,
                fraction: if total_energy > 0.0 {
                    energy / total_energy
                } else {
                    0.0
                },
            })
            .collect();
        // Dominant = max energy, ties broken by first occurrence (a stable, deterministic scan).
        let (dominant_component, dominant_fraction) = components
            .iter()
            .fold(None::<&BudgetComponent>, |best, c| match best {
                Some(b) if b.energy >= c.energy => Some(b),
                _ => Some(c),
            })
            .filter(|_| total_energy > 0.0)
            .map(|c| (c.label.clone(), c.fraction))
            .unwrap_or_else(|| ("none".to_string(), 0.0));
        let mut b = ResidualEnergyBudgetV1 {
            episode_ref: episode_ref.into(),
            components,
            total_energy,
            dominant_component,
            dominant_fraction,
            budget_hash: String::new(),
        };
        b.budget_hash = b.seal();
        b
    }

    /// Convenience: build from named residual *streams*, computing each component's energy as the sum of
    /// squares of its stream ([`stream_energy`]).
    pub fn from_streams(episode_ref: impl Into<String>, streams: &[(String, &[f64])]) -> Self {
        let raw: Vec<(String, f64)> = streams
            .iter()
            .map(|(l, s)| (l.clone(), stream_energy(s)))
            .collect();
        Self::build(episode_ref, &raw)
    }

    /// Re-derive the fractions, total, and dominant from the component energies and re-seal — catches a
    /// tampered fraction, total, dominant label, or hash (all are a pure function of the energies + order).
    pub fn verify(&self) -> bool {
        let raw: Vec<(String, f64)> = self
            .components
            .iter()
            .map(|c| (c.label.clone(), c.energy))
            .collect();
        Self::build(self.episode_ref.clone(), &raw) == *self
    }

    /// One-line-per-component render (share %, energy) + the dominant, for reports.
    pub fn render(&self) -> String {
        let mut s = format!(
            "residual-energy budget [{}] (total {:.4}):\n",
            self.episode_ref, self.total_energy
        );
        for c in &self.components {
            s.push_str(&format!(
                "  {:>6.1}%  {}  (energy {:.4})\n",
                100.0 * c.fraction,
                c.label,
                c.energy
            ));
        }
        s.push_str(&format!(
            "dominant: {} ({:.1}%)\nbudget_hash: {}\n",
            self.dominant_component,
            100.0 * self.dominant_fraction,
            self.budget_hash
        ));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractions_sum_to_one_and_dominant_is_the_largest() {
        let raw = vec![
            (category::BALANCE_CLOSURE.to_string(), 62.0),
            (category::SPECTRAL.to_string(), 24.0),
            (category::CONTROLLER_EFFORT.to_string(), 14.0),
        ];
        let b = ResidualEnergyBudgetV1::build("idx=120..168", &raw);
        assert!((b.total_energy - 100.0).abs() < 1e-12);
        let frac_sum: f64 = b.components.iter().map(|c| c.fraction).sum();
        assert!((frac_sum - 1.0).abs() < 1e-12);
        assert_eq!(b.dominant_component, category::BALANCE_CLOSURE);
        assert!((b.dominant_fraction - 0.62).abs() < 1e-12);
        assert!(b.budget_hash.len() == 64 && b.verify());
    }

    #[test]
    fn from_streams_uses_sum_of_squares_energy() {
        // residual stream [3,4] has energy 9+16 = 25; spectral [0,0] has 0 ⇒ residual dominates 100%.
        let resid = [3.0, 4.0];
        let spec = [0.0, 0.0];
        let b = ResidualEnergyBudgetV1::from_streams(
            "e",
            &[
                (category::RESIDUAL.to_string(), &resid),
                (category::SPECTRAL.to_string(), &spec),
            ],
        );
        assert!((b.total_energy - 25.0).abs() < 1e-12);
        assert_eq!(b.dominant_component, category::RESIDUAL);
        assert!((b.dominant_fraction - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_energy_budget_is_well_defined() {
        let b = ResidualEnergyBudgetV1::build(
            "quiet",
            &[("score".into(), 0.0), ("residual".into(), 0.0)],
        );
        assert_eq!(b.total_energy, 0.0);
        assert_eq!(b.dominant_component, "none");
        assert_eq!(b.dominant_fraction, 0.0);
        assert!(b.components.iter().all(|c| c.fraction == 0.0));
        assert!(b.verify());
    }

    #[test]
    fn tampering_a_fraction_breaks_the_seal() {
        let mut b = ResidualEnergyBudgetV1::build("e", &[("a".into(), 1.0), ("b".into(), 3.0)]);
        assert!(b.verify());
        b.components[0].fraction = 0.9; // forge a's share upward
        assert!(!b.verify());
    }
}
