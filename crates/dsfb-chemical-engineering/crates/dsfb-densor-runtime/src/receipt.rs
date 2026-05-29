//! The run-level sealed receipt — the tamper-evident record of a whole pipeline run.
//!
//! A `RuntimeReceiptV1` collects the ordered [`StageReceiptSummary`]s of an executed pipeline and seals them
//! (with the manifest's pipeline id + frozen-authority digest) into a single `receipt_hash`. It is the substrate
//! analogue of the chemical court record's `bundle_root`: a deterministic, re-derivable digest over exactly what
//! ran, so a verifier can confirm the run was not altered after the fact.

use crate::manifest::DensorManifest;
use crate::seal::{to_hex, CanonicalHasher};
use crate::stage::StageReceiptSummary;
use serde::{Deserialize, Serialize};

/// A sealed record of one pipeline run (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeReceiptV1 {
    pub pipeline_id: String,
    /// The ordered per-stage receipt summaries (input/output hashes + authorities).
    pub stages: Vec<StageReceiptSummary>,
    /// The load-bearing non-claim.
    pub non_claim: String,
    /// SHA-256 (via [`CanonicalHasher`]) over the pipeline id, the frozen-authority allow-list, and every stage
    /// summary in order — the run's tamper-evident root.
    pub receipt_hash: String,
}

impl RuntimeReceiptV1 {
    pub const NON_CLAIM: &'static str = "A deterministic record of which stages ran over which densors against \
        which frozen authorities — NOT a claim that the result is correct, optimal, or meaningful in any domain; \
        the runtime is a mechanism, the meaning lives in the authorities it cites.";

    fn seal(
        pipeline_id: &str,
        manifest: &DensorManifest,
        stages: &[StageReceiptSummary],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"dsfb_densor_runtime_receipt_v1");
        h.field("pipeline_id", pipeline_id.as_bytes());
        // The frozen-authority allow-list (name+digest, in manifest order) — binds the run to its anchors.
        for a in &manifest.authorities {
            h.field("authority_name", a.name.as_bytes());
            h.hash32("authority_hash", &a.hash);
        }
        for s in stages {
            h.field("stage_id", s.stage_id.as_bytes());
            h.hash32("input_hash", &s.input_hash);
            h.hash32("output_hash", &s.output_hash);
            for a in &s.authority_hashes {
                h.field("stage_authority", a.name.as_bytes());
                h.hash32("stage_authority_hash", &a.hash);
            }
        }
        h.field("non_claim", Self::NON_CLAIM.as_bytes());
        h.finalize_hex()
    }

    /// Build a sealed run receipt from the manifest + the ordered stage summaries.
    pub fn build(manifest: &DensorManifest, stages: Vec<StageReceiptSummary>) -> Self {
        let receipt_hash = Self::seal(&manifest.pipeline_id, manifest, &stages);
        RuntimeReceiptV1 {
            pipeline_id: manifest.pipeline_id.clone(),
            stages,
            non_claim: Self::NON_CLAIM.to_string(),
            receipt_hash,
        }
    }

    /// Re-derive the seal and confirm it matches (tamper-evident). Requires the same manifest the run used.
    pub fn verify(&self, manifest: &DensorManifest) -> bool {
        self.non_claim == Self::NON_CLAIM
            && self.receipt_hash == Self::seal(&self.pipeline_id, manifest, &self.stages)
    }

    /// The 12-char short form of the receipt hash, for logs.
    pub fn short(&self) -> &str {
        if self.receipt_hash.len() >= 12 {
            &self.receipt_hash[..12]
        } else {
            &self.receipt_hash
        }
    }
}

/// Convenience: hex of a raw digest (re-exported through the crate root for callers building receipts by hand).
pub fn hex(h: &[u8; 32]) -> String {
    to_hex(h)
}
