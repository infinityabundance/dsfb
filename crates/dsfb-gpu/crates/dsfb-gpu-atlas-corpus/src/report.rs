//! Report writer — emits the public dedup report structure.
//!
//! The T.9 deliverable is `detector_corpus_dedup_report.json` plus
//! a human-readable mirror. T.1a ships the report SHAPE; T.4-T.8
//! populate the dedup / genealogy / witness / L-band / usefulness
//! sections with real data. T.1a fills only the trivially-
//! derivable rows (record count, per-family counts, per-domain
//! counts, L-band histogram, lifecycle histogram). T.6 added the
//! witness-role / fusion-plane / role-axis / witness-law /
//! primary-list / confuser-list sections. T.7 added the L-band
//! honesty-invariant section (histogram + GPU whitelist +
//! verifier result). T.8 added the usefulness-ledger honesty
//! section (evidence-level histogram, lifecycle cross-check,
//! no-fabricated-claims invariant, verifier result). T.9 lands
//! the canonical 10-section publication-grade dedup report.
//!
//! The report is plain ASCII text. T.10 adds a JSON variant when
//! `corpus_hash_v1` lands and the report becomes a hashable
//! artifact.

use crate::fusion::{axes_to_planes, FusionPlane};
use crate::seed::SEED;
use crate::types::{
    DomainTagSet, ImplementationLevel, LifecycleState, LiteratureDetector, NegativeWitnessKind,
    PrimitiveFamily, WitnessRole,
};

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

/// Emit the public dedup report as a UTF-8 string.
///
/// The report has six sections at T.1a, expanded as later T-sections
/// land:
/// 1. Totals (record count, canonicals, aliases).
/// 2. Per-primitive-family histogram.
/// 3. Per-domain coverage matrix (counts of detectors flagged for
///    each domain bit).
/// 4. Implementation-level (L0..L8) histogram.
/// 5. Lifecycle-state histogram.
/// 6. Witness-role histogram.
///
/// T.6 added:
/// 7. Negative-witness histogram.
/// 8. Fusion-plane histogram.
/// 9. Witness-role × fusion-plane coverage matrix.
/// 10. Witness-law coverage invariants.
/// 11. Primary-witness detector list.
/// 12. Confuser detector list.
///
/// T.7 added:
/// 13. L-band honesty invariants (histogram + GPU whitelist +
///     verifier result).
///
/// T.8 added:
/// 14. Usefulness-ledger honesty invariants (evidence-level
///     histogram + lifecycle cross-check + no-fabricated-claims
///     invariant + verifier result).
///
/// T.9 will add:
/// - Source-ref provenance summary (count by venue category).
/// - Publication-grade 10-section dedup report.
#[must_use]
pub fn render_report(records: &[LiteratureDetector]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== DSFB-GPU-Atlas literature detector corpus report ==="
    );
    let _ = writeln!(
        out,
        "(T.1a-T.8 populated: schema/seed/TOML/identity/court/genealogy/witness-law/"
    );
    let _ = writeln!(
        out,
        " L-band/usefulness-ledger; T.9 lands the publication-grade dedup report)"
    );
    let _ = writeln!(out);

    section_totals(&mut out, records);
    section_court_decision_summary(&mut out);
    section_family_histogram(&mut out, records);
    section_domain_coverage(&mut out, records);
    section_lband_histogram(&mut out, records);
    section_lifecycle_histogram(&mut out, records);
    section_witness_role_histogram(&mut out, records);
    section_negative_witness_histogram(&mut out, records);
    section_fusion_plane_histogram(&mut out, records);
    section_role_axis_matrix(&mut out, records);
    section_witness_law_coverage(&mut out, records);
    section_primary_detector_list(&mut out, records);
    section_confuser_detector_list(&mut out, records);
    section_lband_honesty_invariants(&mut out, records);
    section_usefulness_ledger(&mut out, records);

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "T.8 note: usefulness-ledger sections (T.6) + L-band invariants (T.7)"
    );
    let _ = writeln!(
        out,
        "+ ledger honesty (T.8) populated. Dedup-court (T.4) and"
    );
    let _ = writeln!(
        out,
        "genealogy-edge (T.5) details live in their own dedicated reports."
    );
    let _ = writeln!(
        out,
        "The full publication-grade dedup report (T.9) populates in a later section."
    );

    out
}

