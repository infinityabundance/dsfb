//! T.10 — `CaseFileV2` receipt header.
//!
//! **Panel-locked scope (T.10 is header receipt only)**:
//!
//! T.10 introduces a minimal `CaseFileV2Header` that can carry
//! `corpus_hash_v1` and the current detector-ladder identity.
//! It is NOT a full episode transcript, NOT a `registry_hash_v2`
//! claim, NOT an Atlas execution receipt. The Atlas case-file
//! body (full Atlas dispatch verdict) lands in a later
//! commit.
//!
//! The header is hashable: [`casefile_v2_header_hash`] produces
//! a 32-byte SHA-256 commitment over a canonical-byte projection
//! of the header (length-prefixed strings, big-endian integers,
//! enum wire names not Debug output). This header hash is the
//! T.10 receipt anchor; it is NOT the final Atlas case-file
//! hash.
//!
//! **The corpus_hash_v1 field is a raw `[u8; 32]`** so this core
//! crate does not depend on the corpus crate. The caller builds
//! the header from `dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1`
//! and passes the bytes through.

#[cfg(feature = "std")]
extern crate std as alloc_std;

extern crate alloc;
use alloc::vec::Vec;

use crate::hash::sha256;
use crate::motif::DetectorProfile;

/// Domain separator prefix for the case-file-v2 header hash.
/// **Panel-locked**; changing it changes the receipt hash on
/// every header.
pub const CASEFILE_V2_HEADER_DOMAIN: &str = "DSFB-GPU-ATLAS:CASEFILE-V2-HEADER:v1\0";

/// Which projection of the case-file the header represents. At
/// T.10 only the header-only projection exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaseFileV2Schema {
    /// T.10 — receipt header only. No full episode transcript,
    /// no Atlas dispatch body.
    HeaderOnlyT10,
}

impl CaseFileV2Schema {
    /// Canonical wire name. Hand-pinned (not derived from
    /// `Debug`) so a future variant rename cannot silently
    /// shift the header hash.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HeaderOnlyT10 => "HeaderOnlyT10",
        }
    }
}

/// Where the corpus stands at receipt time. Bridges T.9 (internal
/// audit) to T.10 (frozen) cleanly. T.11+ will add later stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorpusStage {
    /// Pre-T.10 internal audit. Used in receipts emitted before
    /// `corpus_hash_v1` was frozen.
    InternalAuditPreFreeze,
    /// T.10 — corpus_hash_v1 frozen and bound into the receipt.
    FrozenT10,
}

impl CorpusStage {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InternalAuditPreFreeze => "InternalAuditPreFreeze",
            Self::FrozenT10 => "FrozenT10",
        }
    }
}

/// Atlas algebra status reflected in the receipt header. The
/// header REFERS to the S1.1 algebra surface without claiming
/// the S1.2 registry has been generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtlasAlgebraStatus {
    /// S1.1 — Atlas algebra type surface only. The 2,000-detector
    /// registry has NOT been generated; `registry_hash_v2` does
    /// NOT exist.
    S1_1TypeSurfaceOnly,
}

impl AtlasAlgebraStatus {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::S1_1TypeSurfaceOnly => "S1_1TypeSurfaceOnly",
        }
    }
}

/// Minimal T.10 receipt header.
///
/// The fields bind, in order:
///
/// - `schema` — case-file shape (HeaderOnlyT10 at T.10).
/// - `corpus_hash_v1` — 32-byte SHA-256 from
///   `dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1`.
/// - `corpus_stage` — `FrozenT10` for receipts emitted from T.10.
/// - `detector_profile` — `D16` / `D64` / `D128` / `D205` etc.
///   This carries the detector-ladder identity at receipt time.
/// - `detector_registry_hash` — the per-profile registry hash
///   (the canonical 16-detector hash for D16; the wider-profile
///   hash for D64+).
/// - `atlas_algebra_status` — `S1_1TypeSurfaceOnly` at T.10.
/// - `semantic_non_bypass` — MUST be `true`. The verifier
///   rejects `false` because the Semantic Non-Bypass Axiom is
///   non-negotiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaseFileV2Header {
    /// Case-file shape.
    pub schema: CaseFileV2Schema,
    /// 32-byte corpus_hash_v1 commitment from T.10.
    pub corpus_hash_v1: [u8; 32],
    /// Corpus stage at receipt time.
    pub corpus_stage: CorpusStage,
    /// Detector profile the receipt is bound to.
    pub detector_profile: DetectorProfile,
    /// Per-profile detector registry hash.
    pub detector_registry_hash: [u8; 32],
    /// Atlas algebra status at receipt time.
    pub atlas_algebra_status: AtlasAlgebraStatus,
    /// Semantic Non-Bypass Axiom flag. **MUST be `true`.**
    pub semantic_non_bypass: bool,
}

/// One verification failure on a `CaseFileV2Header`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseFileV2HeaderError {
    /// `corpus_hash_v1` is the all-zero sentinel. The verifier
    /// rejects this because the canonical compute path always
    /// produces a non-zero digest on a non-empty corpus.
    ZeroCorpusHash,
    /// `semantic_non_bypass` was `false`. The Semantic Non-Bypass
    /// Axiom is non-negotiable; every receipt MUST carry `true`.
    SemanticNonBypassFalse,
    /// `corpus_stage` is `InternalAuditPreFreeze`. T.10 receipts
    /// must carry `FrozenT10`.
    CorpusStagePreFreeze,
}

/// Verify a `CaseFileV2Header` against the T.10 invariants.
/// Returns `Ok(())` on success or the first failing invariant.
///
/// # Errors
///
/// Returns `CaseFileV2HeaderError::ZeroCorpusHash` if the
/// header's `corpus_hash_v1` is all zeros,
/// `CaseFileV2HeaderError::SemanticNonBypassFalse` if the flag
/// is `false`, or `CaseFileV2HeaderError::CorpusStagePreFreeze`
/// if `corpus_stage` is `InternalAuditPreFreeze` (T.10 receipts
/// must declare `FrozenT10`).
pub fn verify_casefile_v2_header(header: &CaseFileV2Header) -> Result<(), CaseFileV2HeaderError> {
    if header.corpus_hash_v1 == [0u8; 32] {
        return Err(CaseFileV2HeaderError::ZeroCorpusHash);
    }
    if !header.semantic_non_bypass {
        return Err(CaseFileV2HeaderError::SemanticNonBypassFalse);
    }
    if header.corpus_stage == CorpusStage::InternalAuditPreFreeze {
        return Err(CaseFileV2HeaderError::CorpusStagePreFreeze);
    }
    Ok(())
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

/// Compute the `CaseFileV2Header` receipt hash. Deterministic
/// across two builds. Two headers differing in any byte produce
/// different hashes.
#[must_use]
pub fn casefile_v2_header_hash(header: &CaseFileV2Header) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(256);
    buf.extend_from_slice(CASEFILE_V2_HEADER_DOMAIN.as_bytes());
    write_str(&mut buf, header.schema.as_str());
    buf.extend_from_slice(&header.corpus_hash_v1);
    write_str(&mut buf, header.corpus_stage.as_str());
    write_str(&mut buf, header.detector_profile.name());
    buf.extend_from_slice(&(header.detector_profile.active_detector_count()).to_be_bytes());
    buf.extend_from_slice(&header.detector_registry_hash);
    write_str(&mut buf, header.atlas_algebra_status.as_str());
    buf.push(u8::from(header.semantic_non_bypass));
    sha256(&buf)
}
