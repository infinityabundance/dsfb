//! The forensic court: passport, hash-linked case file, execution attestation, challenge docket,
//! precedent, and byte-exact replay.
//!
//! Doctrine: **the GPU (or CPU reference) produces evidence; the court decides what that evidence
//! is allowed to mean.** The *evidence root* is a deterministic, reproducible hash over the
//! contract, the input digest, and the per-lane evidence — it excludes timing/device so it replays
//! byte-for-byte. The *execution attestation* then seals the per-run facts (backend, device,
//! elapsed time, throughput, cross-backend agreement) over that root.
//!
//! # Hash construction discipline
//!
//! All hashes use a length-prefixed field encoding via [`upd`]:
//! ```text
//! SHA-256 ← len(label as u64 LE) ‖ label ‖ len(bytes as u64 LE) ‖ bytes
//! ```
//! This prevents field collisions (different field boundaries that produce the same byte stream)
//! while keeping the construction simple and auditable. The same pattern is used for the evidence
//! root, the attestation hash, and any replay verification.
//!
//! # Reproducibility
//!
//! The `evidence_root` field is a pure function of the contract, input, and lane evidence. Running
//! the same input through either the CPU or GPU path yields the same root (verified by `dispatch`).
//! Timing, device name, and throughput are intentionally excluded from the root and stored only in
//! the `ExecutionAttestation`, which is sealed as a separate `attestation_hash` over the fixed root.

use crate::evidence::{hex_to_bytes, merkle_root, LaneEvidence, CONTRACT_ID, DRIFT_WINDOW, SCALE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Provenance of the analysed input, sealed into the evidence root.
///
/// The passport records all parameters that affect how the evidence was produced. Any change to
/// these parameters produces a different evidence root. Stored verbatim in the case file so that a
/// verifier can re-derive the root without access to the original code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passport {
    /// Dataset name (human-readable identifier, e.g. a filename stem).
    pub dataset: String,
    /// SHA-256 of the raw residual matrix in sample-major f64 LE layout — seals the exact bytes.
    pub input_sha256: String,
    /// Number of residual lanes (== number of process variables).
    pub n_lanes: u32,
    /// Total sample count across all lanes (`n_lanes * n_samples_per_lane`).
    pub n_samples_total: u64,
    /// Fixed-point scale factor used during quantisation (must equal [`evidence::SCALE`]).
    pub scale: f64,
    /// Causal drift ring-buffer width in samples (must equal [`evidence::DRIFT_WINDOW`]).
    pub drift_window: u32,
    /// Evidence contract identifier string (must equal [`evidence::CONTRACT_ID`]).
    pub contract_id: String,
    /// Crate version at the time the case file was produced.
    pub crate_version: String,
    /// Frozen `atlas_hash_v1` of the authority crate that defines what the evidence is *allowed to
    /// mean* — i.e. *which atlas interpreted this case*. Provenance only: it is deliberately NOT folded
    /// into [`evidence_root`] (which seals contract + input + lanes), so recording it leaves every
    /// previously-sealed evidence root byte-identical while still binding the case file to its authority.
    pub atlas_hash: String,
}

impl Passport {
    pub fn new(dataset: &str, input_sha256: String, n_lanes: u32, n_samples_total: u64) -> Self {
        Passport {
            dataset: dataset.to_string(),
            input_sha256,
            n_lanes,
            n_samples_total,
            scale: SCALE,
            drift_window: DRIFT_WINDOW as u32,
            contract_id: CONTRACT_ID.to_string(),
            crate_version: crate::VERSION.to_string(),
            atlas_hash: dsfb_chemical_engineering_atlas::hashes::atlas_hash_v1(),
        }
    }
}

/// A re-derivable claim with the backing hash that allows independent verification.
///
/// The `claim` is a human-readable assertion about the evidence; the `backing_hash` is a hash
/// (evidence root or Merkle root) that a verifier can re-compute to check the claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub claim: String,
    pub backing_hash: String,
}

