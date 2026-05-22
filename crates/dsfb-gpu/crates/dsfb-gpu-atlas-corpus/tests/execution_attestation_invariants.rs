//! T.11e acceptance tests for the execution-attestation receipt.
//!
//! Panel-required invariants (30+ including two load-bearing
//! negatives:
//! `execution_attestation_rejects_slsa_compliance_claim` and
//! `execution_attestation_rejects_dirty_repo_without_override`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::execution_attestation::{
    build_t11e_test_fixture, compute_execution_attestation_hash_v1,
    render_execution_attestation_json, render_execution_attestation_text,
    verify_execution_attestation, AttestationNonClaim, AttestationVerifyErrorKind, CommandPurpose,
    ExecutedCommand, ExecutionAttestationSchema, GateSummary, MaterialDigest, MaterialKind,
    R12bEpisodeCounts, SubjectDigest, SubjectKind, TestSummary, EXECUTION_ATTESTATION_SCHEMA_V1,
};

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn fixture_is_deterministic_across_two_builds() {
    let a = build_t11e_test_fixture();
    let b = build_t11e_test_fixture();
    assert_eq!(a.receipt_hash_v1, b.receipt_hash_v1);
}

#[test]
fn fixture_verifies_clean() {
    let r = build_t11e_test_fixture();
    let errors = verify_execution_attestation(&r);
    assert!(
        errors.is_empty(),
        "fixture must verify clean; got {errors:?}"
    );
}

#[test]
fn schema_wire_name_is_panel_locked() {
    assert_eq!(
        ExecutionAttestationSchema::V1UnsignedDsfbNative.as_str(),
        "V1UnsignedDsfbNative"
    );
    assert_eq!(
        EXECUTION_ATTESTATION_SCHEMA_V1,
        "DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1"
    );
}

#[test]
fn fixture_pins_panel_locked_hashes() {
    let r = build_t11e_test_fixture();
    assert_eq!(
        hex(&r.corpus_hash_v1),
        "35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7"
    );
    assert_eq!(
        hex(&r.registry_hash_v2),
        "d3cf63000cee922818e8dbc79ffecbc27d288063efbaed589e1eb1812bc37a08"
    );
    assert_eq!(
        hex(&r.precedent_hash_v1),
        "6721f511f1eb951ba7eff4fa36832f233331507f6e4208d4f97866afd984dd14"
    );
    assert_eq!(
        hex(&r.admissibility_grammar_hash_v1),
        "ff66706a726d0cddc5f343e21f2ffbd8f81392a1504ff1b2002f8609d14a5ba7"
    );
    assert_eq!(
        hex(&r.trial_transcript_hash_v1),
        "37618a45c1e60da3bb66ddae4161d94ed762287483caf88c21a5db3cff64bbee"
    );
}

#[test]
fn fixture_includes_all_required_non_claims() {
    let r = build_t11e_test_fixture();
    for required in AttestationNonClaim::all_required() {
        assert!(
            r.non_claims.contains(&required),
            "non_claims missing required entry {}",
            required.as_str()
        );
    }
}

#[test]
fn fixture_does_not_claim_slsa_level() {
    let r = build_t11e_test_fixture();
    assert_eq!(r.claimed_slsa_level, None);
}

#[test]
fn fixture_does_not_claim_signed_attestation() {
    let r = build_t11e_test_fixture();
    assert!(!r.claimed_signed_attestation);
}

#[test]
fn fixture_r12b_episode_counts_match_baseline() {
    let r = build_t11e_test_fixture();
    assert_eq!(r.r12b_episode_counts, R12bEpisodeCounts::baseline());
    assert_eq!(r.r12b_episode_counts.canonical_16x128, 13);
    assert_eq!(r.r12b_episode_counts.mid_64x512, 89);
    assert_eq!(r.r12b_episode_counts.full_256x4096, 1917);
}

#[test]
fn fixture_test_summary_records_zero_failed() {
    let r = build_t11e_test_fixture();
    assert_eq!(r.test_summary.workspace_failed, 0);
}

#[test]
fn fixture_gate_summary_is_fully_clean() {
    let r = build_t11e_test_fixture();
    let g = r.gate_summary;
    assert!(g.fmt_clean);
    assert!(g.clippy_clean);
    assert!(g.scrub_clean);
    assert!(g.docs_freshness_clean);
    assert!(g.r12_byte_stability_clean);
}

