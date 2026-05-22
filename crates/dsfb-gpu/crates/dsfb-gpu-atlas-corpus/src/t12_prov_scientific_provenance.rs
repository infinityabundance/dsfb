//! T.12.PROV --- Scientific Provenance Credit Pass.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **T.12.PROV verifies that every T.12.a..T.12.p
//! > `CanonicalAddition` preserves visible scientific
//! > provenance: named scientist / author lineage,
//! > `SourceRef` linkage, contribution note, and DSFB-
//! > specific non-claim. It does not claim DSFB invented the
//! > underlying detector families. It records that
//! > DSFB-GPU-Atlas canonizes, deduplicates, contracts,
//! > activates, and maps prior detector science into a
//! > deterministic evidence court.**
//!
//! Core doctrine (panel-locked):
//!
//! > DSFB-GPU-Atlas does not erase prior detector science.
//! > It preserves named scientific lineage while converting
//! > detector primitives into deterministic, replayable
//! > witness records.
//!
//! ## Why
//!
//! Every T.12.a..T.12.p amendment proposal already carries
//! provenance fields in its existing schema:
//! [`crate::amendment::ProposedPrimitive::motivation`] names
//! the scientist or method family being canonicalised,
//! [`crate::amendment::ProposedSourceRef`] records the citation
//! key + title + year + venue, and
//! [`crate::amendment::ProposedDedupRecord::reason`] carries
//! the per-canonical court reason that names the scientist
//! again. What was missing was an explicit cross-proposal
//! verifier that every `CanonicalAddition` actually pins this
//! provenance.
//!
//! T.12.PROV is **derivation-only**: it walks the live
//! T.12.a..T.12.p proposals through
//! [`crate::consolidate::load_all_t12_proposals`], derives a
//! [`ScientistCredit`] row per `CanonicalAddition`, and emits
//! three artifacts:
//!
//! 1. The **scientist credit index** ([`ScientistCreditIndex`])
//!    --- one [`ScientistCredit`] per `CanonicalAddition`
//!    sorted ascending by canonical id.
//! 2. The **source bibliography index** ([`SourceBibliographyIndex`])
//!    --- one entry per unique `(citation_key, source_class)`
//!    pair sorted ascending by `(source_class, citation_key)`.
//! 3. The **provenance credit report** ([`ProvenanceCreditReport`])
//!    --- the top-level META-hash binding both indexes plus
//!    `corpus_hash_v1` and the panel-locked doctrine.
//!
//! Because T.12.PROV reads existing fields without mutating
//! them, every prior T.12.x proposal hash stays byte-
//! identical and no rebaseline is needed.
//!
//! ## Panel-locked non-claims
//!
//! T.12.PROV does NOT:
//!
//! - claim that DSFB-GPU-Atlas invented any of the detector
//!   primitives it canonizes;
//! - mutate any T.12.x / T.12.consolidate / FF.x / S1.3.x
//!   hash anchor;
//! - alter `SEED.len()` (stays at 54);
//! - change S1.3a / FF.2 / FF.3 / S1.3d / S1.3e / S1.3f /
//!   S1.3g court decisions;
//! - emit detector outputs, witness records, fusion tensors,
//!   candidate intervals, or episodes;
//! - generate CUDA kernels;
//! - decide contraindications or challenges;
//! - modify the registry crate.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes:
//!
//! - `scientist_credit_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:SCIENTIST-CREDIT-INDEX:v1\0`.
//! - `source_bibliography_index_hash_v1` under
//!   `DSFB-GPU-ATLAS:SOURCE-BIBLIOGRAPHY-INDEX:v1\0`.
//! - `provenance_credit_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROVENANCE-CREDIT-REPORT:v1\0`.
//!
//! ## Panel-locked verdict (one line)
//!
//! > The identity commit says what DSFB-GPU is; T.12.PROV
//! > makes sure the scientists whose methods became court
//! > witnesses are visibly credited.

use core::fmt::Write;
use std::collections::{BTreeMap, BTreeSet};

use dsfb_gpu_debug_core::sha256;

use crate::amendment::CorpusAmendmentProposal;
use crate::consolidate::load_all_t12_proposals;
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::seed::SEED;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `scientist_credit_index_hash_v1`.
pub const SCIENTIST_CREDIT_INDEX_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:SCIENTIST-CREDIT-INDEX:v1\0";
/// Schema identifier for `scientist_credit_index_hash_v1`.
pub const SCIENTIST_CREDIT_INDEX_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:SCIENTIST-CREDIT-INDEX:v1";