/// Per-execution facts about a single run of the evidence factory.
///
/// This struct records *how* the evidence was produced (backend, device, timing) but is intentionally
/// **excluded** from the evidence root. Two runs of the same input must produce the same evidence root
/// regardless of backend, device, or elapsed time. The attestation is then sealed over the
/// already-fixed root, so the attestation hash changes if timing changes but the evidence root does not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAttestation {
    /// Backend label: "cpu-reference" or "cuda".
    pub backend: String,
    /// Device identifier (e.g. CUDA device name or "cpu-reference").
    pub device: String,
    /// Kernel name run to produce the evidence (e.g. "evidence_kernel" or "lane_evidence_cpu").
    pub kernel: String,
    /// Kernel-only elapsed time in nanoseconds (from CUDA events for GPU; `Instant` for CPU).
    pub elapsed_ns: u64,
    /// Number of bytes in the input residual matrix (used as the denominator for throughput).
    pub bytes_processed: u64,
    /// Throughput in GB/s: `bytes_processed / elapsed_ns` (bytes/ns == GB/s). Informational only.
    pub throughput_gbps: f64,
    /// UTC timestamp of the run, ISO 8601 format (e.g. "2025-09-01T12:00:00Z").
    pub timestamp_utc: String,
    /// True if this run's sealed evidence is self-consistent (CPU reference ran, or a GPU run matched
    /// the CPU reference). True on the CPU-only path — see `gpu_cross_verified` to disambiguate.
    pub cross_backend_verified: bool,
    /// True **only** when an actual GPU run was cross-checked equal against the CPU reference; false
    /// on the CPU-only path (no second backend) and on a GPU mismatch. Lets a reader of the case file
    /// tell a CPU-reference attestation apart from genuine two-backend agreement. Excluded from the
    /// evidence root and the attestation hash (it is provenance metadata, not part of the evidence).
    pub gpu_cross_verified: bool,
    /// SHA-256 over `(evidence_root ‖ backend ‖ device ‖ kernel ‖ elapsed ‖ bytes)` using the
    /// length-prefixed scheme in [`upd`]. Changes with every run but is linked to the fixed root.
    pub attestation_hash: String,
}

/// The hash-linked forensic case file assembled by the court.
///
/// All hash fields form a chain: `lane.digest_hex` → `merkle_root` → `evidence_root`. The
/// `evidence_root` then anchors the `attestation_hash`. Replay verification re-derives the root
/// independently and checks it equals `evidence_root`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFile {
    /// Provenance record for this case (dataset, scale, contract, crate version).
    pub passport: Passport,
    /// Per-lane evidence summaries, one per process variable, in lane-id order.
    pub lanes: Vec<LaneEvidence>,
    /// SHA-256 of the concatenated raw lane digests (in lane order). See [`evidence::merkle_root`].
    pub merkle_root: String,
    /// Deterministic, reproducible root over contract + input + per-lane evidence.
    /// This is the court's single ground-truth hash; it does not depend on timing or device.
    pub evidence_root: String,
    /// Human-readable claims backed by hashes, assembled at case-file creation time.
    pub challenge_docket: Vec<Challenge>,
    /// Evidence root of the immediately preceding case file in the run sequence, if any.
    /// Provides a hash-linked precedent chain across datasets processed in the same run.
    pub precedent_root: Option<String>,
    /// Execution attestation, attached after the evidence root is fixed (may be `None` if the
    /// case file was assembled without running `attest`).
    pub attestation: Option<ExecutionAttestation>,
}

/// Construct a fresh SHA-256 hasher for a canonical hash.
fn canonical_hasher() -> Sha256 {
    Sha256::new()
}

