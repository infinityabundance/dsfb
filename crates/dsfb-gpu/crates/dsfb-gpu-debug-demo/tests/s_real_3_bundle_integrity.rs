//! S-REAL.3.1 bundle integrity test — turns the S-REAL.3
//! 20-dataset Zenodo-publishable bundle from "reported as sealed"
//! into "continuously guarded by CI."
//!
//! WHY THIS TEST EXISTS (for the future engineer reading cold):
//!
//! The S-REAL.3 commit at `a8aaa04` shipped:
//!   reports/s_real_3/bundle_manifest.toml     (20 dataset records)
//!   reports/s_real_3/bundle_hash_chain.txt    (60 SHA-256 rows:
//!                                              20 datasets × 3
//!                                              byte-stable artifacts:
//!                                              casefile.json,
//!                                              dataset_manifest.toml,
//!                                              episodes.jsonl)
//!   reports/s_real_3/zenodo_metadata.json     (deposit template)
//!   reports/INDEX.md                          (executive surface)
//!
//! Plus the 20 per-dataset audit directories under
//!   reports/s_real_<1|2|3>/<dataset_id>/
//! each holding the 9 panel-locked artifacts: dataset_manifest.toml,
//! schema_map.toml, run_receipt.txt, casefile.json, episodes.jsonl,
//! audit_report.html, replay_verification.txt, limitations.md,
//! perf_profile.txt.
//!
//! Without a mechanical gate, a future commit could:
//!   - drift the bundle_manifest.toml total_admitted_episodes
//!     away from the actual sum across `[datasets.*]` entries
//!   - mutate a sealed casefile.json without updating the manifest's
//!     sealed_case_file_hash
//!   - corrupt the bundle_hash_chain.txt SHA-256 against live bytes
//!   - delete a per-dataset artifact (e.g. limitations.md)
//!   - flip a replay_verification.txt from "YES" without re-running
//!
//! This test runs in `cargo test` (no CUDA needed) and catches
//! every one of those drifts. It does NOT re-run the audit
//! pipeline; it asserts that what's CURRENTLY on disk matches
//! what bundle_manifest.toml CLAIMS is on disk.
//!
//! Panel-locked assertion contract:
//!
//!   bundle_manifest.toml:
//!     total_datasets             == 20
//!     total_admitted_episodes    == 316
//!     total_source_class_families == 5
//!     family distribution        F1=4, F2=4, F3=2, F4=6, F5=4
//!
//!   bundle_hash_chain.txt:
//!     row_count                  == 60 (20 datasets × 3 artifacts)
//!     every SHA-256              matches live file bytes
//!
//!   per dataset (20):
//!     tier_dir exists
//!     all 9 artifacts exist
//!     replay_verification.txt    contains "byte-identical replay: YES"
//!     casefile.json final_case_file_hash matches manifest.sealed_case_file_hash
//!     casefile.json episodes array length matches manifest.admitted_episodes
//!
//! No CUDA, no GPU, no audit binary re-run — pure file-bytes
//! verification. Runs in ~100ms.

// Test-file-level clippy relaxation: bundle-integrity assertions
// idiomatically panic on missing-file / parse-failure paths
// (a loud abort with a printed dataset id is the correct posture);
// the minimal in-test TOML parser uses bare-statement helpers that
// fire several "clean library code" lints which would only add
// noise for this verification-style test.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::uninlined_format_args,
    clippy::missing_docs_in_private_items,
    clippy::doc_lazy_continuation,
    clippy::map_unwrap_or,
    clippy::ref_option
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dsfb_gpu_debug_core::hash::sha256;

/// One `[datasets.<id>]` entry parsed from bundle_manifest.toml.
/// Each field is panel-locked: any future schema change MUST
/// update the parser below and the assertions in the test.
#[derive(Debug)]
struct ManifestRecord {
    id: String,
    family: String,
    tier_dir: String,
    admitted_episodes: u32,
    sealed_case_file_hash: String,
    replay_verified: bool,
}

