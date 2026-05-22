//! FF.5 acceptance suite — proposal-schema upgrade policy
//! invariants for the forward-looking governance contract.
//!
//! Nine panel-required load-bearing negatives pin the
//! discipline FF.5 exists to prove:
//!
//! * `ff5_rejects_schema_rerender_without_old_hash`
//! * `ff5_rejects_schema_rerender_without_new_schema_hash`
//! * `ff5_rejects_schema_rerender_without_migration_table`
//! * `ff5_rejects_schema_rerender_without_reason`
//! * `ff5_rejects_migration_table_with_duplicate_old_hash`
//! * `ff5_rejects_migration_table_with_duplicate_new_hash`
//! * `ff5_rejects_claim_that_old_artifact_hash_was_invalid`
//! * `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v1`
//! * `ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v2_without_freeze_campaign`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > Schema upgrade != silent artifact rewrite.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
};
use dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use dsfb_gpu_atlas_corpus::ff4_readme_authority_boundary::build_ff4_readme_authority_boundary_policy;
use dsfb_gpu_atlas_corpus::proposal_schema_policy::{
    build_empty_migration_table, build_migration_table_from_rows,
    build_proposal_schema_upgrade_policy, build_schema_upgrade_receipt,
    render_ff5_migration_table_json, render_ff5_migration_table_text, render_ff5_policy_json,
    render_ff5_policy_text, verify_migration_table, verify_schema_upgrade_receipt, Ff5VerifyError,
    Ff5VerifyErrorKind, MigrationRow, FF5_POLICY_DOCTRINE_LINES,
    PROPOSAL_SCHEMA_MIGRATION_TABLE_DOMAIN_V1, PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1,
    PROPOSAL_SCHEMA_UPGRADE_POLICY_DOMAIN_V1, PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1,
    SCHEMA_UPGRADE_RECEIPT_DOMAIN_V1, SCHEMA_UPGRADE_RECEIPT_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

fn canonical_good_row() -> MigrationRow {
    MigrationRow {
        old_artifact_id: "t12_x_example",
        old_schema_version: "v1",
        new_schema_version: "v2",
        old_artifact_hash: [0x11; 32],
        new_artifact_hash: [0x22; 32],
        reason_byte_diff: "ProposedSourceRef gained authors + doi_or_url fields.",
    }
}

fn canonical_good_receipt(
) -> dsfb_gpu_atlas_corpus::proposal_schema_policy::ProposalSchemaUpgradeReceipt {
    build_schema_upgrade_receipt(
        "corpus_amendment_proposal_v1_to_v2",
        "v1",
        "v2",
        "Extend ProposedSourceRef with authors + doi_or_url + source_kind.",
        vec![canonical_good_row()],
        true,
        true,
        None,
        true,
    )
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #1
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_rerender_without_old_hash() {
    let bad_row = MigrationRow {
        old_artifact_hash: [0u8; 32],
        ..canonical_good_row()
    };
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![bad_row],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaRerenderWithoutOldHash { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #2
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_rerender_without_new_schema_hash() {
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "",
        "Reason.",
        vec![canonical_good_row()],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaRerenderWithoutNewSchemaHash { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #3
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_rerender_without_migration_table() {
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaRerenderWithoutMigrationTable { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #4
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_rerender_without_reason() {
    let receipt_no_reason = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "",
        vec![canonical_good_row()],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt_no_reason);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaRerenderWithoutReason { .. }
    )));

    let bad_row = MigrationRow {
        reason_byte_diff: "",
        ..canonical_good_row()
    };
    let receipt_no_row_reason = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![bad_row],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt_no_row_reason);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaRerenderWithoutReason { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #5
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_migration_table_with_duplicate_old_hash() {
    let row1 = MigrationRow {
        old_artifact_id: "art_a",
        old_artifact_hash: [0x33; 32],
        new_artifact_hash: [0x44; 32],
        ..canonical_good_row()
    };
    let row2 = MigrationRow {
        old_artifact_id: "art_b",
        old_artifact_hash: [0x33; 32],
        new_artifact_hash: [0x55; 32],
        ..canonical_good_row()
    };
    let table = build_migration_table_from_rows(vec![row1, row2]);
    let errs = verify_migration_table(&table);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::MigrationTableWithDuplicateOldHash { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #6
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_migration_table_with_duplicate_new_hash() {
    let row1 = MigrationRow {
        old_artifact_id: "art_a",
        old_artifact_hash: [0x66; 32],
        new_artifact_hash: [0x77; 32],
        ..canonical_good_row()
    };
    let row2 = MigrationRow {
        old_artifact_id: "art_b",
        old_artifact_hash: [0x88; 32],
        new_artifact_hash: [0x77; 32],
        ..canonical_good_row()
    };
    let table = build_migration_table_from_rows(vec![row1, row2]);
    let errs = verify_migration_table(&table);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::MigrationTableWithDuplicateNewHash { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #7
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_claim_that_old_artifact_hash_was_invalid() {
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        true,
        None,
        false, // declares_old_artifact_hash_valid = false
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::ClaimThatOldArtifactHashWasInvalid { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #8
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v1() {
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        false, // preserves_corpus_hash_v1 = false
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaUpgradeThatMutatesCorpusHashV1 { .. }
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #9
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_schema_upgrade_that_mutates_corpus_hash_v2_without_freeze_campaign() {
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        false, // preserves_corpus_hash_v2 = false
        None,  // and no freeze_campaign_id
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::SchemaUpgradeThatMutatesCorpusHashV2WithoutFreezeCampaign { .. }
    )));
}

#[test]
fn ff5_admits_schema_upgrade_that_mutates_corpus_hash_v2_with_freeze_campaign() {
    let receipt = build_schema_upgrade_receipt(
        "good_corpus_v3_freeze",
        "v1",
        "v2",
        "Re-freeze campaign producing corpus_hash_v3.",
        vec![canonical_good_row()],
        true,
        false,
        Some("corpus_hash_v3_freeze_campaign"),
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(
        errs.is_empty(),
        "freeze-campaign-bearing receipt must verify: {errs:?}"
    );
}

// ---------------------------------------------------------------
// Canonical good receipt verifies
// ---------------------------------------------------------------

#[test]
fn ff5_canonical_good_receipt_verifies() {
    let r = canonical_good_receipt();
    let errs = verify_schema_upgrade_receipt(&r);
    assert!(
        errs.is_empty(),
        "canonical good receipt must verify: {errs:?}"
    );
}

#[test]
fn ff5_empty_migration_table_verifies() {
    let table = build_empty_migration_table();
    let errs = verify_migration_table(&table);
    assert!(errs.is_empty());
}

// ---------------------------------------------------------------
// Default policy invariants
// ---------------------------------------------------------------

#[test]
fn ff5_default_policy_has_empty_migration_table() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(p.migration_table.rows.len(), 0);
}

#[test]
fn ff5_default_policy_has_canonical_doctrine() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(p.doctrine_lines, FF5_POLICY_DOCTRINE_LINES);
}

#[test]
fn ff5_doctrine_lines_non_empty() {
    assert!(!FF5_POLICY_DOCTRINE_LINES.is_empty());
    for line in FF5_POLICY_DOCTRINE_LINES {
        assert!(!line.is_empty());
    }
}

#[test]
fn ff5_doctrine_first_line_is_core_rule() {
    assert_eq!(
        FF5_POLICY_DOCTRINE_LINES[0],
        "Schema upgrade != silent artifact rewrite."
    );
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn ff5_policy_hash_is_deterministic_across_two_builds() {
    let a = build_proposal_schema_upgrade_policy();
    let b = build_proposal_schema_upgrade_policy();
    assert_eq!(
        a.proposal_schema_upgrade_policy_hash_v1,
        b.proposal_schema_upgrade_policy_hash_v1
    );
}

#[test]
fn ff5_migration_table_hash_is_deterministic_across_two_builds() {
    let a = build_empty_migration_table();
    let b = build_empty_migration_table();
    assert_eq!(
        a.proposal_schema_migration_table_hash_v1,
        b.proposal_schema_migration_table_hash_v1
    );
}

#[test]
fn ff5_receipt_hash_is_deterministic_across_two_builds() {
    let r1 = canonical_good_receipt();
    let r2 = canonical_good_receipt();
    assert_eq!(
        r1.schema_upgrade_receipt_hash_v1,
        r2.schema_upgrade_receipt_hash_v1
    );
}

#[test]
fn ff5_policy_text_render_byte_stable() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(render_ff5_policy_text(&p), render_ff5_policy_text(&p));
}

#[test]
fn ff5_policy_json_render_byte_stable() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(render_ff5_policy_json(&p), render_ff5_policy_json(&p));
}

#[test]
fn ff5_migration_table_text_render_byte_stable() {
    let t = build_empty_migration_table();
    assert_eq!(
        render_ff5_migration_table_text(&t),
        render_ff5_migration_table_text(&t)
    );
}

#[test]
fn ff5_migration_table_json_render_byte_stable() {
    let t = build_empty_migration_table();
    assert_eq!(
        render_ff5_migration_table_json(&t),
        render_ff5_migration_table_json(&t)
    );
}

// ---------------------------------------------------------------
// Sensitivity
// ---------------------------------------------------------------

#[test]
fn ff5_migration_table_hash_changes_when_rows_added() {
    let empty = build_empty_migration_table();
    let single = build_migration_table_from_rows(vec![canonical_good_row()]);
    assert_ne!(
        empty.proposal_schema_migration_table_hash_v1,
        single.proposal_schema_migration_table_hash_v1
    );
}

#[test]
fn ff5_receipt_hash_changes_when_freeze_campaign_added() {
    let bare = build_schema_upgrade_receipt(
        "upgrade_x",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        true,
        None,
        true,
    );
    let with_freeze = build_schema_upgrade_receipt(
        "upgrade_x",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        false,
        Some("freeze_campaign_v3"),
        true,
    );
    assert_ne!(
        bare.schema_upgrade_receipt_hash_v1,
        with_freeze.schema_upgrade_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Upstream-anchor invariance
// ---------------------------------------------------------------

#[test]
fn ff5_does_not_mutate_corpus_hash_v1() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_proposal_schema_upgrade_policy();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_corpus_hash_v2() {
    let before = build_consolidation_report().corpus_hash_v2;
    let _ = build_proposal_schema_upgrade_policy();
    let after = build_consolidation_report().corpus_hash_v2;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let before = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    let _ = build_proposal_schema_upgrade_policy();
    let after = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_ff2_gate_hash() {
    let r = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&r);
    let ids = default_candidate_ids(&idx);
    let before = build_ff2_activation_ratification_gate_from(&r, &idx, &ids)
        .ff2_activation_ratification_gate_hash_v1;
    let _ = build_proposal_schema_upgrade_policy();
    let after = build_ff2_activation_ratification_gate_from(&r, &idx, &ids)
        .ff2_activation_ratification_gate_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_ff3_gate_hash() {
    let before = build_ff3_registry_generation_gate().ff3_registry_generation_gate_hash_v1;
    let _ = build_proposal_schema_upgrade_policy();
    let after = build_ff3_registry_generation_gate().ff3_registry_generation_gate_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_ff4_policy_hash() {
    let before =
        build_ff4_readme_authority_boundary_policy().ff4_readme_authority_boundary_policy_hash_v1;
    let _ = build_proposal_schema_upgrade_policy();
    let after =
        build_ff4_readme_authority_boundary_policy().ff4_readme_authority_boundary_policy_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff5_does_not_mutate_seed_len() {
    let before = SEED.len();
    let _ = build_proposal_schema_upgrade_policy();
    let after = SEED.len();
    assert_eq!(before, 54);
    assert_eq!(after, 54);
}

// ---------------------------------------------------------------
// Pinned anchor cross-checks
// ---------------------------------------------------------------

#[test]
fn ff5_policy_pins_live_corpus_hash_v1() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(p.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn ff5_policy_pins_live_corpus_hash_v2() {
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(
        p.corpus_hash_v2,
        build_consolidation_report().corpus_hash_v2
    );
}

#[test]
fn ff5_policy_pins_live_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(
        p.ff1_passport_index_hash_v1,
        build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1
    );
}

#[test]
fn ff5_policy_pins_live_ff2_gate_hash() {
    let r = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&r);
    let ids = default_candidate_ids(&idx);
    let live = build_ff2_activation_ratification_gate_from(&r, &idx, &ids);
    let p = build_proposal_schema_upgrade_policy();
    assert_eq!(
        p.ff2_activation_ratification_gate_hash_v1,
        live.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff5_policy_pins_live_ff3_gate_hash() {
    let p = build_proposal_schema_upgrade_policy();
    let live = build_ff3_registry_generation_gate();
    assert_eq!(
        p.ff3_registry_generation_gate_hash_v1,
        live.ff3_registry_generation_gate_hash_v1
    );
}

#[test]
fn ff5_policy_pins_live_ff4_policy_hash() {
    let p = build_proposal_schema_upgrade_policy();
    let live = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        live.ff4_readme_authority_boundary_policy_hash_v1
    );
}

// ---------------------------------------------------------------
// Pinned constants
// ---------------------------------------------------------------

#[test]
fn ff5_policy_domain_pin() {
    assert_eq!(
        PROPOSAL_SCHEMA_UPGRADE_POLICY_DOMAIN_V1,
        "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1\0"
    );
}

#[test]
fn ff5_policy_schema_pin() {
    assert_eq!(
        PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1,
        "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-UPGRADE-POLICY:v1"
    );
}

#[test]
fn ff5_migration_table_domain_pin() {
    assert_eq!(
        PROPOSAL_SCHEMA_MIGRATION_TABLE_DOMAIN_V1,
        "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1\0"
    );
}

#[test]
fn ff5_migration_table_schema_pin() {
    assert_eq!(
        PROPOSAL_SCHEMA_MIGRATION_TABLE_SCHEMA_V1,
        "DSFB-GPU-ATLAS:PROPOSAL-SCHEMA-MIGRATION-TABLE:v1"
    );
}

#[test]
fn ff5_receipt_domain_pin() {
    assert_eq!(
        SCHEMA_UPGRADE_RECEIPT_DOMAIN_V1,
        "DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1\0"
    );
}

#[test]
fn ff5_receipt_schema_pin() {
    assert_eq!(
        SCHEMA_UPGRADE_RECEIPT_SCHEMA_V1,
        "DSFB-GPU-ATLAS:SCHEMA-UPGRADE-RECEIPT:v1"
    );
}

// ---------------------------------------------------------------
// Structural defect rules
// ---------------------------------------------------------------

#[test]
fn ff5_rejects_migration_row_with_identical_schema_versions() {
    let bad_row = MigrationRow {
        old_schema_version: "v1",
        new_schema_version: "v1",
        ..canonical_good_row()
    };
    let receipt = build_schema_upgrade_receipt(
        "no_op_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![bad_row],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::MigrationRowWithIdenticalSchemaVersions { .. }
    )));
}

#[test]
fn ff5_rejects_receipt_with_empty_upgrade_id() {
    let receipt = build_schema_upgrade_receipt(
        "",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff5VerifyErrorKind::ReceiptWithEmptyUpgradeId)));
}

#[test]
fn ff5_rejects_receipt_with_empty_freeze_campaign_id() {
    let receipt = build_schema_upgrade_receipt(
        "bad_freeze",
        "v1",
        "v2",
        "Reason.",
        vec![canonical_good_row()],
        true,
        false,
        Some(""),
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::ReceiptFreezeCampaignIdEmpty { .. }
    )));
}

#[test]
fn ff5_rejects_migration_row_with_empty_artifact_id() {
    let bad_row = MigrationRow {
        old_artifact_id: "",
        ..canonical_good_row()
    };
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![bad_row],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff5VerifyErrorKind::MigrationRowWithEmptyArtifactId)));
}

#[test]
fn ff5_rejects_migration_row_with_empty_schema_version() {
    let bad_row = MigrationRow {
        old_schema_version: "",
        ..canonical_good_row()
    };
    let receipt = build_schema_upgrade_receipt(
        "bad_upgrade",
        "v1",
        "v2",
        "Reason.",
        vec![bad_row],
        true,
        true,
        None,
        true,
    );
    let errs = verify_schema_upgrade_receipt(&receipt);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff5VerifyErrorKind::MigrationRowWithEmptySchemaVersion { .. }
    )));
}

