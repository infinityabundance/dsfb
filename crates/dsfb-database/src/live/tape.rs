//! Append-only residual tape + SHA-256 finalisation.
//!
//! The tape is a JSONL file — one serialised [`ResidualSample`] per
//! line — that persists the *exact* residual stream the live adapter
//! observed. Given a tape, the replay subcommand
//! (`dsfb-database replay-tape`) deterministically reproduces the
//! episode stream by re-running [`crate::grammar::MotifEngine::run`]
//! against the parsed residuals.
//!
//! The seventh non-claim is explicit about the direction of the
//! determinism guarantee: **tape → episodes is byte-stable; engine →
//! tape is not.** Two live invocations against the same PostgreSQL
//! workload produce different tapes because sampling jitter,
//! counter-advance timing, and concurrent workload shape the
//! samples. This module finalises a SHA-256 of the tape bytes so an
//! operator can cryptographically pin a specific run and prove to
//! themselves that a later replay reproduced the same episodes.
//!
//! ## File layout
//!
//! ```text
//! <tape-path>            # jsonl, one ResidualSample per line
//! <tape-path>.hash       # json manifest with SHA-256, sample count,
//!                        #   first_t, last_t, crate_version, source
//! ```
//!
//! `sha256` is computed over the raw bytes of `<tape-path>` after the
//! final flush. The replay path recomputes the hash and refuses to
//! proceed if it mismatches the manifest — this catches silent
//! truncation or deliberate tampering.

use crate::residual::{ResidualSample, ResidualStream};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Sidecar manifest persisted next to the tape at finalisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeManifest {
    /// Lowercase hex SHA-256 of the tape bytes.
    pub sha256: String,
    /// Number of `ResidualSample`s written.
    pub sample_count: u64,
    /// Minimum `t` observed (seconds; None if empty).
    pub first_t: Option<f64>,
    /// Maximum `t` observed (seconds; None if empty).
    pub last_t: Option<f64>,
    /// `CARGO_PKG_VERSION` of the crate that wrote the tape.
    pub crate_version: String,
    /// Source label (e.g. `"live-postgres:host=..."`).
    pub source: String,
}

/// Append-only tape writer.
pub struct Tape {
    path: PathBuf,
    writer: BufWriter<File>,
    sample_count: u64,
    first_t: Option<f64>,
    last_t: Option<f64>,
    source: String,
}

impl Tape {
    /// Create (or truncate) a tape at `path`. Returns an error if the
    /// file cannot be opened for writing.
    pub fn create(path: &Path, source: impl Into<String>) -> Result<Self> {
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("opening tape file {}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            writer: BufWriter::new(f),
            sample_count: 0,
            first_t: None,
            last_t: None,
            source: source.into(),
        })
    }

    /// Append a batch of samples. Each sample becomes one JSON line.
    pub fn append(&mut self, samples: &[ResidualSample]) -> Result<()> {
        for s in samples {
            let line = serde_json::to_string(s)
                .with_context(|| "serializing ResidualSample to tape")?;
            self.writer
                .write_all(line.as_bytes())
                .with_context(|| "writing tape line")?;
            self.writer
                .write_all(b"\n")
                .with_context(|| "writing tape newline")?;
            self.sample_count += 1;
            self.first_t = Some(self.first_t.map(|t| t.min(s.t)).unwrap_or(s.t));
            self.last_t = Some(self.last_t.map(|t| t.max(s.t)).unwrap_or(s.t));
        }
        Ok(())
    }

    /// Flush, fsync, close the tape file, then hash its bytes and
    /// emit the sidecar manifest.
    pub fn finalize(mut self) -> Result<TapeManifest> {
        self.writer
            .flush()
            .with_context(|| "flushing tape writer")?;
        // Get the inner File back and fsync it, so a crash after
        // finalize cannot leave the manifest pointing at incomplete
        // tape bytes.
        let f = self
            .writer
            .into_inner()
            .with_context(|| "unwrapping BufWriter to fsync")?;
        f.sync_all().with_context(|| "fsyncing tape file")?;
        drop(f);
        let tape_bytes = std::fs::read(&self.path)
            .with_context(|| format!("re-reading tape for hashing: {}", self.path.display()))?;
        let mut h = Sha256::new();
        h.update(&tape_bytes);
        let sha = h.finalize();
        let sha_hex: String = sha.iter().map(|b| format!("{:02x}", b)).collect();
        let manifest = TapeManifest {
            sha256: sha_hex,
            sample_count: self.sample_count,
            first_t: self.first_t,
            last_t: self.last_t,
            crate_version: crate::CRATE_VERSION.to_string(),
            source: self.source.clone(),
        };
        let manifest_path = manifest_path_for(&self.path);
        let json = serde_json::to_string_pretty(&manifest)
            .with_context(|| "serializing tape manifest")?;
        std::fs::write(&manifest_path, json)
            .with_context(|| format!("writing tape manifest {}", manifest_path.display()))?;
        Ok(manifest)
    }
}

