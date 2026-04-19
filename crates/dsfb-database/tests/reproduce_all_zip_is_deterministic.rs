//! Byte-determinism lock for `dsfb-database reproduce-all`.
//!
//! The `reproduce-all` subcommand bundles every offline artefact into
//! `out/dsfb_database_artifacts.zip`. The paper's supplementary material
//! and the Colab reproducibility notebook both cite a pinned SHA-256 of
//! this zip; any accidental non-determinism (unsorted entries,
//! wall-clock mtimes, hashmap iteration order) breaks that link. This
//! test invokes the binary twice with the same seed in separate temp
//! directories and asserts the two resulting zips have identical
//! SHA-256 digests.
//!
//! Skipped gracefully when the release binary is not built — we want the
//! test to run on CI but not block a debug-only `cargo test` run.

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn release_binary() -> PathBuf {
    // The binary name matches the `[package] name` so cargo emits it at
    // the standard location.
    repo_root().join("target/release/dsfb-database")
}

fn sha256_of_file(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).expect("read zip");
    let mut h = Sha256::new();
    h.update(&bytes);
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn reproduce_all_zip_is_byte_stable() {
    let bin = release_binary();
    if !bin.exists() {
        eprintln!(
            "skipping reproduce_all_zip_is_byte_stable: {} missing (cargo build --release --features report to populate)",
            bin.display()
        );
        return;
    }

    let out_a = tempfile::tempdir().expect("tempdir a");
    let out_b = tempfile::tempdir().expect("tempdir b");

    for out in [out_a.path(), out_b.path()] {
        let status = Command::new(&bin)
            .args([
                "reproduce-all",
                "--seed",
                "42",
                "--out",
                out.to_str().unwrap(),
            ])
            .status()
            .expect("spawn");
        assert!(status.success(), "reproduce-all failed in {}", out.display());
    }

    let zip_a = out_a.path().join("dsfb_database_artifacts.zip");
    let zip_b = out_b.path().join("dsfb_database_artifacts.zip");
    assert!(zip_a.exists(), "zip not emitted at {}", zip_a.display());
    assert!(zip_b.exists(), "zip not emitted at {}", zip_b.display());

    let sha_a = sha256_of_file(&zip_a);
    let sha_b = sha256_of_file(&zip_b);
    assert_eq!(
        sha_a, sha_b,
        "reproduce-all zip is not byte-stable across runs (a={}, b={})",
        sha_a, sha_b
    );
}