// ---------------------------------------------------------------
// Hash-namespace distinctness
// ---------------------------------------------------------------

#[test]
fn ff5_policy_hash_distinct_from_migration_table_hash() {
    let p = build_proposal_schema_upgrade_policy();
    assert_ne!(
        p.proposal_schema_upgrade_policy_hash_v1,
        p.migration_table.proposal_schema_migration_table_hash_v1
    );
}

#[test]
fn ff5_policy_hash_distinct_from_corpus_hash_v1() {
    let p = build_proposal_schema_upgrade_policy();
    assert_ne!(p.proposal_schema_upgrade_policy_hash_v1, p.corpus_hash_v1);
}

#[test]
fn ff5_policy_hash_distinct_from_corpus_hash_v2() {
    let p = build_proposal_schema_upgrade_policy();
    assert_ne!(p.proposal_schema_upgrade_policy_hash_v1, p.corpus_hash_v2);
}

#[test]
fn ff5_policy_hash_distinct_from_ff4_policy_hash() {
    let p = build_proposal_schema_upgrade_policy();
    assert_ne!(
        p.proposal_schema_upgrade_policy_hash_v1,
        p.ff4_readme_authority_boundary_policy_hash_v1
    );
}

#[test]
fn ff5_receipt_hash_distinct_from_policy_hash() {
    let p = build_proposal_schema_upgrade_policy();
    let r = canonical_good_receipt();
    assert_ne!(
        r.schema_upgrade_receipt_hash_v1,
        p.proposal_schema_upgrade_policy_hash_v1
    );
}

