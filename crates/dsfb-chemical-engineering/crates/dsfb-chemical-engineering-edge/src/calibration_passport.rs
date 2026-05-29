//! `CalibrationModelPassportV1` — the validation provenance of a PAT / chemometric calibration model (panel #9).
//!
//! Soft-sensor and spectroscopy (NIR/Raman) predictions are only as trustworthy as the calibration behind them.
//! An elite chemometrician asks, immediately: what was the calibration vs validation set, the preprocessing, the
//! external RMSEP and bias, the leverage / Q-residual (Hotelling T²) outlier policy, and whether the model was
//! transferred/standardised across instruments. This object records that passport so a soft-sensor witness can
//! state the *validated domain* of its model — and flag a prediction made OUTSIDE it as a non-claim rather than a
//! confident number.
//!
//! **NON-CLAIM:** this documents the model's stated calibration/validation provenance; it does not re-fit, re-validate,
//! or assert accuracy on a new instrument, new operating regime, or outside the validation range — predictions
//! beyond the validated domain are explicitly not asserted (extrapolation). Self-sealed; not part of any frozen
//! authority hash. Additive, read-only.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The validation passport of one calibration model (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationModelPassportV1 {
    /// Stable model identifier.
    pub model_id: String,
    /// The analyte / property predicted (e.g. `"moisture %"`, `"API concentration"`, `"octane number"`).
    pub analyte: String,
    /// The instrument the model was calibrated on — transfer to another unit is non-trivial (`transfer_standardization`).
    pub instrument_id: String,
    /// Spectral / signal preprocessing, as an inspectable string (e.g. `"SNV + 1st-deriv Savitzky-Golay (w=15)"`).
    pub preprocessing: String,
    /// Analyte range spanned by the CALIBRATION set `[min, max]`.
    pub calibration_range: (f64, f64),
    /// Analyte range covered by EXTERNAL validation `[min, max]` — predictions outside this are extrapolation.
    pub validation_range: (f64, f64),
    /// Number of calibration samples.
    pub n_calibration: u32,
    /// Number of (independent) validation samples.
    pub n_validation: u32,
    /// Root-mean-square error of prediction on the external validation set (in analyte units).
    pub rmsep: f64,
    /// Mean prediction bias on the validation set (signed, analyte units).
    pub bias: f64,
    /// High-leverage / outlier handling policy (e.g. `"samples with leverage > 3·(p+1)/n flagged for review"`).
    pub leverage_policy: String,
    /// Spectral residual policy — the Q-residual (SPE) / Hotelling T² limit beyond which a spectrum is
    /// out-of-model and the prediction is withheld (e.g. `"Q > 95% limit ⇒ prediction withheld"`).
    pub q_residual_policy: String,
    /// Instrument-transfer / standardisation method, or `"none (single-instrument)"` (e.g. `"PDS"`, `"DS"`, `"slope/bias"`).
    pub transfer_standardization: String,
    /// SHA-256 (via [`CanonicalHasher`]) sealing the passport.
    pub model_hash: String,
}

impl CalibrationModelPassportV1 {
    /// The standing non-claim, part of the seal preimage.
    pub const NON_CLAIM: &'static str = "Documents the model's stated calibration/validation provenance — it does \
        NOT re-validate, assert accuracy on a new instrument/regime, or claim a prediction outside the validation \
        range; out-of-range predictions are extrapolation and are not asserted.";