#[test]
fn fixture_verification_commands_cover_required_purposes() {
    let r = build_t11e_test_fixture();
    let required = [
        CommandPurpose::Format,
        CommandPurpose::Clippy,
        CommandPurpose::Scrub,
        CommandPurpose::DocsFreshness,
        CommandPurpose::WorkspaceTest,
    ];
    for p in required {
        assert!(
            r.verification_commands.iter().any(|c| c.purpose == p),
            "missing required verification purpose {}",
            p.as_str()
        );
    }
}

#[test]
fn fixture_records_all_five_hash_chain_anchors_as_materials() {
    let r = build_t11e_test_fixture();
    let names: Vec<&str> = r.materials.iter().map(|m| m.name.as_str()).collect();
    for required in [
        "corpus_hash_v1",
        "registry_hash_v2",
        "precedent_hash_v1",
        "admissibility_grammar_hash_v1",
    ] {
        assert!(
            names.contains(&required),
            "materials missing entry `{required}`"
        );
    }
}

#[test]
fn fixture_records_subjects_for_transcript_grammar_precedent() {
    let r = build_t11e_test_fixture();
    let names: Vec<&str> = r.subjects.iter().map(|s| s.name.as_str()).collect();
    for required in [
        "trial_transcript_v1",
        "admissibility_grammar_v1",
        "court_precedents_v1",
    ] {
        assert!(
            names.contains(&required),
            "subjects missing entry `{required}`"
        );
    }
}

#[test]
fn hash_changes_when_corpus_hash_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.corpus_hash_v1[0] ^= 0xFF;
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_registry_hash_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.registry_hash_v2[0] ^= 0xFF;
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_precedent_hash_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.precedent_hash_v1[0] ^= 0xFF;
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_grammar_hash_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.admissibility_grammar_hash_v1[0] ^= 0xFF;
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_transcript_hash_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.trial_transcript_hash_v1[0] ^= 0xFF;
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_repo_commit_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.repo_commit = "abcd1234abcd1234abcd1234abcd1234abcd1234".to_string();
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn hash_changes_when_command_list_changes() {
    let mut r = build_t11e_test_fixture();
    let baseline = r.receipt_hash_v1;
    r.build_commands.push(ExecutedCommand {
        command: "cargo build --release".to_string(),
        purpose: CommandPurpose::Build,
        exit_code: 0,
    });
    let mutated = compute_execution_attestation_hash_v1(&r);
    assert_ne!(baseline, mutated);
}

#[test]
fn rendered_text_is_deterministic() {
    let r = build_t11e_test_fixture();
    let a = render_execution_attestation_text(&r);
    let b = render_execution_attestation_text(&r);
    assert_eq!(a, b);
}

#[test]
fn rendered_json_is_deterministic() {
    let r = build_t11e_test_fixture();
    let a = render_execution_attestation_json(&r);
    let b = render_execution_attestation_json(&r);
    assert_eq!(a, b);
}

#[test]
fn no_publication_language_in_t11e_reports() {
    let r = build_t11e_test_fixture();
    let text = render_execution_attestation_text(&r);
    let json = render_execution_attestation_json(&r);
    let forbidden = ["Zenodo", "DOI", "publication-grade", "peer-reviewed"];
    for body in [&text, &json] {
        for word in forbidden {
            assert!(
                !body.contains(word),
                "T.11e artifact contains forbidden word `{word}`"
            );
        }
    }
}

#[test]
fn rendered_text_mentions_non_claims() {
    let r = build_t11e_test_fixture();
    let text = render_execution_attestation_text(&r);
    // Operator-facing rendering MUST surface each non-claim.
    assert!(text.contains("NotSlsaComplianceClaim"));
    assert!(text.contains("NotInTotoSignedStatement"));
    assert!(text.contains("UnsignedLocalReceipt"));
}

#[test]
fn execution_attestation_rejects_slsa_compliance_claim() {
    // Panel-required load-bearing negative.
    let mut r = build_t11e_test_fixture();
    r.claimed_slsa_level = Some(3);
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == AttestationVerifyErrorKind::ClaimedSlsaLevelPresent),
        "verifier MUST reject any non-None claimed_slsa_level"
    );
}

#[test]
fn execution_attestation_rejects_dirty_repo_without_override() {
    // Panel-required load-bearing negative.
    let mut r = build_t11e_test_fixture();
    r.repo_dirty = true;
    r.dirty_override_acknowledged = false;
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == AttestationVerifyErrorKind::DirtyRepoWithoutOverride),
        "verifier MUST reject dirty-tree attestations without explicit override"
    );
}