/// Parse a tiny subset of TOML — enough for bundle_manifest.toml:
///   - `[section.subsection]` headers (we care about `datasets.<id>`)
///   - `key = value` lines with `value` either bare integer / bool
///     or a double-quoted string
/// The project elsewhere uses a hand-rolled TOML parser at
/// `crates/dsfb-gpu-debug-demo/src/cli/ingest.rs` for residual-
/// projection metadata. We re-implement an equally minimal parser
/// here (≈30 lines) to keep this test crate-self-contained and
/// dependency-free.
fn parse_bundle_records(text: &str) -> Vec<ManifestRecord> {
    let mut out = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current: BTreeMap<String, String> = BTreeMap::new();

    fn flush(out: &mut Vec<ManifestRecord>, id: &Option<String>, m: &BTreeMap<String, String>) {
        let Some(id) = id.as_ref() else { return };
        // Skip the `[bundle]` and `[families]` sections (their
        // "id" prefix is not `datasets.`); only emit
        // `datasets.<id>` records here.
        if !m.contains_key("__from_datasets__") {
            return;
        }
        let admitted_episodes = m
            .get("admitted_episodes")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let replay_verified = m
            .get("replay_verified")
            .map(|s| s == "true")
            .unwrap_or(false);
        let family = m.get("family").cloned().unwrap_or_default();
        let tier_dir = m.get("tier_dir").cloned().unwrap_or_default();
        let sealed_case_file_hash = m.get("sealed_case_file_hash").cloned().unwrap_or_default();
        out.push(ManifestRecord {
            id: id.clone(),
            family,
            tier_dir,
            admitted_episodes,
            sealed_case_file_hash,
            replay_verified,
        });
    }

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            // Flush whatever we were collecting before starting a
            // new section.
            flush(&mut out, &current_id, &current);
            current.clear();
            let header = &line[1..line.len() - 1];
            if let Some(id) = header.strip_prefix("datasets.") {
                current_id = Some(id.to_string());
                current.insert("__from_datasets__".to_string(), "1".to_string());
            } else {
                current_id = Some(header.to_string());
            }
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim().to_string();
            let val_raw = line[eq + 1..].trim();
            // Strip surrounding double quotes if present; TOML
            // strings in bundle_manifest.toml are always double-
            // quoted.
            let val = if val_raw.starts_with('"') && val_raw.ends_with('"') {
                val_raw[1..val_raw.len() - 1].to_string()
            } else {
                val_raw.to_string()
            };
            current.insert(key, val);
        }
    }
    // Flush the last section.
    flush(&mut out, &current_id, &current);
    out
}

fn parse_kv_u64(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{key} ")) || line.starts_with(&format!("{key}=")) {
            if let Some(eq) = line.find('=') {
                return line[eq + 1..].trim().parse().ok();
            }
        }
    }
    None
}

/// Tiny SHA-256 hex helper. The bundle_hash_chain.txt format is
/// `<64-hex-sha256>  <path>` per line; we recompute the hash of
/// the live file and compare hex-string against the chain.
fn sha256_hex_lower(bytes: &[u8]) -> String {
    let h = sha256(bytes);
    let mut s = String::with_capacity(64);
    for b in h {
        let lo = b & 0x0f;
        let hi = b >> 4;
        s.push(if hi < 10 {
            (b'0' + hi) as char
        } else {
            (b'a' + hi - 10) as char
        });
        s.push(if lo < 10 {
            (b'0' + lo) as char
        } else {
            (b'a' + lo - 10) as char
        });
    }
    s
}

/// Locate the repo root from the crate dir, mirroring the pattern
/// the existing R.9.c and S-PERF tests use.
fn workspace_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_dir.to_path_buf())
}

