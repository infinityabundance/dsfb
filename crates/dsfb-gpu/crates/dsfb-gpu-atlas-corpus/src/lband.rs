//! T.7 — implementation-status (L-band) ladder verification.
//!
//! Every literature detector in the corpus carries an
//! [`ImplementationLevel`] (`L0_CitedOnly` .. `L8_LedgerCharacterised`).
//! T.1a declared the enum; T.7 enforces that the value is honest —
//! a detector cannot claim L5/L6/L7/L8 unless the corresponding
//! evidence actually exists.
//!
//! **The L-band is an honesty marker, not a quality score.** A
//! detector at L1 is not "worse" than one at L6 — it is simply
//! cited and canonicalised, not implemented. Inflating L-bands
//! ("corpus inflation by implication") is the failure mode T.7
//! is designed to prevent. The verifier rejects any record whose
//! L-band exceeds what the corpus + workspace can demonstrate.
//!
//! **Honesty rules** (panel-locked, T.7 acceptance bar):
//!
//! - **L5/L6** (`GpuImplemented`, `CpuGpuByteEquivalent`): the
//!   record's `canonical_id` MUST appear in
//!   [`GPU_IMPLEMENTED_CANONICAL_IDS`]. That static list mirrors
//!   the canonical IDs that actually have a CUDA implementation
//!   in `dsfb-gpu-debug-core`'s 8-motif bank. Adding a new entry
//!   requires landing a real GPU kernel + acceptance tests; the
//!   list is not editable as a documentation flourish.
//! - **L7** (`BenchmarkCharacterised`): forbidden until a
//!   per-detector benchmark artifact exists that pins runtime
//!   characterisation against the corpus. No detector can claim
//!   L7 in the absence of that artifact.
//! - **L8** (`LedgerCharacterised`): forbidden until the
//!   usefulness ledger contains measured `(task × dataset)`
//!   empirical rows. **T.8 landed the ledger shell and
//!   conservative `NotScored` seed rows; it did not land
//!   measured empirical L8 characterisation.** The verifier
//!   continues to reject L8 records until a row backed by a
//!   real benchmark artifact is committed.
//!
//! Module scope (panel-locked, post-T.8):
//!
//! - Verifier emits `LBandError` for any record outside the
//!   admissible bands (L0–L6 with the GPU-implementation
//!   whitelist for L5/L6).
//! - No bench artifacts here (those land via a future T.7.*
//!   benchmark harness commit).
//! - No usefulness-ledger row mutation (`usefulness::USEFULNESS_LEDGER`
//!   is the source of truth for T.8).
//! - No CaseFileV2 integration (that's T.11d's transcript body).
//! - No `corpus_hash_v1` mutation (T.10 sealed it).
//! - No GPU code, no kernel changes, no contract changes.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{DetectorCanonicalId, ImplementationLevel, LiteratureDetector};

/// Canonical IDs that have a real CUDA implementation in
/// `dsfb-gpu-debug-core` today (the R.13-sealed v1 prior-art
/// proof). Adding an entry MUST be accompanied by a landed GPU
/// kernel and acceptance tests; this list is reviewed at every
/// T-commit and is the source of truth for L5/L6 admissibility.
///
/// Mapping (corpus canonical_id -> dsfb-gpu-debug bank motif):
///
/// - 14 -> `LatencyRamp`
/// - 15 -> `ConfuserTransient` / single-window spike confuser
/// - 41 -> `ErrorBurst`
/// - 42 -> `SlewShockRecovery`
/// - 43 -> `FanoutCascadeCandidate`
///
/// The remaining three bank motifs (`SustainedDegradation`,
/// `OscillationInstability`, `LocalizedRouteFault`) are NOT in
/// the corpus seed at T.7 — the corpus does not yet claim them.
/// They land as new canonical seed entries in T.7.* when their
/// literature provenance is finalised.
pub static GPU_IMPLEMENTED_CANONICAL_IDS: &[DetectorCanonicalId] = &[
    DetectorCanonicalId(14),
    DetectorCanonicalId(15),
    DetectorCanonicalId(41),
    DetectorCanonicalId(42),
    DetectorCanonicalId(43),
];

/// One L-band verification failure.
#[derive(Debug, Clone)]
pub struct LBandError {
    /// Which record failed.
    pub canonical_id: DetectorCanonicalId,
    /// What's wrong.
    pub kind: LBandErrorKind,
}

/// Structured failure category for L-band verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LBandErrorKind {
    /// The record claims L5/L6 but its canonical_id is not in
    /// [`GPU_IMPLEMENTED_CANONICAL_IDS`]. Corpus inflation:
    /// claiming GPU implementation status without an actual
    /// GPU kernel.
    GpuImplementationClaimedWithoutGpuSurface {
        /// The L-band the record claims.
        claimed: ImplementationLevel,
    },
    /// The record claims L7 but no benchmark artifact exists yet.
    /// L7 stays forbidden until a real per-detector benchmark
    /// harness lands.
    BenchmarkCharacterisedWithoutArtifact,
    /// The record claims L8 but the usefulness ledger does not
    /// yet contain measured `(task × dataset)` empirical rows.
    /// T.8 landed the ledger shell and conservative `NotScored`
    /// seed rows; it did not land measured L8 characterisation.
    LedgerCharacterisedWithoutLedger,
}