// =========================================================
// T.6 — witness-law report sections.
//
// Why these exist: T.6 declares "what evidentiary role each
// detector is allowed to play". The corpus needs an at-a-glance
// view of the role distribution AND a per-role-per-axis matrix so
// a future engineer (often the user, months later) can see at a
// glance whether the corpus has appropriate coverage on each
// fusion plane. Hard counts only — no fuzzy / probabilistic data.
// =========================================================

fn section_negative_witness_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(7) Negative-witness histogram (T.6)");
    let kinds: [(&str, NegativeWitnessKind); 10] = [
        (
            "SmallSampleConfuser",
            NegativeWitnessKind::SmallSampleConfuser,
        ),
        (
            "SingleWindowSpikeConfuser",
            NegativeWitnessKind::SingleWindowSpikeConfuser,
        ),
        (
            "PeriodicBoundaryConfuser",
            NegativeWitnessKind::PeriodicBoundaryConfuser,
        ),
        (
            "MissingnessArtifactConfuser",
            NegativeWitnessKind::MissingnessArtifactConfuser,
        ),
        (
            "SchemaChangeConfuser",
            NegativeWitnessKind::SchemaChangeConfuser,
        ),
        (
            "UnitScaleChangeConfuser",
            NegativeWitnessKind::UnitScaleChangeConfuser,
        ),
        (
            "DeploymentMarkerConfuser",
            NegativeWitnessKind::DeploymentMarkerConfuser,
        ),
        ("ClockSkewConfuser", NegativeWitnessKind::ClockSkewConfuser),
        (
            "BatchBoundaryConfuser",
            NegativeWitnessKind::BatchBoundaryConfuser,
        ),
        (
            "NotANegativeWitness",
            NegativeWitnessKind::NotANegativeWitness,
        ),
    ];
    for (name, kind) in kinds {
        let count = records
            .iter()
            .filter(|r| r.negative_witness_kind == kind)
            .count();
        let _ = writeln!(out, "  {name:<32} : {count}");
    }
    let _ = writeln!(out);
}

fn section_fusion_plane_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(8) Fusion-plane histogram (T.6)");
    let _ = writeln!(out, "  (each detector contributes to one or more planes)");
    for plane in FusionPlane::all() {
        let count = records
            .iter()
            .filter(|r| axes_to_planes(r.fusion_axes).contains(*plane))
            .count();
        let _ = writeln!(out, "  {name:<32} : {count}", name = plane.as_str());
    }
    let _ = writeln!(out);
}