/// Test #1 — guards the panel-locked headline against silent drift.
/// Specifically catches any change to the `total_datasets = 20`,
/// `total_admitted_episodes = 316`, or `total_source_class_families = 5`
/// fields in `reports/s_real_3/bundle_manifest.toml`. If a future
/// commit adds a 21st dataset, retunes the bank, or merges a family,
/// the headline numbers MUST be updated atomically with the schema
/// change; this test refuses to admit a manifest where the top-level
/// block disagrees with the panel-pinned constants.
#[test]
fn s_real_3_bundle_manifest_top_level_invariants() {
    let root = workspace_root();
    let manifest_path = root.join("reports/s_real_3/bundle_manifest.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .expect("bundle_manifest.toml must exist; S-REAL.3 sealed at a8aaa04");

    // Top-level [bundle] block invariants. These three integers
    // are the panel-locked S-REAL.3 headline; any drift means
    // either a dataset was added without updating the totals,
    // or a casefile mutation changed the admitted-episode count
    // without re-sealing the manifest.
    assert_eq!(
        parse_kv_u64(&text, "total_datasets"),
        Some(20),
        "bundle_manifest total_datasets must be 20"
    );
    assert_eq!(
        parse_kv_u64(&text, "total_admitted_episodes"),
        Some(316),
        "bundle_manifest total_admitted_episodes must be 316"
    );
    assert_eq!(
        parse_kv_u64(&text, "total_source_class_families"),
        Some(5),
        "bundle_manifest total_source_class_families must be 5"
    );
}

/// Test #2 — guards against arithmetic drift between the per-dataset
/// `[datasets.*]` records and the top-level `[bundle]` totals. Sums
/// `admitted_episodes` across the 20 records and asserts it equals
/// `total_admitted_episodes`; counts the per-family memberships and
/// asserts the F1=4, F2=4, F3=2, F4=6, F5=4 histogram; verifies every
/// dataset record has `replay_verified=true`. Catches the failure mode
/// where a future commit edits one dataset's admitted-episode count
/// in isolation without refreshing the top-level total.
#[test]
fn s_real_3_bundle_records_match_top_level_totals() {
    let root = workspace_root();
    let manifest_path = root.join("reports/s_real_3/bundle_manifest.toml");
    let text = std::fs::read_to_string(&manifest_path).unwrap();
    let records = parse_bundle_records(&text);

    assert_eq!(
        records.len(),
        20,
        "must have exactly 20 [datasets.*] records"
    );

    let total_episodes: u32 = records.iter().map(|r| r.admitted_episodes).sum();
    assert_eq!(
        total_episodes, 316,
        "sum of [datasets.*].admitted_episodes must equal 316"
    );

    // Family distribution (panel-locked): F1=4, F2=4, F3=2, F4=6, F5=4.
    // Any drift means a dataset was reclassified or a record was
    // dropped without updating the [families] block.
    let mut family_counts: BTreeMap<&str, u32> = BTreeMap::new();
    for r in &records {
        *family_counts.entry(r.family.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        family_counts.get("F1").copied(),
        Some(4),
        "F1 must have 4 datasets"
    );
    assert_eq!(
        family_counts.get("F2").copied(),
        Some(4),
        "F2 must have 4 datasets"
    );
    assert_eq!(
        family_counts.get("F3").copied(),
        Some(2),
        "F3 must have 2 datasets"
    );
    assert_eq!(
        family_counts.get("F4").copied(),
        Some(6),
        "F4 must have 6 datasets"
    );
    assert_eq!(
        family_counts.get("F5").copied(),
        Some(4),
        "F5 must have 4 datasets"
    );

    // All 20 records must be replay_verified = true. If any
    // future dataset is admitted without replay verification,
    // this gate refuses to let the bundle seal.
    for r in &records {
        assert!(
            r.replay_verified,
            "dataset {} must have replay_verified = true",
            r.id
        );
        assert_eq!(
            r.sealed_case_file_hash.len(),
            64,
            "dataset {} sealed_case_file_hash must be 64 hex chars",
            r.id
        );
    }
}

/// Test #3 — the load-bearing chain-integrity gate. Walks every row
/// in `reports/s_real_3/bundle_hash_chain.txt` (60 rows = 20 datasets
/// × 3 byte-stable artifacts: casefile.json, dataset_manifest.toml,
/// episodes.jsonl), recomputes SHA-256 over the live file bytes, and
/// asserts the live hash byte-equals the chain hash. Catches the
/// failure mode where ANY of the 60 chain-pinned files drifts by even
/// one byte without a matching update to `bundle_hash_chain.txt`.
/// This is the Zenodo-publishable bundle's primary integrity guarantee.
#[test]
fn s_real_3_bundle_hash_chain_rows_match_live_files() {
    let root = workspace_root();
    let chain_path = root.join("reports/s_real_3/bundle_hash_chain.txt");
    let chain_text =
        std::fs::read_to_string(&chain_path).expect("bundle_hash_chain.txt must exist");

    // Each line: `<64-hex sha>  <relative_path>`. Skip blank
    // and comment lines so the chain can carry header
    // commentary if a future commit needs it.
    let rows: Vec<(String, String)> = chain_text
        .lines()
        .filter_map(|raw| {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut it = line.splitn(2, char::is_whitespace);
            let h = it.next()?.trim();
            let p = it.next()?.trim();
            if h.len() != 64
                || !h
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            {
                return None;
            }
            Some((h.to_string(), p.to_string()))
        })
        .collect();

    assert_eq!(
        rows.len(),
        60,
        "bundle_hash_chain must have exactly 60 rows (20 datasets × 3 artifacts)"
    );

    for (expected_hex, rel_path) in &rows {
        let abs = root.join(rel_path);
        let bytes = std::fs::read(&abs).unwrap_or_else(|_| {
            panic!("hash-chain row references missing file: {}", abs.display())
        });
        let actual_hex = sha256_hex_lower(&bytes);
        assert_eq!(
            &actual_hex, expected_hex,
            "hash-chain row drift: {} expected {} actual {}",
            rel_path, expected_hex, actual_hex
        );
    }
}

/// Test #4 — guards against silent deletion of per-dataset artifacts
/// AND against replay-verification regression. For each of the 20
/// datasets, asserts every one of the 9 panel-locked artifact files
/// (dataset_manifest.toml, schema_map.toml, run_receipt.txt,
/// casefile.json, episodes.jsonl, audit_report.html,
/// replay_verification.txt, limitations.md, perf_profile.txt) exists
/// AND that `replay_verification.txt` contains the literal substring
/// `byte-identical replay: YES`. Catches the failure modes where a
/// future commit deletes `limitations.md` to "clean up" the bundle,
/// or where a future audit re-run admits a fixture whose two replay
/// dispatches did NOT produce byte-identical casefiles.
#[test]
fn s_real_3_per_dataset_artifacts_present_and_replay_yes() {
    let root = workspace_root();
    let manifest_path = root.join("reports/s_real_3/bundle_manifest.toml");
    let text = std::fs::read_to_string(&manifest_path).unwrap();
    let records = parse_bundle_records(&text);

    // The 9 panel-locked per-dataset artifacts. Any future
    // refactor that adds a 10th MUST add it here; any rename
    // MUST also be reflected.
    const ARTIFACTS: &[&str] = &[
        "dataset_manifest.toml",
        "schema_map.toml",
        "run_receipt.txt",
        "casefile.json",
        "episodes.jsonl",
        "audit_report.html",
        "replay_verification.txt",
        "limitations.md",
        "perf_profile.txt",
    ];

    for r in &records {
        let dir = root.join(&r.tier_dir);
        assert!(
            dir.is_dir(),
            "tier_dir for {} not found: {}",
            r.id,
            dir.display()
        );
        for art in ARTIFACTS {
            let p = dir.join(art);
            assert!(
                p.is_file(),
                "artifact missing for {}: {}",
                r.id,
                p.display()
            );
        }

        // The replay_verification.txt MUST carry the verbatim
        // panel-locked string `byte-identical replay: YES`.
        // A `NO` here means the second run produced different
        // bytes — bundle would be inadmissible.
        let rv_path = dir.join("replay_verification.txt");
        let rv = std::fs::read_to_string(&rv_path).unwrap();
        assert!(
            rv.contains("byte-identical replay: YES"),
            "{}: replay_verification.txt missing 'byte-identical replay: YES'",
            r.id
        );
    }
}

/// Test #5 — cross-checks the live casefile.json against the manifest
/// for each dataset on two axes: (a) the live casefile's
/// `final_case_file_hash` field byte-equals
/// `manifest[dataset].sealed_case_file_hash`, and (b) the live
/// casefile's `episodes` array length equals
/// `manifest[dataset].admitted_episodes`. Catches the failure mode
/// where an audit re-run produces a casefile under a different
/// bank/registry (changing the final hash) without refreshing the
/// manifest, OR
/// where an audit re-run drops/adds episodes without updating the
/// admitted-episode count. This is the per-dataset complement to
/// Test #2's aggregate sum-check.
#[test]
fn s_real_3_casefile_hash_and_episode_count_match_manifest() {
    let root = workspace_root();
    let manifest_path = root.join("reports/s_real_3/bundle_manifest.toml");
    let text = std::fs::read_to_string(&manifest_path).unwrap();
    let records = parse_bundle_records(&text);

    for r in &records {
        let cf_path = root.join(&r.tier_dir).join("casefile.json");
        let cf_text = std::fs::read_to_string(&cf_path)
            .unwrap_or_else(|_| panic!("casefile.json missing for {}", r.id));

        // Pull out final_case_file_hash. Format inside the JSON
        // is `"final_case_file_hash":"sha256:<64-hex>"` — we scan
        // textually because the project keeps no JSON crate in
        // deps. Anchored on the literal field name; if a future
        // refactor renames the field, this test catches it.
        let needle = "\"final_case_file_hash\":\"sha256:";
        let pos = cf_text
            .find(needle)
            .unwrap_or_else(|| panic!("{}: final_case_file_hash field not found", r.id));
        let start = pos + needle.len();
        let end = cf_text[start..]
            .find('"')
            .map(|n| start + n)
            .unwrap_or_else(|| panic!("{}: malformed final_case_file_hash field", r.id));
        let cf_hash = &cf_text[start..end];

        assert_eq!(
            cf_hash, r.sealed_case_file_hash,
            "{}: casefile.json final_case_file_hash != manifest sealed_case_file_hash\n  cf:       {}\n  manifest: {}",
            r.id, cf_hash, r.sealed_case_file_hash
        );

        // Episode count: count occurrences of `"admitted":"true"`
        // in the casefile body. The episode array is one JSON
        // object per admitted episode; each carries the field.
        // The bundle's admitted_episodes MUST match this count.
        let needle_ep = "\"admitted\":\"true\"";
        let mut count = 0u32;
        let mut search = cf_text.as_str();
        while let Some(p) = search.find(needle_ep) {
            count += 1;
            search = &search[p + needle_ep.len()..];
        }
        assert_eq!(
            count, r.admitted_episodes,
            "{}: casefile.json admitted-episode count ({}) != manifest admitted_episodes ({})",
            r.id, count, r.admitted_episodes
        );
    }
}