// ---------------------------------------------------------------
// Render coverage
// ---------------------------------------------------------------

#[test]
fn ff5_render_text_contains_pinned_anchors_and_doctrine() {
    let p = build_proposal_schema_upgrade_policy();
    let text = render_ff5_policy_text(&p);
    assert!(text.contains("FF.5 Proposal Schema Upgrade Policy"));
    assert!(text.contains("corpus_hash_v1"));
    assert!(text.contains("corpus_hash_v2"));
    assert!(text.contains("ff1_passport_index_hash_v1"));
    assert!(text.contains("ff2_activation_ratification_gate_hash_v1"));
    assert!(text.contains("ff3_registry_generation_gate_hash_v1"));
    assert!(text.contains("ff4_readme_authority_boundary_policy_hash_v1"));
    assert!(text.contains("Schema upgrade != silent artifact rewrite."));
    assert!(text.contains("proposal_schema_upgrade_policy_hash_v1"));
}

#[test]
fn ff5_render_json_contains_schema_field() {
    let p = build_proposal_schema_upgrade_policy();
    let json = render_ff5_policy_json(&p);
    assert!(json.contains(PROPOSAL_SCHEMA_UPGRADE_POLICY_SCHEMA_V1));
}

#[test]
fn ff5_migration_table_render_text_indicates_zero_rows_at_baseline() {
    let t = build_empty_migration_table();
    let text = render_ff5_migration_table_text(&t);
    assert!(text.contains("Row count: 0"));
    assert!(text.contains("proposal_schema_migration_table_hash_v1"));
}

// ---------------------------------------------------------------
// Verify-all-clean baseline
// ---------------------------------------------------------------

#[test]
fn ff5_default_build_admissible_under_table_verifier() {
    let p = build_proposal_schema_upgrade_policy();
    let errs: Vec<Ff5VerifyError> = verify_migration_table(&p.migration_table);
    assert!(errs.is_empty(), "default table must verify: {errs:?}");
}
