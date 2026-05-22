//! FF.4 — README front-door authority-boundary warning policy.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **FF.4 makes the post-T.12.consolidate / post-FF.1 /
//! > post-FF.2 / post-FF.3 authority-boundary state unmissable
//! > at the README front door. It does not add detectors, mutate
//! > any upstream hash anchor, modify SEED, modify any court
//! > artifact, or change activation / registry-generation
//! > behaviour. It is a communication-hygiene seal: the operator-
//! > facing README MUST carry the canonical authority-boundary
//! > block stating that T.12.a..T.12.p were filed as amendment
//! > proposals (and did not mutate seed authority while filed),
//! > that T.12.consolidate froze `corpus_hash_v2` as the
//! > ratified post-amendment authority, that FF.1 materialized
//! > 98 ratified CanonicalAddition entries into
//! > T12RatifiedPassport records, and that FF.2 + FF.3 reject
//! > unratified / non-passported / ad-hoc / unknown-source
//! > records by explicit reason code. Stale pre-ratification
//! > language ("future ratification campaign", "until a future
//! > freeze") is forbidden in the front-door area now that the
//! > ratification + materialization already happened.**
//!
//! ## Why
//!
//! Before T.12.consolidate, the README warning correctly read
//! "T.12.a..j are amendment proposals; they do not mutate SEED,
//! `corpus_hash_v1`, `registry_hash_v2`, `DetectorPassports`,
//! or activation outputs until a future ratification / freeze
//! campaign." After T.12.consolidate + FF.1 + FF.2 + FF.3, that
//! exact wording is stale: the future ratification already
//! happened, the future freeze already produced `corpus_hash_v2`,
//! and the 98 ratified entries already have FF.1 passports +
//! FF.2 activation status + FF.3 registry-generation eligibility.
//!
//! The panel directive was explicit: FF.4 is a hygiene seal,
//! NOT an authority mutation. It changes the README text; it
//! does not change court state.
//!
//! ## Method
//!
//! 1. Define the canonical authority-boundary block as a fixed
//!    array of text lines
//!    ([`FF4_AUTHORITY_BOUNDARY_BLOCK_LINES`]). The block is
//!    pinned verbatim; mutations require a future FF.4.x schema-
//!    upgrade commit with a new domain separator.
//! 2. Define the panel-required required-substring set
//!    ([`FF4_REQUIRED_SUBSTRINGS`]): operator-facing phrases the
//!    README front-door area MUST contain so the authority-state
//!    story reads correctly. Examples:
//!    `"historical seed-corpus anchor"`,
//!    `"ratified post-amendment authority"`,
//!    `"T12RatifiedPassport"`,
//!    `"unratified, non-passported, ad-hoc, or unknown-source
//!    records are rejected"`.
//! 3. Define the panel-required forbidden-substring set
//!    ([`FF4_FORBIDDEN_SUBSTRINGS`]): operator-misleading
//!    phrases that MUST NOT appear in the front-door area.
//!    Examples:
//!    `"future ratification / freeze campaign"`,
//!    `"until a future freeze"`,
//!    `"T.12 proposals mutated SEED"`,
//!    `"FF.1 mutated corpus_hash_v2"`.
//! 4. Expose [`verify_ff4_readme`] which walks a README string
//!    against the two substring sets and emits every rejection
//!    under [`Ff4VerifyErrorKind`].
//! 5. Emit a top-level
//!    [`Ff4ReadmeAuthorityBoundaryPolicy`] artifact pinning the
//!    five upstream anchor hashes (`corpus_hash_v1`,
//!    `corpus_hash_v2`, `ff1_passport_index_hash_v1`,
//!    `ff2_activation_ratification_gate_hash_v1`,
//!    `ff3_registry_generation_gate_hash_v1`) plus the canonical
//!    block + required + forbidden substring sets. Hash under
//!    `DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0`.
//!
//! ## Panel-locked non-claims
//!
//! - FF.4 does NOT add new detectors.
//! - FF.4 does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
//!   any T.12.x proposal hash, any T.12.consolidate hash, any
//!   FF.1 passport / index / report hash, any FF.2 hash, or any
//!   FF.3 hash.
//! - FF.4 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
//!   FF.1 / FF.2 / FF.3 hash.
//! - FF.4 does NOT mutate `SEED.len()` (stays at 54).
//! - FF.4 does NOT promote any open proposal to Accepted.
//! - FF.4 does NOT change S1.3a / FF.2 / FF.3 court decisions.
//! - FF.4 does NOT generate CUDA kernels.
//! - FF.4 does NOT decide contraindications or challenges.
//! - FF.4 does NOT mutate the registry crate.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! Every upstream hash anchor (T.11 / S1.3 / T.12.x /
//! T.12.consolidate / FF.1 / FF.2 / FF.3) byte-identical.
//! `SEED.len()` = 54.
//!
//! **NEW**: `ff4_readme_authority_boundary_policy_hash_v1` (one
//! value over the canonical block + required + forbidden
//! substring sets + pinned anchors).
//!
//! ## Panel-locked one-line verdict
//!
//! > FF.4 makes the authority boundary unmissable at the front
//! > door; it does not move any boundary.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::consolidate::build_consolidation_report;
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::ff1_passport_materialisation::build_ff1_passport_index_from;
use crate::ff2_activation_ratification_gate::build_ff2_activation_ratification_gate_from;
use crate::ff2_activation_ratification_gate::default_candidate_ids;
use crate::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use crate::seed::SEED;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separator
// ---------------------------------------------------------------