impl LBandErrorKind {
    /// Human-readable description for the report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::GpuImplementationClaimedWithoutGpuSurface { claimed } => format!(
                "record claims L-band {claimed:?} but its canonical_id is not in GPU_IMPLEMENTED_CANONICAL_IDS; L5/L6 requires an actual GPU kernel in dsfb-gpu-debug-core"
            ),
            Self::BenchmarkCharacterisedWithoutArtifact => String::from(
                "record claims L7 (BenchmarkCharacterised) but no per-detector benchmark artifact exists yet; L7 stays forbidden until a real benchmark harness lands",
            ),
            Self::LedgerCharacterisedWithoutLedger => String::from(
                "record claims L8 (LedgerCharacterised) but the usefulness ledger does not yet contain measured (task x dataset) empirical rows; T.8 landed the ledger shell and conservative NotScored seed rows, not measured L8 characterisation",
            ),
        }
    }
}

/// Aggregate L-band verification report.
#[derive(Debug, Clone, Default)]
pub struct LBandVerifyReport {
    /// Number of records inspected.
    pub records_inspected: usize,
    /// Per-record failures (empty if clean).
    pub errors: Vec<LBandError>,
}

impl LBandVerifyReport {
    /// True if no errors were recorded.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Verify the L-band of a single record. Returns the list of
/// errors found (empty when the record is clean).
#[must_use]
pub fn verify_record_lband(record: &LiteratureDetector) -> Vec<LBandError> {
    let id = record.canonical_id;
    let mut errors = Vec::new();
    let gpu_implemented = GPU_IMPLEMENTED_CANONICAL_IDS.contains(&id);
    match record.implementation_status {
        ImplementationLevel::L5_GpuImplemented | ImplementationLevel::L6_CpuGpuByteEquivalent => {
            if !gpu_implemented {
                errors.push(LBandError {
                    canonical_id: id,
                    kind: LBandErrorKind::GpuImplementationClaimedWithoutGpuSurface {
                        claimed: record.implementation_status,
                    },
                });
            }
        }
        ImplementationLevel::L7_BenchmarkCharacterised => {
            errors.push(LBandError {
                canonical_id: id,
                kind: LBandErrorKind::BenchmarkCharacterisedWithoutArtifact,
            });
        }
        ImplementationLevel::L8_LedgerCharacterised => {
            errors.push(LBandError {
                canonical_id: id,
                kind: LBandErrorKind::LedgerCharacterisedWithoutLedger,
            });
        }
        // L0..L4 are always admissible at T.7. L0 (CitedOnly)
        // means the corpus references the primitive without any
        // claim; L1..L4 are pure-corpus claims that don't
        // require an external workspace artifact to be honest.
        ImplementationLevel::L0_CitedOnly
        | ImplementationLevel::L1_Canonicalised
        | ImplementationLevel::L2_DeterministicFormula
        | ImplementationLevel::L3_CpuImplemented
        | ImplementationLevel::L4_CpuVerified => {}
    }
    errors
}

/// Verify every record's L-band against the T.7 honesty rules.
#[must_use]
pub fn verify_corpus_lband(records: &[LiteratureDetector]) -> LBandVerifyReport {
    let mut report = LBandVerifyReport {
        records_inspected: records.len(),
        errors: Vec::new(),
    };
    for record in records {
        let mut record_errors = verify_record_lband(record);
        report.errors.append(&mut record_errors);
    }
    report
}

/// Count of records at each L-band, in canonical band order.
/// Useful for the report's invariant block.
#[derive(Debug, Clone, Copy, Default)]
pub struct LBandHistogram {
    /// L0 CitedOnly count.
    pub l0: usize,
    /// L1 Canonicalised count.
    pub l1: usize,
    /// L2 DeterministicFormula count.
    pub l2: usize,
    /// L3 CpuImplemented count.
    pub l3: usize,
    /// L4 CpuVerified count.
    pub l4: usize,
    /// L5 GpuImplemented count.
    pub l5: usize,
    /// L6 CpuGpuByteEquivalent count.
    pub l6: usize,
    /// L7 BenchmarkCharacterised count.
    pub l7: usize,
    /// L8 LedgerCharacterised count.
    pub l8: usize,
}

impl LBandHistogram {
    /// Sum across every band.
    #[must_use]
    pub fn total(&self) -> usize {
        self.l0 + self.l1 + self.l2 + self.l3 + self.l4 + self.l5 + self.l6 + self.l7 + self.l8
    }
}

/// Compute the histogram over a slice of detector records.
#[must_use]
pub fn compute_histogram(records: &[LiteratureDetector]) -> LBandHistogram {
    let mut h = LBandHistogram::default();
    for r in records {
        match r.implementation_status {
            ImplementationLevel::L0_CitedOnly => h.l0 += 1,
            ImplementationLevel::L1_Canonicalised => h.l1 += 1,
            ImplementationLevel::L2_DeterministicFormula => h.l2 += 1,
            ImplementationLevel::L3_CpuImplemented => h.l3 += 1,
            ImplementationLevel::L4_CpuVerified => h.l4 += 1,
            ImplementationLevel::L5_GpuImplemented => h.l5 += 1,
            ImplementationLevel::L6_CpuGpuByteEquivalent => h.l6 += 1,
            ImplementationLevel::L7_BenchmarkCharacterised => h.l7 += 1,
            ImplementationLevel::L8_LedgerCharacterised => h.l8 += 1,
        }
    }
    h
}
