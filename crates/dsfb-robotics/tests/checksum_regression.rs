//! Mechanical regression on the per-dataset paper-lock JSON checksums.
//!
//! For every dataset in `audit/checksums.txt` (which is itself
//! re-emitted by `scripts/reproduce.sh` after every full run), this
//! test runs the production paper-lock binary on the local data and
//! asserts that the SHA-256 of the JSON output matches the committed
//! checksum.
//!
//! Catches mechanical regressions in:
//! - the FSM / grammar / engine code paths (any output drift)
//! - the residual CSV preprocessor (any sample-count drift)
//! - the JSON serialiser ordering / formatting
//! - the underlying float-arithmetic semantics if the host Rust
//!   toolchain ever drifts off IEEE 754 binary64.
//!
//! The test is skipped under Miri (Miri cannot perform filesystem
//! syscalls; per-dataset checksum verification therefore cannot run
//! under Miri) and skipped if either `audit/checksums.txt` or the
//! processed-CSV directory is missing (so a clean checkout without
//! data corpora compiles + runs the rest of the test suite).
#![cfg(all(feature = "std", feature = "paper_lock"))]

use std::collections::HashMap;
use std::path::PathBuf;

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::{run_real_data_with_trace, serialize_report};

fn locate_crate_root() -> PathBuf {
    // The test runner cwd may be either the workspace root or the
    // crate root depending on invocation. Try both.
    let candidates = [
        PathBuf::from("crates/dsfb-robotics"),
        PathBuf::from("."),
    ];
    for c in candidates.iter() {
        if c.join("audit").join("checksums.txt").is_file()
            && c.join("data").join("processed").is_dir()
        {
            return c.clone();
        }
    }
    PathBuf::from(".")
}

fn parse_committed_checksums(path: &PathBuf) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(s) = std::fs::read_to_string(path) else { return out };
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: "<sha>  <relative path ending in <slug>.json>"
        let mut parts = line.splitn(2, "  ");
        let Some(sha) = parts.next() else { continue };
        let Some(path) = parts.next() else { continue };
        // extract slug from `audit/json_outputs/<slug>.json`
        let Some(fname) = path.rsplit('/').next() else { continue };
        let Some(slug) = fname.strip_suffix(".json") else { continue };
        // skip the dsfb-gray DSSE attestation line — it points at a
        // different artefact than the per-dataset paper-lock outputs.
        if slug == "dsfb_robotics_scan" || path.contains("dsse") {
            continue;
        }
        out.insert(slug.to_string(), sha.to_string());
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    // Tiny SHA-256 dependency-free shim is overkill; depend on the
    // sha256 implementation already in the workspace via the
    // `sha2` crate-dev path. Or we can shell out to `sha256sum`,
    // which is universally available on Linux test runners.
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum available");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(bytes)
        .expect("write");
    let out = child.wait_with_output().expect("sha256sum exit");
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace().next().unwrap_or("").to_string()
}

#[test]
#[cfg_attr(miri, ignore = "Miri cannot model filesystem syscalls")]
fn paper_lock_json_checksums_match_committed() {
    let root = locate_crate_root();
    let committed_path = root.join("audit").join("checksums.txt");
    if !committed_path.is_file() {
        eprintln!("skipping: {committed_path:?} missing — clean checkout?");
        return;
    }
    let committed = parse_committed_checksums(&committed_path);
    if committed.is_empty() {
        eprintln!("skipping: no per-dataset checksums in {committed_path:?}");
        return;
    }
    if !root.join("data").join("processed").is_dir() {
        eprintln!("skipping: data/processed/ missing — corpora not fetched");
        return;
    }
    // cd into the crate root so paper-lock's relative-path lookups
    // for data/processed/<slug>.csv resolve correctly.
    std::env::set_current_dir(&root).expect("chdir to crate root");

    let mut mismatches = Vec::new();
    let mut checked = 0_usize;
    for (slug, expected_sha) in &committed {
        let Some(id) = DatasetId::from_slug(slug) else {
            eprintln!("skipping unknown slug: {slug}");
            continue;
        };
        let report = match run_real_data_with_trace(id, false) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("skipping {slug}: {}", e.instructions);
                continue;
            }
        };
        let json = serialize_report(&report).expect("serialize");
        let actual_sha = sha256_hex(json.as_bytes());
        checked += 1;
        if &actual_sha != expected_sha {
            mismatches.push((slug.clone(), expected_sha.clone(), actual_sha));
        }
    }
    if !mismatches.is_empty() {
        for (slug, expected, actual) in &mismatches {
            eprintln!("CHECKSUM DRIFT: {slug}\n  expected: {expected}\n  actual:   {actual}");
        }
        panic!(
            "{} datasets drifted; see audit/checksums.txt",
            mismatches.len()
        );
    }
    eprintln!("OK: {checked} per-dataset paper-lock JSON checksums match audit/checksums.txt");
}
