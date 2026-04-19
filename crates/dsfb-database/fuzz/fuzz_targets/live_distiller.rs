#![no_main]

//! Fuzz target for `live::distiller::DistillerState::ingest`.
//!
//! The live PostgreSQL adapter pipes `pg_stat_statements`, `pg_stat_activity`,
//! `pg_stat_io`, and `pg_stat_database` snapshots through this distiller
//! to produce typed `ResidualSample`s. The distiller is the second
//! trust boundary on the live path (after the SHA-256-pinned allow-list
//! that gates which queries the adapter may send): an engine returning
//! adversarial counter values — a counter that wraps, a query_id with
//! NUL bytes, an entropy that overflows on `ln`, simultaneous snapshots
//! at the same t — must not panic the observer.
//!
//! Invariants asserted on every emission:
//!   1. `DistillerState::ingest` must never panic.
//!   2. Every returned `ResidualSample.t` is finite.
//!   3. Every returned `ResidualSample.value` is finite.
//!   4. The class of every emitted residual is one of the five
//!      published classes (no orphan variants from a casting bug).
//!   5. Snapshots arriving with non-monotone or duplicate `t` are
//!      tolerated — emission may be empty but never panics.
//!
//! Per Pass-2 plan: this target does NOT modify `src/live/distiller.rs`.
//! If a counterexample fires, it is documented in paper §44
//! (adversarial workload) and deferred to a future pass with a new
//! paper edition.

use dsfb_database::live::distiller::{
    ActivityRow, DistillerState, PgssRow, Snapshot, StatDatabaseRow, StatIoRow,
};
use dsfb_database::residual::ResidualClass;
use libfuzzer_sys::fuzz_target;

/// Decode a u64 counter from 8 bytes of fuzz data, defaulting to 0 on
/// short input.
fn take_u64(data: &[u8], i: &mut usize) -> u64 {
    if *i + 8 > data.len() {
        return 0;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[*i..*i + 8]);
    *i += 8;
    u64::from_le_bytes(buf)
}

fn take_f64(data: &[u8], i: &mut usize) -> f64 {
    if *i + 8 > data.len() {
        return 0.0;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[*i..*i + 8]);
    *i += 8;
    let v = f64::from_le_bytes(buf);
    if !v.is_finite() {
        // The engine is documented to expect finite scrape inputs; the
        // `is_finite` guard here mirrors the adapter-layer's bound and
        // keeps the fuzzer focused on the distiller's logic, not the
        // f64-NaN propagation surface.
        0.0
    } else {
        v
    }
}

fn take_str(data: &[u8], i: &mut usize, max_len: usize) -> String {
    if *i + 1 > data.len() {
        return String::new();
    }
    let len = (data[*i] as usize % max_len.max(1)).min(max_len);
    *i += 1;
    if *i + len > data.len() {
        return String::new();
    }
    let s: String = data[*i..*i + len]
        .iter()
        .map(|b| (b32_alphabet(*b)) as char)
        .collect();
    *i += len;
    s
}

fn b32_alphabet(b: u8) -> u8 {
    // Map every byte to a printable ASCII identifier char so the
    // string-based hash maps in the distiller exercise their hashing
    // path without exotic-codepoint quirks. The fuzzer doesn't need
    // unicode coverage to find bugs here.
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789_-:";
    alphabet[(b as usize) % alphabet.len()]
}

fn build_snapshot(data: &[u8], i: &mut usize) -> Snapshot {
    let t = (take_u64(data, i) as f64) / 1_000.0; // ms granularity, finite-only

    let pgss_n = (take_u64(data, i) as usize) % 8;
    let pgss = (0..pgss_n)
        .map(|_| PgssRow {
            query_id: take_str(data, i, 16),
            calls: take_u64(data, i),
            total_exec_time_ms: take_f64(data, i).abs().min(1e12),
        })
        .collect();

    let act_n = (take_u64(data, i) as usize) % 8;
    let activity = (0..act_n)
        .map(|_| ActivityRow {
            wait_event_type: take_str(data, i, 8),
            wait_event: take_str(data, i, 8),
            state: take_str(data, i, 8),
        })
        .collect();

    let io_n = (take_u64(data, i) as usize) % 8;
    let stat_io = (0..io_n)
        .map(|_| StatIoRow {
            backend_type: take_str(data, i, 8),
            object: take_str(data, i, 8),
            context: take_str(data, i, 8),
            reads: take_u64(data, i),
            hits: take_u64(data, i),
            read_time_ms: take_f64(data, i).abs().min(1e12),
        })
        .collect();

    let db_n = (take_u64(data, i) as usize) % 4;
    let stat_database = (0..db_n)
        .map(|_| StatDatabaseRow {
            datname: take_str(data, i, 8),
            blks_hit: take_u64(data, i),
            blks_read: take_u64(data, i),
        })
        .collect();

    Snapshot {
        t,
        pgss,
        activity,
        stat_io,
        stat_database,
    }
}

fuzz_target!(|data: &[u8]| {
    let mut state = DistillerState::new();
    let mut i = 0;
    // Cap snapshots per iteration so the fuzzer turns over inputs
    // quickly. Coverage on the distiller saturates at ~6 snapshots.
    let max_snaps = 6_usize;
    let mut count = 0;
    while i < data.len() && count < max_snaps {
        let snap = build_snapshot(data, &mut i);
        let out = state.ingest(&snap);
        for r in &out {
            assert!(r.t.is_finite(), "non-finite residual t = {}", r.t);
            assert!(
                r.value.is_finite(),
                "non-finite residual value = {}",
                r.value
            );
            assert!(
                ResidualClass::ALL.contains(&r.class),
                "orphan residual class: {:?}",
                r.class
            );
        }
        count += 1;
    }
});
