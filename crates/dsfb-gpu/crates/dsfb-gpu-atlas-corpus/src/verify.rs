//! Verifier — walks the seed corpus and rejects records missing
//! schema-required fields.
//!
//! This is the gate that keeps the corpus from becoming a junk
//! drawer. The panel-locked rule is exhaustive: a detector may
//! enter the corpus only if it declares every constitution flag
//! AND populates every structurally-required field. Verifier
//! failures are deterministic and reproducible — two builds on
//! different machines see identical reports for the same seed.
//!
//! The verifier is a pure function over the seed slice. It does
//! NOT mutate the corpus, does NOT touch the filesystem, and does
//! NOT depend on the GPU. Future T.* commits extend the verifier
//! with five-hash identity checks (T.3) and genealogy-DAG cycle
//! checks (T.5); T.1a establishes the verifier shape.

use crate::types::{ConstitutionFlags, DetectorCanonicalId, LiteratureDetector, SourceRef};

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// One verification failure for a specific detector record.
///
/// Failures carry the offending canonical ID and a structured kind
/// so the report can group failures by category. Construction is
/// in [`verify_record`].
#[derive(Debug, Clone)]
pub struct VerifyError {
    /// Which detector record failed.
    pub canonical_id: DetectorCanonicalId,
    /// What kind of failure.
    pub kind: VerifyErrorKind,
}

/// Structured failure category for a verification error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyErrorKind {
    /// One of the eight constitution flags is `false`. The variant
    /// carries the flag name so the report can be specific.
    MissingConstitutionFlag(&'static str),
    /// `display_name` is the empty string.
    EmptyDisplayName,
    /// Neither a `SourceRef` nor an engineering-practice note is
    /// declared, so provenance is missing.
    MissingProvenance,
    /// A declared `SourceRef` has `year == 0` (engineering practice)
    /// but its `notes` field is empty — the verifier requires an
    /// explicit note when the year is omitted.
    EngineeringPracticeWithoutNote(&'static str),
    /// `origin_domains` bitset is empty.
    NoOriginDomains,
    /// `input_requirements` bitset is empty.
    NoInputRequirements,
    /// `fusion_axes` bitset is empty.
    NoFusionAxes,
    /// `duplicate_group` value does not match `canonical_id` for a
    /// canonical record (T.1a invariant; T.4 lifts this to allow
    /// proper alias-to-canonical mappings).
    DuplicateGroupMismatch,
    /// `canonical_id` is 0 (reserved as the null / not-applicable
    /// sentinel).
    ReservedCanonicalIdZero,
}

impl VerifyErrorKind {
    /// Human-readable description for the report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::MissingConstitutionFlag(name) => {
                format!("constitution flag `{name}` is false; every flag must be declared true")
            }
            Self::EmptyDisplayName => String::from("display_name is empty"),
            Self::MissingProvenance => String::from(
                "no SourceRef declared and no engineering-practice note populated",
            ),
            Self::EngineeringPracticeWithoutNote(key) => format!(
                "SourceRef `{key}` declares year=0 (engineering practice) but `notes` is empty"
            ),
            Self::NoOriginDomains => String::from(
                "origin_domains bitset is empty; every primitive must declare at least one domain",
            ),
            Self::NoInputRequirements => String::from(
                "input_requirements bitset is empty; every primitive operates over SOME input contract",
            ),
            Self::NoFusionAxes => String::from(
                "fusion_axes bitset is empty; every primitive must bind to at least one fusion axis",
            ),
            Self::DuplicateGroupMismatch => String::from(
                "duplicate_group does not match canonical_id; T.1a entries are all canonical heads",
            ),
            Self::ReservedCanonicalIdZero => {
                String::from("canonical_id is 0; that value is reserved as a null sentinel")
            }
        }
    }
}

/// Aggregate report from verifying a slice of records.
///
/// Returned by [`verify_corpus`]. The verifier never panics on a
/// bad record; it accumulates errors and returns them all in one
/// pass so a future engineer can fix many records in one commit.
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    /// Number of records inspected.
    pub records_inspected: usize,
    /// Per-record errors. Empty if the corpus is clean.
    pub errors: Vec<VerifyError>,
}

impl VerifyReport {
    /// True if no errors were recorded.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }

    /// Number of unique records that had at least one error.
    #[must_use]
    pub fn unique_failed_records(&self) -> usize {
        let mut ids: Vec<DetectorCanonicalId> =
            self.errors.iter().map(|e| e.canonical_id).collect();
        ids.sort();
        ids.dedup();
        ids.len()
    }
}

/// Verify one record against the schema invariants. Returns the
/// list of errors found (empty when the record is clean).
///
/// Public so the CLI can run per-record diagnostics and so tests
/// can pin individual cases.
#[must_use]
pub fn verify_record(record: &LiteratureDetector) -> Vec<VerifyError> {
    let id = record.canonical_id;
    let mut errors = Vec::new();

    if id.0 == 0 {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::ReservedCanonicalIdZero,
        });
    }

    check_constitution_flags(id, record.constitution_compliance, &mut errors);

    if record.display_name.is_empty() {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::EmptyDisplayName,
        });
    }

    check_provenance(id, record.source_refs, &mut errors);

    if record.origin_domains.is_empty() {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::NoOriginDomains,
        });
    }

    if record.input_requirements.is_empty() {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::NoInputRequirements,
        });
    }

    if record.fusion_axes.is_empty() {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::NoFusionAxes,
        });
    }

    if record.duplicate_group.0 != id.0 {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::DuplicateGroupMismatch,
        });
    }

    errors
}

/// Verify every record in a slice and return the aggregated report.
#[must_use]
pub fn verify_corpus(records: &[LiteratureDetector]) -> VerifyReport {
    let mut report = VerifyReport {
        records_inspected: records.len(),
        errors: Vec::new(),
    };
    for record in records {
        let mut record_errors = verify_record(record);
        report.errors.append(&mut record_errors);
    }
    report
}

fn check_constitution_flags(
    id: DetectorCanonicalId,
    flags: ConstitutionFlags,
    errors: &mut Vec<VerifyError>,
) {
    let cases = [
        ("declared_input_contract", flags.declared_input_contract),
        ("declared_output_type", flags.declared_output_type),
        (
            "declared_deterministic_form",
            flags.declared_deterministic_form,
        ),
        ("declared_provenance", flags.declared_provenance),
        (
            "declared_equivalence_status",
            flags.declared_equivalence_status,
        ),
        ("declared_witness_role", flags.declared_witness_role),
        (
            "declared_activation_conditions",
            flags.declared_activation_conditions,
        ),
        (
            "declared_failure_confuser_modes",
            flags.declared_failure_confuser_modes,
        ),
    ];
    for (name, value) in cases {
        if !value {
            errors.push(VerifyError {
                canonical_id: id,
                kind: VerifyErrorKind::MissingConstitutionFlag(name),
            });
        }
    }
}

fn check_provenance(
    id: DetectorCanonicalId,
    source_refs: &[SourceRef],
    errors: &mut Vec<VerifyError>,
) {
    if source_refs.is_empty() {
        errors.push(VerifyError {
            canonical_id: id,
            kind: VerifyErrorKind::MissingProvenance,
        });
        return;
    }
    for s in source_refs {
        if s.year == 0 && s.notes.is_empty() {
            errors.push(VerifyError {
                canonical_id: id,
                kind: VerifyErrorKind::EngineeringPracticeWithoutNote(s.citation_key),
            });
        }
    }
}
