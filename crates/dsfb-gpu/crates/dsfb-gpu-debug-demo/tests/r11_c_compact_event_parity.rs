//! R.11c — `GpuTraceEventCompact` projection + provenance hash
//! acceptance tests.
//!
//! The throughput dispatch now H2Ds a 16-byte compact projection of
//! `TraceEvent` instead of the 48-byte audit form. The compact
//! bytes are a deterministic function of the events; the audit
//! invariants (D64 episode counts + case-file replay determinism)
//! continue to be tested in `r11_b_window_feature_parity.rs`. This
//! file pins the compact-projection-specific invariants:
//!
//!   1. Pack determinism — `pack_compact_event_projection(events)`
//!      is byte-stable across two calls.
//!   2. Pack round-trip — every field a `TraceEvent` carries that
//!      the throughput path reads (`ts_ns`, `entity_id`,
//!      `latency_us`, `error_code != 0`) is recoverable from the
//!      compact form.
//!   3. Hash determinism — `compact_event_projection_hash` is
//!      stable across two calls on identical input.
//!   4. Hash sensitivity — modifying any input event changes the
//!      hash. The hash is a load-bearing provenance anchor; if it
//!      didn't change on input mutation it'd be meaningless.
//!   5. Byte-width budget — the compact form is exactly 16 bytes
//!      per event (3× tighter than the 48-byte `TraceEvent`).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::event::{
    compact_event_projection_hash, pack_compact_event_projection, GpuTraceEventCompact, TraceEvent,
};
use dsfb_gpu_debug_core::fixture::{synthesize, synthesize_scaled, DEFAULT_SEED};

#[test]
fn compact_event_is_exactly_sixteen_bytes() {
    // Locks the byte budget that R.11c was sold on: 48 B/event ->
    // 16 B/event = 3x PCIe reduction at full scale. A regression
    // here (e.g., someone adding a field) would change every
    // recorded throughput-mode case-file hash via the H2D byte
    // count even before any kernel deviation.
    assert_eq!(core::mem::size_of::<GpuTraceEventCompact>(), 16);
    assert_eq!(GpuTraceEventCompact::SIZE, 16);
}

#[test]
fn compact_pack_is_deterministic_across_two_runs() {
    let events = synthesize(DEFAULT_SEED);
    let a = pack_compact_event_projection(&events);
    let b = pack_compact_event_projection(&events);
    assert_eq!(a, b);
}

#[test]
fn compact_pack_round_trips_all_throughput_fields() {
    // Every field the throughput-mode window_feature kernel reads
    // must survive the compact projection without loss:
    //   ts_ns       (u64)  via GpuTraceEventCompact::ts_ns
    //   entity_id   (u32)  via entity_id()
    //   latency_us  (u32)  via latency_us field
    //   error_flag  (bool) via error_nonzero()
    //
    // The structured fixture's `entity_id` values stay well below
    // 2^15, so the 31-bit cap on the packed entity_id field is
    // not exercised here (a u32 max-value entity stays in the
    // unsupported regime).
    let events = synthesize_scaled(DEFAULT_SEED, 64, 512, 4);
    let compact = pack_compact_event_projection(&events);
    assert_eq!(compact.len(), events.len());
    for (i, (ev, c)) in events.iter().zip(compact.iter()).enumerate() {
        assert_eq!(c.ts_ns, ev.ts_ns, "ts_ns mismatch at index {i}");
        assert_eq!(
            c.entity_id(),
            ev.entity_id,
            "entity_id mismatch at index {i}"
        );
        assert_eq!(
            c.latency_us, ev.latency_us,
            "latency_us mismatch at index {i}"
        );
        assert_eq!(
            c.error_nonzero(),
            ev.error_code != 0,
            "error flag mismatch at index {i}"
        );
    }
}

#[test]
fn compact_projection_hash_is_deterministic() {
    let events = synthesize(DEFAULT_SEED);
    let compact = pack_compact_event_projection(&events);
    let h1 = compact_event_projection_hash(&compact);
    let h2 = compact_event_projection_hash(&compact);
    assert_eq!(h1, h2);
    // Repack from the original events; same hash.
    let compact_again = pack_compact_event_projection(&events);
    let h3 = compact_event_projection_hash(&compact_again);
    assert_eq!(h1, h3);
}

#[test]
fn compact_projection_hash_differs_when_events_differ() {
    // Mutate one byte of one field and confirm the projection
    // hash flips. Without this property, a verifier couldn't
    // detect substituted compact bytes during ingest.
    let mut events = synthesize(DEFAULT_SEED);
    let compact_a = pack_compact_event_projection(&events);
    let hash_a = compact_event_projection_hash(&compact_a);

    // Flip `latency_us` on event 0 (any throughput-visible field
    // works; latency is the most "physical" change).
    events[0].latency_us = events[0].latency_us.wrapping_add(1);
    let compact_b = pack_compact_event_projection(&events);
    let hash_b = compact_event_projection_hash(&compact_b);

    assert_ne!(hash_a, hash_b, "hash must respond to a 1-µs latency edit");

    // Restore and flip `error_code` instead; same expectation.
    events[0].latency_us = events[0].latency_us.wrapping_sub(1);
    events[0].error_code = events[0].error_code.wrapping_add(1);
    let compact_c = pack_compact_event_projection(&events);
    let hash_c = compact_event_projection_hash(&compact_c);
    assert_ne!(hash_a, hash_c, "hash must respond to an error_code edit");
}

#[test]
fn compact_pack_does_not_mutate_the_source_events() {
    // The audit-grade `TraceEvent[]` slice must be untouched by
    // packing. The throughput projection is read-only over its
    // input.
    let events_a = synthesize(DEFAULT_SEED);
    let snapshot: Vec<TraceEvent> = events_a.clone();
    let _ = pack_compact_event_projection(&events_a);
    assert_eq!(events_a, snapshot);
}
