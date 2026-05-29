//! `ConfuserDocketV1` — a per-episode confuser docket keyed to the matched fault (P60).
//!
//! The atlas fault-signature bank stores, *statically*, each signature's `confuser_faults` (the
//! mechanisms it can be mistaken for) and its `discriminating_signature` (what tells them apart). That
//! is authority data. This is the **execution-side emission**: when an episode is matched to a fault
//! signature, emit a first-class, hash-sealed docket *for that episode* recording which confusers were
//! considered and the discriminating evidence — so the case file shows not just *what* was matched but
//! *what was ruled out and why*. Reads the atlas authority (the docket cites the catalogued confusers,
//! it does not invent them).
//!
//! Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;
use dsfb_chemical_engineering_atlas as authority;

/// A per-episode confuser docket (schema v1): the confusers considered for a matched fault and the
/// discriminating signature that justified ruling them out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfuserDocketV1 {
    /// Which episode this docket is about (e.g. `"idx=120..168"` or an episode id).
    pub episode_ref: String,
    /// The matched fault signature's id (atlas `fault_id`).
    pub matched_fault_id: String,
    /// The matched fault's human-readable name.
    pub matched_fault_name: String,
    /// What distinguishes the matched fault from its confusers (the atlas `discriminating_signature`).
    pub discriminating_signature: String,
    /// The confuser mechanisms considered + ruled out (the atlas `confuser_faults`), in order.
    pub confusers: Vec<String>,
    /// SHA-256 (via [`CanonicalHasher`]) sealing the docket.
    pub docket_hash: String,
}

impl ConfuserDocketV1 {
    fn seal(
        episode_ref: &str,
        matched_fault_id: &str,
        matched_fault_name: &str,
        discriminating_signature: &str,
        confusers: &[String],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"confuser_docket_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        h.field("matched_fault_id", matched_fault_id.as_bytes());
        h.field("matched_fault_name", matched_fault_name.as_bytes());
        h.field(
            "discriminating_signature",
            discriminating_signature.as_bytes(),
        );
        for c in confusers {
            h.field("confuser", c.as_bytes());
        }
        h.finalize_hex()
    }

    /// Emit a confuser docket for an episode matched to atlas `fault_id`. Returns `None` if the fault id
    /// is not catalogued (the docket only ever cites real authority records). The confusers + the
    /// discriminating signature are read from the atlas; the docket is sealed for this episode.
    pub fn for_fault(episode_ref: impl Into<String>, fault_id: &str) -> Option<Self> {
        let f = authority::FAULT_SIGNATURES
            .iter()
            .find(|s| s.fault_id == fault_id)?;
        let episode_ref = episode_ref.into();
        let matched_fault_id = f.fault_id.to_string();
        let matched_fault_name = f.name.to_string();
        let discriminating_signature = f.discriminating_signature.to_string();
        let confusers: Vec<String> = f.confuser_faults.iter().map(|c| c.to_string()).collect();
        let docket_hash = Self::seal(
            &episode_ref,
            &matched_fault_id,
            &matched_fault_name,
            &discriminating_signature,
            &confusers,
        );
        Some(ConfuserDocketV1 {
            episode_ref,
            matched_fault_id,
            matched_fault_name,
            discriminating_signature,
            confusers,
            docket_hash,
        })
    }

    /// Re-derive the seal from the record's own fields and check it matches `docket_hash`.
    pub fn verify(&self) -> bool {
        Self::seal(
            &self.episode_ref,
            &self.matched_fault_id,
            &self.matched_fault_name,
            &self.discriminating_signature,
            &self.confusers,
        ) == self.docket_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docket_cites_real_atlas_confusers_and_self_verifies() {
        // F1 (valve stiction) is a catalogued, executed fault signature with confusers.
        let fid = authority::FAULT_SIGNATURES[0].fault_id;
        let d = ConfuserDocketV1::for_fault("idx=10..40", fid).expect("catalogued fault");
        assert_eq!(d.matched_fault_id, fid);
        assert!(
            !d.confusers.is_empty(),
            "the matched signature lists confusers to rule out"
        );
        // The confusers match the atlas record exactly (the docket does not invent any).
        let atlas_confusers: Vec<String> = authority::FAULT_SIGNATURES[0]
            .confuser_faults
            .iter()
            .map(|c| c.to_string())
            .collect();
        assert_eq!(d.confusers, atlas_confusers);
        assert!(d.verify());
        assert_eq!(d.docket_hash.len(), 64);
    }

    #[test]
    fn uncatalogued_fault_yields_no_docket() {
        assert!(ConfuserDocketV1::for_fault("e", "not_a_real_fault_id").is_none());
    }

    #[test]
    fn deterministic_and_tamper_evident() {
        let fid = authority::FAULT_SIGNATURES[0].fault_id;
        let a = ConfuserDocketV1::for_fault("e", fid).unwrap();
        let b = ConfuserDocketV1::for_fault("e", fid).unwrap();
        assert_eq!(a.docket_hash, b.docket_hash);
        let mut t = a.clone();
        t.confusers.push("fabricated confuser".into());
        assert!(!t.verify());
    }
}
