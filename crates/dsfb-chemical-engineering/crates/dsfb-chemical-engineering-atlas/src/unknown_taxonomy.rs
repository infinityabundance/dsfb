//! `UnknownTaxonomyV1` — the authority taxonomy of *why* an episode is left unknown (P60).
//!
//! When the heuristics bank declines to assign an operator label, the reason matters forensically: an
//! "unknown" that is a 3-sample transient is a different disposition from one where the baseline itself
//! is drifting. The edge pipeline already distinguishes a handful of unknown reasons inline; this is the
//! **authority** definition — a curated, hash-sealed 7-class taxonomy of unknown dispositions that the
//! execution side maps episodes onto, and that is folded into `atlas_hash_v1` so the taxonomy a case
//! file was interpreted against is sealed.
//!
//! Each class is **advisory and bounded**: it records *why no positive label was warranted*, never a
//! positive fault claim. The taxonomy is fixed at seven classes; new classes are added by commit +
//! deliberate atlas re-freeze, never by mutation.

/// One unknown-disposition class (schema v1). All fields are `&'static`; the record is a pure constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnknownClassRecordV1 {
    /// Stable snake_case identifier (the key the edge classifier maps to).
    pub class_id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// What the class means — the structural reason no positive label was assigned.
    pub description: &'static str,
    /// Advisory disposition: how an operator / the court should read an episode in this class
    /// (what to check, what it rules out). Never a positive fault claim.
    pub disposition: &'static str,
}

/// The seven unknown-disposition classes. Order is fixed and part of the sealed `atlas_hash_v1`.
pub const UNKNOWN_TAXONOMY: &[UnknownClassRecordV1] = &[
    UnknownClassRecordV1 {
        class_id: "short_transient",
        name: "Short transient",
        description: "The episode is too brief to support a confident structural label — a few-sample \
                      excursion that recovers before drift/slew structure stabilises.",
        disposition: "Treat as a watch item, not an alarm; rules out a sustained structural fault. \
                      Re-examine only if recurrence increases.",
    },
    UnknownClassRecordV1 {
        class_id: "detector_conflict",
        name: "Detector conflict",
        description: "Firing detectors contradict on the grammar state — high disagreement entropy with \
                      no dominant motif, so no single heuristic is warranted.",
        disposition: "Inspect the DetectorDisagreementForensics report; the conflict itself is the \
                      evidence. Rules out a clean single-mechanism fault.",
    },
    UnknownClassRecordV1 {
        class_id: "out_of_regime_envelope",
        name: "Out of regime envelope",
        description: "The sample falls outside every calibrated regime/phase admissibility envelope, so \
                      no regime's nominal model applies to interpret the residual.",
        disposition: "Check whether the operating regime/phase is mislabelled or uncalibrated before \
                      reading the residual as a fault. Rules out an in-regime structural label.",
    },
    UnknownClassRecordV1 {
        class_id: "residual_degeneracy",
        name: "Residual degeneracy",
        description: "The residual structure is degenerate (near-zero variance, flat, or saturated), so \
                      drift/slew/envelope coordinates carry no discriminating information.",
        disposition: "Verify sensor liveness / quantisation before interpreting; a degenerate residual \
                      rules out a resolvable structural motif.",
    },
    UnknownClassRecordV1 {
        class_id: "missing_process_context",
        name: "Missing process context",
        description: "No heuristic in the bank applies to this process type / variable set — the \
                      episode is real but outside the catalogued domain.",
        disposition: "A candidate for extending the heuristics bank; rules out *known* mechanisms only, \
                      not an unmodelled one. Catalogue the context if recurrent.",
    },
    UnknownClassRecordV1 {
        class_id: "non_stationary_baseline",
        name: "Non-stationary baseline",
        description: "The baseline reference itself is drifting (the nominal window is non-stationary), \
                      so the residual cannot be trusted as a deviation from a stable normal.",
        disposition: "Re-fit / re-window the baseline before classifying; rules out a fault attribution \
                      that assumes a stationary reference (the flagship gas-sensor-drift failure mode).",
    },
    UnknownClassRecordV1 {
        class_id: "insufficient_witness_diversity",
        name: "Insufficient witness diversity",
        description: "Too few distinct detector families corroborate — the quorum is barely met by a \
                      single family, so a confident multi-witness label is not warranted.",
        disposition: "Treat as low-confidence; rules out a broadly-corroborated fault. Raise only if \
                      additional families recruit.",
    },
];

/// Number of unknown-disposition classes (fixed at 7).
pub const UNKNOWN_CLASS_COUNT: usize = 7;
