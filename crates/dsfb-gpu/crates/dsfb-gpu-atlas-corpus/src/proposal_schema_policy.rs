//! FF.5 — `ProposalSchemaUpgradePolicy`: schema-upgrade
//! migration law for proposal artifacts.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **FF.5 defines how proposal schema upgrades are allowed to
//! > re-render historical proposal artifacts without erasing
//! > the old artifact hashes or confusing the court lineage.
//! > Core rule: schema upgrade ≠ silent artifact rewrite.
//! > Required doctrine: if a schema change re-renders old
//! > proposals, the migration must preserve the old artifact
//! > hash, emit the new schema hash, explain why the rendered
//! > bytes changed, and provide an explicit `old_hash →
//! > new_hash` migration table. The old artifact remains part
//! > of the evidence trail; the new artifact becomes the
//! > active schema rendering only through an explicit
//! > migration receipt.**
//!
//! ## Why
//!
//! The post-T.12.consolidate / post-FF.1 / post-FF.2 / post-FF.3
//! arc surfaced several future-work items (e.g. richer
//! `ProposedSourceRef` with `authors` / `doi_or_url`,
//! structured contract flags on `ProposedPrimitive`,
//! `ratification_commit` split) that will eventually require
//! re-rendering historical T.12.x proposal artifacts under a
//! v2 schema. Without an upfront policy, the most likely
//! failure mode is a silent re-render — new bytes overwrite
//! the historical hash anchors without provenance, and the
//! court lineage becomes ambiguous.
//!
//! FF.5 lands the policy BEFORE any schema upgrade so every
//! future upgrade has a known contract to satisfy:
//!
//! 1. Preserve the old artifact hash in the archived receipt.
//! 2. Emit the new schema/version hash.
//! 3. Explain the semantic reason the rendered bytes changed.
//! 4. Provide an explicit `old_hash → new_hash` migration
//!    table.
//! 5. Declare the old artifact hash was VALID at filing time
//!    (the migration is not a correction; the old bytes were
//!    a faithful rendering under the old schema).
//! 6. Never mutate `corpus_hash_v1` — that anchor is
//!    historical and frozen at T.10.
//! 7. Never mutate `corpus_hash_v2` outside a declared freeze
//!    campaign — `corpus_hash_v2` is the ratified-corpus
//!    authority and any change requires a panel-locked
//!    re-freeze.
//!
//! ## Status at FF.5 time
//!
//! No schema upgrades have happened. The migration table is
//! empty. The artifact this module emits is the POLICY (the
//! contract) plus the migration-table SHELL (empty), so
//! future schema-upgrade commits have a typed surface to
//! produce receipts against.
//!
//! ## Method
//!
//! 1. Define [`ProposalSchemaUpgradeReceipt`] — the receipt
//!    shape every future schema-upgrade commit must emit.
//! 2. Define [`MigrationRow`] — one row per re-rendered
//!    artifact, carrying `old_artifact_hash`,
//!    `new_artifact_hash`, `old_schema_version`,
//!    `new_schema_version`, `old_artifact_id`, and
//!    `reason_byte_diff`.
//! 3. Define [`ProposalSchemaMigrationTable`] — the rolling
//!    list of every migration row across all schema upgrades.
//!    Empty at FF.5; future commits append rows.
//! 4. Define [`ProposalSchemaUpgradePolicy`] — the top-level
//!    artifact pinning the doctrine + the migration table +
//!    the six upstream anchor hashes (`corpus_hash_v1`,
//!    `corpus_hash_v2`, `ff1_passport_index_hash_v1`,
//!    `ff2_activation_ratification_gate_hash_v1`,
//!    `ff3_registry_generation_gate_hash_v1`,
//!    `ff4_readme_authority_boundary_policy_hash_v1`).
//! 5. Expose [`verify_schema_upgrade_receipt`] which walks a
//!    receipt against the policy and emits every rejection
//!    under [`Ff5VerifyErrorKind`].
//! 6. Expose [`verify_migration_table`] for the table-shape
//!    invariants (duplicate-old-hash and duplicate-new-hash
//!    checks across rows).
//!
//! ## Three new own-namespace hash layers
//!
//! - `proposal_schema_upgrade_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0`.
//! - `proposal_schema_migration_table_hash_v1` under
//!   `DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0`.
//! - `schema_upgrade_receipt_hash_v1` (per-receipt) under
//!   `DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0`.
//!
//! ## Panel-locked non-claims
//!
//! - FF.5 does NOT add new detectors.
//! - FF.5 does NOT alter any upstream hash anchor
//!   (`corpus_hash_v1`, `corpus_hash_v2`, any T.12.x proposal
//!   hash, any T.12.consolidate hash, any FF.1 / FF.2 / FF.3
//!   / FF.4 hash).
//! - FF.5 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
//!   FF.1 / FF.2 / FF.3 / FF.4 hash.
//! - FF.5 does NOT mutate `SEED.len()` (stays at 54).
//! - FF.5 does NOT itself perform any schema upgrade. It is a
//!   forward-looking governance artifact pinning the contract
//!   future upgrades MUST satisfy.
//! - FF.5 does NOT change S1.3a / FF.2 / FF.3 court
//!   decisions.
//! - FF.5 does NOT generate CUDA kernels.
//! - FF.5 does NOT decide contraindications or challenges.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! Every upstream hash anchor (T.11 / S1.3 / T.12.x /
//! T.12.consolidate / FF.1 / FF.2 / FF.3 / FF.4) byte-
//! identical. `SEED.len()` = 54.
//!
//! **NEW**: three own-namespace hashes (policy + table +
//! per-receipt).
//!
//! ## Panel-locked one-line verdict
//!
//! > Schema upgrade ≠ silent artifact rewrite.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::consolidate::build_consolidation_report;
use crate::ff1_passport_materialisation::build_ff1_passport_index_from;
use crate::ff2_activation_ratification_gate::build_ff2_activation_ratification_gate_from;
use crate::ff2_activation_ratification_gate::default_candidate_ids;
use crate::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use crate::ff4_readme_authority_boundary::build_ff4_readme_authority_boundary_policy;
use crate::seed::SEED;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators
// ---------------------------------------------------------------

