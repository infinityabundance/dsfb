//! T.4 — first explicit dedup-court alias claims.
//!
//! The 54-record static seed stays canonical. T.4 introduces a
//! SEPARATE static array of `DetectorClaim` records: literature
//! names that the dedup court will judge as `AliasOf(...)` against
//! the canonical they collapse into. The panel-locked discipline
//! is that the 54 seed records are NOT mutated; alias claims live
//! beside them so every judgment is auditable.
//!
//! Alias-ID space (panel-locked):
//!
//! - 0 is the null sentinel.
//! - 1..=999 reserved for future internal use.
//! - 1000+ assigned to T.4 alias claims. Future T.4.* expansions
//!   add more entries with monotonically increasing IDs.
//!
//! T.4's first batch (panel-named):
//!
//! - Robust z / median-MAD: 3 aliases of canonical 6
//! - PCA SPE / Q residual: 3 aliases of canonical 20
//! - Hotelling T2: 2 aliases of canonical 5
//! - Page-Hinkley: 2 aliases of canonical 4
//! - Jensen-Shannon: 2 aliases of canonical 32
//!
//! Western Electric (canonical 16) and Nelson rules (canonical 17)
//! are NOT in this claims array — they remain canonical primitives
//! but the dedup-court pass over the seed will emit
//! `CompositionOf([SHEWHART_CHART])` records for them. See
//! `crate::court` for the policy.

use crate::types::{CanonicalisationDecision, DedupReason, DetectorAliasId, DetectorCanonicalId};

/// One T.4 literature-name claim, paired with the court's
/// pre-declared decision.
///
/// Each claim is its own audit unit: it carries its own alias_id
/// handle, its own reason code, its own notes string. The court
/// uses these directly; T.4 does not yet attempt automated
/// equivalence-class detection (T.4.1+ may add a parallel pass
/// that auto-discovers aliases via `detector_identity_hash`
/// collisions — at T.4 the judgments are explicit).
#[derive(Debug, Clone, Copy)]
pub struct DetectorClaim {
    /// Alias-side handle (alias-ID space 1000+).
    pub alias_id: DetectorAliasId,
    /// Literature name as it appears in source documents.
    pub literature_name: &'static str,
    /// The pre-declared court decision (panel-locked at T.4).
    pub decision: CanonicalisationDecision,
    /// The reason code attached to the decision.
    pub reason_code: DedupReason,
    /// Free-form note explaining the judgment for the public report.
    pub notes: &'static str,
}

/// The T.4 first-batch alias claims.
///
/// Each entry's `decision` field is the court's canonical judgment
/// the acceptance tests pin. The `notes` field is the auditable
/// explanation that future readers (and the future Zenodo dedup
/// report at T.9) will see.
pub static CLAIMS: &[DetectorClaim] = &[
    // ===== Robust z (canonical_id 6) =====
    DetectorClaim {
        alias_id: DetectorAliasId(1001),
        literature_name: "robust z-score",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(6)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Same modified-z robust z-score formula; alternate literature name without parameter change.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1002),
        literature_name: "median-MAD z",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(6)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Alternate name emphasising the median + MAD substitution; same canonical formula.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1003),
        literature_name: "MAD outlier detector",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(6)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Outlier-flagging framing of the same modified-z formula; one threshold, one formula.",
    },
    // ===== PCA SPE / Q residual (canonical_id 20) =====
    DetectorClaim {
        alias_id: DetectorAliasId(1004),
        literature_name: "SPE",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(20)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Squared prediction error; one of the canonical short forms of the Q statistic.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1005),
        literature_name: "Q statistic",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(20)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Q statistic in PCA-based process monitoring; identical to SPE under the standard control-limit framework.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1006),
        literature_name: "squared prediction error",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(20)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Long-form expansion of SPE; same canonical formula.",
    },
    // ===== Hotelling T-squared (canonical_id 5) =====
    DetectorClaim {
        alias_id: DetectorAliasId(1007),
        literature_name: "Hotelling T2",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(5)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Notational variant of Hotelling T-squared; same quadratic form on the score vector.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1008),
        literature_name: "multivariate T-square",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(5)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Long-form name emphasising the multivariate generalisation of Student's t; same statistic.",
    },
    // ===== Page-Hinkley (canonical_id 4) =====
    DetectorClaim {
        alias_id: DetectorAliasId(1009),
        literature_name: "Page-Hinkley",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(4)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Short form of Page-Hinkley test; same sequential cumulative-mean-shift detector.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1010),
        literature_name: "cumulative mean shift detector",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(4)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Descriptive framing of the Page-Hinkley sequential change-point statistic; same formula.",
    },
    // ===== Jensen-Shannon divergence (canonical_id 32) =====
    DetectorClaim {
        alias_id: DetectorAliasId(1011),
        literature_name: "JS divergence",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(32)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Short form of Jensen-Shannon divergence; same symmetric-KL midpoint formula.",
    },
    DetectorClaim {
        alias_id: DetectorAliasId(1012),
        literature_name: "Jensen-Shannon distance",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(32)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "Distance framing (square root of the divergence); same canonical primitive in the corpus.",
    },
];