/// Domain separator for `source_bibliography_index_hash_v1`.
pub const SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:SOURCE-BIBLIOGRAPHY-INDEX:v1\0";
/// Schema identifier for `source_bibliography_index_hash_v1`.
pub const SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:SOURCE-BIBLIOGRAPHY-INDEX:v1";

/// Domain separator for `provenance_credit_report_hash_v1`.
pub const PROVENANCE_CREDIT_REPORT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:PROVENANCE-CREDIT-REPORT:v1\0";
/// Schema identifier for `provenance_credit_report_hash_v1`.
pub const PROVENANCE_CREDIT_REPORT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:PROVENANCE-CREDIT-REPORT:v1";

// ---------------------------------------------------------------
// Forbidden-substring sets (panel-locked)
// ---------------------------------------------------------------

/// Phrases a `CanonicalAddition` motivation or dedup-record
/// reason must NOT contain. Caught by the R.5 verifier rule
/// (`DsfbInventionClaimForPriorDetector`). The court's
/// position is: DSFB-GPU-Atlas canonizes prior detector
/// science; it does not claim invention.
const T12_PROV_FORBIDDEN_DSFB_INVENTION_SUBSTRINGS: &[&str] = &[
    "dsfb invented",
    "dsfb-gpu-atlas invented",
    "dsfb-gpu-debug invented",
    "we invented",
    "atlas invented",
    "originally introduced by dsfb",
    "first introduced by dsfb",
    "first proposed by dsfb",
];

/// The panel-locked DSFB-specific non-claim credit note that
/// accompanies every emitted [`ScientistCredit`] row. Stored
/// as a single `&'static str` constant so the bytes the
/// hash sees are pinned identical to the bytes the renderer
/// emits.
pub const T12_PROV_DSFB_CREDIT_NOTE: &str =
    "DSFB-GPU-Atlas canonizes, deduplicates, normalizes, contracts, and activates this detector primitive into a deterministic, replayable witness record. DSFB-GPU-Atlas does not claim invention of this primitive; named scientists and source papers above carry the original credit.";

// ---------------------------------------------------------------
// CanonicalAddition wire name (mirrors
// CanonicalisationDecision::CanonicalAddition.as_str())
// ---------------------------------------------------------------

const CANONICAL_ADDITION_WIRE_NAME: &str = "CanonicalAddition";
/// T.12.a-era legacy wire name. Treated as the same role as
/// `CanonicalAddition` for credit aggregation; surfaced in
/// the credit row's `contribution` field with the original
/// wire-name bytes.
const CANONICAL_T12A_HISTORICAL_WIRE_NAME: &str = "Canonical";
const PARAMETERIZATION_OF_WIRE_NAME: &str = "ParameterizationOf";
const REJECTED_WIRE_NAME: &str = "RejectedNotDeterministic";

fn is_canonical_addition(wire_name: &str) -> bool {
    wire_name == CANONICAL_ADDITION_WIRE_NAME || wire_name == CANONICAL_T12A_HISTORICAL_WIRE_NAME
}

// ---------------------------------------------------------------
// ScientistCredit row
// ---------------------------------------------------------------

/// One scientist-credit row derived from a single
/// `CanonicalAddition` dedup-record + its `ProposedPrimitive`
/// motivation + the batch's `ProposedSourceRef` list.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `scientist_credit_index_hash_v1`.
#[derive(Debug, Clone)]
pub struct ScientistCredit {
    /// Canonical id of the `CanonicalAddition`.
    pub canonical_id: u32,
    /// Operator-readable detector display name (mirrors the
    /// `ProposedPrimitive::display_name`).
    pub detector_name: &'static str,
    /// Wire name of the source class.
    pub source_class_wire_name: &'static str,
    /// Origin proposal id (e.g. `"t12_b_scd_proposal"`).
    pub origin_proposal_id: &'static str,
    /// Source ref citation keys associated with this
    /// canonical id (the verifier's R.4 rule requires every
    /// key here also appear in the batch's
    /// `proposed_source_refs` list).
    pub source_ref_keys: Vec<&'static str>,
    /// Contribution text --- the `ProposedDedupRecord::reason`
    /// for the canonical id. Non-empty (R.3 verifier rule).
    pub contribution: &'static str,
    /// Motivation text --- the `ProposedPrimitive::motivation`
    /// for the canonical id. Non-empty (R.1 rule via
    /// derivation: a `CanonicalAddition` without a backing
    /// `ProposedPrimitive` would never reach this row).
    pub motivation: &'static str,
    /// Panel-locked DSFB-specific non-claim credit note (the
    /// same [`T12_PROV_DSFB_CREDIT_NOTE`] constant for every
    /// row so the renderer + hash stay byte-stable).
    pub credit_note: &'static str,
}