/// Domain separator for `proposal_schema_upgrade_policy_hash_v1`.
pub const PROPOSAL_SCHEMA_UPGRADE_POLICY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0";

/// Schema identifier embedded in the policy hash material.
pub const PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1";

/// Domain separator for `proposal_schema_migration_table_hash_v1`.
pub const PROPOSAL_SCHEMA_MIGRATION_TABLE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0";

/// Schema identifier embedded in the migration-table hash
/// material.
pub const PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1";

/// Domain separator for per-receipt
/// `schema_upgrade_receipt_hash_v1`.
pub const SCHEMA_UPGRADE_RECEIPT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0";

/// Schema identifier embedded in per-receipt hash material.
pub const SCHEMA_UPGRADE_RECEIPT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1";

// ---------------------------------------------------------------
// Policy doctrine lines (pinned)
// ---------------------------------------------------------------

/// Panel-locked policy doctrine text lines. Pinned verbatim
/// into the policy hash material; any change requires a new
/// domain separator (FF.5.x schema-upgrade commit).
pub const FF5_POLICY_DOCTRINE_LINES: &[&str] = &[
    "Schema upgrade != silent artifact rewrite.",
    "Historical proposal artifacts are court evidence.",
    "A schema upgrade may produce new rendered artifacts, but it must not pretend the old bytes never existed.",
    "Every schema-driven re-render must preserve the archived old hash, emit the new schema/version hash, explain the semantic reason for the byte change, and record a migration table from old hash to new hash.",
    "The old artifact remains part of the evidence trail; the new artifact becomes the active schema rendering only through an explicit migration receipt.",
    "corpus_hash_v1 is the historical seed-corpus anchor frozen at T.10; no schema upgrade may mutate it.",
    "corpus_hash_v2 is the ratified-corpus authority anchor; any schema upgrade that mutates it requires a declared freeze campaign id.",
    "Every migration row must declare a non-zero old_artifact_hash, a non-zero new_artifact_hash, the old schema version, the new schema version, the old_artifact_id, and the semantic reason the bytes changed.",
    "Migration table rows have unique old_artifact_hash values and unique new_artifact_hash values across the whole table.",
    "Every receipt must declare the old artifact hash was VALID at filing time; a migration is not a correction.",
];

// ---------------------------------------------------------------
// Migration-row record
// ---------------------------------------------------------------

