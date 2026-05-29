//! `FirstPrinciplesWitnessAdapterV1` + `EquationResidualPassportV1` (Wave-3 physics) — turn a
//! **first-principles relation** into a residual witness channel, with a sealed provenance passport.
//!
//! A statistical residual ("the SPE is high") is dimensionless and opaque to a chemical engineer. When a
//! governing equation *and* its parameters are available — an Arrhenius rate, an Antoine vapour pressure, a
//! Raoult partial pressure, a Henry's-law solubility, a Newton heat-transfer duty, a pump head curve, a
//! valve `Cv` flow — the engineer can compute what the equation *predicts* from the measured inputs and read
//! the **model–plant residual** `measured − predicted` directly in engineering units. That residual is then
//! admissible as an ordinary DSFB witness channel (exactly like the balance-closure witness), so
//! model–plant mismatch becomes court-admissible evidence rather than a black-box score.
//!
//! Two coupled objects:
//!   * [`FirstPrinciplesEquation`] — a small, extensible bank of governing equations, each a *pure* function
//!     `predict(inputs) → output` with a documented input layout, output unit, assumptions, and validity range.
//!   * [`FirstPrinciplesWitnessAdapterV1`] — runs an equation over a measured series + per-sample inputs,
//!     produces the residual stream summary, counts how many samples fell inside the equation's validity
//!     range, and seals it.
//!   * [`EquationResidualPassportV1`] — the per-equation passport: equation id / form / units / assumptions /
//!     validity / parameter source / a `calibration_hash` over the exact parameters / known-invalid regimes.
//!
//! Bounded (non-claims): the equation is an **added witness channel, not exact physical modelling**. A small
//! residual is consistency with the supplied correlation + parameters, not proof the model is correct; a
//! large residual is candidate evidence of model–plant mismatch, *never* a root-cause or a calibrated state
//! estimate. Outside the stated validity range the residual is reported but flagged out-of-validity. Additive
//! + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Universal gas constant (J·mol⁻¹·K⁻¹) — the only physical constant the bank needs.
const R_GAS: f64 = 8.314_462_618;

/// A governing equation with its parameters. Each variant documents the **per-sample input layout** its
/// [`predict`](FirstPrinciplesEquation::predict) expects; the adapter aligns one such input vector with each
/// measured sample. The bank is deliberately broad (prior-art disclosure of the first-principles surface
/// DSFB reasons over) and extensible — new correlations slot in as new variants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FirstPrinciplesEquation {
    /// Arrhenius rate constant `k = k0·exp(−Ea/(R·T))`. Inputs `[T_K]`; predicts `k` (units of `k0`).
    Arrhenius { k0: f64, ea_j_per_mol: f64 },
    /// Antoine vapour pressure `log10(P*) = A − B/(C + T_C)` (chosen convention: `P*` in kPa, `T` in °C).
    /// Inputs `[T_C]`; predicts saturation pressure `P*`. Valid only for `T ∈ [t_min, t_max]`.
    Antoine {
        a: f64,
        b: f64,
        c: f64,
        t_min_c: f64,
        t_max_c: f64,
    },
    /// Raoult partial pressure `p_i = x_i · Psat`. Inputs `[x_i, Psat]`; predicts `p_i` (Psat's units).
    Raoult,
    /// Henry's-law dissolved concentration `c = p / H`. Inputs `[p]`; predicts `c`.
    Henry { h: f64 },
    /// Newton heat-transfer duty `Q = U·A·ΔT`. Inputs `[ΔT]`; predicts duty `Q` (W if `ua` in W/K).
    HeatTransferUADt { ua_w_per_k: f64 },
    /// Centrifugal-pump head curve `H = h0 − k·Q²`. Inputs `[Q]`; predicts developed head `H`.
    PumpHeadCurve { h0: f64, k: f64 },
    /// Control-valve flow `Q = Cv · opening · sqrt(ΔP / SG)`. Inputs `[opening_frac, ΔP, SG]`; predicts `Q`.
    ValveFlowCv { cv: f64 },
}

