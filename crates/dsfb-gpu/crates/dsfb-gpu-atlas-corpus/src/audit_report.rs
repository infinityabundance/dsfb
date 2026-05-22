//! T.9 — internal corpus audit report bundle.
//!
//! **Panel-locked scope (T.9 is INTERNAL, NOT publication)**:
//!
//! T.9 is a cold-reader internal audit artifact, not an external
//! release artifact. It exists so future T.10 / T.11 work has a
//! clean factual baseline, **not** so the corpus is deposited.
//!
//! T.9 explicitly does NOT do:
//!
//! - No Zenodo deposit, no DOI, no publication metadata.
//! - No `corpus_hash_v1` (deferred to T.10).
//! - No `CaseFileV2` integration (deferred to T.11).
//! - No GPU code changes.
//! - No mutation of T.1-T.8 schemas.
//! - No new theory layer.
//!
//! T.9 ships a deterministic [`AuditReportBundle`] with two
//! mirror artifacts (human-readable TXT + machine-readable JSON)
//! plus refreshed genealogy DOT / JSON exports. The bundle
//! consolidates T.1-T.8 counts into ten stable top-level
//! sections so a reviewer (often the user, months later) can
//! understand at a glance what exists, what is canonical, what
//! is merely cited, what is implemented, what is GPU-ready, and
//! what is explicitly not claimed.
//!
//! Honesty invariants enforced by the bundle (and pinned by
//! [`crate::audit_report`] acceptance tests):
//!
//! - Aliases are reported separately from canonical primitives;
//!   they are NOT counted as unique primitives.
//! - L0-L4 records are reported as cited / formula-only /
//!   CPU-implemented / CPU-verified; they are NOT described as
//!   GPU-ready.
//! - L7 / L8 counts are reported only as "forbidden until
//!   benchmark / ledger evidence" categories with zero counts at
//!   T.9.
//! - The usefulness ledger is reported with its evidence-level
//!   ladder; no row is described as "useful" or "ranked" unless
//!   it carries `score_kind != NotScored`. At T.9 every row is
//!   `NotScored`.
//! - No "Zenodo", "DOI", "publish", or "deposit" language appears
//!   anywhere in the rendered TXT or JSON.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::court::{classify_all, count_decisions, CourtReportCounts};
use crate::fusion::{axes_to_planes, FusionPlane};
use crate::genealogy::{
    build_genealogy, export_dot, export_json, verify_genealogy, GenealogyGraph,
};
use crate::lband::{compute_histogram as compute_lband_histogram, GPU_IMPLEMENTED_CANONICAL_IDS};
use crate::seed::SEED;
use crate::types::{ImplementationLevel, LifecycleState, PrimitiveFamily, WitnessRole};
use crate::usefulness::{
    compute_evidence_histogram, compute_lifecycle_histogram, verify_usefulness_ledger,
    UsefulnessEvidenceLevel, UsefulnessLedgerRow, USEFULNESS_LEDGER,
};

/// Canonical schema string the JSON artifact carries. Two builds
/// emit byte-identical JSON beginning with this schema name.
pub const AUDIT_REPORT_SCHEMA: &str = "DSFB-GPU-ATLAS:CORPUS-AUDIT-REPORT:v1";

/// Internal release stage label rendered into both TXT and JSON.
/// T.9 is the internal-audit-pre-freeze stage; T.10 advances it.
pub const RELEASE_STAGE: &str = "internal-audit-pre-freeze";

/// Aggregate corpus counts surfaced into the audit bundle.
#[derive(Debug, Clone, Copy)]
pub struct CorpusCounts {
    /// Distinct literature primitive records in SEED.
    pub literature_primitives: usize,
    /// Alias claims in T.4 CLAIMS (separately counted; NOT added
    /// to `literature_primitives` for headline purposes).
    pub alias_claims: usize,
    /// Court subjects = SEED + CLAIMS (sum of the two; the
    /// authoritative dedup tally is the court summary, not this
    /// total).
    pub court_subjects: usize,
}

/// Court-decision summary — mirrors
/// [`crate::court::CourtReportCounts`] but carries an explicit
/// total so JSON consumers don't recompute.
#[derive(Debug, Clone, Copy)]
pub struct CourtSummary {
    /// Raw counts from `count_decisions`.
    pub counts: CourtReportCounts,
}