fn section_role_axis_matrix(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(9) Witness-role x fusion-plane coverage matrix (T.6)");
    let _ = writeln!(out, "  rows = WitnessRole; columns = FusionPlane; cell = count of detectors with that role bound to that plane.");
    let roles: [(WitnessRole, &str); 10] = [
        (WitnessRole::Primary, "Primary"),
        (WitnessRole::Corroborating, "Corroborating"),
        (WitnessRole::Confuser, "Confuser"),
        (WitnessRole::Boundary, "Boundary"),
        (WitnessRole::CleanWindow, "CleanWindow"),
        (WitnessRole::Recovery, "Recovery"),
        (WitnessRole::Timing, "Timing"),
        (WitnessRole::Distribution, "Distribution"),
        (WitnessRole::Topology, "Topology"),
        (WitnessRole::CausalityProxy, "CausalityProxy"),
    ];
    // Header
    let _ = write!(out, "  {empty:<16}", empty = "");
    for plane in FusionPlane::all() {
        let abbrev = match plane {
            FusionPlane::ProvenanceAdmissibility => "Prov",
            FusionPlane::NumericStrength => "Num",
            FusionPlane::TemporalStructure => "Temp",
            FusionPlane::CrossSignalStructure => "XSig",
            FusionPlane::DistributionStructure => "Dist",
            FusionPlane::SemanticBankStructure => "Bank",
            FusionPlane::ReliabilityConfuserControl => "Rel",
            FusionPlane::TaskUtility => "Util",
        };
        let _ = write!(out, " {abbrev:>5}");
    }
    let _ = writeln!(out);
    // Rows
    for (role, role_name) in roles {
        let _ = write!(out, "  {role_name:<16}");
        for plane in FusionPlane::all() {
            let count = records
                .iter()
                .filter(|r| {
                    r.witness_role == role && axes_to_planes(r.fusion_axes).contains(*plane)
                })
                .count();
            let _ = write!(out, " {count:>5}");
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out);
}

fn section_witness_law_coverage(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(10) Witness-law coverage (T.6 invariants)");
    let missing_role = records.iter().filter(|_| false).count();
    // (Every record has a WitnessRole enum value; the "missing" check
    // is structurally impossible with the current schema. Print 0 to
    // document the invariant.)
    let missing_axis = records.iter().filter(|r| r.fusion_axes.is_empty()).count();
    let missing_plane = records
        .iter()
        .filter(|r| axes_to_planes(r.fusion_axes).is_empty())
        .count();
    let primary_with_neg = records
        .iter()
        .filter(|r| {
            r.witness_role == WitnessRole::Primary
                && r.negative_witness_kind != NegativeWitnessKind::NotANegativeWitness
        })
        .count();
    let confuser_without_neg = records
        .iter()
        .filter(|r| {
            r.witness_role == WitnessRole::Confuser
                && r.negative_witness_kind == NegativeWitnessKind::NotANegativeWitness
        })
        .count();
    let _ = writeln!(
        out,
        "  detectors_missing_roles                  : {missing_role}"
    );
    let _ = writeln!(
        out,
        "  detectors_missing_axis_bindings          : {missing_axis}"
    );
    let _ = writeln!(
        out,
        "  detectors_missing_fusion_planes          : {missing_plane}"
    );
    let _ = writeln!(
        out,
        "  primary_witnesses_with_negative_kind     : {primary_with_neg}  (invariant: must be 0)"
    );
    let _ = writeln!(
        out,
        "  confuser_witnesses_without_negative_kind : {confuser_without_neg}  (invariant: must be 0)"
    );
    let _ = writeln!(out);
}

fn section_primary_detector_list(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(11) Primary-witness detector list (T.6)");
    for r in records
        .iter()
        .filter(|r| r.witness_role == WitnessRole::Primary)
    {
        let _ = writeln!(
            out,
            "  [{id:>3}] {name}",
            id = r.canonical_id.0,
            name = r.display_name
        );
    }
    let _ = writeln!(out);
}

fn section_confuser_detector_list(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(12) Confuser detector list (T.6)");
    for r in records
        .iter()
        .filter(|r| r.witness_role == WitnessRole::Confuser)
    {
        let _ = writeln!(
            out,
            "  [{id:>3}] {name}  (kind: {kind:?})",
            id = r.canonical_id.0,
            name = r.display_name,
            kind = r.negative_witness_kind
        );
    }
    let _ = writeln!(out);
}

// =========================================================
// T.7 — L-band honesty-invariant report section.
//
// Why this exists: T.7 turns the L-band field into an
// auditable claim. The histogram tells a future engineer which
// rungs of the implementation ladder the corpus actually
// occupies; the invariant block tells them which rungs are
// forbidden until later sections (T.7.*, T.8) ship real
// evidence. Together they prevent "corpus inflation by
// implication".
// =========================================================

fn section_lband_honesty_invariants(out: &mut String, records: &[LiteratureDetector]) {
    use crate::lband::{compute_histogram, verify_corpus_lband, GPU_IMPLEMENTED_CANONICAL_IDS};
    let _ = writeln!(out, "(13) L-band honesty invariants (T.7)");
    let _ = writeln!(
        out,
        "  L-band is an honesty marker, not a quality score. A detector"
    );
    let _ = writeln!(
        out,
        "  at L1 is cited and canonicalised; it is not inferior to L6."
    );
    let _ = writeln!(out);
    let histogram = compute_histogram(records);
    let _ = writeln!(out, "  Histogram (records per L-band):");
    let _ = writeln!(out, "    L0_CitedOnly                  : {}", histogram.l0);
    let _ = writeln!(out, "    L1_Canonicalised              : {}", histogram.l1);
    let _ = writeln!(out, "    L2_DeterministicFormula       : {}", histogram.l2);
    let _ = writeln!(out, "    L3_CpuImplemented             : {}", histogram.l3);
    let _ = writeln!(out, "    L4_CpuVerified                : {}", histogram.l4);
    let _ = writeln!(out, "    L5_GpuImplemented             : {}", histogram.l5);
    let _ = writeln!(out, "    L6_CpuGpuByteEquivalent       : {}", histogram.l6);
    let _ = writeln!(
        out,
        "    L7_BenchmarkCharacterised     : {}  (forbidden at T.7)",
        histogram.l7
    );
    let _ = writeln!(
        out,
        "    L8_LedgerCharacterised        : {}  (forbidden at T.7)",
        histogram.l8
    );
    let _ = writeln!(
        out,
        "    total                         : {}",
        histogram.total()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  GPU-implemented canonical IDs (load-bearing whitelist):"
    );
    let mut ids: alloc::vec::Vec<u32> = GPU_IMPLEMENTED_CANONICAL_IDS.iter().map(|c| c.0).collect();
    ids.sort_unstable();
    for id in ids {
        let name = records
            .iter()
            .find(|r| r.canonical_id.0 == id)
            .map_or("<unknown>", |r| r.display_name);
        let _ = writeln!(out, "    [{id:>3}] {name}");
    }
    let _ = writeln!(out);
    let verify = verify_corpus_lband(records);
    let _ = writeln!(out, "  Verifier result:");
    let _ = writeln!(
        out,
        "    records_inspected             : {}",
        verify.records_inspected
    );
    let _ = writeln!(
        out,
        "    errors                        : {}",
        verify.errors.len()
    );
    if !verify.is_clean() {
        for err in &verify.errors {
            let _ = writeln!(
                out,
                "      [{id:>3}] {desc}",
                id = err.canonical_id.0,
                desc = err.kind.describe()
            );
        }
    }
    let _ = writeln!(out);
}

/// Default-render the seed corpus (convenience wrapper).
#[must_use]
pub fn render_seed_report() -> String {
    render_report(SEED)
}

// =========================================================
// T.8 — usefulness-ledger report section.
//
// Why this exists: T.8 introduces the deterministic detector
// usefulness ledger keyed by (canonical_id, task_id, domain,
// dataset_id). Section 14 is the public audit surface: it
// reports the evidence-level histogram, the lifecycle-state
// histogram (cross-checking the schema-level
// LifecycleState in Section 5), the no-fabricated-claims
// invariant, and the verifier result. The block deliberately
// names the panel-locked thesis verbatim so a reviewer cannot
// mistake the ledger for a learned ranking model.
// =========================================================

// One large rule-by-rule renderer so a reviewer can read every
// sub-block (header / evidence histogram / lifecycle / invariant /
// verifier) in source order without jumping helpers. The
// `too_many_lines` clippy warning is locally allowed for that
// reason.
#[allow(clippy::too_many_lines)]
fn section_usefulness_ledger(out: &mut String, records: &[LiteratureDetector]) {
    use crate::usefulness::{
        compute_evidence_histogram, compute_lifecycle_histogram, verify_usefulness_ledger,
        UsefulnessLedgerRow, USEFULNESS_LEDGER,
    };
    let _ = writeln!(out, "(14) Usefulness ledger honesty invariants (T.8)");
    let _ = writeln!(
        out,
        "  The usefulness ledger is an audit surface, not a learned"
    );
    let _ = writeln!(
        out,
        "  ranking model. T.8 records declared evidence levels and"
    );
    let _ = writeln!(
        out,
        "  conservative contribution fields; empirical usefulness"
    );
    let _ = writeln!(out, "  remains unclaimed until a row is backed by a named");
    let _ = writeln!(out, "  benchmark artifact.");
    let _ = writeln!(out);

    let total_rows = USEFULNESS_LEDGER.len();
    let _ = writeln!(out, "  rows loaded                   : {total_rows}");
    let _ = writeln!(out, "  detectors covered             : {}", records.len());
    let _ = writeln!(out);

    let ev = compute_evidence_histogram(USEFULNESS_LEDGER);
    let _ = writeln!(out, "  Evidence-level histogram:");
    let _ = writeln!(out, "    Unmeasured                  : {}", ev.unmeasured);
    let _ = writeln!(
        out,
        "    LiteraturePrior             : {}",
        ev.literature_prior
    );
    let _ = writeln!(out, "    RoleSeeded                  : {}", ev.role_seeded);
    let _ = writeln!(
        out,
        "    SyntheticFixtureMeasured    : {}  (forbidden without pinned fixture)",
        ev.synthetic_fixture
    );
    let _ = writeln!(
        out,
        "    RealDatasetMeasured         : {}  (forbidden without hashed dataset)",
        ev.real_dataset
    );
    let _ = writeln!(
        out,
        "    CrossDomainReplicated       : {}  (forbidden without 2+ domain runs)",
        ev.cross_domain
    );
    let _ = writeln!(
        out,
        "    RetiredByEvidence           : {}  (forbidden without measured negative)",
        ev.retired_by_evidence
    );
    let _ = writeln!(out, "    total                       : {}", ev.total());
    let _ = writeln!(out);

    let lc = compute_lifecycle_histogram(USEFULNESS_LEDGER);
    let _ = writeln!(
        out,
        "  Lifecycle-state histogram (cross-check vs Section 5):"
    );
    let _ = writeln!(out, "    Active                      : {}", lc.active);
    let _ = writeln!(out, "    Dormant                     : {}", lc.dormant);
    let _ = writeln!(
        out,
        "    RetiredRedundant            : {}",
        lc.retired_redundant
    );
    let _ = writeln!(
        out,
        "    RetiredHighFalsePositive    : {}",
        lc.retired_high_fp
    );
    let _ = writeln!(
        out,
        "    RetiredTooExpensive         : {}",
        lc.retired_too_expensive
    );
    let _ = writeln!(out, "    QuarantinedUnstable         : {}", lc.quarantined);
    let _ = writeln!(out, "    ResurrectedForDomain        : {}", lc.resurrected);
    let _ = writeln!(out, "    total                       : {}", lc.total());
    let _ = writeln!(out);

    // No-fabricated-claims invariant: every row must have zero
    // empirical fields at evidence levels < SyntheticFixtureMeasured.
    let mut fabricated = 0usize;
    for r in USEFULNESS_LEDGER {
        if r.evidence_level.forbids_empirical_claims()
            && !UsefulnessLedgerRow::has_zero_empirical_fields(r)
        {
            fabricated += 1;
        }
    }
    let _ = writeln!(out, "  No-fabricated-claims invariant:");
    let _ = writeln!(
        out,
        "    rows at Unmeasured/Literature/Role with nonzero empirical fields : {fabricated}"
    );
    let _ = writeln!(
        out,
        "    (must be 0 at T.8; any nonzero count is a verifier-rejected row)"
    );
    let _ = writeln!(out);

    let verify = verify_usefulness_ledger(records, USEFULNESS_LEDGER);
    let _ = writeln!(out, "  Verifier result:");
    let _ = writeln!(
        out,
        "    records_inspected           : {}",
        verify.records_inspected
    );
    let _ = writeln!(
        out,
        "    rows_inspected              : {}",
        verify.rows_inspected
    );
    let _ = writeln!(
        out,
        "    errors                      : {}",
        verify.errors.len()
    );
    if !verify.is_clean() {
        for err in &verify.errors {
            let _ = writeln!(
                out,
                "      [{id:>3}] {desc}",
                id = err.canonical_id.0,
                desc = err.kind.describe()
            );
        }
    }
    let _ = writeln!(out);
}

fn section_totals(out: &mut String, records: &[LiteratureDetector]) {
    let total = records.len();
    let canonicals = records
        .iter()
        .filter(|r| r.duplicate_group.0 == r.canonical_id.0)
        .count();
    let aliases = total - canonicals;
    let _ = writeln!(out, "(1) Totals");
    let _ = writeln!(out, "  total records              : {total}");
    let _ = writeln!(out, "  canonical primitives       : {canonicals}");
    let _ = writeln!(
        out,
        "  alias / duplicate records  : {aliases}  (schema-level duplicate_group rollup; the authoritative dedup tally lives in (1b), produced by the T.4 court)"
    );
    let _ = writeln!(out);
}

// =========================================================
// (1b) Dedup-court decision summary.
//
// Why this exists: the (1) Totals block reports SCHEMA-LEVEL
// duplicate-group membership, which T.1a-T.1b populated by
// pointing every record at its own canonical_id. After T.4's
// court machinery landed, the AUTHORITATIVE dedup picture is
// the count of `CanonicalisationDecision` variants the court
// emits over SEED + CLAIMS. (1b) names that source-of-truth
// explicitly so a reviewer is not misled by the (1) row.
// =========================================================

fn section_court_decision_summary(out: &mut String) {
    let records = crate::court::classify_all();
    let counts = crate::court::count_decisions(&records);
    let _ = writeln!(out, "(1b) Dedup-court decision summary (T.4)");
    let _ = writeln!(
        out,
        "  source              : crate::court::classify_all() over SEED + CLAIMS"
    );
    let _ = writeln!(out, "  court subjects             : {}", counts.total());
    let _ = writeln!(out, "  canonical decisions        : {}", counts.canonical);
    let _ = writeln!(out, "  alias decisions            : {}", counts.aliases);
    let _ = writeln!(
        out,
        "  parameterisation decisions : {}",
        counts.parameterisations
    );
    let _ = writeln!(
        out,
        "  composition decisions      : {}",
        counts.compositions
    );
    let _ = writeln!(
        out,
        "  stochastic-reduction decs. : {}",
        counts.stochastic_reductions
    );
    let _ = writeln!(out, "  rejected records           : {}", counts.rejected);
    let _ = writeln!(out, "  deferred records           : {}", counts.deferred);
    let _ = writeln!(out);
}

fn section_family_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(2) Per-primitive-family histogram");
    let mut counts: Vec<(PrimitiveFamily, usize)> = Vec::new();
    for r in records {
        if let Some(slot) = counts.iter_mut().find(|(f, _)| *f == r.primitive_family) {
            slot.1 += 1;
        } else {
            counts.push((r.primitive_family, 1));
        }
    }
    counts.sort_by_key(|(f, _)| *f);
    for (family, count) in counts {
        let _ = writeln!(out, "  {family:>30?} : {count}");
    }
    let _ = writeln!(out);
}

