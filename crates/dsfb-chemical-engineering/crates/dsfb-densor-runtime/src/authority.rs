//! Authority hashes — the frozen digests a stage was built against.
//!
//! An `AuthorityHash` names a piece of frozen authority (e.g. a detector atlas, an envelope policy, a dataset
//! corpus) by `(name, 32-byte digest)`. A stage must declare the authorities it consulted; the runtime checks
//! those against the manifest's frozen set before admitting the stage's output. This is the substrate analogue of
//! the chemical crate's `atlas_hash_v1` / `corpus_hash_v1` pinning: an execution is only meaningful relative to a
//! named, frozen authority.

use crate::seal::to_hex;
use serde::{Deserialize, Serialize};

/// A named, frozen authority digest a stage was built/validated against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityHash {
    pub name: String,
    /// 32-byte SHA-256 of the frozen authority's canonical bytes.
    pub hash: [u8; 32],
}

impl AuthorityHash {
    pub fn new(name: impl Into<String>, hash: [u8; 32]) -> Self {
        AuthorityHash {
            name: name.into(),
            hash,
        }
    }

    /// Lowercase hex of the digest, for display / receipts.
    pub fn hex(&self) -> String {
        to_hex(&self.hash)
    }
}