/// Identity-hash summary (T.3).
#[derive(Debug, Clone, Copy)]
pub struct IdentitySummary {
    /// Records carrying a five-hash identity (= SEED length;
    /// every record has identity hashes computed on demand).
    pub records_with_identity_hashes: usize,
}

/// Genealogy-graph summary (T.5).
#[derive(Debug, Clone, Copy)]
pub struct GenealogySummary {
    /// Node count in the built genealogy graph.
    pub nodes: usize,
    /// Edge count in the built genealogy graph.
    pub edges: usize,
    /// Origin records (no ancestors).
    pub origin_records: usize,
    /// Derived records (>= 1 ancestor).
    pub derived_records: usize,
    /// True if `verify_genealogy` reported a clean DAG.
    pub dag_verified_clean: bool,
}

/// Witness-role + fusion-plane summary (T.6).
#[derive(Debug, Clone, Copy)]
pub struct WitnessSummary {
    /// Records per witness role, in canonical order
    /// (Primary, Corroborating, Confuser, Boundary, CleanWindow,
    /// Recovery, Timing, Distribution, Topology, CausalityProxy).
    pub role_counts: [usize; 10],
    /// Records per fusion plane, in canonical order
    /// (ProvenanceAdmissibility, NumericStrength, TemporalStructure,
    /// CrossSignalStructure, DistributionStructure, SemanticBankStructure,
    /// ReliabilityConfuserControl, TaskUtility).
    pub plane_counts: [usize; 8],
}

/// L-band summary (T.7).
#[derive(Debug, Clone, Copy)]
pub struct LBandSummary {
    /// Records at L0..L8, in canonical order.
    pub counts: [usize; 9],
    /// Whitelisted canonical IDs (sourced from
    /// [`GPU_IMPLEMENTED_CANONICAL_IDS`]).
    pub gpu_whitelist_size: usize,
    /// True if the T.7 verifier reports no errors over SEED.
    pub verifier_clean: bool,
}

/// Usefulness-ledger summary (T.8).
#[derive(Debug, Clone, Copy)]
pub struct UsefulnessSummary {
    /// Rows in USEFULNESS_LEDGER.
    pub rows_loaded: usize,
    /// Per-evidence-level counts (Unmeasured, LiteraturePrior,
    /// RoleSeeded, SyntheticFixtureMeasured, RealDatasetMeasured,
    /// CrossDomainReplicated, RetiredByEvidence).
    pub evidence_level_counts: [usize; 7],
    /// Per-lifecycle counts (Active, Dormant, RetiredRedundant,
    /// RetiredHighFalsePositive, RetiredTooExpensive,
    /// QuarantinedUnstable, ResurrectedForDomain).
    pub lifecycle_counts: [usize; 7],
    /// Rows at Unmeasured/Literature/Role with nonzero empirical
    /// fields (must be 0 at T.9 — the no-fabricated-claims gate).
    pub fabricated_claims: usize,
    /// True if `verify_usefulness_ledger` reports no errors.
    pub verifier_clean: bool,
}

/// All ten stable summary blocks the audit bundle exposes.
///
/// Why a single struct: testability. Tests assert specific
/// summary fields without re-parsing the TXT / JSON; the
/// `render_audit_report_*` functions are pure projections from
/// this struct so the two artifacts cannot disagree.
#[derive(Debug, Clone)]
pub struct AuditReportData {
    /// Canonical counts (literature primitives, alias claims,
    /// court subjects).
    pub counts: CorpusCounts,
    /// Per-`PrimitiveFamily` histogram (used as the source-class
    /// coverage indicator since the panel-named source classes
    /// map onto the existing family taxonomy).
    pub family_histogram: Vec<(PrimitiveFamily, usize)>,
    /// Court-decision summary.
    pub court: CourtSummary,
    /// Identity-hash summary.
    pub identity: IdentitySummary,
    /// Genealogy-graph summary.
    pub genealogy: GenealogySummary,
    /// Witness-role + fusion-plane summary.
    pub witness: WitnessSummary,
    /// L-band summary.
    pub lband: LBandSummary,
    /// Usefulness-ledger summary.
    pub usefulness: UsefulnessSummary,
}