/// Feed a labelled field into a SHA-256 hasher using the length-prefixed scheme.
///
/// Feeds: `len(label) as u64 LE ‖ label ‖ len(bytes) as u64 LE ‖ bytes`.
/// The length prefixes prevent field boundary ambiguities: two different field splits cannot
/// produce the same byte stream for different inputs. Every field in the evidence root and
/// attestation hash is written through this function.
fn upd(h: &mut Sha256, label: &str, bytes: &[u8]) {
    h.update((label.len() as u64).to_le_bytes());
    h.update(label.as_bytes());
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

/// Finalise a SHA-256 hasher and return the digest as a lowercase hex string (64 chars).
fn hexdigest(h: Sha256) -> String {
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Compute the deterministic evidence root from passport + lanes + merkle root.
///
/// The root is a SHA-256 over all fields that define the evidence: the contract parameters (from
/// the passport), the input digest, the Merkle root, and each lane's full evidence record. Fields
/// are fed in a fixed order through [`upd`] (length-prefixed) so the result is unambiguous.
///
/// The `scale` field is hashed as its IEEE-754 bit representation (via `to_bits().to_le_bytes()`)
/// rather than as a decimal string, to avoid any floating-point formatting ambiguity.
///
/// This function is called both during case file assembly and during replay verification, so both
/// calls must receive structurally identical inputs to produce a matching root.
pub fn evidence_root(passport: &Passport, lanes: &[LaneEvidence], merkle: &str) -> String {
    let mut h = canonical_hasher();
    // Contract parameters from the passport — any change here changes the root.
    upd(&mut h, "contract", passport.contract_id.as_bytes());
    upd(&mut h, "input", passport.input_sha256.as_bytes());
    upd(&mut h, "n_lanes", &passport.n_lanes.to_le_bytes());
    upd(&mut h, "n_samples", &passport.n_samples_total.to_le_bytes());
    // Scale hashed as raw IEEE-754 bits to avoid decimal representation ambiguity.
    upd(&mut h, "scale", &passport.scale.to_bits().to_le_bytes());
    upd(&mut h, "window", &passport.drift_window.to_le_bytes());
    // Merkle root decoded from hex back to 32 raw bytes before hashing.
    upd(&mut h, "merkle", &hex_to_bytes(merkle));
    // Per-lane evidence, in lane-id order. Every field is included so the root depends on the
    // full evidence record, not just the digest.
    for l in lanes {
        upd(&mut h, "lane", &l.lane_id.to_le_bytes());
        upd(&mut h, "ns", &l.n_samples.to_le_bytes());
        upd(&mut h, "exc", &l.n_exceedances.to_le_bytes());
        upd(&mut h, "oob", &l.oob_count.to_le_bytes());
        upd(&mut h, "pke", &l.peak_exceedance.to_le_bytes());
        upd(&mut h, "pkd", &l.peak_abs_drift.to_le_bytes());
        upd(&mut h, "pks", &l.peak_abs_slew.to_le_bytes());
        // Lane digest decoded from hex to 32 raw bytes.
        upd(&mut h, "dig", &hex_to_bytes(&l.digest_hex));
    }
    hexdigest(h)
}

impl CaseFile {
    /// Assemble a case file from lane evidence + input provenance.
    ///
    /// Computes the Merkle root, builds a [`Passport`], derives the [`evidence_root`], and
    /// populates the challenge docket with three verifiable claims. `precedent_root`, if provided,
    /// is the evidence root of the preceding dataset in the run sequence, forming a hash chain.
    /// `attestation` is left as `None`; call [`CaseFile::attest`] after construction to seal it.
    pub fn assemble(
        dataset: &str,
        input_sha256: String,
        n_samples_total: u64,
        lanes: Vec<LaneEvidence>,
        precedent_root: Option<String>,
    ) -> Self {
        let merkle = merkle_root(&lanes);
        let passport = Passport::new(dataset, input_sha256, lanes.len() as u32, n_samples_total);
        let root = evidence_root(&passport, &lanes, &merkle);
        // Aggregate counts across lanes for the docket claims.
        let total_exc: u64 = lanes.iter().map(|l| l.n_exceedances as u64).sum();
        let total_oob: u64 = lanes.iter().map(|l| l.oob_count as u64).sum();
        // Challenge docket: human-readable claims backed by reproducible hashes.
        let docket = vec![
            Challenge {
                // Claim 1: the entire evidence set is reproducible — backed by the full evidence root.
                claim: format!("Evidence over {} lanes / {} samples is reproducible from the input under contract {}.", lanes.len(), n_samples_total, CONTRACT_ID),
                backing_hash: root.clone(),
            },
            Challenge {
                // Claim 2: exceedance count — backed by the Merkle root (covers all lane digests).
                claim: format!("{total_exc} sample-level control-limit exceedances were observed (one-sided e=max(0,q))."),
                backing_hash: merkle.clone(),
            },
            Challenge {
                // Claim 3: out-of-band count — backed by Merkle root to show they were sealed, not dropped.
                claim: format!("{total_oob} non-finite (out-of-band) samples were sealed into the record rather than silently dropped."),
                backing_hash: merkle.clone(),
            },
        ];
        CaseFile {
            passport,
            lanes,
            merkle_root: merkle,
            evidence_root: root,
            challenge_docket: docket,
            precedent_root,
            attestation: None,
        }
    }

    /// Seal an execution attestation over the (already-fixed) evidence root.
    ///
    /// This must be called *after* `assemble`, so that the evidence root is fixed before being
    /// included in the attestation hash. The `throughput_gbps` is computed as `bytes/ns` = GB/s.
    // Each parameter is an independent per-run fact sealed into the attestation; bundling them into a
    // struct would only invert the dispatch->court layering (the verification flags originate in
    // `dispatch::EvidenceRun`), so we keep the explicit primitives.
    #[allow(clippy::too_many_arguments)]
    pub fn attest(
        &mut self,
        backend: &str,
        device: &str,
        kernel: &str,
        elapsed_ns: u64,
        bytes_processed: u64,
        cross_backend_verified: bool,
        gpu_cross_verified: bool,
        timestamp_utc: &str,
    ) {
        // bytes/ns is dimensionally equivalent to GB/s (1 GB = 1e9 bytes, 1 ns = 1e-9 s).
        let throughput = if elapsed_ns > 0 {
            bytes_processed as f64 / elapsed_ns as f64
        } else {
            0.0
        };
        // The attestation hash seals the per-run facts over the already-fixed evidence root.
        // Note: timing and device are part of the attestation hash but NOT the evidence root.
        let mut h = canonical_hasher();
        upd(&mut h, "evidence_root", &hex_to_bytes(&self.evidence_root));
        upd(&mut h, "backend", backend.as_bytes());
        upd(&mut h, "device", device.as_bytes());
        upd(&mut h, "kernel", kernel.as_bytes());
        upd(&mut h, "elapsed", &elapsed_ns.to_le_bytes());
        upd(&mut h, "bytes", &bytes_processed.to_le_bytes());
        self.attestation = Some(ExecutionAttestation {
            backend: backend.to_string(),
            device: device.to_string(),
            kernel: kernel.to_string(),
            elapsed_ns,
            bytes_processed,
            throughput_gbps: throughput,
            timestamp_utc: timestamp_utc.to_string(),
            cross_backend_verified,
            gpu_cross_verified,
            attestation_hash: hexdigest(h),
        });
    }

    /// Byte-exact replay: recompute the evidence root from the same input lanes and confirm it
    /// equals the stored root.
    ///
    /// `recomputed_lanes` must be produced by the caller by running the evidence factory over the
    /// same residual data with the same contract parameters. If the result matches, the evidence
    /// root is verified to be reproducible. A mismatch indicates either a contract violation or
    /// data corruption.
    pub fn verify_replay(&self, recomputed_lanes: &[LaneEvidence]) -> ReplayResult {
        let merkle = merkle_root(recomputed_lanes);
        let root = evidence_root(&self.passport, recomputed_lanes, &merkle);
        ReplayResult {
            matched: root == self.evidence_root,
            recomputed_root: root,
            stored_root: self.evidence_root.clone(),
        }
    }
}

/// Result of a byte-exact replay verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// `true` if the recomputed root equals the stored root byte-for-byte.
    pub matched: bool,
    /// The root derived from `recomputed_lanes` in the replay call.
    pub recomputed_root: String,
    /// The root stored in the case file at assembly time.
    pub stored_root: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::lane_evidence_cpu;

    #[test]
    fn case_file_replays_byte_exact() {
        let lanes: Vec<LaneEvidence> = (0..4)
            .map(|l| {
                lane_evidence_cpu(
                    l,
                    &(0..512)
                        .map(|i| ((i + l as usize) as f64 * 0.05).sin())
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let cf = CaseFile::assemble("unit", "deadbeef".into(), 4 * 512, lanes.clone(), None);
        let rr = cf.verify_replay(&lanes);
        assert!(rr.matched, "evidence root must replay byte-exact");
        assert_eq!(rr.recomputed_root, cf.evidence_root);
    }
}