// ---------------------------------------------------------------
// Source bibliography entry
// ---------------------------------------------------------------

/// One bibliography row derived from a unique
/// `(citation_key, source_class)` pair. Sorted ascending by
/// `(source_class_wire_name, citation_key)`.
#[derive(Debug, Clone)]
pub struct SourceBibliographyEntry {
    /// Citation key (mirrors `ProposedSourceRef::citation_key`).
    pub citation_key: &'static str,
    /// Publication title.
    pub title: &'static str,
    /// Publication year (0 for engineering-practice records;
    /// the R.6 verifier rule requires year-0 records carry a
    /// non-empty venue / provenance note).
    pub year: u16,
    /// Publication venue.
    pub venue: &'static str,
    /// Source class wire name.
    pub source_class_wire_name: &'static str,
    /// Origin proposal id this bibliography entry was first
    /// encountered in.
    pub origin_proposal_id: &'static str,
}

// ---------------------------------------------------------------
// Top-level scientist credit index
// ---------------------------------------------------------------

/// The full scientist credit index. One [`ScientistCredit`]
/// per `CanonicalAddition` across T.12.a..T.12.p, sorted
/// ascending by canonical id.
#[derive(Debug, Clone)]
pub struct ScientistCreditIndex {
    /// Sorted list of credits.
    pub credits: Vec<ScientistCredit>,
    /// Total `CanonicalAddition` count (mirrors
    /// `credits.len()`).
    pub canonical_addition_count: u32,
    /// `scientist_credit_index_hash_v1`.
    pub scientist_credit_index_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level source bibliography index
// ---------------------------------------------------------------

/// The full source bibliography index. One
/// [`SourceBibliographyEntry`] per unique
/// `(citation_key, source_class)` pair, sorted ascending by
/// `(source_class_wire_name, citation_key)`.
#[derive(Debug, Clone)]
pub struct SourceBibliographyIndex {
    /// Sorted list of bibliography entries.
    pub entries: Vec<SourceBibliographyEntry>,
    /// Total unique entry count.
    pub unique_entry_count: u32,
    /// `source_bibliography_index_hash_v1`.
    pub source_bibliography_index_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level provenance credit report
// ---------------------------------------------------------------

/// The top-level T.12.PROV artifact. Wraps the two indexes
/// plus the corpus authority anchors so one hash pins the
/// entire provenance credit pass.
#[derive(Debug, Clone)]
pub struct ProvenanceCreditReport {
    /// Wrapped scientist credit index.
    pub scientist_credit_index: ScientistCreditIndex,
    /// Wrapped source bibliography index.
    pub source_bibliography_index: SourceBibliographyIndex,
    /// Historical seed-corpus anchor.
    pub corpus_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Number of T.12.a..T.12.p proposals walked.
    pub proposal_count: u32,
    /// Number of rejection records seen across all proposals
    /// (the R.7 verifier rule requires every rejection record
    /// carries a non-empty method-family credit reason).
    pub rejection_record_count: u32,
    /// Number of parameterization records seen across all
    /// proposals (the R.8 verifier rule requires every
    /// parameterization record carries a parent lineage
    /// note in its reason text).
    pub parameterization_record_count: u32,
    /// `provenance_credit_report_hash_v1`.
    pub provenance_credit_report_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why T.12.PROV rejected a proposal-set. Eight panel-required
/// load-bearing negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum T12ProvVerifyErrorKind {
    /// Panel-required negative #1. A `CanonicalAddition`
    /// dedup record has no matching `ProposedPrimitive` in
    /// the same batch (which is where the motivation /
    /// scientist-credit text lives).
    CanonicalAdditionWithoutScientistCredit {
        /// Canonical id missing a credit.
        canonical_id: u32,
        /// Origin proposal id.
        origin_proposal_id: &'static str,
    },
    /// Panel-required negative #2. A `CanonicalAddition`
    /// belongs to a batch with an empty
    /// `proposed_source_refs` slice.
    CanonicalAdditionWithoutSourceRef {
        /// Canonical id without a source ref.
        canonical_id: u32,
        /// Origin proposal id.
        origin_proposal_id: &'static str,
    },
    /// Panel-required negative #3. A `CanonicalAddition`'s
    /// dedup-record reason text is empty.
    ScientistCreditWithoutContributionNote {
        /// Canonical id with empty contribution.
        canonical_id: u32,
    },
    /// Panel-required negative #4. A
    /// [`ScientistCredit::source_ref_keys`] entry names a
    /// citation key not present in the batch's
    /// `proposed_source_refs` list.
    SourceRefKeyNotInProposalSources {
        /// Canonical id naming the orphan key.
        canonical_id: u32,
        /// The orphan citation key.
        orphan_citation_key: &'static str,
    },
    /// Panel-required negative #5. A `CanonicalAddition`'s
    /// motivation or contribution text claims DSFB invention
    /// of a prior detector family.
    DsfbInventionClaimForPriorDetector {
        /// Canonical id carrying the forbidden text.
        canonical_id: u32,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #6. A `ProposedSourceRef` with
    /// `year = 0` (engineering practice) carries an empty
    /// `venue` (the only field left to record the provenance
    /// note).
    EngineeringPracticeRecordWithoutProvenanceNote {
        /// Citation key with the missing provenance note.
        citation_key: &'static str,
        /// Origin proposal id.
        origin_proposal_id: &'static str,
    },
    /// Panel-required negative #7. A `RejectedNotDeterministic`
    /// dedup record carries an empty reason (the reason must
    /// credit the method family being refused, even though
    /// the method is not admitted).
    RejectedRecordWithoutMethodFamilyCredit {
        /// Canonical id of the rejection shell.
        canonical_id: u32,
        /// Origin proposal id.
        origin_proposal_id: &'static str,
    },
    /// Panel-required negative #8. A `ParameterizationOf`
    /// dedup record carries an empty reason (the reason must
    /// name the parent canonical that the parameterization
    /// collapses into).
    ParameterizationWithoutParentLineageNote {
        /// Canonical id of the parameterization shell.
        canonical_id: u32,
        /// Origin proposal id.
        origin_proposal_id: &'static str,
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
    /// `corpus_hash_v1` pinned on the report does not equal
    /// the live `compute_corpus_hash_v1()` result.
    CorpusHashV1Mismatch {
        /// Hash the report claims.
        claimed: [u8; 32],
        /// Hash the live builder returns.
        actual: [u8; 32],
    },
    /// Scientist credit list is not sorted ascending by
    /// canonical id.
    ScientistCreditNotSortedAscending,
    /// Bibliography list is not sorted ascending by
    /// `(source_class_wire_name, citation_key)`.
    BibliographyNotSortedAscending,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T12ProvVerifyError {
    /// Error kind (see [`T12ProvVerifyErrorKind`]).
    pub kind: T12ProvVerifyErrorKind,
}

// ---------------------------------------------------------------
// Build helpers (derivation from live T.12.a..T.12.p proposals)
// ---------------------------------------------------------------

/// Build the production T.12.PROV scientist credit index from
/// the live T.12.a..T.12.p proposal set.
#[must_use]
pub fn build_scientist_credit_index() -> ScientistCreditIndex {
    let proposals = load_all_t12_proposals();
    build_scientist_credit_index_from(&proposals)
}

/// Build the scientist credit index from an injected
/// proposal list. Used by tests to inject mutated proposals.
#[must_use]
pub fn build_scientist_credit_index_from(
    proposals: &[CorpusAmendmentProposal],
) -> ScientistCreditIndex {
    let mut credits: Vec<ScientistCredit> = Vec::new();
    for proposal in proposals {
        // Skip the T.12.0 proof-of-life proposal: it has no
        // CanonicalAdditions and no source refs.
        if proposal.body.proposed_primitives.is_empty()
            && proposal.body.proposed_dedup_records.is_empty()
        {
            continue;
        }

        // Build the (canonical_id -> motivation) lookup from
        // ProposedPrimitive records.
        let prim_motivation_by_id: BTreeMap<u32, &'static str> = proposal
            .body
            .proposed_primitives
            .iter()
            .map(|p| (p.reserved_canonical_id.0, p.motivation))
            .collect();

        let prim_name_by_id: BTreeMap<u32, &'static str> = proposal
            .body
            .proposed_primitives
            .iter()
            .map(|p| (p.reserved_canonical_id.0, p.display_name))
            .collect();

        // Walk dedup records; for each CanonicalAddition
        // (post-T.12.b naming) or `Canonical` (T.12.a-era
        // legacy alias) emit a ScientistCredit row.
        for d in &proposal.body.proposed_dedup_records {
            if !is_canonical_addition(d.decision_wire_name) {
                continue;
            }
            let canonical_id = d.canonical_id.0;
            let motivation = prim_motivation_by_id
                .get(&canonical_id)
                .copied()
                .unwrap_or("");
            let detector_name = prim_name_by_id.get(&canonical_id).copied().unwrap_or("");
            let source_ref_keys: Vec<&'static str> = proposal
                .body
                .proposed_source_refs
                .iter()
                .map(|r| r.citation_key)
                .collect();
            credits.push(ScientistCredit {
                canonical_id,
                detector_name,
                source_class_wire_name: proposal.target_source_class.as_str(),
                origin_proposal_id: proposal.proposal_id,
                source_ref_keys,
                contribution: d.reason,
                motivation,
                credit_note: T12_PROV_DSFB_CREDIT_NOTE,
            });
        }
    }

    credits.sort_by_key(|c| c.canonical_id);
    let canonical_addition_count = u32::try_from(credits.len()).unwrap_or(u32::MAX);
    let scientist_credit_index_hash_v1 = compute_scientist_credit_index_hash(&credits);
    ScientistCreditIndex {
        credits,
        canonical_addition_count,
        scientist_credit_index_hash_v1,
    }
}

/// Build the production T.12.PROV source bibliography index
/// from the live T.12.a..T.12.p proposal set.
#[must_use]
pub fn build_source_bibliography_index() -> SourceBibliographyIndex {
    let proposals = load_all_t12_proposals();
    build_source_bibliography_index_from(&proposals)
}

/// Build the source bibliography index from an injected
/// proposal list.
#[must_use]
pub fn build_source_bibliography_index_from(
    proposals: &[CorpusAmendmentProposal],
) -> SourceBibliographyIndex {
    // Deduplicate by (source_class_wire_name, citation_key).
    // The first proposal an entry is encountered in wins the
    // origin_proposal_id slot; later duplicates with the
    // same (class, key) are absorbed silently.
    let mut by_key: BTreeMap<(&'static str, &'static str), SourceBibliographyEntry> =
        BTreeMap::new();
    for proposal in proposals {
        for r in &proposal.body.proposed_source_refs {
            let key = (proposal.target_source_class.as_str(), r.citation_key);
            by_key
                .entry(key)
                .or_insert_with(|| SourceBibliographyEntry {
                    citation_key: r.citation_key,
                    title: r.title,
                    year: r.year,
                    venue: r.venue,
                    source_class_wire_name: proposal.target_source_class.as_str(),
                    origin_proposal_id: proposal.proposal_id,
                });
        }
    }
    let entries: Vec<SourceBibliographyEntry> = by_key.into_values().collect();
    let unique_entry_count = u32::try_from(entries.len()).unwrap_or(u32::MAX);
    let source_bibliography_index_hash_v1 = compute_source_bibliography_index_hash(&entries);
    SourceBibliographyIndex {
        entries,
        unique_entry_count,
        source_bibliography_index_hash_v1,
    }
}

/// Build the production T.12.PROV provenance credit report.
#[must_use]
pub fn build_provenance_credit_report() -> ProvenanceCreditReport {
    let proposals = load_all_t12_proposals();
    build_provenance_credit_report_from(&proposals)
}

/// Build the provenance credit report from an injected
/// proposal list.
#[must_use]
pub fn build_provenance_credit_report_from(
    proposals: &[CorpusAmendmentProposal],
) -> ProvenanceCreditReport {
    let scientist_credit_index = build_scientist_credit_index_from(proposals);
    let source_bibliography_index = build_source_bibliography_index_from(proposals);
    let mut rejection_record_count: u32 = 0;
    let mut parameterization_record_count: u32 = 0;
    for proposal in proposals {
        for d in &proposal.body.proposed_dedup_records {
            match d.decision_wire_name {
                REJECTED_WIRE_NAME => {
                    rejection_record_count = rejection_record_count.saturating_add(1);
                }
                PARAMETERIZATION_OF_WIRE_NAME => {
                    parameterization_record_count = parameterization_record_count.saturating_add(1);
                }
                _ => {}
            }
        }
    }
    let mut report = ProvenanceCreditReport {
        scientist_credit_index,
        source_bibliography_index,
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        seed_len: u32::try_from(SEED.len()).unwrap_or(u32::MAX),
        proposal_count: u32::try_from(proposals.len()).unwrap_or(u32::MAX),
        rejection_record_count,
        parameterization_record_count,
        provenance_credit_report_hash_v1: [0u8; 32],
    };
    report.provenance_credit_report_hash_v1 = compute_report_hash(&report);
    report
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_scientist_credit_index_hash(credits: &[ScientistCredit]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(SCIENTIST_CREDIT_INDEX_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(SCIENTIST_CREDIT_INDEX_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(
        &u32::try_from(credits.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for c in credits {
        buf.push(0x1e);
        buf.extend_from_slice(&c.canonical_id.to_be_bytes());
        push_len_prefixed(&mut buf, c.detector_name.as_bytes());
        push_len_prefixed(&mut buf, c.source_class_wire_name.as_bytes());
        push_len_prefixed(&mut buf, c.origin_proposal_id.as_bytes());
        push_str_slice(&mut buf, &c.source_ref_keys);
        push_len_prefixed(&mut buf, c.contribution.as_bytes());
        push_len_prefixed(&mut buf, c.motivation.as_bytes());
        push_len_prefixed(&mut buf, c.credit_note.as_bytes());
    }
    sha256(&buf)
}

fn compute_source_bibliography_index_hash(entries: &[SourceBibliographyEntry]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(
        &u32::try_from(entries.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for e in entries {
        buf.push(0x1e);
        push_len_prefixed(&mut buf, e.citation_key.as_bytes());
        push_len_prefixed(&mut buf, e.title.as_bytes());
        buf.extend_from_slice(&e.year.to_be_bytes());
        push_len_prefixed(&mut buf, e.venue.as_bytes());
        push_len_prefixed(&mut buf, e.source_class_wire_name.as_bytes());
        push_len_prefixed(&mut buf, e.origin_proposal_id.as_bytes());
    }
    sha256(&buf)
}

fn compute_report_hash(r: &ProvenanceCreditReport) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(PROVENANCE_CREDIT_REPORT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(PROVENANCE_CREDIT_REPORT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&r.corpus_hash_v1);
    buf.extend_from_slice(&r.seed_len.to_be_bytes());
    buf.extend_from_slice(&r.proposal_count.to_be_bytes());
    buf.extend_from_slice(&r.scientist_credit_index.scientist_credit_index_hash_v1);
    buf.extend_from_slice(
        &r.source_bibliography_index
            .source_bibliography_index_hash_v1,
    );
    buf.extend_from_slice(&r.rejection_record_count.to_be_bytes());
    buf.extend_from_slice(&r.parameterization_record_count.to_be_bytes());
    push_len_prefixed(&mut buf, T12_PROV_DSFB_CREDIT_NOTE.as_bytes());
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn push_str_slice(buf: &mut Vec<u8>, slice: &[&str]) {
    let len = u32::try_from(slice.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    for s in slice {
        push_len_prefixed(buf, s.as_bytes());
    }
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify a T.12.PROV provenance credit report against the
/// injected proposal set. Returns a vector of errors (empty
/// when the report satisfies every panel-required +
/// structural rule).
#[must_use]
#[allow(clippy::too_many_lines)] // 12 rules; splitting would obscure panel numbering
pub fn verify_t12_prov(
    report: &ProvenanceCreditReport,
    proposals: &[CorpusAmendmentProposal],
) -> Vec<T12ProvVerifyError> {
    let mut errors: Vec<T12ProvVerifyError> = Vec::new();

    // Walk every proposal to enforce R.1, R.2, R.6, R.7, R.8.
    for proposal in proposals {
        let prim_id_set: BTreeSet<u32> = proposal
            .body
            .proposed_primitives
            .iter()
            .map(|p| p.reserved_canonical_id.0)
            .collect();
        let source_refs = &proposal.body.proposed_source_refs;
        let batch_has_source_ref = !source_refs.is_empty();

        for d in &proposal.body.proposed_dedup_records {
            if is_canonical_addition(d.decision_wire_name) {
                let canonical_id = d.canonical_id.0;

                // R.1 CanonicalAdditionWithoutScientistCredit.
                if !prim_id_set.contains(&canonical_id) {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::CanonicalAdditionWithoutScientistCredit {
                            canonical_id,
                            origin_proposal_id: proposal.proposal_id,
                        },
                    });
                }

                // R.2 CanonicalAdditionWithoutSourceRef.
                if !batch_has_source_ref {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::CanonicalAdditionWithoutSourceRef {
                            canonical_id,
                            origin_proposal_id: proposal.proposal_id,
                        },
                    });
                }

                // R.3 ScientistCreditWithoutContributionNote.
                if d.reason.is_empty() {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::ScientistCreditWithoutContributionNote {
                            canonical_id,
                        },
                    });
                }

                // R.5 DsfbInventionClaimForPriorDetector.
                for &forbidden in T12_PROV_FORBIDDEN_DSFB_INVENTION_SUBSTRINGS {
                    if contains_ascii_case_insensitive(d.reason, forbidden) {
                        errors.push(T12ProvVerifyError {
                            kind: T12ProvVerifyErrorKind::DsfbInventionClaimForPriorDetector {
                                canonical_id,
                                forbidden_substring: forbidden,
                            },
                        });
                    }
                }
            } else if d.decision_wire_name == REJECTED_WIRE_NAME {
                // R.7 RejectedRecordWithoutMethodFamilyCredit.
                if d.reason.is_empty() {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::RejectedRecordWithoutMethodFamilyCredit {
                            canonical_id: d.canonical_id.0,
                            origin_proposal_id: proposal.proposal_id,
                        },
                    });
                }
            } else if d.decision_wire_name == PARAMETERIZATION_OF_WIRE_NAME {
                // R.8 ParameterizationWithoutParentLineageNote.
                if d.reason.is_empty() {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::ParameterizationWithoutParentLineageNote {
                            canonical_id: d.canonical_id.0,
                            origin_proposal_id: proposal.proposal_id,
                        },
                    });
                }
            }
        }

        // R.6 EngineeringPracticeRecordWithoutProvenanceNote.
        for r in source_refs {
            if r.year == 0 && r.venue.is_empty() {
                errors.push(T12ProvVerifyError {
                    kind: T12ProvVerifyErrorKind::EngineeringPracticeRecordWithoutProvenanceNote {
                        citation_key: r.citation_key,
                        origin_proposal_id: proposal.proposal_id,
                    },
                });
            }
        }
    }

    // R.4 SourceRefKeyNotInProposalSources.
    //
    // For each credit row, every key in `source_ref_keys`
    // must appear in the originating proposal's
    // `proposed_source_refs` list.
    let proposal_source_keys: BTreeMap<&'static str, BTreeSet<&'static str>> = proposals
        .iter()
        .map(|p| {
            (
                p.proposal_id,
                p.body
                    .proposed_source_refs
                    .iter()
                    .map(|r| r.citation_key)
                    .collect(),
            )
        })
        .collect();
    for c in &report.scientist_credit_index.credits {
        if let Some(known) = proposal_source_keys.get(c.origin_proposal_id) {
            for &key in &c.source_ref_keys {
                if !known.contains(key) {
                    errors.push(T12ProvVerifyError {
                        kind: T12ProvVerifyErrorKind::SourceRefKeyNotInProposalSources {
                            canonical_id: c.canonical_id,
                            orphan_citation_key: key,
                        },
                    });
                }
            }
        }
    }

    // R.5 DsfbInventionClaimForPriorDetector --- also scan
    // the motivation text on each credit row (the loop above
    // already scanned the reason text via the dedup records).
    for c in &report.scientist_credit_index.credits {
        for &forbidden in T12_PROV_FORBIDDEN_DSFB_INVENTION_SUBSTRINGS {
            if contains_ascii_case_insensitive(c.motivation, forbidden) {
                errors.push(T12ProvVerifyError {
                    kind: T12ProvVerifyErrorKind::DsfbInventionClaimForPriorDetector {
                        canonical_id: c.canonical_id,
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
    }

    // Structural defects.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if report.corpus_hash_v1 != live_v1 {
        errors.push(T12ProvVerifyError {
            kind: T12ProvVerifyErrorKind::CorpusHashV1Mismatch {
                claimed: report.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(T12ProvVerifyError {
            kind: T12ProvVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }
    for w in report.scientist_credit_index.credits.windows(2) {
        if w[0].canonical_id > w[1].canonical_id {
            errors.push(T12ProvVerifyError {
                kind: T12ProvVerifyErrorKind::ScientistCreditNotSortedAscending,
            });
            break;
        }
    }
    for w in report.source_bibliography_index.entries.windows(2) {
        let a = (w[0].source_class_wire_name, w[0].citation_key);
        let b = (w[1].source_class_wire_name, w[1].citation_key);
        if a > b {
            errors.push(T12ProvVerifyError {
                kind: T12ProvVerifyErrorKind::BibliographyNotSortedAscending,
            });
            break;
        }
    }

    errors
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    for window_start in 0..=h.len() - n.len() {
        let mut ok = true;
        for i in 0..n.len() {
            if !h[window_start + i].eq_ignore_ascii_case(&n[i]) {
                ok = false;
                break;
            }
        }
        if ok {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the provenance credit report as deterministic text.
#[must_use]
pub fn render_provenance_credit_report_text(r: &ProvenanceCreditReport) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "T.12.PROV Provenance Credit Report (v1)");
    let _ = writeln!(s, "=======================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned corpus anchors");
    let _ = writeln!(s, "  corpus_hash_v1 : {}", hex32(&r.corpus_hash_v1));
    let _ = writeln!(s, "  SEED.len()     : {}", r.seed_len);
    let _ = writeln!(s);
    let _ = writeln!(s, "Walk shape");
    let _ = writeln!(
        s,
        "  proposal_count                    : {}",
        r.proposal_count
    );
    let _ = writeln!(
        s,
        "  canonical_addition_count          : {}",
        r.scientist_credit_index.canonical_addition_count
    );
    let _ = writeln!(
        s,
        "  unique_bibliography_entry_count   : {}",
        r.source_bibliography_index.unique_entry_count
    );
    let _ = writeln!(
        s,
        "  rejection_record_count            : {}",
        r.rejection_record_count
    );
    let _ = writeln!(
        s,
        "  parameterization_record_count     : {}",
        r.parameterization_record_count
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Component hashes");
    let _ = writeln!(
        s,
        "  scientist_credit_index_hash_v1    : {}",
        hex32(&r.scientist_credit_index.scientist_credit_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "  source_bibliography_index_hash_v1 : {}",
        hex32(
            &r.source_bibliography_index
                .source_bibliography_index_hash_v1
        )
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked DSFB credit note");
    let _ = writeln!(s, "  {T12_PROV_DSFB_CREDIT_NOTE}");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "provenance_credit_report_hash_v1 : {}",
        hex32(&r.provenance_credit_report_hash_v1)
    );
    s
}

/// Render the scientist credit index as deterministic text.
/// Prints one block per credit row in canonical order.
#[must_use]
#[allow(clippy::too_many_lines)] // wide schema; one writeln per field
pub fn render_scientist_credit_index_text(idx: &ScientistCreditIndex) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "T.12.PROV Scientist Credit Index (v1)");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "canonical_addition_count : {}",
        idx.canonical_addition_count
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "scientist_credit_index_hash_v1 : {}",
        hex32(&idx.scientist_credit_index_hash_v1)
    );
    s
}

/// Render the source bibliography index as deterministic text.
#[must_use]
pub fn render_source_bibliography_index_text(idx: &SourceBibliographyIndex) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "T.12.PROV Source Bibliography Index (v1)");
    let _ = writeln!(s, "========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "unique_entry_count : {}", idx.unique_entry_count);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "source_bibliography_index_hash_v1 : {}",
        hex32(&idx.source_bibliography_index_hash_v1)
    );
    s
}

/// Render the provenance credit report as canonical JSON.
#[must_use]
pub fn render_provenance_credit_report_json(r: &ProvenanceCreditReport) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", PROVENANCE_CREDIT_REPORT_SCHEMA_V1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v1", &r.corpus_hash_v1);
    s.push(',');
    let _ = write!(s, "\"seed_len\":{}", r.seed_len);
    s.push(',');
    let _ = write!(s, "\"proposal_count\":{}", r.proposal_count);
    s.push(',');
    let _ = write!(
        s,
        "\"canonical_addition_count\":{}",
        r.scientist_credit_index.canonical_addition_count
    );
    s.push(',');
    let _ = write!(
        s,
        "\"unique_bibliography_entry_count\":{}",
        r.source_bibliography_index.unique_entry_count
    );
    s.push(',');
    let _ = write!(s, "\"rejection_record_count\":{}", r.rejection_record_count);
    s.push(',');
    let _ = write!(
        s,
        "\"parameterization_record_count\":{}",
        r.parameterization_record_count
    );
    s.push(',');
    json_hex(
        &mut s,
        "scientist_credit_index_hash_v1",
        &r.scientist_credit_index.scientist_credit_index_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "source_bibliography_index_hash_v1",
        &r.source_bibliography_index
            .source_bibliography_index_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "provenance_credit_report_hash_v1",
        &r.provenance_credit_report_hash_v1,
    );
    s.push('}');
    s
}

/// Render the scientist credit index as canonical JSON.
#[must_use]
pub fn render_scientist_credit_index_json(idx: &ScientistCreditIndex) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", SCIENTIST_CREDIT_INDEX_SCHEMA_V1);
    s.push(',');
    let _ = write!(
        s,
        "\"canonical_addition_count\":{}",
        idx.canonical_addition_count
    );
    s.push(',');
    json_hex(
        &mut s,
        "scientist_credit_index_hash_v1",
        &idx.scientist_credit_index_hash_v1,
    );
    s.push('}');
    s
}

/// Render the source bibliography index as canonical JSON.
#[must_use]
pub fn render_source_bibliography_index_json(idx: &SourceBibliographyIndex) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1);
    s.push(',');
    let _ = write!(s, "\"unique_entry_count\":{}", idx.unique_entry_count);
    s.push(',');
    json_hex(
        &mut s,
        "source_bibliography_index_hash_v1",
        &idx.source_bibliography_index_hash_v1,
    );
    s.push('}');
    s
}

fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_hex(s: &mut String, key: &str, value: &[u8; 32]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    let _ = s.write_str(&hex32(value));
    s.push('"');
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Read-only access to the panel-locked forbidden-substring
/// set for tests.
#[doc(hidden)]
#[must_use]
pub fn forbidden_dsfb_invention_substrings() -> &'static [&'static str] {
    T12_PROV_FORBIDDEN_DSFB_INVENTION_SUBSTRINGS
}
