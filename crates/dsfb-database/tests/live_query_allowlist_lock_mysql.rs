//! Allow-list byte-lock for the live MySQL adapter.
//!
//! The live-mysql adapter can only execute SQL strings enumerated in
//! [`dsfb_database::live_mysql::AllowedMySqlQuery`]. This test pins
//! the SHA-256 of the concatenated SQL texts so that any edit — even
//! a cosmetic reformat — forces an intentional lock bump. The lock
//! value must be updated together with the matching paper revision
//! (`spec/permissions.mysql.sql`, §Live-Eval MySQL subsection).
//!
//! The allow-list enum is always compiled (it lives in library mode
//! behind `src/live_mysql/queries.rs` without requiring
//! `mysql_async`), so this lock runs on every `cargo test` invocation
//! regardless of feature flags. This is deliberate: the statement-
//! level control's auditability is independent of whether the
//! runtime connection wrapper is compiled in.
//!
//! If the fingerprint below is wrong, this test prints the actual
//! value so it is easy to update deliberately. Never update the
//! fingerprint without also checking the paper and the permissions
//! manifest.

use dsfb_database::live_mysql::AllowedMySqlQuery;
use sha2::{Digest, Sha256};

const PINNED_SHA256: &str =
    "f6581d915ba6707669f839e317beaa13d6cf06cbc672a07b8dae3226de12a7f1";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn allow_list_is_pinned() {
    let concat = AllowedMySqlQuery::sql_concat_for_lock();
    let got = sha256_hex(concat.as_bytes());
    assert_eq!(
        got, PINNED_SHA256,
        "AllowedMySqlQuery allow-list drifted from pinned SHA-256. \
         Expected {}, got {}. Update the paper's §Live-Eval MySQL \
         subsection and spec/permissions.mysql.sql together with the \
         new hash.",
        PINNED_SHA256, got
    );
}

#[test]
fn allow_list_cardinality_is_four() {
    // A silent addition of a fifth variant would change the
    // concatenation (picked up by `allow_list_is_pinned`) but a
    // re-order without content change would not. Pin the cardinality
    // explicitly so a refactor that swaps variants out-for-in also
    // fails the lock.
    assert_eq!(
        AllowedMySqlQuery::ALL.len(),
        4,
        "AllowedMySqlQuery variant count drifted; update the paper §Live-Eval MySQL table"
    );
}

#[test]
fn every_variant_is_unique() {
    let mut seen = std::collections::HashSet::new();
    for q in AllowedMySqlQuery::ALL {
        assert!(
            seen.insert(q),
            "duplicate variant in AllowedMySqlQuery::ALL: {:?}",
            q
        );
    }
}
