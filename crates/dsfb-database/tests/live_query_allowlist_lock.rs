//! Allow-list byte-lock for the live PostgreSQL adapter.
//!
//! The live adapter can only execute SQL strings enumerated in
//! [`dsfb_database::live::AllowedQuery`]. This test pins the
//! SHA-256 of the concatenated SQL texts so that any edit — even a
//! cosmetic reformat — forces an intentional lock bump. The lock
//! value must be updated together with the matching paper revision
//! (Appendix F permission manifest, §Live table of queries).
//!
//! If the fingerprint below is wrong, this test prints the actual
//! value so it is easy to update deliberately. Never update the
//! fingerprint without also checking the paper.

#![cfg(feature = "live-postgres")]

use dsfb_database::live::AllowedQuery;
use sha2::{Digest, Sha256};

const PINNED_SHA256: &str =
    "94c3268154a09be6f71ea5adb72dc85ff68eebb7ad7a2773336e02d02cc749db";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn allow_list_is_pinned() {
    let concat = AllowedQuery::sql_concat_for_lock();
    let got = sha256_hex(concat.as_bytes());
    // First-run bootstrap: compute the fingerprint at test time and
    // write it out in a diagnostic — the first reviewer pins it by
    // replacing `PINNED_SHA256` with the computed value below, and
    // subsequent edits fail the test unless they are intentional.
    //
    // We keep `PINNED_SHA256 = "__DYNAMIC__"` as a sentinel while
    // the allow-list is still being authored; the test is a
    // pre-commit reminder that a lock bump is owed, not a hard
    // failure, until the sentinel is replaced.
    if PINNED_SHA256 == "__DYNAMIC__" {
        eprintln!(
            "ALLOW-LIST LOCK SENTINEL: pin to PINNED_SHA256 = \"{}\" once text stabilises",
            got
        );
        return;
    }
    assert_eq!(
        got, PINNED_SHA256,
        "allow-list SHA-256 drifted.\n  expected {}\n  got      {}\nDid you edit src/live/queries.rs without updating the paper?",
        PINNED_SHA256, got
    );
}

#[test]
fn every_allowed_query_is_a_select() {
    // Independent sanity layer on top of the SHA-256 lock: defence
    // in depth, so that a malicious edit cannot simultaneously
    // silence the lock test by also editing the sentinel back.
    for q in AllowedQuery::ALL.iter() {
        let sql = q.sql();
        assert!(
            sql.trim_start().starts_with("SELECT"),
            "allowed query {:?} is not a SELECT: {}",
            q,
            sql
        );
        for kw in &[
            "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "TRUNCATE", "GRANT", "REVOKE",
            "COPY", "LOCK",
        ] {
            assert!(
                !sql.to_uppercase().contains(kw),
                "allowed query {:?} contains forbidden keyword {}: {}",
                q,
                kw,
                sql
            );
        }
    }
}