/// One row of the proposal-schema migration table. Field
/// order is the canonical hash order; do not reorder without
/// rebaselining `proposal_schema_migration_table_hash_v1`.
#[derive(Debug, Clone)]
pub struct MigrationRow {
    /// Wire identifier of the proposal artifact being
    /// migrated (e.g. `"t12_a_spc"`,
    /// `"corpus_amendment_proposal_v1"`).
    pub old_artifact_id: &'static str,
    /// Schema version the old artifact was rendered under
    /// (e.g. `"v1"`).
    pub old_schema_version: &'static str,
    /// Schema version the new artifact is rendered under
    /// (e.g. `"v2"`).
    pub new_schema_version: &'static str,
    /// 32-byte hash of the old artifact, preserved verbatim
    /// from the historical receipt.
    pub old_artifact_hash: [u8; 32],
    /// 32-byte hash of the new artifact under the new schema.
    pub new_artifact_hash: [u8; 32],
    /// Operator-readable explanation of why the rendered
    /// bytes differ (e.g. `"ProposedSourceRef gained authors
    /// + doi_or_url fields"`). Non-empty.
    pub reason_byte_diff: &'static str,
}

// ---------------------------------------------------------------
// Migration-table record
// ---------------------------------------------------------------

/// The rolling migration table. Empty at FF.5 (no schema
/// upgrades have happened yet); future schema-upgrade commits
/// append rows. Sorted by `old_artifact_hash` ascending so
/// the table hash is byte-stable across appends.
#[derive(Debug, Clone)]
pub struct ProposalSchemaMigrationTable {
    /// Migration rows, sorted by `old_artifact_hash`
    /// ascending.
    pub rows: Vec<MigrationRow>,
    /// `proposal_schema_migration_table_hash_v1` — domain-
    /// separated SHA-256 over the sorted row list + row
    /// count.
    pub proposal_schema_migration_table_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Schema-upgrade receipt (per-upgrade)
// ---------------------------------------------------------------

/// One schema-upgrade receipt. Every future schema-upgrade
/// commit MUST emit one of these and append its rows into the
/// rolling migration table. Verified by
/// [`verify_schema_upgrade_receipt`].
#[derive(Debug, Clone)]
pub struct ProposalSchemaUpgradeReceipt {
    /// Operator-readable upgrade id (e.g.
    /// `"corpus_amendment_proposal_v1_to_v2"`).
    pub upgrade_id: &'static str,
    /// Schema version the receipt migrates FROM.
    pub old_schema_version: &'static str,
    /// Schema version the receipt migrates TO.
    pub new_schema_version: &'static str,
    /// Operator-readable explanation of WHY the schema is
    /// changing (the semantic motivation). Non-empty.
    pub semantic_reason: &'static str,
    /// Per-artifact migration rows the receipt is appending
    /// to the rolling table.
    pub migration_rows: Vec<MigrationRow>,
    /// MUST be true: the schema upgrade does not mutate
    /// `corpus_hash_v1`.
    pub preserves_corpus_hash_v1: bool,
    /// MUST be true UNLESS `freeze_campaign_id` is `Some`:
    /// the schema upgrade does not mutate `corpus_hash_v2`
    /// outside a declared freeze campaign.
    pub preserves_corpus_hash_v2: bool,
    /// `Some(id)` iff the schema upgrade is being performed
    /// under a declared corpus-freeze campaign. Required when
    /// `preserves_corpus_hash_v2 = false`.
    pub freeze_campaign_id: Option<&'static str>,
    /// MUST be true: the receipt declares the old artifact
    /// hash was valid at filing time. A migration is a schema
    /// evolution, not a correction.
    pub declares_old_artifact_hash_valid: bool,
    /// `schema_upgrade_receipt_hash_v1` — domain-separated
    /// SHA-256 over every field above.
    pub schema_upgrade_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level policy artifact
// ---------------------------------------------------------------

/// The FF.5 proposal-schema upgrade policy artifact. Pins the
/// doctrine + the migration table + the six upstream anchor
/// hashes proving FF.5 did not mutate any upstream authority.
#[derive(Debug, Clone)]
pub struct ProposalSchemaUpgradePolicy {
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
    /// FF.4 README authority-boundary policy hash.
    pub ff4_readme_authority_boundary_policy_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Policy doctrine text lines (mirror of
    /// [`FF5_POLICY_DOCTRINE_LINES`]).
    pub doctrine_lines: &'static [&'static str],
    /// Current migration table (empty at FF.5; appended by
    /// future schema-upgrade commits).
    pub migration_table: ProposalSchemaMigrationTable,
    /// `proposal_schema_upgrade_policy_hash_v1` — domain-
    /// separated SHA-256 over every field above.
    pub proposal_schema_upgrade_policy_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why FF.5 rejected a schema-upgrade receipt or migration
/// table. The nine panel-required negatives map onto rules
/// R.1–R.9; additional structural rules emit under their own
/// kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ff5VerifyErrorKind {
    /// Panel-required negative #1. A migration row is missing
    /// the `old_artifact_hash` (all-zero sentinel).
    SchemaRerenderWithoutOldHash {
        /// The artifact id with the missing old hash.
        old_artifact_id: &'static str,
    },
    /// Panel-required negative #2. A receipt is missing the
    /// new schema version (empty `new_schema_version`).
    SchemaRerenderWithoutNewSchemaHash {
        /// The receipt upgrade id with the missing version.
        upgrade_id: &'static str,
    },
    /// Panel-required negative #3. A schema-upgrade receipt
    /// emits no migration rows.
    SchemaRerenderWithoutMigrationTable {
        /// The receipt upgrade id with no rows.
        upgrade_id: &'static str,
    },
    /// Panel-required negative #4. A receipt is missing the
    /// semantic reason for the schema change (empty
    /// `semantic_reason`), or a migration row is missing
    /// `reason_byte_diff`.
    SchemaRerenderWithoutReason {
        /// The receipt or row identifier whose reason is
        /// missing.
        identifier: &'static str,
    },
    /// Panel-required negative #5. Two migration rows in the
    /// same table share the same `old_artifact_hash`.
    MigrationTableWithDuplicateOldHash {
        /// The duplicated old hash.
        duplicated_old_artifact_hash: [u8; 32],
    },
    /// Panel-required negative #6. Two migration rows in the
    /// same table share the same `new_artifact_hash`.
    MigrationTableWithDuplicateNewHash {
        /// The duplicated new hash.
        duplicated_new_artifact_hash: [u8; 32],
    },
    /// Panel-required negative #7. A receipt declares
    /// `declares_old_artifact_hash_valid = false`. A migration
    /// is a schema evolution, not a correction; the old hash
    /// was valid at filing time.
    ClaimThatOldArtifactHashWasInvalid {
        /// The receipt upgrade id.
        upgrade_id: &'static str,
    },
    /// Panel-required negative #8. A receipt declares
    /// `preserves_corpus_hash_v1 = false`. corpus_hash_v1 is
    /// the historical seed-corpus anchor frozen at T.10; no
    /// schema upgrade may mutate it.
    SchemaUpgradeThatMutatesCorpusHashV1 {
        /// The receipt upgrade id.
        upgrade_id: &'static str,
    },
    /// Panel-required negative #9. A receipt declares
    /// `preserves_corpus_hash_v2 = false` without a
    /// `freeze_campaign_id`. corpus_hash_v2 mutation requires
    /// a declared freeze campaign.
    SchemaUpgradeThatMutatesCorpusHashV2WithoutFreezeCampaign {
        /// The receipt upgrade id.
        upgrade_id: &'static str,
    },
    /// Structural defect: a migration row has the same
    /// `old_schema_version` and `new_schema_version` (no
    /// actual schema change).
    MigrationRowWithIdenticalSchemaVersions {
        /// The artifact id.
        old_artifact_id: &'static str,
    },
    /// Structural defect: a migration row has an empty
    /// `old_artifact_id`.
    MigrationRowWithEmptyArtifactId,
    /// Structural defect: a migration row has an empty
    /// `old_schema_version` or `new_schema_version`.
    MigrationRowWithEmptySchemaVersion {
        /// The artifact id with the empty schema version.
        old_artifact_id: &'static str,
    },
    /// Structural defect: a receipt has an empty `upgrade_id`.
    ReceiptWithEmptyUpgradeId,
    /// Structural defect: a receipt's `preserves_corpus_hash_v2
    /// = false` with `freeze_campaign_id = Some(empty)`.
    ReceiptFreezeCampaignIdEmpty {
        /// The receipt upgrade id.
        upgrade_id: &'static str,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff5VerifyError {
    /// Error kind (see [`Ff5VerifyErrorKind`]).
    pub kind: Ff5VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production FF.5 policy artifact from live state.
/// The migration table is empty (no schema upgrades have
/// happened at FF.5 time).
#[must_use]
pub fn build_proposal_schema_upgrade_policy() -> ProposalSchemaUpgradePolicy {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let activation_candidate_ids = default_candidate_ids(&passport_index);
    let ff2_gate = build_ff2_activation_ratification_gate_from(
        &report,
        &passport_index,
        &activation_candidate_ids,
    );
    let ff3_gate = build_ff3_registry_generation_gate();
    let ff4_policy = build_ff4_readme_authority_boundary_policy();
    let migration_table = build_empty_migration_table();
    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut policy = ProposalSchemaUpgradePolicy {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        ff1_passport_index_hash_v1: passport_index.ff1_passport_index_hash_v1,
        ff2_activation_ratification_gate_hash_v1: ff2_gate.ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: ff3_gate.ff3_registry_generation_gate_hash_v1,
        ff4_readme_authority_boundary_policy_hash_v1: ff4_policy
            .ff4_readme_authority_boundary_policy_hash_v1,
        seed_len,
        doctrine_lines: FF5_POLICY_DOCTRINE_LINES,
        migration_table,
        proposal_schema_upgrade_policy_hash_v1: [0u8; 32],
    };
    policy.proposal_schema_upgrade_policy_hash_v1 =
        compute_proposal_schema_upgrade_policy_hash(&policy);
    policy
}

/// Build the empty migration table (FF.5 baseline state).
#[must_use]
pub fn build_empty_migration_table() -> ProposalSchemaMigrationTable {
    build_migration_table_from_rows(Vec::new())
}

/// Build a migration table from a row vector. Sorts rows by
/// `old_artifact_hash` ascending and computes the table hash.
#[must_use]
pub fn build_migration_table_from_rows(
    mut rows: Vec<MigrationRow>,
) -> ProposalSchemaMigrationTable {
    rows.sort_by(|a, b| a.old_artifact_hash.cmp(&b.old_artifact_hash));
    let mut table = ProposalSchemaMigrationTable {
        rows,
        proposal_schema_migration_table_hash_v1: [0u8; 32],
    };
    table.proposal_schema_migration_table_hash_v1 =
        compute_proposal_schema_migration_table_hash(&table);
    table
}

/// Build a schema-upgrade receipt from its declared fields.
/// Computes the receipt hash over the canonical projection.
#[must_use]
#[allow(clippy::too_many_arguments)] // each argument is a declared receipt field, panel-locked
pub fn build_schema_upgrade_receipt(
    upgrade_id: &'static str,
    old_schema_version: &'static str,
    new_schema_version: &'static str,
    semantic_reason: &'static str,
    migration_rows: Vec<MigrationRow>,
    preserves_corpus_hash_v1: bool,
    preserves_corpus_hash_v2: bool,
    freeze_campaign_id: Option<&'static str>,
    declares_old_artifact_hash_valid: bool,
) -> ProposalSchemaUpgradeReceipt {
    let mut receipt = ProposalSchemaUpgradeReceipt {
        upgrade_id,
        old_schema_version,
        new_schema_version,
        semantic_reason,
        migration_rows,
        preserves_corpus_hash_v1,
        preserves_corpus_hash_v2,
        freeze_campaign_id,
        declares_old_artifact_hash_valid,
        schema_upgrade_receipt_hash_v1: [0u8; 32],
    };
    receipt.schema_upgrade_receipt_hash_v1 = compute_schema_upgrade_receipt_hash(&receipt);
    receipt
}

// ---------------------------------------------------------------
// Hash builders
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

fn write_migration_row(out: &mut Vec<u8>, r: &MigrationRow) {
    write_str(out, r.old_artifact_id);
    write_str(out, r.old_schema_version);
    write_str(out, r.new_schema_version);
    write_bytes_fixed(out, &r.old_artifact_hash);
    write_bytes_fixed(out, &r.new_artifact_hash);
    write_str(out, r.reason_byte_diff);
}

fn compute_proposal_schema_migration_table_hash(table: &ProposalSchemaMigrationTable) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(PROPOSAL_SCHEMA_MIGRATION_TABLE_DOMAIN_V1.as_bytes());
    write_str(&mut buf, PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1);
    write_u32(
        &mut buf,
        u32::try_from(table.rows.len()).unwrap_or(u32::MAX),
    );
    for r in &table.rows {
        write_migration_row(&mut buf, r);
    }
    sha256(&buf)
}

fn compute_proposal_schema_upgrade_policy_hash(p: &ProposalSchemaUpgradePolicy) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(PROPOSAL_SCHEMA_UPGRADE_POLICY_DOMAIN_V1.as_bytes());
    write_str(&mut buf, PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &p.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &p.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &p.ff1_passport_index_hash_v1);
    write_bytes_fixed(&mut buf, &p.ff2_activation_ratification_gate_hash_v1);
    write_bytes_fixed(&mut buf, &p.ff3_registry_generation_gate_hash_v1);
    write_bytes_fixed(&mut buf, &p.ff4_readme_authority_boundary_policy_hash_v1);
    write_u32(&mut buf, p.seed_len);
    write_u32(
        &mut buf,
        u32::try_from(p.doctrine_lines.len()).unwrap_or(u32::MAX),
    );
    for line in p.doctrine_lines {
        write_str(&mut buf, line);
    }
    write_bytes_fixed(
        &mut buf,
        &p.migration_table.proposal_schema_migration_table_hash_v1,
    );
    sha256(&buf)
}

fn compute_schema_upgrade_receipt_hash(r: &ProposalSchemaUpgradeReceipt) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(SCHEMA_UPGRADE_RECEIPT_DOMAIN_V1.as_bytes());
    write_str(&mut buf, SCHEMA_UPGRADE_RECEIPT_SCHEMA_V1);
    write_str(&mut buf, r.upgrade_id);
    write_str(&mut buf, r.old_schema_version);
    write_str(&mut buf, r.new_schema_version);
    write_str(&mut buf, r.semantic_reason);
    write_u32(
        &mut buf,
        u32::try_from(r.migration_rows.len()).unwrap_or(u32::MAX),
    );
    for row in &r.migration_rows {
        write_migration_row(&mut buf, row);
    }
    write_u32(&mut buf, u32::from(r.preserves_corpus_hash_v1));
    write_u32(&mut buf, u32::from(r.preserves_corpus_hash_v2));
    match r.freeze_campaign_id {
        Some(s) => {
            write_u32(&mut buf, 1);
            write_str(&mut buf, s);
        }
        None => {
            write_u32(&mut buf, 0);
        }
    }
    write_u32(&mut buf, u32::from(r.declares_old_artifact_hash_valid));
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifiers
// ---------------------------------------------------------------

/// Verify a schema-upgrade receipt against the FF.5 policy
/// doctrine. Empty return means the receipt is admissible.
#[must_use]
pub fn verify_schema_upgrade_receipt(
    receipt: &ProposalSchemaUpgradeReceipt,
) -> Vec<Ff5VerifyError> {
    let mut errors: Vec<Ff5VerifyError> = Vec::new();

    // Structural: empty upgrade id.
    if receipt.upgrade_id.is_empty() {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::ReceiptWithEmptyUpgradeId,
        });
    }