/// Panel-locked limitations / non-claims block. Renders into
/// both TXT (Section 10) and JSON (`limitations` array).
pub const LIMITATIONS: &[&str] = &[
    "This T.9 audit report does not claim that all listed detectors are implemented.",
    "It does not claim that all detectors are useful.",
    "It does not claim that literature primitives are unique merely because names differ.",
    "It does not claim GPU execution for L0-L4 detectors.",
    "It does not claim corpus_hash_v1 or CaseFileV2 integration; those are deferred to T.10/T.11.",
    "It does not claim Atlas activation planning; that is deferred to T.11 / S1.2+.",
    "It is an internal audit artifact, not an external release artifact.",
];

/// Panel-locked deferred-gates block. Names the campaign
/// decisions that follow T.9.
pub const DEFERRED_GATES: &[&str] = &[
    "T.10 corpus_hash_v1 + CaseFileV2 integration",
    "T.11 GPU family-kernel mapping + Atlas activation",
    "R.9.d.2 D205 scaling ladder",
    "Section S Phase 1: Atlas algebra types + activation planner",
];

/// All four bundle artifacts as in-memory strings. The CLI
/// writes them to `reports/corpus_t9_*.{txt,json,dot}`.
#[derive(Debug, Clone)]
pub struct AuditReportBundle {
    /// Human-readable internal audit report.
    pub audit_report_txt: String,
    /// Machine-readable internal audit report.
    pub audit_report_json: String,
    /// Genealogy graph as Graphviz DOT (refresh of the T.5 export).
    pub genealogy_dot: String,
    /// Genealogy graph as flat-list JSON (refresh of the T.5 export).
    pub genealogy_json: String,
}

/// Compute every summary from the live T.1-T.8 sources.
#[must_use]
pub fn collect_audit_report_data() -> AuditReportData {
    let counts = CorpusCounts {
        literature_primitives: SEED.len(),
        alias_claims: crate::claims::CLAIMS.len(),
        court_subjects: SEED.len() + crate::claims::CLAIMS.len(),
    };

    let mut family_histogram: Vec<(PrimitiveFamily, usize)> = Vec::new();
    for r in SEED {
        if let Some(slot) = family_histogram
            .iter_mut()
            .find(|(f, _)| *f == r.primitive_family)
        {
            slot.1 += 1;
        } else {
            family_histogram.push((r.primitive_family, 1));
        }
    }

    let court_records = classify_all();
    let court = CourtSummary {
        counts: count_decisions(&court_records),
    };

    let identity = IdentitySummary {
        records_with_identity_hashes: SEED.len(),
    };

    let graph = build_genealogy();
    let dag_verified_clean = verify_genealogy(&graph).errors.is_empty();
    let origin_records = SEED.iter().filter(|r| r.genealogy.is_origin).count();
    let genealogy = GenealogySummary {
        nodes: graph.nodes.len(),
        edges: graph.edges.len(),
        origin_records,
        derived_records: SEED.len() - origin_records,
        dag_verified_clean,
    };

    let role_counts = compute_role_counts();
    let plane_counts = compute_plane_counts();
    let witness = WitnessSummary {
        role_counts,
        plane_counts,
    };

    let lband_hist = compute_lband_histogram(SEED);
    let lband_verifier = crate::lband::verify_corpus_lband(SEED);
    let lband = LBandSummary {
        counts: [
            lband_hist.l0,
            lband_hist.l1,
            lband_hist.l2,
            lband_hist.l3,
            lband_hist.l4,
            lband_hist.l5,
            lband_hist.l6,
            lband_hist.l7,
            lband_hist.l8,
        ],
        gpu_whitelist_size: GPU_IMPLEMENTED_CANONICAL_IDS.len(),
        verifier_clean: lband_verifier.is_clean(),
    };

    let ev = compute_evidence_histogram(USEFULNESS_LEDGER);
    let lc = compute_lifecycle_histogram(USEFULNESS_LEDGER);
    let fabricated_claims = USEFULNESS_LEDGER
        .iter()
        .filter(|r| {
            r.evidence_level.forbids_empirical_claims()
                && !UsefulnessLedgerRow::has_zero_empirical_fields(r)
        })
        .count();
    let usefulness_verifier = verify_usefulness_ledger(SEED, USEFULNESS_LEDGER);
    let usefulness = UsefulnessSummary {
        rows_loaded: USEFULNESS_LEDGER.len(),
        evidence_level_counts: [
            ev.unmeasured,
            ev.literature_prior,
            ev.role_seeded,
            ev.synthetic_fixture,
            ev.real_dataset,
            ev.cross_domain,
            ev.retired_by_evidence,
        ],
        lifecycle_counts: [
            lc.active,
            lc.dormant,
            lc.retired_redundant,
            lc.retired_high_fp,
            lc.retired_too_expensive,
            lc.quarantined,
            lc.resurrected,
        ],
        fabricated_claims,
        verifier_clean: usefulness_verifier.is_clean(),
    };

    AuditReportData {
        counts,
        family_histogram,
        court,
        identity,
        genealogy,
        witness,
        lband,
        usefulness,
    }
}

