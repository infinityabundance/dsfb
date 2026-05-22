// Test-file-level clippy relaxation: expect/unwrap/panic patterns are
// idiomatic in tests where a fixture-read failure should abort the test
// loudly, and format-push-string is fine for one-shot hex assembly.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::format_push_string,
    clippy::missing_docs_in_private_items
)]

//! S-REAL.1 — replay byte-identity acceptance test (tier-scoped).
//!
//! WHY: The S-REAL.1 audit gauntlet's central credibility claim is that
//! two consecutive dispatches on the same SHA-pinned fixture bytes produce
//! byte-identical case-file + episodes + audit-report artifacts. This
//! acceptance test pins that claim against the committed receipts under
//! `reports/s_real_1/<dataset>/`.
//!
//! Scope (panel-locked, post-S-REAL.3): this test covers ONLY the
//! original S-REAL.1 tier — 3 datasets × 8 artifacts. The full
//! 20-dataset × 9-artifact bundle surface (S-REAL.1 + S-REAL.2 +
//! S-REAL.3 combined, with the S-REAL.PERF `perf_profile.txt`
//! addition bringing the per-dataset artifact count from 8 to 9) is
//! covered by the sibling `s_real_3_bundle_integrity.rs` test, which
//! re-hashes every row of `reports/s_real_3/bundle_hash_chain.txt`
//! against the live bytes on disk. Keeping the S-REAL.1 test
//! tier-scoped preserves a clean per-tier admission story: each
//! sealed tier has its own byte-identity gate that does not couple
//! to later tiers' artifact counts.
//!
//! Design choice: the test does NOT re-invoke CUDA (a) because requiring
//! CUDA at test-time would make the acceptance suite hostile to
//! non-GPU hosts and (b) because the replay-byte-identity claim
//! is already proven by the committed `replay_verification.txt`
//! receipts, which were produced by an actual dispatcher run and carry
//! per-artifact SHA-256 receipts for both runs. The test instead
//! verifies:
//!
//! 1. All 24 S-REAL.1 audit artifacts (3 datasets × 8 each) exist
//!    and are non-empty.
//! 2. Every `replay_verification.txt` contains the literal admission
//!    line `byte-identical replay: YES` and individual YES marks for
//!    casefile.json, episodes.jsonl, and audit_report.html.
//! 3. Every `dataset_manifest.toml` still pins the SHA-256 the live
//!    vendored fixture bytes hash to.
//! 4. The committed casefile.json + episodes.jsonl + audit_report.html
//!    SHA-256s match the values recorded inside replay_verification.txt
//!    (no on-disk tampering between commit time and now).
//!
//! License: Apache-2.0. Background IP: Invariant Forge LLC.

use std::fs;
use std::path::{Path, PathBuf};

use dsfb_gpu_debug_core::hash::sha256;

const ARTIFACTS: &[&str] = &[
    "dataset_manifest.toml",
    "schema_map.toml",
    "run_receipt.txt",
    "casefile.json",
    "episodes.jsonl",
    "audit_report.html",
    "replay_verification.txt",
    "limitations.md",
];

const DATASETS: &[&str] = &["aiops_kpi", "illinois_socialnet", "tadbench_f11"];

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to the demo crate root; the workspace
    // root is two levels up.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn dataset_dir(dataset: &str) -> PathBuf {
    workspace_root()
        .join("reports")
        .join("s_real_1")
        .join(dataset)
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn hex(b: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for byte in b {
        s.push_str(&format!("{byte:02x}"));
    }
    s
}

#[test]
fn s_real_1_audit_emits_24_artifacts() {
    for dataset in DATASETS {
        let dir = dataset_dir(dataset);
        assert!(
            dir.is_dir(),
            "{} is not a directory; run `s-real-1-audit --dataset {dataset}` first",
            dir.display()
        );
        for artifact in ARTIFACTS {
            let p = dir.join(artifact);
            let bytes = read_bytes(&p);
            assert!(!bytes.is_empty(), "{} is empty", p.display());
        }
    }
}

#[test]
fn s_real_1_each_dataset_admits_byte_identical_replay() {
    for dataset in DATASETS {
        let path = dataset_dir(dataset).join("replay_verification.txt");
        let text = read_text(&path);
        assert!(
            text.contains("byte-identical replay: YES"),
            "{} does not admit byte-identical replay:\n{text}",
            path.display()
        );
        for sub in [
            "casefile.json:        YES",
            "episodes.jsonl:       YES",
            "audit_report.html:    YES",
        ] {
            assert!(
                text.contains(sub),
                "{} missing per-artifact admission line: {sub}",
                path.display()
            );
        }
    }
}

#[test]
fn s_real_1_each_dataset_pin_matches_live_vendored_bytes() {
    // Vendored fixtures live in the dsfb-gpu repo under
    // `data/fixtures/`. Paths are resolved relative to the workspace
    // root (CARGO_MANIFEST_DIR / ..) so the test runs without
    // depending on any external repository being mounted at a fixed
    // absolute path.
    let vendored: &[(&str, &str)] = &[
        ("aiops_kpi", "data/fixtures/aiops_challenge.tsv"),
        (
            "illinois_socialnet",
            "data/fixtures/illinois_socialnetwork.tsv",
        ),
        ("tadbench_f11", "data/fixtures/tadbench_trainticket_F11.tsv"),
    ];
    for (dataset, rel_path) in vendored {
        let manifest_path = dataset_dir(dataset).join("dataset_manifest.toml");
        let manifest = read_text(&manifest_path);
        let abs_path = workspace_root().join(rel_path);
        let actual_bytes = read_bytes(&abs_path);
        let actual_hex = hex(&sha256(&actual_bytes));
        assert!(
            manifest.contains(&actual_hex),
            "{}: manifest does not carry the live vendored SHA-256 {actual_hex}",
            manifest_path.display()
        );
    }
}

#[test]
fn s_real_1_committed_artifacts_match_recorded_replay_sha256() {
    // The replay_verification.txt records the SHA-256 of each artifact
    // (run 1 == run 2 by the byte-identity claim). If a reader tampered
    // with casefile.json / episodes.jsonl / audit_report.html on disk
    // after the audit commit landed, the recomputed hash would no
    // longer match the recorded one and this test would fail.
    for dataset in DATASETS {
        let dir = dataset_dir(dataset);
        let replay = read_text(&dir.join("replay_verification.txt"));

        for (artifact, label_in_replay) in [
            ("casefile.json", "casefile.json:        "),
            ("episodes.jsonl", "episodes.jsonl:       "),
            ("audit_report.html", "audit_report.html:    "),
        ] {
            let bytes = read_bytes(&dir.join(artifact));
            let actual_hex = hex(&sha256(&bytes));

            // The replay file lists each hash twice (Run 1 and Run 2 blocks).
            // Verify both occurrences match the live on-disk SHA-256.
            let occurrences = replay
                .lines()
                .filter(|line| {
                    line.contains(label_in_replay)
                        && !line.trim_end().ends_with("YES")
                        && !line.trim_end().ends_with("NO")
                })
                .filter_map(|line| line.split_whitespace().last())
                .collect::<Vec<&str>>();
            assert!(
                occurrences.len() >= 2,
                "{}/replay_verification.txt has fewer than 2 SHA-256 lines for {artifact}",
                dir.display()
            );
            for recorded in &occurrences {
                assert_eq!(
                    *recorded,
                    actual_hex,
                    "{}/{} live SHA-256 {actual_hex} does not match recorded {recorded}",
                    dir.display(),
                    artifact
                );
            }
        }
    }
}