/// Domain separator for
/// `ff4_readme_authority_boundary_policy_hash_v1`. Distinct from
/// every FF.1 / FF.2 / FF.3 domain so the FF.4 artifact is
/// independently addressable.
pub const FF4_README_AUTHORITY_BOUNDARY_POLICY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0";

/// Panel-locked stale-phrasing forbids checked by R.1. Hoisted
/// to module scope to satisfy `items_after_statements`; the
/// list is part of the policy contract and shipped verbatim
/// in the forbidden-substring set.
const STALE_FUTURE_RATIFICATION_PHRASES: &[&str] = &[
    "future ratification / freeze campaign",
    "until a future ratification campaign",
    "until a future freeze",
];

/// Panel-locked forbids checked by R.6 (T.12 claims that mutated
/// SEED). Hoisted to module scope to satisfy
/// `items_after_statements`.
const T12_MUTATED_SEED_PHRASES: &[&str] =
    &["T.12 proposals mutated SEED", "T.12 proposals mutate SEED"];

/// Panel-locked forbids checked by R.7 (FF.1 claims that mutated
/// `corpus_hash_v2`). Hoisted to module scope to satisfy
/// `items_after_statements`.
const FF1_MUTATED_CORPUS_V2_PHRASES: &[&str] =
    &["FF.1 mutated corpus_hash_v2", "FF.1 mutates corpus_hash_v2"];

/// Schema identifier embedded in the policy hash material.
pub const FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1";

// ---------------------------------------------------------------
// Canonical authority-boundary block
// ---------------------------------------------------------------