/// Static metadata describing one equation for the passport and for human reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquationMetadata {
    pub id: String,
    pub form: String,
    pub output_unit: String,
    pub input_layout: String,
    pub assumptions: String,
    pub validity: String,
    pub known_invalid_regimes: String,
}

impl FirstPrinciplesEquation {
    /// Predict the equation's output for one sample's inputs. Returns `None` on wrong input arity or a
    /// non-finite / structurally invalid argument (e.g. a zero denominator) — the adapter records such a
    /// sample as a non-finite residual rather than fabricating a value.
    pub fn predict(&self, inputs: &[f64]) -> Option<f64> {
        let finite = inputs.iter().all(|x| x.is_finite());
        if !finite {
            return None;
        }
        match *self {
            FirstPrinciplesEquation::Arrhenius { k0, ea_j_per_mol } => {
                let t = *inputs.first()?;
                if t <= 0.0 {
                    return None; // absolute temperature must be positive
                }
                Some(k0 * (-ea_j_per_mol / (R_GAS * t)).exp())
            }
            FirstPrinciplesEquation::Antoine { a, b, c, .. } => {
                let t = *inputs.first()?;
                let denom = c + t;
                if denom == 0.0 {
                    return None;
                }
                Some(10f64.powf(a - b / denom))
            }
            FirstPrinciplesEquation::Raoult => {
                let (&x, &psat) = (inputs.first()?, inputs.get(1)?);
                Some(x * psat)
            }
            FirstPrinciplesEquation::Henry { h } => {
                if h == 0.0 {
                    return None;
                }
                Some(*inputs.first()? / h)
            }
            FirstPrinciplesEquation::HeatTransferUADt { ua_w_per_k } => {
                Some(ua_w_per_k * *inputs.first()?)
            }
            FirstPrinciplesEquation::PumpHeadCurve { h0, k } => {
                let q = *inputs.first()?;
                Some(h0 - k * q * q)
            }
            FirstPrinciplesEquation::ValveFlowCv { cv } => {
                let (&open, &dp, &sg) = (inputs.first()?, inputs.get(1)?, inputs.get(2)?);
                if sg <= 0.0 || dp < 0.0 {
                    return None;
                }
                Some(cv * open * (dp / sg).sqrt())
            }
        }
    }

    /// True iff this sample's inputs lie inside the equation's documented validity range.
    pub fn within_validity(&self, inputs: &[f64]) -> bool {
        if !inputs.iter().all(|x| x.is_finite()) {
            return false;
        }
        match *self {
            FirstPrinciplesEquation::Arrhenius { .. } => inputs.first().is_some_and(|&t| t > 0.0),
            FirstPrinciplesEquation::Antoine {
                t_min_c, t_max_c, ..
            } => inputs
                .first()
                .is_some_and(|&t| t >= t_min_c && t <= t_max_c),
            FirstPrinciplesEquation::Raoult => {
                inputs.first().is_some_and(|&x| (0.0..=1.0).contains(&x))
                    && inputs.get(1).is_some_and(|&p| p >= 0.0)
            }
            FirstPrinciplesEquation::Henry { h } => {
                h > 0.0 && inputs.first().is_some_and(|&p| p >= 0.0)
            }
            FirstPrinciplesEquation::HeatTransferUADt { .. } => !inputs.is_empty(),
            FirstPrinciplesEquation::PumpHeadCurve { h0, k } => {
                // Valid up to the curve's zero-head runout flow Q* = sqrt(h0/k); beyond it the quadratic
                // model goes negative and is non-physical (a known-invalid regime).
                inputs
                    .first()
                    .is_some_and(|&q| q >= 0.0 && (k <= 0.0 || h0 < 0.0 || q * q <= h0 / k))
            }
            FirstPrinciplesEquation::ValveFlowCv { .. } => {
                inputs.first().is_some_and(|&o| (0.0..=1.0).contains(&o))
                    && inputs.get(1).is_some_and(|&dp| dp >= 0.0)
                    && inputs.get(2).is_some_and(|&sg| sg > 0.0)
            }
        }
    }