fn compute_role_counts() -> [usize; 10] {
    let roles: [WitnessRole; 10] = [
        WitnessRole::Primary,
        WitnessRole::Corroborating,
        WitnessRole::Confuser,
        WitnessRole::Boundary,
        WitnessRole::CleanWindow,
        WitnessRole::Recovery,
        WitnessRole::Timing,
        WitnessRole::Distribution,
        WitnessRole::Topology,
        WitnessRole::CausalityProxy,
    ];
    let mut counts = [0usize; 10];
    for (i, role) in roles.iter().enumerate() {
        counts[i] = SEED.iter().filter(|r| r.witness_role == *role).count();
    }
    counts
}

fn compute_plane_counts() -> [usize; 8] {
    let mut counts = [0usize; 8];
    for (i, plane) in FusionPlane::all().iter().enumerate() {
        counts[i] = SEED
            .iter()
            .filter(|r| axes_to_planes(r.fusion_axes).contains(*plane))
            .count();
    }
    counts
}

/// Build every bundle artifact from a single
/// [`AuditReportData`] snapshot so the four files cannot drift.
#[must_use]
pub fn generate_audit_report_bundle() -> AuditReportBundle {
    let data = collect_audit_report_data();
    let graph = build_genealogy();
    AuditReportBundle {
        audit_report_txt: render_audit_report_txt(&data),
        audit_report_json: render_audit_report_json(&data),
        genealogy_dot: export_dot(&graph),
        genealogy_json: export_json(&graph),
    }
}

// ===================================================================
// TXT renderer.
// ===================================================================

