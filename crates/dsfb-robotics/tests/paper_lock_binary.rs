//! Integration test for the `paper-lock` binary. Exercises the full
//! CLI end-to-end to lock the contract:
//!
//! - unknown slug → exit 64 (EX_USAGE), usage printed.
//! - `--list` → exit 0, all ten supported slugs on stdout.
//! - `--help` → exit 0, usage text on stdout.
//! - `<slug> --fixture` → exit 0, JSON report on stdout.
//! - `<slug>` (real-data) → exit 64, oracle-protocol pointer on stderr.
//! - Two consecutive fixture runs for the same dataset produce
//!   byte-identical stdout (the bit-exact tolerance gate).
//!
//! Only compiled under `--features std,paper_lock` since the binary
//! itself is feature-gated.

#![cfg(feature = "paper_lock")]

use std::path::PathBuf;
use std::process::Command;

/// Canonical slug list in paper §10 order — must stay in lock-step
/// with `main.rs::SUPPORTED_SLUGS` and `datasets::DatasetId::slug`.
const SLUGS: &[&str] = &[
    "cwru",
    "ims",
    "kuka_lwr",
    "femto_st",
    "panda_gaz",
    "dlr_justin",
    "ur10_kufieta",
    "cheetah3",
    "icub_pushrecovery",
    "droid",
    "openx",
    "anymal_parkour",
    "unitree_g1",
    "aloha_static",
    "icub3_sorrentino",
    "mobile_aloha",
    "so100",
    "aloha_static_tape",
    "aloha_static_screw_driver",
    "aloha_static_pingpong_test",
];

fn binary_path() -> PathBuf {
    // Cargo sets CARGO_BIN_EXE_<name> at build time for integration tests.
    PathBuf::from(env!("CARGO_BIN_EXE_paper-lock"))
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary_path())
        .args(args)
        .output()
        .expect("spawn paper-lock");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn help_exits_zero_and_prints_usage() {
    let (code, stdout, _stderr) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Usage: paper-lock"));
    assert!(stdout.contains("--fixture"));
}

#[test]
fn no_args_prints_help_with_exit_zero() {
    let (code, stdout, _stderr) = run(&[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Usage: paper-lock"));
}

#[test]
fn list_prints_all_supported_slugs() {
    let (code, stdout, _stderr) = run(&["--list"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), SLUGS.len(), "expected exactly {} slugs on stdout, got {:?}", SLUGS.len(), lines);
    for slug in SLUGS {
        assert!(lines.contains(slug), "slug {slug} missing from --list output");
    }
}

#[test]
fn unknown_slug_exits_64() {
    let (code, _stdout, stderr) = run(&["not-a-real-dataset"]);
    assert_eq!(code, 64, "unknown slug must yield EX_USAGE");
    assert!(stderr.contains("unknown dataset"));
}

#[test]
fn unknown_flag_exits_64() {
    let (code, _stdout, stderr) = run(&["--not-a-flag"]);
    assert_eq!(code, 64);
    assert!(stderr.contains("unknown flag"));
}

#[test]
fn real_data_mode_produces_report_or_actionable_error() {
    // After Phase 8 the real-data path consumes `data/processed/<slug>.csv`.
    // Where that file exists the run succeeds with mode="real-data";
    // where it is absent the binary must exit 64 (EX_USAGE) with a
    // clear pointer to the preprocess script.
    for slug in SLUGS {
        let (code, stdout, stderr) = run(&[slug]);
        if code == 0 {
            assert!(stdout.contains("\"mode\": \"real-data\""), "{slug}: expected real-data report");
            assert!(stdout.contains(&format!("\"dataset\": \"{slug}\"")));
            assert!(stdout.trim_end().ends_with('}'));
        } else {
            assert_eq!(code, 64, "{slug}: real-data absent → EX_USAGE, got {code}");
            assert!(stderr.contains("preprocess_datasets.py"));
            assert!(stderr.contains("silently substitute"));
        }
    }
}

#[test]
fn fixture_mode_produces_valid_json_report_for_every_dataset() {
    for slug in SLUGS {
        let (code, stdout, _stderr) = run(&[slug, "--fixture"]);
        assert_eq!(code, 0, "{slug}: fixture mode must exit 0");
        // Structural JSON checks — no dep on serde_json in the test.
        assert!(stdout.starts_with("{"), "{slug}: JSON must start with '{{'");
        assert!(stdout.trim_end().ends_with("}"), "{slug}: JSON must end with '}}'");
        assert!(stdout.contains(&format!("\"dataset\": \"{slug}\"")));
        assert!(stdout.contains("\"mode\": \"fixture-smoke-test\""));
        assert!(stdout.contains("\"paper_lock_version\": \"0.1.0\""));
        assert!(stdout.contains("\"run_configuration\""));
        assert!(stdout.contains("\"aggregate\""));
        assert!(stdout.contains("\"total_samples\""));
        assert!(stdout.ends_with('\n'), "{slug}: report must end with newline");
    }
}

#[test]
fn fixture_output_is_bit_exact_across_repeat_invocations() {
    for slug in SLUGS {
        let (_, a, _) = run(&[slug, "--fixture"]);
        let (_, b, _) = run(&[slug, "--fixture"]);
        let (_, c, _) = run(&[slug, "--fixture"]);
        assert_eq!(a, b, "{slug}: fixture run 1 vs 2 drifted");
        assert_eq!(b, c, "{slug}: fixture run 2 vs 3 drifted");
    }
}

#[test]
fn fixture_mode_writes_banner_to_stderr() {
    let (_code, _stdout, stderr) = run(&["kuka_lwr", "--fixture"]);
    assert!(stderr.contains("FIXTURE MODE"), "banner missing from stderr: {stderr}");
    assert!(stderr.contains("NOT empirical data") || stderr.contains("not empirical"));
}