    /// Stable id used in the passport, the seal, and reports.
    pub fn id(&self) -> &'static str {
        match self {
            FirstPrinciplesEquation::Arrhenius { .. } => "arrhenius",
            FirstPrinciplesEquation::Antoine { .. } => "antoine",
            FirstPrinciplesEquation::Raoult => "raoult",
            FirstPrinciplesEquation::Henry { .. } => "henry",
            FirstPrinciplesEquation::HeatTransferUADt { .. } => "heat_transfer_ua_dt",
            FirstPrinciplesEquation::PumpHeadCurve { .. } => "pump_head_curve",
            FirstPrinciplesEquation::ValveFlowCv { .. } => "valve_flow_cv",
        }
    }

    /// Full static metadata for the passport. The five descriptive fields are static per variant; only the
    /// validity string is parameter-dependent (Antoine's fitted band), so it is built in its own match.
    pub fn metadata(&self) -> EquationMetadata {
        let (form, output_unit, input_layout, assumptions, known_invalid): (
            &str,
            &str,
            &str,
            &str,
            &str,
        ) = match self {
            FirstPrinciplesEquation::Arrhenius { .. } => (
                "k = k0·exp(−Ea/(R·T))",
                "rate (units of k0)",
                "[T_K]",
                "single-step Arrhenius kinetics; constant k0, Ea; well-mixed",
                "near/above decomposition T; multi-step or diffusion-limited regimes",
            ),
            FirstPrinciplesEquation::Antoine { .. } => (
                "log10(P*) = A − B/(C + T)",
                "kPa (convention)",
                "[T_C]",
                "pure-component Antoine correlation; coefficients fitted for the stated range",
                "outside [t_min,t_max]; near the critical point",
            ),
            FirstPrinciplesEquation::Raoult => (
                "p_i = x_i · Psat",
                "pressure (Psat's units)",
                "[x_i, Psat]",
                "ideal solution; vapour–liquid equilibrium; activity coefficient ≈ 1",
                "strongly non-ideal mixtures (azeotropes, electrolytes)",
            ),
            FirstPrinciplesEquation::Henry { .. } => (
                "c = p / H",
                "concentration",
                "[p]",
                "dilute solution; linear Henry's-law regime; constant H at fixed T",
                "high concentration; reactive/dissociating solutes",
            ),
            FirstPrinciplesEquation::HeatTransferUADt { .. } => (
                "Q = U·A·ΔT",
                "W (if UA in W/K)",
                "[ΔT]",
                "lumped U·A; ΔT is the effective driving temperature difference",
                "strong fouling/phase-change shifting U·A; large ΔT-profile curvature (use ΔT_lm)",
            ),
            FirstPrinciplesEquation::PumpHeadCurve { .. } => (
                "H = h0 − k·Q²",
                "head (units of h0)",
                "[Q]",
                "single fixed-speed centrifugal curve; constant fluid density",
                "beyond runout (model goes negative); cavitation; speed change (affinity laws)",
            ),
            FirstPrinciplesEquation::ValveFlowCv { .. } => (
                "Q = Cv·opening·sqrt(ΔP/SG)",
                "flow (Cv's units)",
                "[opening_frac, ΔP, SG]",
                "linear installed characteristic; turbulent, non-choked flow",
                "choked/flashing flow; non-linear trim characteristic",
            ),
        };
        let validity = match *self {
            FirstPrinciplesEquation::Arrhenius { .. } => "T > 0 K".to_string(),
            FirstPrinciplesEquation::Antoine {
                t_min_c, t_max_c, ..
            } => format!("T ∈ [{t_min_c}, {t_max_c}] °C"),
            FirstPrinciplesEquation::Raoult => "0 ≤ x_i ≤ 1, Psat ≥ 0".to_string(),
            FirstPrinciplesEquation::Henry { .. } => "p ≥ 0, H > 0".to_string(),
            FirstPrinciplesEquation::HeatTransferUADt { .. } => "any finite ΔT".to_string(),
            FirstPrinciplesEquation::PumpHeadCurve { .. } => "0 ≤ Q ≤ sqrt(h0/k)".to_string(),
            FirstPrinciplesEquation::ValveFlowCv { .. } => {
                "0 ≤ opening ≤ 1, ΔP ≥ 0, SG > 0".to_string()
            }
        };
        EquationMetadata {
            id: self.id().to_string(),
            form: form.to_string(),
            output_unit: output_unit.to_string(),
            input_layout: input_layout.to_string(),
            assumptions: assumptions.to_string(),
            validity,
            known_invalid_regimes: known_invalid.to_string(),
        }
    }

    /// Hash the equation id + its exact parameters → the `calibration_hash` that pins the parameter set a
    /// witness/passport was computed with (so a re-parameterised equation is a different, detectable seal).
    pub fn param_hash(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("equation_id", self.id().as_bytes());
        let mut p = |label: &str, v: f64| {
            h.f64q(label, v);
        };
        match *self {
            FirstPrinciplesEquation::Arrhenius { k0, ea_j_per_mol } => {
                p("k0", k0);
                p("ea_j_per_mol", ea_j_per_mol);
            }
            FirstPrinciplesEquation::Antoine {
                a,
                b,
                c,
                t_min_c,
                t_max_c,
            } => {
                p("a", a);
                p("b", b);
                p("c", c);
                p("t_min_c", t_min_c);
                p("t_max_c", t_max_c);
            }
            FirstPrinciplesEquation::Raoult => {}
            FirstPrinciplesEquation::Henry { h: hp } => p("h", hp),
            FirstPrinciplesEquation::HeatTransferUADt { ua_w_per_k } => p("ua_w_per_k", ua_w_per_k),
            FirstPrinciplesEquation::PumpHeadCurve { h0, k } => {
                p("h0", h0);
                p("k", k);
            }
            FirstPrinciplesEquation::ValveFlowCv { cv } => p("cv", cv),
        }
        h.finalize_hex()
    }
}