/// Render the audit report's human-readable mirror — the ten
/// stable top-level sections in canonical order. Two calls on the
/// same data produce byte-identical strings.
///
/// The function is intentionally one large section-by-section
/// renderer so a reviewer can read every block in source order
/// without jumping helpers. The `too_many_lines` clippy warning
/// is locally allowed for that reason.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_audit_report_txt(data: &AuditReportData) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "=== DSFB-GPU-Atlas Corpus Internal Audit Report ===");
    let _ = writeln!(
        out,
        "(T.9 internal audit artifact; cold-reader baseline for future T.10/T.11 work."
    );
    let _ = writeln!(
        out,
        " NOT a release artifact; corpus_hash_v1 and CaseFileV2 integration deferred.)"
    );
    let _ = writeln!(out);

    // (1) Corpus identity and release state.
    let _ = writeln!(out, "(1) Corpus identity and release state");
    let _ = writeln!(out, "  schema                : {AUDIT_REPORT_SCHEMA}");
    let _ = writeln!(out, "  generated by section  : T.9");
    let _ = writeln!(out, "  release stage         : {RELEASE_STAGE}");
    let _ = writeln!(out, "  corpus_hash_v1        : deferred until T.10");
    let _ = writeln!(out, "  CaseFileV2 link       : deferred until T.11");
    let _ = writeln!(out);

    // (2) Literature primitive count and source-class coverage.
    let _ = writeln!(
        out,
        "(2) Literature primitive count and source-class coverage"
    );
    let _ = writeln!(
        out,
        "  literature primitives (canonical) : {}",
        data.counts.literature_primitives
    );
    let _ = writeln!(
        out,
        "  alias / duplicate claims          : {}   (T.4 CLAIMS; counted separately)",
        data.counts.alias_claims
    );
    let _ = writeln!(
        out,
        "  court subjects (= primitives + claims) : {}",
        data.counts.court_subjects
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  Per-primitive-family coverage (source-class indicator):"
    );
    for (family, count) in &data.family_histogram {
        let _ = writeln!(out, "    {family:?} : {count}");
    }
    let _ = writeln!(out);

    // (3) Alias / duplicate / canonicalisation summary.
    let _ = writeln!(out, "(3) Alias / duplicate / canonicalisation summary");
    let _ = writeln!(
        out,
        "  Aliases are NOT counted as unique primitives. The dedup-court (T.4)"
    );
    let _ = writeln!(
        out,
        "  emits AliasOf / ParameterisationOf / CompositionOf decisions over the"
    );
    let _ = writeln!(
        out,
        "  alias claims; see (4) below for the authoritative tally."
    );
    let _ = writeln!(out);

    // (4) Deduplication court decision summary.
    let _ = writeln!(out, "(4) Deduplication court decision summary (T.4)");
    let _ = writeln!(
        out,
        "  source              : crate::court::classify_all over SEED + CLAIMS"
    );
    let _ = writeln!(
        out,
        "  court subjects             : {}",
        data.court.counts.total()
    );
    let _ = writeln!(
        out,
        "  canonical decisions        : {}",
        data.court.counts.canonical
    );
    let _ = writeln!(
        out,
        "  alias decisions            : {}",
        data.court.counts.aliases
    );
    let _ = writeln!(
        out,
        "  parameterisation decisions : {}",
        data.court.counts.parameterisations
    );
    let _ = writeln!(
        out,
        "  composition decisions      : {}",
        data.court.counts.compositions
    );
    let _ = writeln!(
        out,
        "  stochastic-reduction decs. : {}",
        data.court.counts.stochastic_reductions
    );
    let _ = writeln!(
        out,
        "  rejected records           : {}",
        data.court.counts.rejected
    );
    let _ = writeln!(
        out,
        "  deferred records           : {}",
        data.court.counts.deferred
    );
    let _ = writeln!(out);

    // (5) Detector identity hash summary.
    let _ = writeln!(out, "(5) Detector identity hash summary (T.3)");
    let _ = writeln!(
        out,
        "  records with five-hash identity : {}",
        data.identity.records_with_identity_hashes
    );
    let _ = writeln!(
        out,
        "  composite hash policy           : SHA256(domain || formula || parameter || semantic_role)"
    );
    let _ = writeln!(
        out,
        "  omitted from composite          : source_hash, implementation_hash"
    );
    let _ = writeln!(
        out,
        "    (so citation fixes and L-band upgrades do not break equivalence classes)"
    );
    let _ = writeln!(out);

    // (6) Genealogy graph summary.
    let _ = writeln!(out, "(6) Genealogy graph summary (T.5)");
    let _ = writeln!(
        out,
        "  nodes                     : {}",
        data.genealogy.nodes
    );
    let _ = writeln!(
        out,
        "  edges                     : {}",
        data.genealogy.edges
    );
    let _ = writeln!(
        out,
        "  origin records            : {}",
        data.genealogy.origin_records
    );
    let _ = writeln!(
        out,
        "  derived records           : {}",
        data.genealogy.derived_records
    );
    let _ = writeln!(
        out,
        "  DAG verified clean        : {}",
        data.genealogy.dag_verified_clean
    );
    let _ = writeln!(
        out,
        "  artifacts                 : reports/corpus_t9_genealogy.dot"
    );
    let _ = writeln!(
        out,
        "                              reports/corpus_t9_genealogy.json"
    );
    let _ = writeln!(out);

    // (7) Witness-role and fusion-plane coverage.
    let _ = writeln!(out, "(7) Witness-role and fusion-plane coverage (T.6)");
    let role_names = [
        "Primary",
        "Corroborating",
        "Confuser",
        "Boundary",
        "CleanWindow",
        "Recovery",
        "Timing",
        "Distribution",
        "Topology",
        "CausalityProxy",
    ];
    for (name, count) in role_names.iter().zip(data.witness.role_counts.iter()) {
        let _ = writeln!(out, "  role {name:<14} : {count}");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "  Fusion-plane membership counts:");
    for (plane, count) in FusionPlane::all()
        .iter()
        .zip(data.witness.plane_counts.iter())
    {
        let _ = writeln!(out, "    {name:<32} : {count}", name = plane.as_str());
    }
    let _ = writeln!(out);

    // (8) Implementation-status L-band histogram.
    let _ = writeln!(out, "(8) Implementation-status L-band histogram (T.7)");
    let _ = writeln!(
        out,
        "  (L-band is an honesty marker, not a quality score. L0-L4 are NOT GPU-ready;"
    );
    let _ = writeln!(
        out,
        "   L7/L8 are forbidden until benchmark/ledger evidence exists.)"
    );
    let lband_names = [
        "L0_CitedOnly",
        "L1_Canonicalised",
        "L2_DeterministicFormula",
        "L3_CpuImplemented",
        "L4_CpuVerified",
        "L5_GpuImplemented",
        "L6_CpuGpuByteEquivalent",
        "L7_BenchmarkCharacterised",
        "L8_LedgerCharacterised",
    ];
    for (name, count) in lband_names.iter().zip(data.lband.counts.iter()) {
        let suffix = match *name {
            "L7_BenchmarkCharacterised" | "L8_LedgerCharacterised" => "  (forbidden at T.9)",
            _ => "",
        };
        let _ = writeln!(out, "  {name:<28} : {count}{suffix}");
    }
    let _ = writeln!(
        out,
        "  GPU-implemented whitelist size : {}",
        data.lband.gpu_whitelist_size
    );
    let _ = writeln!(
        out,
        "  T.7 verifier clean             : {}",
        data.lband.verifier_clean
    );
    let _ = writeln!(out);

    // (9) Usefulness ledger honesty summary.
    let _ = writeln!(out, "(9) Usefulness ledger honesty summary (T.8)");
    let _ = writeln!(
        out,
        "  The usefulness ledger is an audit surface, not a learned"
    );
    let _ = writeln!(
        out,
        "  ranking model. No row is described as ranked or useful unless"
    );
    let _ = writeln!(
        out,
        "  it carries score_kind != NotScored. At T.9 every row is NotScored."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  rows loaded                       : {}",
        data.usefulness.rows_loaded
    );
    let ev_names = [
        "Unmeasured",
        "LiteraturePrior",
        "RoleSeeded",
        "SyntheticFixtureMeasured",
        "RealDatasetMeasured",
        "CrossDomainReplicated",
        "RetiredByEvidence",
    ];
    for (name, count) in ev_names
        .iter()
        .zip(data.usefulness.evidence_level_counts.iter())
    {
        let suffix = match *name {
            "SyntheticFixtureMeasured" => "  (forbidden without pinned fixture)",
            "RealDatasetMeasured" => "  (forbidden without hashed dataset)",
            "CrossDomainReplicated" => "  (forbidden without 2+ domain runs)",
            "RetiredByEvidence" => "  (forbidden without measured negative)",
            _ => "",
        };
        let _ = writeln!(out, "  {name:<28} : {count}{suffix}");
    }
    let _ = writeln!(
        out,
        "  no-fabricated-claims violators    : {}  (must be 0 at T.9)",
        data.usefulness.fabricated_claims
    );
    let _ = writeln!(
        out,
        "  T.8 verifier clean                : {}",
        data.usefulness.verifier_clean
    );
    let _ = writeln!(out);

    // (10) Limitations, non-claims, and deferred gates.
    let _ = writeln!(out, "(10) Limitations, non-claims, and deferred gates");
    for line in LIMITATIONS {
        let _ = writeln!(out, "  - {line}");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "  Deferred gates (campaign decisions following T.9):");
    for line in DEFERRED_GATES {
        let _ = writeln!(out, "    * {line}");
    }
    let _ = writeln!(out);

    out
}

