//! A small inventory view over a sealed run — "what ran, against what, sealing to what".
//!
//! `RuntimeIndex` is the navigable summary of a [`RuntimeReceiptV1`]: the pipeline id, the ordered stage ids, the
//! distinct authorities cited, and the sealed receipt hash. It is the runtime's analogue of the artifact index —
//! a read-only re-presentation of already-sealed evidence, useful for logs and dashboards.

use crate::receipt::RuntimeReceiptV1;
use std::collections::BTreeSet;

/// A read-only summary of one sealed run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIndex {
    pub pipeline_id: String,
    pub stage_ids: Vec<String>,
    pub authorities: Vec<String>,
    pub receipt_hash: String,
}

impl RuntimeIndex {
    /// Summarise a sealed receipt. Authorities are de-duplicated and sorted (deterministic).
    pub fn of(receipt: &RuntimeReceiptV1) -> Self {
        let stage_ids = receipt.stages.iter().map(|s| s.stage_id.clone()).collect();
        let mut auth: BTreeSet<String> = BTreeSet::new();
        for s in &receipt.stages {
            for a in &s.authority_hashes {
                auth.insert(a.name.clone());
            }
        }
        RuntimeIndex {
            pipeline_id: receipt.pipeline_id.clone(),
            stage_ids,
            authorities: auth.into_iter().collect(),
            receipt_hash: receipt.receipt_hash.clone(),
        }
    }

    /// A one-line human summary.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: {} stage(s) [{}] over authorities [{}] -> {}",
            self.pipeline_id,
            self.stage_ids.len(),
            self.stage_ids.join(", "),
            self.authorities.join(", "),
            &self.receipt_hash[..self.receipt_hash.len().min(12)],
        )
    }
}