#[test]
fn dirty_repo_with_override_is_admitted() {
    // The override allows diagnostic dirty-tree snapshots.
    let mut r = build_t11e_test_fixture();
    r.repo_dirty = true;
    r.dirty_override_acknowledged = true;
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(!errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::DirtyRepoWithoutOverride));
}

#[test]
fn verifier_rejects_claimed_signed_attestation_true() {
    let mut r = build_t11e_test_fixture();
    r.claimed_signed_attestation = true;
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ClaimedSignedAttestation));
}

#[test]
fn verifier_rejects_zero_corpus_hash() {
    let mut r = build_t11e_test_fixture();
    r.corpus_hash_v1 = [0u8; 32];
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ZeroCorpusHash));
}

#[test]
fn verifier_rejects_zero_registry_hash() {
    let mut r = build_t11e_test_fixture();
    r.registry_hash_v2 = [0u8; 32];
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ZeroRegistryHash));
}

#[test]
fn verifier_rejects_zero_precedent_hash() {
    let mut r = build_t11e_test_fixture();
    r.precedent_hash_v1 = [0u8; 32];
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ZeroPrecedentHash));
}

#[test]
fn verifier_rejects_zero_grammar_hash() {
    let mut r = build_t11e_test_fixture();
    r.admissibility_grammar_hash_v1 = [0u8; 32];
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ZeroGrammarHash));
}

#[test]
fn verifier_rejects_zero_transcript_hash() {
    let mut r = build_t11e_test_fixture();
    r.trial_transcript_hash_v1 = [0u8; 32];
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ZeroTranscriptHash));
}

#[test]
fn verifier_rejects_empty_repo_commit() {
    let mut r = build_t11e_test_fixture();
    r.repo_commit.clear();
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::EmptyRepoCommit));
}

#[test]
fn verifier_rejects_invalid_repo_commit_format() {
    let mut r = build_t11e_test_fixture();
    r.repo_commit = "not-a-git-sha".to_string();
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::InvalidRepoCommitFormat));
}

#[test]
fn verifier_rejects_empty_build_commands() {
    let mut r = build_t11e_test_fixture();
    r.build_commands.clear();
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::EmptyBuildCommands));
}

#[test]
fn verifier_rejects_missing_required_gate_command() {
    let mut r = build_t11e_test_fixture();
    // Strip the Clippy gate command.
    r.verification_commands
        .retain(|c| c.purpose != CommandPurpose::Clippy);
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::MissingRequiredGateCommand));
}

#[test]
fn verifier_rejects_gate_not_clean() {
    let mut r = build_t11e_test_fixture();
    r.gate_summary = GateSummary {
        fmt_clean: true,
        clippy_clean: false,
        scrub_clean: true,
        docs_freshness_clean: true,
        r12_byte_stability_clean: true,
    };
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::GateNotClean));
}

#[test]
fn verifier_rejects_workspace_test_failed() {
    let mut r = build_t11e_test_fixture();
    r.test_summary = TestSummary {
        workspace_test_groups: 57,
        workspace_passed: 668,
        workspace_failed: 2,
        workspace_ignored: 0,
    };
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::WorkspaceTestFailed));
}

#[test]
fn verifier_rejects_r12b_episode_count_drift() {
    let mut r = build_t11e_test_fixture();
    r.r12b_episode_counts = R12bEpisodeCounts {
        canonical_16x128: 99,
        mid_64x512: 89,
        full_256x4096: 1917,
    };
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::R12bEpisodeCountsDrift));
}

#[test]
fn verifier_rejects_empty_subjects() {
    let mut r = build_t11e_test_fixture();
    r.subjects.clear();
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::SubjectDigestMissing));
}

#[test]
fn verifier_rejects_empty_materials() {
    let mut r = build_t11e_test_fixture();
    r.materials.clear();
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::MaterialDigestMissing));
}

#[test]
fn verifier_rejects_receipt_hash_mismatch() {
    let mut r = build_t11e_test_fixture();
    r.rustc_version = "rustc 99.99.99".to_string();
    // Do NOT recompute the hash; the verifier should catch.
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ReceiptHashMismatch));
}

#[test]
fn verifier_rejects_incomplete_non_claims() {
    let mut r = build_t11e_test_fixture();
    r.non_claims
        .retain(|c| *c != AttestationNonClaim::NotSlsaComplianceClaim);
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    let errors = verify_execution_attestation(&r);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::NonClaimsIncomplete));
}