// ===================================================================
// JSON renderer.
// ===================================================================

/// Render the audit report's machine-readable mirror as
/// deterministic JSON. Schema:
/// [`AUDIT_REPORT_SCHEMA`].
///
/// Field ordering is canonical and stable; no timestamps, no
/// floats. Two calls on the same data produce byte-identical
/// strings.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_audit_report_json(data: &AuditReportData) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"schema\": \"{AUDIT_REPORT_SCHEMA}\",");
    let _ = writeln!(out, "  \"generated_by_section\": \"T.9\",");

    let _ = writeln!(out, "  \"corpus_release_state\": {{");
    let _ = writeln!(out, "    \"stage\": \"{RELEASE_STAGE}\",");
    let _ = writeln!(out, "    \"corpus_hash_v1\": null,");
    let _ = writeln!(out, "    \"case_file_v2_link\": null");
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"counts\": {{");
    let _ = writeln!(
        out,
        "    \"literature_primitives\": {},",
        data.counts.literature_primitives
    );
    let _ = writeln!(out, "    \"alias_claims\": {},", data.counts.alias_claims);
    let _ = writeln!(
        out,
        "    \"court_subjects\": {}",
        data.counts.court_subjects
    );
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"primitive_family_histogram\": {{");
    for (i, (family, count)) in data.family_histogram.iter().enumerate() {
        let comma = if i + 1 < data.family_histogram.len() {
            ","
        } else {
            ""
        };
        let _ = writeln!(out, "    \"{family:?}\": {count}{comma}");
    }
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"court_summary\": {{");
    let _ = writeln!(out, "    \"canonical\": {},", data.court.counts.canonical);
    let _ = writeln!(out, "    \"aliases\": {},", data.court.counts.aliases);
    let _ = writeln!(
        out,
        "    \"parameterisations\": {},",
        data.court.counts.parameterisations
    );
    let _ = writeln!(
        out,
        "    \"compositions\": {},",
        data.court.counts.compositions
    );
    let _ = writeln!(
        out,
        "    \"stochastic_reductions\": {},",
        data.court.counts.stochastic_reductions
    );
    let _ = writeln!(out, "    \"rejected\": {},", data.court.counts.rejected);
    let _ = writeln!(out, "    \"deferred\": {},", data.court.counts.deferred);
    let _ = writeln!(out, "    \"total\": {}", data.court.counts.total());
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"identity_summary\": {{");
    let _ = writeln!(
        out,
        "    \"records_with_identity_hashes\": {},",
        data.identity.records_with_identity_hashes
    );
    let _ = writeln!(
        out,
        "    \"composite_hash_policy\": \"SHA256(domain || formula || parameter || semantic_role)\","
    );
    let _ = writeln!(
        out,
        "    \"omitted_from_composite\": [\"source_hash\", \"implementation_hash\"]"
    );
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"genealogy_summary\": {{");
    let _ = writeln!(out, "    \"nodes\": {},", data.genealogy.nodes);
    let _ = writeln!(out, "    \"edges\": {},", data.genealogy.edges);
    let _ = writeln!(
        out,
        "    \"origin_records\": {},",
        data.genealogy.origin_records
    );
    let _ = writeln!(
        out,
        "    \"derived_records\": {},",
        data.genealogy.derived_records
    );
    let _ = writeln!(
        out,
        "    \"dag_verified_clean\": {}",
        data.genealogy.dag_verified_clean
    );
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"witness_summary\": {{");
    let role_names = [
        "Primary",
        "Corroborating",
        "Confuser",
        "Boundary",
        "CleanWindow",
        "Recovery",
        "Timing",
        "Distribution",
        "Topology",
        "CausalityProxy",
    ];
    let _ = writeln!(out, "    \"role_counts\": {{");
    for (i, (name, count)) in role_names
        .iter()
        .zip(data.witness.role_counts.iter())
        .enumerate()
    {
        let comma = if i + 1 < role_names.len() { "," } else { "" };
        let _ = writeln!(out, "      \"{name}\": {count}{comma}");
    }
    let _ = writeln!(out, "    }},");
    let _ = writeln!(out, "    \"plane_counts\": {{");
    let planes = FusionPlane::all();
    for (i, (plane, count)) in planes
        .iter()
        .zip(data.witness.plane_counts.iter())
        .enumerate()
    {
        let comma = if i + 1 < planes.len() { "," } else { "" };
        let _ = writeln!(
            out,
            "      \"{name}\": {count}{comma}",
            name = plane.as_str()
        );
    }
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"lband_summary\": {{");
    let lband_names = [
        "L0_CitedOnly",
        "L1_Canonicalised",
        "L2_DeterministicFormula",
        "L3_CpuImplemented",
        "L4_CpuVerified",
        "L5_GpuImplemented",
        "L6_CpuGpuByteEquivalent",
        "L7_BenchmarkCharacterised",
        "L8_LedgerCharacterised",
    ];
    let _ = writeln!(out, "    \"counts\": {{");
    for (i, (name, count)) in lband_names.iter().zip(data.lband.counts.iter()).enumerate() {
        let comma = if i + 1 < lband_names.len() { "," } else { "" };
        let _ = writeln!(out, "      \"{name}\": {count}{comma}");
    }
    let _ = writeln!(out, "    }},");
    let _ = writeln!(
        out,
        "    \"gpu_whitelist_size\": {},",
        data.lband.gpu_whitelist_size
    );
    let _ = writeln!(out, "    \"verifier_clean\": {}", data.lband.verifier_clean);
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"usefulness_summary\": {{");
    let _ = writeln!(out, "    \"rows_loaded\": {},", data.usefulness.rows_loaded);
    let ev_names = [
        "Unmeasured",
        "LiteraturePrior",
        "RoleSeeded",
        "SyntheticFixtureMeasured",
        "RealDatasetMeasured",
        "CrossDomainReplicated",
        "RetiredByEvidence",
    ];
    let _ = writeln!(out, "    \"evidence_level_counts\": {{");
    for (i, (name, count)) in ev_names
        .iter()
        .zip(data.usefulness.evidence_level_counts.iter())
        .enumerate()
    {
        let comma = if i + 1 < ev_names.len() { "," } else { "" };
        let _ = writeln!(out, "      \"{name}\": {count}{comma}");
    }
    let _ = writeln!(out, "    }},");
    let lc_names = [
        "Active",
        "Dormant",
        "RetiredRedundant",
        "RetiredHighFalsePositive",
        "RetiredTooExpensive",
        "QuarantinedUnstable",
        "ResurrectedForDomain",
    ];
    let _ = writeln!(out, "    \"lifecycle_counts\": {{");
    for (i, (name, count)) in lc_names
        .iter()
        .zip(data.usefulness.lifecycle_counts.iter())
        .enumerate()
    {
        let comma = if i + 1 < lc_names.len() { "," } else { "" };
        let _ = writeln!(out, "      \"{name}\": {count}{comma}");
    }
    let _ = writeln!(out, "    }},");
    let _ = writeln!(
        out,
        "    \"fabricated_claims\": {},",
        data.usefulness.fabricated_claims
    );
    let _ = writeln!(
        out,
        "    \"verifier_clean\": {}",
        data.usefulness.verifier_clean
    );
    let _ = writeln!(out, "  }},");

    let _ = writeln!(out, "  \"limitations\": [");
    for (i, line) in LIMITATIONS.iter().enumerate() {
        let comma = if i + 1 < LIMITATIONS.len() { "," } else { "" };
        let _ = writeln!(out, "    \"{}\"{}", json_escape(line), comma);
    }
    let _ = writeln!(out, "  ],");

    let _ = writeln!(out, "  \"deferred_gates\": [");
    for (i, line) in DEFERRED_GATES.iter().enumerate() {
        let comma = if i + 1 < DEFERRED_GATES.len() {
            ","
        } else {
            ""
        };
        let _ = writeln!(out, "    \"{}\"{}", json_escape(line), comma);
    }
    let _ = writeln!(out, "  ]");

    let _ = writeln!(out, "}}");
    out
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            other => out.push(other),
        }
    }
    out
}