fn section_domain_coverage(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(3) Per-domain coverage matrix");
    let domains: [(&str, u16); 13] = [
        ("Debug", DomainTagSet::DEBUG),
        ("Telemetry", DomainTagSet::TELEMETRY),
        ("Tabular", DomainTagSet::TABULAR),
        ("TimeSeries", DomainTagSet::TIME_SERIES),
        ("Graph", DomainTagSet::GRAPH),
        ("Industrial", DomainTagSet::INDUSTRIAL),
        ("Categorical", DomainTagSet::CATEGORICAL),
        ("Missingness", DomainTagSet::MISSINGNESS),
        ("EventStream", DomainTagSet::EVENT_STREAM),
        ("Medical", DomainTagSet::MEDICAL),
        ("RFComms", DomainTagSet::RF_COMMS),
        ("Chemometrics", DomainTagSet::CHEMOMETRICS),
        ("Database", DomainTagSet::DATABASE),
    ];
    for (name, mask) in domains {
        let count = records
            .iter()
            .filter(|r| (r.origin_domains.0 & mask) != 0)
            .count();
        let _ = writeln!(out, "  {name:<15} : {count}");
    }
    let _ = writeln!(out);
}

fn section_lband_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(4) Implementation-level (L-band) histogram");
    let bands: [(&str, ImplementationLevel); 9] = [
        ("L0 CitedOnly", ImplementationLevel::L0_CitedOnly),
        ("L1 Canonicalised", ImplementationLevel::L1_Canonicalised),
        (
            "L2 DeterministicFormula",
            ImplementationLevel::L2_DeterministicFormula,
        ),
        ("L3 CpuImplemented", ImplementationLevel::L3_CpuImplemented),
        ("L4 CpuVerified", ImplementationLevel::L4_CpuVerified),
        ("L5 GpuImplemented", ImplementationLevel::L5_GpuImplemented),
        (
            "L6 CpuGpuByteEquivalent",
            ImplementationLevel::L6_CpuGpuByteEquivalent,
        ),
        (
            "L7 BenchmarkCharacterised",
            ImplementationLevel::L7_BenchmarkCharacterised,
        ),
        (
            "L8 LedgerCharacterised",
            ImplementationLevel::L8_LedgerCharacterised,
        ),
    ];
    for (name, level) in bands {
        let count = records
            .iter()
            .filter(|r| r.implementation_status == level)
            .count();
        let _ = writeln!(out, "  {name:<28} : {count}");
    }
    let _ = writeln!(out);
}