/// A hash-sealed first-principles witness (schema v1): the model–plant residual stream's summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirstPrinciplesWitnessAdapterV1 {
    pub equation_id: String,
    /// Name of the measured variable being compared against the prediction (e.g. `"reactor_P"`).
    pub measured_var: String,
    pub calibration_hash: String,
    pub n_samples: usize,
    /// Samples whose inputs were inside the equation's validity range.
    pub n_in_validity: usize,
    /// Samples whose residual could be computed (finite prediction and finite measurement).
    pub n_residual_finite: usize,
    /// Maximum `|measured − predicted|` over the finite residuals (the model–plant mismatch evidence).
    pub peak_abs_residual: f64,
    /// Root-mean-square of the finite residuals (overall mismatch magnitude).
    pub rms_residual: f64,
    /// SHA-256 of the residual stream (quantised f64), so the exact stream is sealed.
    pub residual_hash: String,
    pub witness_hash: String,
}

impl FirstPrinciplesWitnessAdapterV1 {
    fn hash_residual(residual: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"first_principles_residual_stream_v1");
        for &v in residual {
            h.f64q("r", v);
        }
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"first_principles_witness_v1");
        h.field("equation_id", self.equation_id.as_bytes());
        h.field("measured_var", self.measured_var.as_bytes());
        h.field("calibration_hash", self.calibration_hash.as_bytes());
        h.u64("n_samples", self.n_samples as u64);
        h.u64("n_in_validity", self.n_in_validity as u64);
        h.u64("n_residual_finite", self.n_residual_finite as u64);
        h.f64q("peak_abs_residual", self.peak_abs_residual);
        h.f64q("rms_residual", self.rms_residual);
        h.field("residual_hash", self.residual_hash.as_bytes());
        h.finalize_hex()
    }

    /// Compute the model–plant residual stream `measured − predicted` and seal its summary. `inputs[i]` is
    /// the input vector for sample `i` (layout documented by the equation); `measured[i]` is the measured
    /// value of the predicted quantity. A sample with a non-computable prediction (bad arity, non-finite,
    /// invalid argument) or a non-finite measurement contributes a non-finite residual (excluded from the
    /// peak/RMS summary but still sealed in the stream, so the gap is visible and tamper-evident).
    pub fn build(
        equation: &FirstPrinciplesEquation,
        measured_var: impl Into<String>,
        measured: &[f64],
        inputs: &[Vec<f64>],
    ) -> Self {
        let n = measured.len().min(inputs.len());
        let mut residual = Vec::with_capacity(n);
        let mut n_in_validity = 0usize;
        for i in 0..n {
            if equation.within_validity(&inputs[i]) {
                n_in_validity += 1;
            }
            let r = match equation.predict(&inputs[i]) {
                Some(pred) if measured[i].is_finite() => measured[i] - pred,
                _ => f64::NAN,
            };
            residual.push(r);
        }
        let finite: Vec<f64> = residual.iter().copied().filter(|x| x.is_finite()).collect();
        let peak_abs_residual = finite.iter().fold(0.0f64, |a, &x| a.max(x.abs()));
        let rms_residual = if finite.is_empty() {
            0.0
        } else {
            (finite.iter().map(|x| x * x).sum::<f64>() / finite.len() as f64).sqrt()
        };
        let mut w = FirstPrinciplesWitnessAdapterV1 {
            equation_id: equation.id().to_string(),
            measured_var: measured_var.into(),
            calibration_hash: equation.param_hash(),
            n_samples: n,
            n_in_validity,
            n_residual_finite: finite.len(),
            peak_abs_residual,
            rms_residual,
            residual_hash: Self::hash_residual(&residual),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// Re-derive the residual hash from the supplied stream and re-seal — catches a tampered stream, summary,
    /// or hash. The caller passes the same `residual` stream `build` produced (or recomputes it identically).
    pub fn verify(&self, residual: &[f64]) -> bool {
        Self::hash_residual(residual) == self.residual_hash && self.seal() == self.witness_hash
    }
}

/// A hash-sealed per-equation provenance passport (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquationResidualPassportV1 {
    pub metadata: EquationMetadata,
    /// Where the parameters came from (e.g. `"plant data sheet rev C"`, `"DIPPR"`, `"vendor pump curve"`).
    pub parameter_source: String,
    /// Hash pinning the exact parameter set (matches the witness's `calibration_hash`).
    pub calibration_hash: String,
    pub passport_hash: String,
}