/// True if the rendered text contains a panel-forbidden
/// publication / deposit / Zenodo / DOI marker. T.9 must never
/// render any of these because the artifact is internal only.
/// The corpus_tN_regression_check.txt receipt is allowed to
/// MENTION "Zenodo" in the context of deferral; this check is for
/// the rendered TXT/JSON only.
#[must_use]
pub fn contains_publication_language(text: &str) -> bool {
    // Case-insensitive substring check for the forbidden tokens.
    let lower = text.to_lowercase();
    for tok in ["zenodo", "doi", "deposit", "publication-grade"] {
        if lower.contains(tok) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------
// Genealogy ancillary helpers — re-exported for the CLI.
// ---------------------------------------------------------------

/// Build a fresh genealogy graph (alias for
/// [`crate::genealogy::build_genealogy`]). T.9 bundles this so
/// the audit CLI doesn't need to import T.5 directly.
#[must_use]
pub fn build_audit_genealogy() -> GenealogyGraph {
    build_genealogy()
}

// Silence unused-import warnings for cross-module types only
// referenced through summaries.
#[allow(dead_code)]
const _LIFECYCLE_HANDLE: Option<LifecycleState> = None;
#[allow(dead_code)]
const _LBAND_HANDLE: Option<ImplementationLevel> = None;
#[allow(dead_code)]
const _USEFULNESS_LEVEL_HANDLE: Option<UsefulnessEvidenceLevel> = None;