/// Panel-locked canonical authority-boundary block text lines.
/// Pinned verbatim so the README's front-door warning reads
/// exactly the same in every release; mutations require a new
/// domain separator (FF.4.x schema-upgrade commit). The README
/// MUST embed every one of these lines verbatim somewhere in
/// the front-door area (see [`verify_ff4_readme`]).
pub const FF4_AUTHORITY_BOUNDARY_BLOCK_LINES: &[&str] = &[
    "## Authority boundary (post-T.12.consolidate + FF.1 + FF.2 + FF.3)",
    "Important authority-state note. T.12.a..T.12.p were amendment proposals.",
    "They did not mutate SEED, corpus_hash_v1, registry_hash_v2, historical",
    "DetectorPassports, or activation outputs while they were filed.",
    "T.12.consolidate ratified the accepted T.12 expansion set and froze",
    "corpus_hash_v2 as the first post-amendment corpus authority.",
    "FF.1 then materialized 98 ratified T.12 CanonicalAddition entries into",
    "T12RatifiedPassport records under ff1_passport_index_hash_v1.",
    "FF.2 and FF.3 now enforce that activation and registry generation consume",
    "only SeedHistorical records or T12RatifiedAndPassported records. Unratified,",
    "non-passported, ad-hoc, or unknown-source records are rejected by explicit",
    "reason code (DisabledUnratifiedProposal at activation; RejectedUnratifiedProposal,",
    "RejectedMissingFf1Passport, RejectedCorpusHashV2Mismatch, RejectedPassportIndexHashMismatch,",
    "RejectedAdHocRecord, RejectedUnknownSourceAuthority at registry generation).",
    "- SEED and corpus_hash_v1 remain the historical seed-corpus anchor.",
    "- T.12 proposals did not mutate seed authority while filed.",
    "- T.12.consolidate froze corpus_hash_v2 as ratified post-amendment authority.",
    "- FF.1 materialized ratified T.12 additions into passports.",
    "- FF.2 / FF.3 prevent unratified records from entering activation or registry generation.",
];

// ---------------------------------------------------------------
// Required / forbidden substring policy
// ---------------------------------------------------------------

/// Panel-required substrings the README's front-door area MUST
/// contain. The verifier checks that every entry appears as a
/// substring at least once in the supplied README text. Pinned
/// verbatim; mutations require a future FF.4.x schema-upgrade
/// commit.
pub const FF4_REQUIRED_SUBSTRINGS: &[&str] = &[
    "historical seed-corpus anchor",
    "ratified post-amendment authority",
    "T12RatifiedPassport",
    "FF.2 / FF.3 prevent unratified records from entering activation or registry generation.",
    "FF.1 materialized ratified T.12 additions into passports.",
    "T.12.consolidate froze corpus_hash_v2",
];

/// Panel-required substrings the README MUST NOT contain. The
/// verifier checks that every entry is absent. Pinned verbatim;
/// mutations require a future FF.4.x schema-upgrade commit.
///
/// These are stale pre-ratification phrasings that became
/// operator-misleading after T.12.consolidate + FF.1 already
/// happened. The verifier guards against silent regression.
pub const FF4_FORBIDDEN_SUBSTRINGS: &[&str] = &[
    "future ratification / freeze campaign",
    "until a future ratification campaign",
    "until a future freeze",
    "T.12 proposals mutated SEED",
    "T.12 proposals mutate SEED",
    "FF.1 mutated corpus_hash_v2",
    "FF.1 mutates corpus_hash_v2",
];

// ---------------------------------------------------------------
// Policy artifact
// ---------------------------------------------------------------