#[test]
fn material_digest_kinds_cover_corpus_registry_precedent_grammar() {
    let r = build_t11e_test_fixture();
    let kinds: Vec<MaterialKind> = r.materials.iter().map(|m| m.kind).collect();
    for required in [
        MaterialKind::Corpus,
        MaterialKind::Registry,
        MaterialKind::Precedent,
        MaterialKind::Grammar,
    ] {
        assert!(
            kinds.contains(&required),
            "materials missing kind {}",
            required.as_str()
        );
    }
}

#[test]
fn subject_digest_kinds_cover_transcript_grammar_precedent() {
    let r = build_t11e_test_fixture();
    let kinds: Vec<SubjectKind> = r.subjects.iter().map(|s| s.kind).collect();
    for required in [
        SubjectKind::Transcript,
        SubjectKind::Grammar,
        SubjectKind::PrecedentLedger,
    ] {
        assert!(
            kinds.contains(&required),
            "subjects missing kind {}",
            required.as_str()
        );
    }
}

#[test]
fn enum_wire_names_cover_every_command_purpose() {
    let names = [
        CommandPurpose::Build.as_str(),
        CommandPurpose::Format.as_str(),
        CommandPurpose::Clippy.as_str(),
        CommandPurpose::Scrub.as_str(),
        CommandPurpose::DocsFreshness.as_str(),
        CommandPurpose::SinglePackageTest.as_str(),
        CommandPurpose::WorkspaceTest.as_str(),
        CommandPurpose::BulkEmit.as_str(),
        CommandPurpose::RegressionCheck.as_str(),
    ];
    for n in names {
        assert!(!n.is_empty());
    }
    assert_eq!(names.len(), 9);
}

#[test]
fn forge_unused_imports_to_keep_referenced() {
    let _ = MaterialDigest {
        kind: MaterialKind::Corpus,
        name: "x".to_string(),
        digest: [0u8; 32],
    };
    let _ = SubjectDigest {
        kind: SubjectKind::Receipt,
        name: "x".to_string(),
        digest: [0u8; 32],
    };
}

#[test]
fn corpus_hash_v1_unchanged_by_t11e() {
    use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
    let live = compute_corpus_hash_v1();
    assert_eq!(
        hex(&live.bytes),
        "35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7"
    );
}

#[test]
fn precedent_hash_v1_unchanged_by_t11e() {
    use dsfb_gpu_atlas_corpus::precedent::collect_court_precedents;
    let p = collect_court_precedents();
    assert_eq!(
        hex(&p.precedent_hash_v1),
        "6721f511f1eb951ba7eff4fa36832f233331507f6e4208d4f97866afd984dd14"
    );
}

#[test]
fn admissibility_grammar_hash_v1_unchanged_by_t11e() {
    use dsfb_gpu_atlas_corpus::admissibility::collect_admissibility_grammar;
    let g = collect_admissibility_grammar();
    assert_eq!(
        hex(&g.admissibility_grammar_hash_v1.0),
        "ff66706a726d0cddc5f343e21f2ffbd8f81392a1504ff1b2002f8609d14a5ba7"
    );
}

#[test]
fn trial_transcript_hash_v1_unchanged_by_t11e() {
    use dsfb_gpu_atlas_corpus::trial_transcript::build_t11d_latency_ramp_fixture;
    let t = build_t11d_latency_ramp_fixture();
    assert_eq!(
        hex(&t.trial_transcript_hash_v1),
        "37618a45c1e60da3bb66ddae4161d94ed762287483caf88c21a5db3cff64bbee"
    );
}

#[test]
fn t11e_attestation_does_not_replace_signed_attestation_pathway() {
    // Belt-and-braces: the receipt explicitly carries the
    // panel-locked non-claims, the renderer mentions
    // `NotInTotoSignedStatement`, and the verifier rejects any
    // attempt to set `claimed_signed_attestation = true`. This
    // test pins all three at once.
    let r = build_t11e_test_fixture();
    assert!(r
        .non_claims
        .contains(&AttestationNonClaim::NotInTotoSignedStatement));
    let text = render_execution_attestation_text(&r);
    assert!(text.contains("NotInTotoSignedStatement"));
    let mut forged = r.clone();
    forged.claimed_signed_attestation = true;
    forged.receipt_hash_v1 = compute_execution_attestation_hash_v1(&forged);
    let errors = verify_execution_attestation(&forged);
    assert!(errors
        .iter()
        .any(|e| e.kind == AttestationVerifyErrorKind::ClaimedSignedAttestation));
}
