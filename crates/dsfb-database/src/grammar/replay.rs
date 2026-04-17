//! Deterministic replay.
//!
//! The grammar produces episodes that are bytewise identical across runs of
//! the same `(stream, grammar)` pair. We hash the episode list with SHA-256
//! and check the digest against a stored value; the test
//! `tests/deterministic_replay.rs` enforces this property end-to-end.

use super::Episode;
use sha2::{Digest, Sha256};

pub fn fingerprint(episodes: &[Episode]) -> [u8; 32] {
    let mut h = Sha256::new();
    for e in episodes {
        h.update((e.motif as u8).to_le_bytes());
        if let Some(c) = &e.channel {
            h.update(c.as_bytes());
        }
        h.update(b"|");
        h.update(e.t_start.to_le_bytes());
        h.update(e.t_end.to_le_bytes());
        h.update(e.peak.to_le_bytes());
        h.update(e.ema_at_boundary.to_le_bytes());
        h.update(e.trust_sum.to_le_bytes());
    }
    h.finalize().into()
}

pub fn fingerprint_hex(episodes: &[Episode]) -> String {
    fingerprint(episodes)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