/// Compute the manifest path `<tape>.hash` for a given tape path.
pub fn manifest_path_for(tape: &Path) -> PathBuf {
    let mut p = tape.as_os_str().to_os_string();
    p.push(".hash");
    PathBuf::from(p)
}

/// Load a tape from disk, verify the SHA-256 against the sidecar
/// manifest, and return a [`ResidualStream`] plus the manifest.
/// Refuses to proceed on hash mismatch, missing manifest, or schema
/// mismatch.
pub fn load_and_verify(path: &Path) -> Result<(ResidualStream, TapeManifest)> {
    let manifest_path = manifest_path_for(path);
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading tape manifest {}", manifest_path.display()))?;
    let manifest: TapeManifest = serde_json::from_str(&manifest_json)
        .with_context(|| "parsing tape manifest")?;
    let tape_bytes = std::fs::read(path)
        .with_context(|| format!("reading tape {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&tape_bytes);
    let sha = h.finalize();
    let got: String = sha.iter().map(|b| format!("{:02x}", b)).collect();
    if got != manifest.sha256 {
        anyhow::bail!(
            "tape hash mismatch at {}: manifest={} actual={}",
            path.display(),
            manifest.sha256,
            got
        );
    }
    let mut stream = ResidualStream::new(manifest.source.clone());
    for (ix, line) in std::str::from_utf8(&tape_bytes)
        .with_context(|| "tape contains non-UTF-8 bytes")?
        .lines()
        .enumerate()
    {
        if line.is_empty() {
            continue;
        }
        let s: ResidualSample = serde_json::from_str(line)
            .with_context(|| format!("parsing tape line {}", ix + 1))?;
        stream.push(s);
    }
    stream.sort();
    if stream.len() as u64 != manifest.sample_count {
        anyhow::bail!(
            "tape sample count mismatch: manifest={} actual={}",
            manifest.sample_count,
            stream.len()
        );
    }
    Ok((stream, manifest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::ResidualClass;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_tape_preserves_samples() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.jsonl");
        let mut tape = Tape::create(&path, "live-postgres:test").unwrap();
        let samples = vec![
            ResidualSample::new(0.0, ResidualClass::PlanRegression, 0.1)
                .with_channel("q1"),
            ResidualSample::new(1.0, ResidualClass::WorkloadPhase, 0.2)
                .with_channel("bucket"),
            ResidualSample::new(2.0, ResidualClass::PlanRegression, 0.3)
                .with_channel("q2"),
        ];
        tape.append(&samples).unwrap();
        let manifest = tape.finalize().unwrap();
        assert_eq!(manifest.sample_count, 3);
        assert_eq!(manifest.first_t, Some(0.0));
        assert_eq!(manifest.last_t, Some(2.0));
        let (stream, m) = load_and_verify(&path).unwrap();
        assert_eq!(stream.len(), 3);
        assert_eq!(m.sha256, manifest.sha256);
    }

    #[test]
    fn tampered_tape_fails_verification() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.jsonl");
        let mut tape = Tape::create(&path, "live-postgres:test").unwrap();
        tape.append(&[
            ResidualSample::new(0.0, ResidualClass::PlanRegression, 0.1),
            ResidualSample::new(1.0, ResidualClass::PlanRegression, 0.2),
        ])
        .unwrap();
        tape.finalize().unwrap();
        // Tamper with the tape after finalisation.
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] = b'X'; // mangle the first character (breaks JSON too)
        std::fs::write(&path, &bytes).unwrap();
        let err = load_and_verify(&path).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("tape hash mismatch") || msg.contains("parsing tape line"),
            "expected hash-mismatch or parse error, got: {}",
            msg
        );
    }

    #[test]
    fn manifest_path_suffix() {
        let p = Path::new("/tmp/a/b.tape");
        assert_eq!(manifest_path_for(p), PathBuf::from("/tmp/a/b.tape.hash"));
    }
}