    #[allow(clippy::too_many_arguments)]
    fn seal(
        model_id: &str,
        analyte: &str,
        instrument_id: &str,
        preprocessing: &str,
        calibration_range: (f64, f64),
        validation_range: (f64, f64),
        n_calibration: u32,
        n_validation: u32,
        rmsep: f64,
        bias: f64,
        leverage_policy: &str,
        q_residual_policy: &str,
        transfer_standardization: &str,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"calibration_model_passport_v1");
        h.field("model_id", model_id.as_bytes());
        h.field("analyte", analyte.as_bytes());
        h.field("instrument_id", instrument_id.as_bytes());
        h.field("preprocessing", preprocessing.as_bytes());
        h.f64q("calibration_min", calibration_range.0);
        h.f64q("calibration_max", calibration_range.1);
        h.f64q("validation_min", validation_range.0);
        h.f64q("validation_max", validation_range.1);
        h.u64("n_calibration", n_calibration as u64);
        h.u64("n_validation", n_validation as u64);
        h.f64q("rmsep", rmsep);
        h.f64q("bias", bias);
        h.field("leverage_policy", leverage_policy.as_bytes());
        h.field("q_residual_policy", q_residual_policy.as_bytes());
        h.field(
            "transfer_standardization",
            transfer_standardization.as_bytes(),
        );
        h.field("non_claim", Self::NON_CLAIM.as_bytes());
        h.finalize_hex()
    }

    /// Build a sealed calibration passport.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        model_id: impl Into<String>,
        analyte: impl Into<String>,
        instrument_id: impl Into<String>,
        preprocessing: impl Into<String>,
        calibration_range: (f64, f64),
        validation_range: (f64, f64),
        n_calibration: u32,
        n_validation: u32,
        rmsep: f64,
        bias: f64,
        leverage_policy: impl Into<String>,
        q_residual_policy: impl Into<String>,
        transfer_standardization: impl Into<String>,
    ) -> Self {
        let (
            model_id,
            analyte,
            instrument_id,
            preprocessing,
            leverage_policy,
            q_residual_policy,
            transfer_standardization,
        ) = (
            model_id.into(),
            analyte.into(),
            instrument_id.into(),
            preprocessing.into(),
            leverage_policy.into(),
            q_residual_policy.into(),
            transfer_standardization.into(),
        );
        let model_hash = Self::seal(
            &model_id,
            &analyte,
            &instrument_id,
            &preprocessing,
            calibration_range,
            validation_range,
            n_calibration,
            n_validation,
            rmsep,
            bias,
            &leverage_policy,
            &q_residual_policy,
            &transfer_standardization,
        );
        CalibrationModelPassportV1 {
            model_id,
            analyte,
            instrument_id,
            preprocessing,
            calibration_range,
            validation_range,
            n_calibration,
            n_validation,
            rmsep,
            bias,
            leverage_policy,
            q_residual_policy,
            transfer_standardization,
            model_hash,
        }
    }

    /// True iff `value` lies within the validated range — a prediction outside is extrapolation and, per the
    /// non-claim, must NOT be asserted as a confident value.
    pub fn predicts_within_validation(&self, value: f64) -> bool {
        value >= self.validation_range.0 && value <= self.validation_range.1
    }

    /// Re-derive the seal and confirm it matches (tamper-evident).
    pub fn verify(&self) -> bool {
        self.model_hash
            == Self::seal(
                &self.model_id,
                &self.analyte,
                &self.instrument_id,
                &self.preprocessing,
                self.calibration_range,
                self.validation_range,
                self.n_calibration,
                self.n_validation,
                self.rmsep,
                self.bias,
                &self.leverage_policy,
                &self.q_residual_policy,
                &self.transfer_standardization,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CalibrationModelPassportV1 {
        CalibrationModelPassportV1::build(
            "nir_moisture_v3",
            "moisture %",
            "FT-NIR unit A",
            "SNV + 1st-deriv Savitzky-Golay (w=15)",
            (2.0, 14.0),
            (3.0, 12.0),
            120,
            40,
            0.31,
            -0.02,
            "leverage > 3·(p+1)/n flagged",
            "Q > 95% limit ⇒ prediction withheld",
            "none (single-instrument)",
        )
    }

    #[test]
    fn build_self_verifies_and_is_deterministic() {
        let p = sample();
        assert!(p.verify());
        assert_eq!(p.model_hash, sample().model_hash);
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut p = sample();
        p.rmsep = 9.99;
        assert!(!p.verify(), "a changed RMSEP must break the seal");
    }

    #[test]
    fn out_of_validation_range_is_extrapolation() {
        let p = sample(); // validated 3..12 %
        assert!(p.predicts_within_validation(7.5));
        assert!(
            !p.predicts_within_validation(13.0),
            "above the validated range ⇒ extrapolation (non-claim)"
        );
        assert!(
            !p.predicts_within_validation(1.0),
            "below the validated range ⇒ extrapolation (non-claim)"
        );
    }
}