    // R.2 SchemaRerenderWithoutNewSchemaHash.
    if receipt.new_schema_version.is_empty() {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::SchemaRerenderWithoutNewSchemaHash {
                upgrade_id: receipt.upgrade_id,
            },
        });
    }

    // Structural: empty old schema version.
    if receipt.old_schema_version.is_empty() {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::MigrationRowWithEmptySchemaVersion {
                old_artifact_id: receipt.upgrade_id,
            },
        });
    }

    // R.3 SchemaRerenderWithoutMigrationTable.
    if receipt.migration_rows.is_empty() {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::SchemaRerenderWithoutMigrationTable {
                upgrade_id: receipt.upgrade_id,
            },
        });
    }

    // R.4 SchemaRerenderWithoutReason.
    if receipt.semantic_reason.is_empty() {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::SchemaRerenderWithoutReason {
                identifier: receipt.upgrade_id,
            },
        });
    }

    // R.7 ClaimThatOldArtifactHashWasInvalid.
    if !receipt.declares_old_artifact_hash_valid {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::ClaimThatOldArtifactHashWasInvalid {
                upgrade_id: receipt.upgrade_id,
            },
        });
    }

    // R.8 SchemaUpgradeThatMutatesCorpusHashV1.
    if !receipt.preserves_corpus_hash_v1 {
        errors.push(Ff5VerifyError {
            kind: Ff5VerifyErrorKind::SchemaUpgradeThatMutatesCorpusHashV1 {
                upgrade_id: receipt.upgrade_id,
            },
        });
    }

    // R.9 SchemaUpgradeThatMutatesCorpusHashV2WithoutFreezeCampaign.
    if !receipt.preserves_corpus_hash_v2 {
        match receipt.freeze_campaign_id {
            None => {
                errors.push(Ff5VerifyError {
                    kind:
                        Ff5VerifyErrorKind::SchemaUpgradeThatMutatesCorpusHashV2WithoutFreezeCampaign {
                            upgrade_id: receipt.upgrade_id,
                        },
                });
            }
            Some("") => {
                errors.push(Ff5VerifyError {
                    kind: Ff5VerifyErrorKind::ReceiptFreezeCampaignIdEmpty {
                        upgrade_id: receipt.upgrade_id,
                    },
                });
            }
            Some(_) => { /* admitted */ }
        }
    }

    // Per-row structural + R.1 / R.4-row / R.X checks.
    for row in &receipt.migration_rows {
        if row.old_artifact_hash == [0u8; 32] {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::SchemaRerenderWithoutOldHash {
                    old_artifact_id: row.old_artifact_id,
                },
            });
        }
        if row.new_artifact_hash == [0u8; 32] {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::SchemaRerenderWithoutNewSchemaHash {
                    upgrade_id: row.old_artifact_id,
                },
            });
        }
        if row.reason_byte_diff.is_empty() {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::SchemaRerenderWithoutReason {
                    identifier: row.old_artifact_id,
                },
            });
        }
        if row.old_schema_version == row.new_schema_version {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationRowWithIdenticalSchemaVersions {
                    old_artifact_id: row.old_artifact_id,
                },
            });
        }
        if row.old_artifact_id.is_empty() {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationRowWithEmptyArtifactId,
            });
        }
        if row.old_schema_version.is_empty() || row.new_schema_version.is_empty() {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationRowWithEmptySchemaVersion {
                    old_artifact_id: row.old_artifact_id,
                },
            });
        }
    }

    // R.5 / R.6 duplicate-hash checks within the receipt's
    // own rows.
    let mut seen_old: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut seen_new: BTreeSet<[u8; 32]> = BTreeSet::new();
    for row in &receipt.migration_rows {
        if !seen_old.insert(row.old_artifact_hash) {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationTableWithDuplicateOldHash {
                    duplicated_old_artifact_hash: row.old_artifact_hash,
                },
            });
        }
        if !seen_new.insert(row.new_artifact_hash) {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationTableWithDuplicateNewHash {
                    duplicated_new_artifact_hash: row.new_artifact_hash,
                },
            });
        }
    }

    errors
}