fn section_lifecycle_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(5) Lifecycle-state histogram");
    let states: [(&str, LifecycleState); 7] = [
        ("Active", LifecycleState::Active),
        ("Dormant", LifecycleState::Dormant),
        ("RetiredRedundant", LifecycleState::RetiredRedundant),
        (
            "RetiredHighFalsePositive",
            LifecycleState::RetiredHighFalsePositive,
        ),
        ("RetiredTooExpensive", LifecycleState::RetiredTooExpensive),
        ("QuarantinedUnstable", LifecycleState::QuarantinedUnstable),
        ("ResurrectedForDomain", LifecycleState::ResurrectedForDomain),
    ];
    for (name, state) in states {
        let count = records
            .iter()
            .filter(|r| r.lifecycle_state == state)
            .count();
        let _ = writeln!(out, "  {name:<28} : {count}");
    }
    let _ = writeln!(out);
}

fn section_witness_role_histogram(out: &mut String, records: &[LiteratureDetector]) {
    let _ = writeln!(out, "(6) Witness-role histogram");
    let roles: [(&str, WitnessRole); 10] = [
        ("Primary", WitnessRole::Primary),
        ("Corroborating", WitnessRole::Corroborating),
        ("Confuser", WitnessRole::Confuser),
        ("Boundary", WitnessRole::Boundary),
        ("CleanWindow", WitnessRole::CleanWindow),
        ("Recovery", WitnessRole::Recovery),
        ("Timing", WitnessRole::Timing),
        ("Distribution", WitnessRole::Distribution),
        ("Topology", WitnessRole::Topology),
        ("CausalityProxy", WitnessRole::CausalityProxy),
    ];
    for (name, role) in roles {
        let count = records.iter().filter(|r| r.witness_role == role).count();
        let _ = writeln!(out, "  {name:<20} : {count}");
    }
    let _ = writeln!(out);
}