impl EquationResidualPassportV1 {
    fn seal(&self) -> String {
        let m = &self.metadata;
        let mut h = CanonicalHasher::new();
        h.field("schema", b"equation_residual_passport_v1");
        h.field("id", m.id.as_bytes());
        h.field("form", m.form.as_bytes());
        h.field("output_unit", m.output_unit.as_bytes());
        h.field("input_layout", m.input_layout.as_bytes());
        h.field("assumptions", m.assumptions.as_bytes());
        h.field("validity", m.validity.as_bytes());
        h.field("known_invalid_regimes", m.known_invalid_regimes.as_bytes());
        h.field("parameter_source", self.parameter_source.as_bytes());
        h.field("calibration_hash", self.calibration_hash.as_bytes());
        h.finalize_hex()
    }

    /// Build + seal a passport for an equation and its parameter source.
    pub fn build(equation: &FirstPrinciplesEquation, parameter_source: impl Into<String>) -> Self {
        let mut p = EquationResidualPassportV1 {
            metadata: equation.metadata(),
            parameter_source: parameter_source.into(),
            calibration_hash: equation.param_hash(),
            passport_hash: String::new(),
        };
        p.passport_hash = p.seal();
        p
    }

    /// True iff this passport documents the equation+parameters a given witness was computed with.
    pub fn matches_witness(&self, witness: &FirstPrinciplesWitnessAdapterV1) -> bool {
        self.metadata.id == witness.equation_id && self.calibration_hash == witness.calibration_hash
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.passport_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrhenius_predicts_and_residual_is_zero_on_consistent_data() {
        let eq = FirstPrinciplesEquation::Arrhenius {
            k0: 7.2e10,
            ea_j_per_mol: 72_750.0,
        };
        // Build "measured" exactly from the equation at three temperatures → residual must be ~0.
        let temps = [350.0, 360.0, 370.0];
        let inputs: Vec<Vec<f64>> = temps.iter().map(|&t| vec![t]).collect();
        let measured: Vec<f64> = inputs.iter().map(|i| eq.predict(i).unwrap()).collect();
        let w = FirstPrinciplesWitnessAdapterV1::build(&eq, "rate_k", &measured, &inputs);
        assert_eq!(w.n_in_validity, 3);
        assert_eq!(w.n_residual_finite, 3);
        assert!(
            w.peak_abs_residual < 1e-6,
            "consistent data ⇒ ~0 residual, got {}",
            w.peak_abs_residual
        );
        assert_eq!(w.witness_hash.len(), 64);
    }

    #[test]
    fn pump_curve_residual_flags_a_degraded_pump_and_self_verifies() {
        // h0 = 50 m, k = 0.2 → H = 50 − 0.2·Q². A degraded pump develops 5 m less head everywhere.
        let eq = FirstPrinciplesEquation::PumpHeadCurve { h0: 50.0, k: 0.2 };
        let flows = [2.0, 4.0, 6.0, 8.0];
        let inputs: Vec<Vec<f64>> = flows.iter().map(|&q| vec![q]).collect();
        let measured: Vec<f64> = inputs
            .iter()
            .map(|i| eq.predict(i).unwrap() - 5.0)
            .collect();
        let residual: Vec<f64> = (0..inputs.len())
            .map(|i| measured[i] - eq.predict(&inputs[i]).unwrap())
            .collect();
        let w = FirstPrinciplesWitnessAdapterV1::build(&eq, "pump_head", &measured, &inputs);
        // Every residual is −5 m (the degradation), so peak |residual| = 5 and RMS = 5.
        assert!((w.peak_abs_residual - 5.0).abs() < 1e-9);
        assert!((w.rms_residual - 5.0).abs() < 1e-9);
        assert!(w.verify(&residual));
        assert!(!w.verify(&[0.0; 4]));
    }

    #[test]
    fn out_of_validity_samples_are_counted_not_silently_dropped() {
        // Antoine for water-ish coefficients valid 1..100 °C; feed one sample at 150 °C (out of range).
        let eq = FirstPrinciplesEquation::Antoine {
            a: 7.0,
            b: 1700.0,
            c: 235.0,
            t_min_c: 1.0,
            t_max_c: 100.0,
        };
        let inputs = vec![vec![25.0], vec![80.0], vec![150.0]];
        let measured: Vec<f64> = inputs.iter().map(|i| eq.predict(i).unwrap()).collect();
        let w = FirstPrinciplesWitnessAdapterV1::build(&eq, "Psat", &measured, &inputs);
        assert_eq!(w.n_samples, 3);
        assert_eq!(w.n_in_validity, 2); // the 150 °C sample is outside the fitted band
        assert_eq!(w.n_residual_finite, 3); // but its residual is still computed (Antoine is finite there)
    }

    #[test]
    fn passport_pins_parameters_and_matches_its_witness() {
        let eq = FirstPrinciplesEquation::HeatTransferUADt { ua_w_per_k: 1200.0 };
        let inputs = vec![vec![10.0], vec![12.0]];
        let measured = vec![12000.0, 14400.0]; // exactly U·A·ΔT
        let w = FirstPrinciplesWitnessAdapterV1::build(&eq, "duty_Q", &measured, &inputs);
        let passport = EquationResidualPassportV1::build(&eq, "plant data sheet rev C");
        assert!(passport.verify());
        assert!(passport.matches_witness(&w));
        assert_eq!(passport.metadata.form, "Q = U·A·ΔT");
        // A different U·A is a different calibration_hash → passport no longer matches.
        let eq2 = FirstPrinciplesEquation::HeatTransferUADt { ua_w_per_k: 1100.0 };
        let w2 = FirstPrinciplesWitnessAdapterV1::build(&eq2, "duty_Q", &measured, &inputs);
        assert!(!passport.matches_witness(&w2));
        assert_ne!(eq.param_hash(), eq2.param_hash());
    }

    #[test]
    fn passport_tamper_breaks_the_seal() {
        let eq = FirstPrinciplesEquation::Henry { h: 0.034 };
        let mut p = EquationResidualPassportV1::build(&eq, "DIPPR");
        assert!(p.verify());
        p.metadata.validity = "valid everywhere".into(); // forge away the validity caveat
        assert!(!p.verify());
    }
}