/// Verify a migration table's whole-table invariants
/// (duplicate-old-hash and duplicate-new-hash checks across
/// the entire table). Empty return means admissible.
#[must_use]
pub fn verify_migration_table(table: &ProposalSchemaMigrationTable) -> Vec<Ff5VerifyError> {
    let mut errors: Vec<Ff5VerifyError> = Vec::new();
    let mut seen_old: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut seen_new: BTreeSet<[u8; 32]> = BTreeSet::new();
    for row in &table.rows {
        if !seen_old.insert(row.old_artifact_hash) {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationTableWithDuplicateOldHash {
                    duplicated_old_artifact_hash: row.old_artifact_hash,
                },
            });
        }
        if !seen_new.insert(row.new_artifact_hash) {
            errors.push(Ff5VerifyError {
                kind: Ff5VerifyErrorKind::MigrationTableWithDuplicateNewHash {
                    duplicated_new_artifact_hash: row.new_artifact_hash,
                },
            });
        }
    }
    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the FF.5 policy artifact as a deterministic text
/// report. Two renders against the same policy produce byte-
/// identical bytes.
#[must_use]
pub fn render_ff5_policy_text(p: &ProposalSchemaUpgradePolicy) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "FF.5 Proposal Schema Upgrade Policy (v1)");
    let _ = writeln!(s, "========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                                : {}",
        hex32(&p.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                                : {}",
        hex32(&p.corpus_hash_v2)
    );
    let _ = writeln!(
        s,
        "  ff1_passport_index_hash_v1                    : {}",
        hex32(&p.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff2_activation_ratification_gate_hash_v1      : {}",
        hex32(&p.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff3_registry_generation_gate_hash_v1          : {}",
        hex32(&p.ff3_registry_generation_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff4_readme_authority_boundary_policy_hash_v1  : {}",
        hex32(&p.ff4_readme_authority_boundary_policy_hash_v1)
    );
    let _ = writeln!(
        s,
        "  SEED.len()                                    : {}",
        p.seed_len
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Doctrine ({} lines)", p.doctrine_lines.len());
    for (i, line) in p.doctrine_lines.iter().enumerate() {
        let _ = writeln!(s, "  {:>2}. {line}", i + 1);
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Migration table: {} rows", p.migration_table.rows.len());
    let _ = writeln!(
        s,
        "  proposal_schema_migration_table_hash_v1 : {}",
        hex32(&p.migration_table.proposal_schema_migration_table_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "proposal_schema_upgrade_policy_hash_v1 : {}",
        hex32(&p.proposal_schema_upgrade_policy_hash_v1)
    );
    s
}

/// Render the FF.5 policy artifact as a deterministic JSON
/// object.
#[must_use]
pub fn render_ff5_policy_json(p: &ProposalSchemaUpgradePolicy) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"schema\":\"{PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1}\""
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
    let _ = write!(
        s,
        ",\"ff4_readme_authority_boundary_policy_hash_v1\":\"{}\"",
        hex32(&p.ff4_readme_authority_boundary_policy_hash_v1)
    );
    let _ = write!(s, ",\"seed_len\":{}", p.seed_len);
    let _ = write!(s, ",\"doctrine_lines\":[");
    for (i, line) in p.doctrine_lines.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{}\"", json_escape(line));
    }
    s.push(']');
    let _ = write!(s, ",\"migration_table\":{{");
    let _ = write!(
        s,
        "\"schema\":\"{PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1}\""
    );
    let _ = write!(s, ",\"row_count\":{}", p.migration_table.rows.len());
    let _ = write!(s, ",\"rows\":[");
    for (i, row) in p.migration_table.rows.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"old_artifact_id\":\"{}\",\"old_schema_version\":\"{}\",\"new_schema_version\":\"{}\",\"old_artifact_hash\":\"{}\",\"new_artifact_hash\":\"{}\",\"reason_byte_diff\":\"{}\"}}",
            json_escape(row.old_artifact_id),
            json_escape(row.old_schema_version),
            json_escape(row.new_schema_version),
            hex32(&row.old_artifact_hash),
            hex32(&row.new_artifact_hash),
            json_escape(row.reason_byte_diff)
        );
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"proposal_schema_migration_table_hash_v1\":\"{}\"",
        hex32(&p.migration_table.proposal_schema_migration_table_hash_v1)
    );
    s.push('}');
    let _ = write!(
        s,
        ",\"proposal_schema_upgrade_policy_hash_v1\":\"{}\"",
        hex32(&p.proposal_schema_upgrade_policy_hash_v1)
    );
    s.push('}');
    s
}

/// Render the migration table as a deterministic text report.
#[must_use]
pub fn render_ff5_migration_table_text(table: &ProposalSchemaMigrationTable) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "FF.5 Proposal Schema Migration Table (v1)");
    let _ = writeln!(s, "=========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Row count: {}", table.rows.len());
    for (i, row) in table.rows.iter().enumerate() {
        let _ = writeln!(s);
        let _ = writeln!(s, "Row {} ({}):", i + 1, row.old_artifact_id);
        let _ = writeln!(s, "  old_schema_version : {}", row.old_schema_version);
        let _ = writeln!(s, "  new_schema_version : {}", row.new_schema_version);
        let _ = writeln!(
            s,
            "  old_artifact_hash  : {}",
            hex32(&row.old_artifact_hash)
        );
        let _ = writeln!(
            s,
            "  new_artifact_hash  : {}",
            hex32(&row.new_artifact_hash)
        );
        let _ = writeln!(s, "  reason_byte_diff   : {}", row.reason_byte_diff);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "proposal_schema_migration_table_hash_v1 : {}",
        hex32(&table.proposal_schema_migration_table_hash_v1)
    );
    s
}

/// Render the migration table as a deterministic JSON object.
#[must_use]
pub fn render_ff5_migration_table_json(table: &ProposalSchemaMigrationTable) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"schema\":\"{PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1}\""
    );
    let _ = write!(s, ",\"row_count\":{}", table.rows.len());
    let _ = write!(s, ",\"rows\":[");
    for (i, row) in table.rows.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"old_artifact_id\":\"{}\",\"old_schema_version\":\"{}\",\"new_schema_version\":\"{}\",\"old_artifact_hash\":\"{}\",\"new_artifact_hash\":\"{}\",\"reason_byte_diff\":\"{}\"}}",
            json_escape(row.old_artifact_id),
            json_escape(row.old_schema_version),
            json_escape(row.new_schema_version),
            hex32(&row.old_artifact_hash),
            hex32(&row.new_artifact_hash),
            json_escape(row.reason_byte_diff)
        );
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"proposal_schema_migration_table_hash_v1\":\"{}\"",
        hex32(&table.proposal_schema_migration_table_hash_v1)
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