/// Human-readable genealogy summary for the `dsfb-corpus genealogy`
/// subcommand. Reports origin / non-origin counts plus the origin
/// list. Machine-consumable exports live in
/// [`crate::genealogy::export_dot`] and [`crate::genealogy::export_json`]
/// (T.5) and are emitted by the `genealogy-dot` and
/// `genealogy-json` subcommands.
#[must_use]
pub fn render_genealogy_summary(records: &[LiteratureDetector]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== DSFB-GPU-Atlas literature detector corpus genealogy ==="
    );
    let _ = writeln!(
        out,
        "(T.5 — human-readable summary; see `genealogy-dot` and `genealogy-json` for machine-consumable exports)"
    );
    let _ = writeln!(out);
    let total = records.len();
    let origins = records.iter().filter(|r| r.genealogy.is_origin).count();
    let derived = total - origins;
    let _ = writeln!(out, "  total records              : {total}");
    let _ = writeln!(out, "  origin (no ancestors)      : {origins}");
    let _ = writeln!(out, "  derived (>= 1 ancestor)    : {derived}");
    let _ = writeln!(out);
    let _ = writeln!(out, "Origin records:");
    for r in records.iter().filter(|r| r.genealogy.is_origin) {
        let _ = writeln!(
            out,
            "  [{id:>3}] {name}",
            id = r.canonical_id.0,
            name = r.display_name
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Derived records (ancestor IDs):");
    for r in records.iter().filter(|r| !r.genealogy.is_origin) {
        let ancestors: Vec<String> = r
            .genealogy
            .derived_from
            .iter()
            .map(|a| format!("{}", a.0))
            .collect();
        let _ = writeln!(
            out,
            "  [{id:>3}] {name} <- [{anc}]",
            id = r.canonical_id.0,
            name = r.display_name,
            anc = ancestors.join(", ")
        );
    }
    out
}