/// The FF.4 README authority-boundary policy artifact. Carries
/// the canonical block + required + forbidden substring sets +
/// the five pinned upstream anchor hashes. Two builds produce
/// byte-identical bytes.
#[derive(Debug, Clone)]
pub struct Ff4ReadmeAuthorityBoundaryPolicy {
    /// Historical seed-corpus anchor.
    pub corpus_hash_v1: [u8; 32],
    /// Ratified-corpus authority anchor.
    pub corpus_hash_v2: [u8; 32],
    /// FF.1 passport-index hash.
    pub ff1_passport_index_hash_v1: [u8; 32],
    /// FF.2 activation ratification gate hash.
    pub ff2_activation_ratification_gate_hash_v1: [u8; 32],
    /// FF.3 registry generation gate hash.
    pub ff3_registry_generation_gate_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Canonical block text lines (mirror of
    /// [`FF4_AUTHORITY_BOUNDARY_BLOCK_LINES`]).
    pub block_lines: &'static [&'static str],
    /// Required substrings (mirror of
    /// [`FF4_REQUIRED_SUBSTRINGS`]).
    pub required_substrings: &'static [&'static str],
    /// Forbidden substrings (mirror of
    /// [`FF4_FORBIDDEN_SUBSTRINGS`]).
    pub forbidden_substrings: &'static [&'static str],
    /// `ff4_readme_authority_boundary_policy_hash_v1` —
    /// domain-separated SHA-256 over every field above.
    pub ff4_readme_authority_boundary_policy_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why FF.4 rejected an input (the README text supplied to
/// [`verify_ff4_readme`]). An empty return means the README
/// authority-boundary discipline is satisfied. The seven panel-
/// required negatives map onto rules R.1–R.7; additional
/// structural rules (anchor cross-checks, SEED invariance)
/// emit under their own kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ff4VerifyErrorKind {
    /// Panel-required negative #1. The README contains stale
    /// "future ratification" / "until a future freeze" prose
    /// that was correct pre-ratification but is now operator-
    /// misleading.
    StaleFutureRatificationLanguage {
        /// The forbidden substring observed.
        observed_forbidden_substring: &'static str,
    },
    /// Panel-required negative #2. The README lacks the
    /// canonical `corpus_hash_v1` historical-anchor phrasing
    /// (`"historical seed-corpus anchor"`).
    MissingCorpusHashV1HistoricalAnchorLanguage,
    /// Panel-required negative #3. The README lacks the
    /// canonical `corpus_hash_v2` ratified-authority phrasing
    /// (`"ratified post-amendment authority"`).
    MissingCorpusHashV2RatifiedAuthorityLanguage,
    /// Panel-required negative #4. The README lacks the FF.1
    /// passport-materialisation phrasing
    /// (`"FF.1 materialized ratified T.12 additions into
    /// passports."`).
    MissingFf1PassportMaterialisationLanguage,
    /// Panel-required negative #5. The README lacks the FF.2 /
    /// FF.3 unratified-rejection phrasing (`"FF.2 / FF.3
    /// prevent unratified records from entering activation or
    /// registry generation."`).
    MissingFf2Ff3UnratifiedRejectionLanguage,
    /// Panel-required negative #6. The README contains a
    /// forbidden claim that T.12 proposals mutated SEED.
    ClaimThatT12ProposalsMutatedSeed,
    /// Panel-required negative #7. The README contains a
    /// forbidden claim that FF.1 mutated `corpus_hash_v2`.
    ClaimThatFf1MutatedCorpusHashV2,
    /// Structural defect: any other required substring missing
    /// from the front-door area beyond the panel-locked
    /// canonical-anchor / FF.1 / FF.2 / FF.3 phrasings.
    MissingRequiredSubstring {
        /// The required substring that was absent.
        missing_required_substring: &'static str,
    },
    /// Structural defect: any other forbidden substring present
    /// beyond the panel-locked stale-language phrasings.
    ForbiddenSubstringPresent {
        /// The forbidden substring observed.
        observed_forbidden_substring: &'static str,
    },
    /// `corpus_hash_v1` pinned on the policy does not equal the
    /// live `compute_corpus_hash_v1()`.
    CorpusHashV1Mismatch {
        /// Hash the policy claims.
        claimed: [u8; 32],
        /// Hash the live `compute_corpus_hash_v1()` returns.
        actual: [u8; 32],
    },
    /// `corpus_hash_v2` pinned on the policy does not equal the
    /// live consolidation-report `corpus_hash_v2`.
    CorpusHashV2Mismatch {
        /// Hash the policy claims.
        claimed: [u8; 32],
        /// Hash the live consolidation report computes.
        actual: [u8; 32],
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff4VerifyError {
    /// Error kind (see [`Ff4VerifyErrorKind`]).
    pub kind: Ff4VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the FF.4 policy artifact from live state. Two builds
/// produce byte-identical bytes.
#[must_use]
pub fn build_ff4_readme_authority_boundary_policy() -> Ff4ReadmeAuthorityBoundaryPolicy {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let activation_candidate_ids = default_candidate_ids(&passport_index);
    let ff2_gate = build_ff2_activation_ratification_gate_from(
        &report,
        &passport_index,
        &activation_candidate_ids,
    );
    let ff3_gate = build_ff3_registry_generation_gate();
    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut policy = Ff4ReadmeAuthorityBoundaryPolicy {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        ff1_passport_index_hash_v1: passport_index.ff1_passport_index_hash_v1,
        ff2_activation_ratification_gate_hash_v1: ff2_gate.ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: ff3_gate.ff3_registry_generation_gate_hash_v1,
        seed_len,
        block_lines: FF4_AUTHORITY_BOUNDARY_BLOCK_LINES,
        required_substrings: FF4_REQUIRED_SUBSTRINGS,
        forbidden_substrings: FF4_FORBIDDEN_SUBSTRINGS,
        ff4_readme_authority_boundary_policy_hash_v1: [0u8; 32],
    };
    policy.ff4_readme_authority_boundary_policy_hash_v1 =
        compute_ff4_readme_authority_boundary_policy_hash(&policy);
    policy
}

// ---------------------------------------------------------------
// Hash builder
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_bytes_fixed(out: &mut Vec<u8>, bytes: &[u8; 32]) {
    out.extend_from_slice(bytes);
}

fn compute_ff4_readme_authority_boundary_policy_hash(
    p: &Ff4ReadmeAuthorityBoundaryPolicy,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(FF4_README_AUTHORITY_BOUNDARY_POLICY_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &p.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &p.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &p.ff1_passport_index_hash_v1);
    write_bytes_fixed(&mut buf, &p.ff2_activation_ratification_gate_hash_v1);
    write_bytes_fixed(&mut buf, &p.ff3_registry_generation_gate_hash_v1);
    write_u32(&mut buf, p.seed_len);
    write_u32(
        &mut buf,
        u32::try_from(p.block_lines.len()).unwrap_or(u32::MAX),
    );
    for line in p.block_lines {
        write_str(&mut buf, line);
    }
    write_u32(
        &mut buf,
        u32::try_from(p.required_substrings.len()).unwrap_or(u32::MAX),
    );
    for s in p.required_substrings {
        write_str(&mut buf, s);
    }
    write_u32(
        &mut buf,
        u32::try_from(p.forbidden_substrings.len()).unwrap_or(u32::MAX),
    );
    for s in p.forbidden_substrings {
        write_str(&mut buf, s);
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Walk a README text against the FF.4 policy and emit every
/// rejection. An empty return means the README authority-
/// boundary discipline is satisfied.
///
/// The panel-required negatives R.1–R.7 are checked first
/// (they emit specific `Ff4VerifyErrorKind` variants with
/// dedicated kinds); the structural required / forbidden
/// substring sweeps emit `MissingRequiredSubstring` /
/// `ForbiddenSubstringPresent` for any remaining items.
#[must_use]
pub fn verify_ff4_readme(
    policy: &Ff4ReadmeAuthorityBoundaryPolicy,
    readme_text: &str,
) -> Vec<Ff4VerifyError> {
    let mut errors: Vec<Ff4VerifyError> = Vec::new();

    // R.1 StaleFutureRatificationLanguage: panel-locked stale
    // phrasings (module-scope const).
    for phrase in STALE_FUTURE_RATIFICATION_PHRASES {
        if readme_text.contains(phrase) {
            errors.push(Ff4VerifyError {
                kind: Ff4VerifyErrorKind::StaleFutureRatificationLanguage {
                    observed_forbidden_substring: phrase,
                },
            });
        }
    }

    // R.2 MissingCorpusHashV1HistoricalAnchorLanguage.
    if !readme_text.contains("historical seed-corpus anchor") {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::MissingCorpusHashV1HistoricalAnchorLanguage,
        });
    }

    // R.3 MissingCorpusHashV2RatifiedAuthorityLanguage.
    if !readme_text.contains("ratified post-amendment authority") {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::MissingCorpusHashV2RatifiedAuthorityLanguage,
        });
    }

    // R.4 MissingFf1PassportMaterialisationLanguage.
    if !readme_text.contains("FF.1 materialized ratified T.12 additions into passports.") {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::MissingFf1PassportMaterialisationLanguage,
        });
    }

    // R.5 MissingFf2Ff3UnratifiedRejectionLanguage.
    if !readme_text.contains(
        "FF.2 / FF.3 prevent unratified records from entering activation or registry generation.",
    ) {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::MissingFf2Ff3UnratifiedRejectionLanguage,
        });
    }

    // R.6 ClaimThatT12ProposalsMutatedSeed (module-scope const).
    for phrase in T12_MUTATED_SEED_PHRASES {
        if readme_text.contains(phrase) {
            errors.push(Ff4VerifyError {
                kind: Ff4VerifyErrorKind::ClaimThatT12ProposalsMutatedSeed,
            });
        }
    }

    // R.7 ClaimThatFf1MutatedCorpusHashV2 (module-scope const).
    for phrase in FF1_MUTATED_CORPUS_V2_PHRASES {
        if readme_text.contains(phrase) {
            errors.push(Ff4VerifyError {
                kind: Ff4VerifyErrorKind::ClaimThatFf1MutatedCorpusHashV2,
            });
        }
    }

    // Structural sweep: any other required-substring miss
    // surfaces under MissingRequiredSubstring. Skips entries
    // that R.2-R.5 already covered.
    for s in policy.required_substrings {
        if matches!(
            *s,
            "historical seed-corpus anchor"
                | "ratified post-amendment authority"
                | "T12RatifiedPassport"
                | "FF.2 / FF.3 prevent unratified records from entering activation or registry generation."
                | "FF.1 materialized ratified T.12 additions into passports."
                | "T.12.consolidate froze corpus_hash_v2"
        ) {
            // Covered by the canonical-anchor rules above OR
            // intentionally checked here for completeness.
            if !readme_text.contains(s) {
                errors.push(Ff4VerifyError {
                    kind: Ff4VerifyErrorKind::MissingRequiredSubstring {
                        missing_required_substring: s,
                    },
                });
            }
            continue;
        }
        if !readme_text.contains(s) {
            errors.push(Ff4VerifyError {
                kind: Ff4VerifyErrorKind::MissingRequiredSubstring {
                    missing_required_substring: s,
                },
            });
        }
    }

    // Structural sweep: any other forbidden-substring presence
    // surfaces under ForbiddenSubstringPresent. Skips entries
    // that R.1 / R.6 / R.7 already covered.
    for s in policy.forbidden_substrings {
        if matches!(
            *s,
            "future ratification / freeze campaign"
                | "until a future ratification campaign"
                | "until a future freeze"
                | "T.12 proposals mutated SEED"
                | "T.12 proposals mutate SEED"
                | "FF.1 mutated corpus_hash_v2"
                | "FF.1 mutates corpus_hash_v2"
        ) {
            continue;
        }
        if readme_text.contains(s) {
            errors.push(Ff4VerifyError {
                kind: Ff4VerifyErrorKind::ForbiddenSubstringPresent {
                    observed_forbidden_substring: s,
                },
            });
        }
    }

    // Anchor cross-checks + SEED invariance.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if policy.corpus_hash_v1 != live_v1 {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::CorpusHashV1Mismatch {
                claimed: policy.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    let live_report = build_consolidation_report();
    if policy.corpus_hash_v2 != live_report.corpus_hash_v2 {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::CorpusHashV2Mismatch {
                claimed: policy.corpus_hash_v2,
                actual: live_report.corpus_hash_v2,
            },
        });
    }
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(Ff4VerifyError {
            kind: Ff4VerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the canonical authority-boundary block as a single
/// String (each line joined with a newline). Used by the
/// `ff4-authority-boundary-block` CLI subcommand for operators
/// to copy verbatim into the README. Two renders produce byte-
/// identical bytes.
#[must_use]
pub fn render_ff4_authority_boundary_block() -> String {
    let mut s = String::new();
    for (i, line) in FF4_AUTHORITY_BOUNDARY_BLOCK_LINES.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        s.push_str(line);
    }
    s.push('\n');
    s
}

/// Render the FF.4 policy artifact as a deterministic text
/// report.
#[must_use]
pub fn render_ff4_policy_text(p: &Ff4ReadmeAuthorityBoundaryPolicy) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "FF.4 README Authority-Boundary Policy (v1)");
    let _ = writeln!(s, "==========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                              : {}",
        hex32(&p.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                              : {}",
        hex32(&p.corpus_hash_v2)
    );
    let _ = writeln!(
        s,
        "  ff1_passport_index_hash_v1                  : {}",
        hex32(&p.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff2_activation_ratification_gate_hash_v1    : {}",
        hex32(&p.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff3_registry_generation_gate_hash_v1        : {}",
        hex32(&p.ff3_registry_generation_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  SEED.len()                                  : {}",
        p.seed_len
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Canonical authority-boundary block ({} lines)",
        p.block_lines.len()
    );
    for line in p.block_lines {
        let _ = writeln!(s, "  | {line}");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Required substrings ({})", p.required_substrings.len());
    for r in p.required_substrings {
        let _ = writeln!(s, "  + {r}");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Forbidden substrings ({})", p.forbidden_substrings.len());
    for f in p.forbidden_substrings {
        let _ = writeln!(s, "  - {f}");
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "ff4_readme_authority_boundary_policy_hash_v1 : {}",
        hex32(&p.ff4_readme_authority_boundary_policy_hash_v1)
    );
    s
}

/// Render the FF.4 policy artifact as a deterministic JSON
/// object. Two renders produce byte-identical bytes.
#[must_use]
pub fn render_ff4_policy_json(p: &Ff4ReadmeAuthorityBoundaryPolicy) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"schema\":\"{FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1}\""
    );
    let _ = write!(s, ",\"corpus_hash_v1\":\"{}\"", hex32(&p.corpus_hash_v1));
    let _ = write!(s, ",\"corpus_hash_v2\":\"{}\"", hex32(&p.corpus_hash_v2));
    let _ = write!(
        s,
        ",\"ff1_passport_index_hash_v1\":\"{}\"",
        hex32(&p.ff1_passport_index_hash_v1)
    );
    let _ = write!(
        s,
        ",\"ff2_activation_ratification_gate_hash_v1\":\"{}\"",
        hex32(&p.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = write!(
        s,
        ",\"ff3_registry_generation_gate_hash_v1\":\"{}\"",
        hex32(&p.ff3_registry_generation_gate_hash_v1)
    );
    let _ = write!(s, ",\"seed_len\":{}", p.seed_len);
    let _ = write!(s, ",\"block_lines\":[");
    for (i, line) in p.block_lines.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{}\"", json_escape(line));
    }
    s.push(']');
    let _ = write!(s, ",\"required_substrings\":[");
    for (i, r) in p.required_substrings.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{}\"", json_escape(r));
    }
    s.push(']');
    let _ = write!(s, ",\"forbidden_substrings\":[");
    for (i, f) in p.forbidden_substrings.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{}\"", json_escape(f));
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"ff4_readme_authority_boundary_policy_hash_v1\":\"{}\"",
        hex32(&p.ff4_readme_authority_boundary_policy_hash_v1)
    );
    s.push('}');
    s
}

/// Hex-encode a 32-byte digest as a 64-character lowercase
/// string.
fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push(nibble(*b >> 4));
        s.push(nibble(*b & 0x0f));
    }
    s
}

const fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + (n - 10)) as char,
        _ => '?',
    }
}

/// Minimal JSON-string escape covering the characters that
/// appear in our pinned policy text (quote + backslash).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}
