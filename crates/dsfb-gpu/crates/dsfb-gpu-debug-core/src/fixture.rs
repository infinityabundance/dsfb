//! Deterministic synthetic trace-event fixture.
//!
//! The fixture is the bounded input on which the prior-art replay claim
//! rests. Given the same seed, the synthesizer emits the same 10 000
//! events with the same three injected episodes — every bit of the
//! output is reproducible from the seed.
//!
//! Why a synthesizer and not vendored real data: the prior-art proof is
//! about the architecture, not the dataset. A small, hand-described fixture
//! is auditable (every detector cell can be traced back to a deterministic
//! cause), CI-friendly (no large blob to vendor), and free of licensing
//! concerns. Real-dataset evaluation is paper-§14 work that lives in a
//! different feature flag and is out of scope for the v0 demo.
//!
//! Why three episodes specifically: the canonical bank in Section F is
//! built around eight motifs; three exemplar episodes (latency ramp, error
//! burst, slew-shock with recovery) exercise the residual, sign, and
//! detector layers in distinct enough ways that the downstream pipeline
//! can be observed end-to-end without overcrowding the fixture.
//!
//! Numbers: 10 000 events, 16 entities, 128 windows × 1 s. The synthesizer
//! distributes events evenly in time so each window contains roughly 78
//! events.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::event::TraceEvent;

/// Default LCG seed for the canonical fixture. The hex spelling `0xD5FB`
/// is a mnemonic for "DSFB" so the seed is recognizable in case-file
/// dumps. Changing this changes every byte of the fixture and therefore
/// invalidates any checked-in golden hash.
pub const DEFAULT_SEED: u64 = 0xD5FB_D5FB_D5FB_D5FB;

/// Number of trace events produced by the canonical fixture.
pub const N_EVENTS: usize = 10_000;

/// Number of distinct entities (services / sources).
pub const N_ENTITIES: u32 = 16;

/// Number of distinct routes per entity. Routes are not consumed by the v0
/// pipeline but are recorded so the canonical bytes reflect the full
/// shape.
pub const N_ROUTES_PER_ENTITY: u32 = 4;

/// Number of 1-second windows.
pub const N_WINDOWS: u32 = 128;

/// Window size in nanoseconds. Matches `contract.toml::[windowing]`.
pub const WINDOW_SIZE_NS: u64 = 1_000_000_000;

/// Baseline latency for clean events, in microseconds.
pub const BASELINE_LATENCY_US: u32 = 1_000;

/// LCG-injected jitter band around the baseline latency, in microseconds.
/// Total jitter is `±LATENCY_JITTER_US`.
pub const LATENCY_JITTER_US: u32 = 200;

/// Configuration for the three injected episodes. The fields here are
/// chosen to be cleanly distinguishable from the baseline so that the
/// detector layer (Section D) can find them without ambiguity.
mod episode {
    /// Entity targeted by the latency-ramp episode.
    pub const RAMP_ENTITY: u32 = 3;
    /// First and last window of the ramp (inclusive, exclusive).
    pub const RAMP_WINDOWS: core::ops::Range<u32> = 20..36;
    /// Per-window step that the latency increases by during the ramp,
    /// in microseconds. With 16 windows × 3 000 the ramp tops out at
    /// ~46 ms, well-clear of the baseline + jitter band.
    pub const RAMP_STEP_US: u32 = 3_000;

    /// Entity targeted by the error-burst episode.
    pub const BURST_ENTITY: u32 = 7;
    /// Window range of the burst.
    pub const BURST_WINDOWS: core::ops::Range<u32> = 60..66;
    /// Error code stamped on burst events.
    pub const BURST_ERROR_CODE: u16 = 500;
    /// Status code stamped on burst events.
    pub const BURST_STATUS_CODE: u16 = 500;

    /// Entity targeted by the slew-shock episode.
    pub const SHOCK_ENTITY: u32 = 11;
    /// Single window of the spike.
    pub const SHOCK_WINDOW: u32 = 90;
    /// Spike latency in microseconds (~100 ms, two orders of magnitude
    /// above baseline).
    pub const SHOCK_LATENCY_US: u32 = 100_000;
    /// Range over which the entity recovers after the spike.
    pub const SHOCK_RECOVERY: core::ops::Range<u32> = 91..96;
}

/// A small, fast, fully deterministic linear-congruential generator.
///
/// Why hand-rolled: this crate is zero-dependency. Why LCG: only the
/// statistical-flatness matters here (we don't need cryptographic quality,
/// just deterministic-from-seed jitter), and the constants chosen are the
/// same ones used in the `dsfb-debug` property tests so any future cross-
/// repo comparison stays apples-to-apples.
#[derive(Clone, Copy)]
pub struct Lcg {
    state: u64,
}

impl Lcg {
    /// Seed the generator. The state is taken verbatim — there is no
    /// scrambling step. Two `Lcg::new(seed)` invocations with the same
    /// seed produce identical byte streams.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the state and return the new value. Public so the fixture
    /// synthesizer can use the same generator for ts_ns jitter, latency
    /// jitter, and entity/route distribution.
    pub fn next_u64(&mut self) -> u64 {
        // Knuth's MMIX multiplier + Numerical Recipes increment. These are
        // identical to the constants used by `dsfb-debug`'s property tests
        // (`tests/property_tests.rs::theorem9_holds_over_pseudorandom_inputs`).
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Uniform integer in `[0, range)`. Uses the high 32 bits of the state
    /// for a slightly better distribution than the low ones; modulo bias
    /// is acceptable here because `range` is always small relative to
    /// `u32::MAX`.
    pub fn next_in(&mut self, range: u32) -> u32 {
        (self.next_u64() >> 32) as u32 % range
    }

    /// Symmetric jitter in `[-magnitude, +magnitude]` for the latency
    /// synthesis. Output is an `i32` so callers can add it to an unsigned
    /// baseline with `.saturating_add_signed`.
    pub fn next_jitter(&mut self, magnitude: u32) -> i32 {
        if magnitude == 0 {
            return 0;
        }
        let span = magnitude.saturating_mul(2).saturating_add(1);
        let raw = self.next_in(span) as i32;
        raw - magnitude as i32
    }
}

/// Build the canonical 10 000-event fixture from `seed`.
///
/// Event placement: event `i` lands at `ts_ns = i * (window_size /
/// events_per_window)`, so events are evenly distributed in time. Each
/// event's entity is `i % N_ENTITIES` and the route within that entity is
/// drawn by the LCG. This pattern guarantees that every (window, entity)
/// cell receives at least a few events even before the LCG fires, which
/// makes the residual stage well-conditioned without any seed-dependent
/// "lucky" runs.
#[must_use]
#[allow(clippy::cast_lossless)]
pub fn synthesize(seed: u64) -> Vec<TraceEvent> {
    let mut events: Vec<TraceEvent> = Vec::with_capacity(N_EVENTS);
    let mut rng = Lcg::new(seed);

    // Ticks per event so the 10_000 events span exactly N_WINDOWS windows
    // of WINDOW_SIZE_NS each. With the values above this is 12_800_000 ns
    // per event — about 12.8 ms between adjacent timestamps.
    let ticks_per_event: u64 = (u64::from(N_WINDOWS) * WINDOW_SIZE_NS) / (N_EVENTS as u64);

    for i in 0..N_EVENTS {
        let ts_ns = (i as u64) * ticks_per_event;
        let window: u32 = (ts_ns / WINDOW_SIZE_NS) as u32;
        let entity_id: u32 = (i as u32) % N_ENTITIES;
        // Route within the entity. Deterministic-but-shuffled via the LCG.
        let route_id: u32 = entity_id * N_ROUTES_PER_ENTITY + rng.next_in(N_ROUTES_PER_ENTITY);

        // Baseline-with-jitter latency. The LCG advances once per event so
        // the synthesizer's bytes change if any earlier event is modified.
        let jitter = rng.next_jitter(LATENCY_JITTER_US);
        let mut latency_us: u32 = (BASELINE_LATENCY_US as i32 + jitter).max(1) as u32;
        let mut status_code: u16 = 200;
        let mut error_code: u16 = 0;

        // Episode 1: latency ramp on entity RAMP_ENTITY across the
        // RAMP_WINDOWS range. The ramp slope is RAMP_STEP_US per window
        // beyond the start of the range.
        if entity_id == episode::RAMP_ENTITY && episode::RAMP_WINDOWS.contains(&window) {
            let steps = window - episode::RAMP_WINDOWS.start;
            latency_us = latency_us.saturating_add(steps * episode::RAMP_STEP_US);
        }

        // Episode 2: error burst on entity BURST_ENTITY across BURST_WINDOWS.
        // All events from that entity within the window range come back
        // with the burst status/error codes; the residual layer will see
        // a sustained error-rate excursion.
        if entity_id == episode::BURST_ENTITY && episode::BURST_WINDOWS.contains(&window) {
            status_code = episode::BURST_STATUS_CODE;
            error_code = episode::BURST_ERROR_CODE;
        }

        // Episode 3a: slew shock — one window of very high latency on
        // entity SHOCK_ENTITY.
        if entity_id == episode::SHOCK_ENTITY && window == episode::SHOCK_WINDOW {
            latency_us = episode::SHOCK_LATENCY_US;
        }

        // Episode 3b: recovery — latency relaxes back toward baseline over
        // the SHOCK_RECOVERY range. Linear taper from 25 ms down to ~1 ms.
        if entity_id == episode::SHOCK_ENTITY && episode::SHOCK_RECOVERY.contains(&window) {
            let steps_in = window - episode::SHOCK_RECOVERY.start;
            let span = episode::SHOCK_RECOVERY.end - episode::SHOCK_RECOVERY.start;
            // Taper from 25_000 µs (start) down to 1_000 µs (just past end).
            let taper_high: u32 = 25_000;
            let taper_low: u32 = BASELINE_LATENCY_US;
            let taper = taper_high - ((taper_high - taper_low) * steps_in) / span.max(1);
            latency_us = taper;
        }

        // Spans: deterministic from event index so a fixture replay carries
        // the same span identity through the case file. Parent of event i
        // is event i-1 within the same entity, else 0.
        let span_id = (i as u64) + 1;
        let parent_span_id = if i >= N_ENTITIES as usize {
            span_id - u64::from(N_ENTITIES)
        } else {
            0
        };

        events.push(TraceEvent {
            ts_ns,
            entity_id,
            route_id,
            span_id,
            parent_span_id,
            latency_us,
            status_code,
            error_code,
            event_kind: 1,
            flags: 0,
        });
    }

    events
}

/// Synthesize a deterministic fixture sized for a scaled contract.
///
/// Unlike [`synthesize`] which produces the v0 canonical 10 000-event
/// fixture, this function emits `events_per_cell × n_entities ×
/// n_windows` events distributed evenly across the grid. The same three
/// injected episodes (latency ramp, error burst, slew-shock + recovery)
/// are scaled to the new grid so the bank stage still has work to do.
///
/// Determinism guarantees: the LCG with `seed` is the only source of
/// randomness; two calls with the same `(seed, n_entities, n_windows,
/// events_per_cell)` produce byte-identical event streams. The cost of
/// generating a 256×1024-grid fixture with 4 events per cell is roughly
/// 1 million events; on a release build this takes ~10 ms.
#[must_use]
#[allow(clippy::cast_possible_wrap)]
pub fn synthesize_scaled(
    seed: u64,
    n_entities: u32,
    n_windows: u32,
    events_per_cell: u32,
) -> Vec<TraceEvent> {
    let total_events: usize =
        (n_entities as usize) * (n_windows as usize) * (events_per_cell as usize);
    let mut events: Vec<TraceEvent> = Vec::with_capacity(total_events);
    let mut rng = Lcg::new(seed);

    // Distribute events evenly in time across n_windows × WINDOW_SIZE_NS.
    // Each event lands in window `i / events_per_window` so the grid is
    // cleanly covered.
    let ticks_per_event: u64 = if total_events == 0 {
        1
    } else {
        (u64::from(n_windows) * WINDOW_SIZE_NS) / (total_events as u64).max(1)
    };

    // Scaled episode windows: keep the same relative position as the v0
    // fixture (ramp at 20/128, burst at 60/128, shock at 90/128).
    let ramp_entity = if n_entities > 3 { 3 } else { 0 };
    let burst_entity = if n_entities > 7 {
        7
    } else {
        n_entities.saturating_sub(1)
    };
    let shock_entity = if n_entities > 11 {
        11
    } else {
        n_entities.saturating_sub(1)
    };
    let scale_w = |w: u32| -> u32 { (u64::from(w) * u64::from(n_windows) / 128) as u32 };
    let ramp_start = scale_w(20);
    let ramp_end = scale_w(36);
    let ramp_step_us: u32 = 3_000;
    let burst_start = scale_w(60);
    let burst_end = scale_w(66);
    let shock_window = scale_w(90);
    let shock_recovery_start = scale_w(91);
    let shock_recovery_end = scale_w(96);

    for i in 0..total_events {
        let ts_ns = (i as u64) * ticks_per_event;
        let window: u32 = ((ts_ns / WINDOW_SIZE_NS) as u32).min(n_windows.saturating_sub(1));
        let entity_id: u32 = (i as u32) % n_entities;
        let route_id: u32 = entity_id * N_ROUTES_PER_ENTITY + rng.next_in(N_ROUTES_PER_ENTITY);
        let jitter = rng.next_jitter(LATENCY_JITTER_US);
        let mut latency_us: u32 = (BASELINE_LATENCY_US as i32 + jitter).max(1) as u32;
        let mut status_code: u16 = 200;
        let mut error_code: u16 = 0;

        if entity_id == ramp_entity && (ramp_start..ramp_end).contains(&window) {
            let steps = window - ramp_start;
            latency_us = latency_us.saturating_add(steps * ramp_step_us);
        }
        if entity_id == burst_entity && (burst_start..burst_end).contains(&window) {
            status_code = 500;
            error_code = 500;
        }
        if entity_id == shock_entity && window == shock_window {
            latency_us = 100_000;
        }
        if entity_id == shock_entity && (shock_recovery_start..shock_recovery_end).contains(&window)
        {
            let span = (shock_recovery_end - shock_recovery_start).max(1);
            let steps_in = window - shock_recovery_start;
            let taper_high: u32 = 25_000;
            let taper_low: u32 = BASELINE_LATENCY_US;
            latency_us = taper_high - ((taper_high - taper_low) * steps_in) / span;
        }

        let span_id = (i as u64) + 1;
        let parent_span_id = if i >= n_entities as usize {
            span_id - u64::from(n_entities)
        } else {
            0
        };

        events.push(TraceEvent {
            ts_ns,
            entity_id,
            route_id,
            span_id,
            parent_span_id,
            latency_us,
            status_code,
            error_code,
            event_kind: 1,
            flags: 0,
        });
    }
    events
}

/// R.2 courthouse-factory fixture — produce `n_catalogs` independent
/// in-memory `TraceEvent` streams at the given `(n_entities, n_windows)`
/// grid, all of them seed-deterministic.
///
/// The "courthouse-factory" framing comes from Section R: a single
/// dsfb-gpu-debug court (one catalog) is too small to show the GPU's
/// natural regime; the real prior-art claim is that the GPU runs **many
/// independent deterministic courts in parallel**. This generator is the
/// data feed that lets the bench evaluate that regime without writing
/// fixtures to disk first.
///
/// Each catalog `c` uses a derived seed
/// `seed ^ (c.wrapping_mul(0x9E37_79B9_7F4A_7C15))` (the Knuth golden
/// ratio constant) so that:
///   * catalog 0 reproduces `synthesize_scaled(seed, ...)` for canonical
///     seeds (when applicable), and
///   * adjacent catalogs are uncorrelated bytewise (no LCG drift between
///     them).
///
/// `events_per_cell` defaults to 4 in the bench harness, matching
/// `synthesize_scaled`. Two consecutive calls with the same arguments
/// produce byte-identical catalog vectors — pinned by the unit test
/// `courthouse_factory_is_deterministic_from_seed`.
///
/// Memory note: at the panel's "money-table" profile (K=64, entities=256,
/// windows=4096, events_per_cell=4) this allocates 64 × 256 × 4096 × 4
/// ≈ 268 M `TraceEvent`s ≈ 12.8 GB. Callers should check available RAM
/// before invoking at large profiles; the bench surfaces an honest
/// `--catalogs` ceiling.
#[must_use]
pub fn synthesize_courthouse_factory(
    seed: u64,
    n_catalogs: u32,
    n_entities: u32,
    n_windows: u32,
    events_per_cell: u32,
) -> Vec<Vec<TraceEvent>> {
    // Knuth's multiplicative constant for u64. Used to spread base-seed
    // bits before XORing with the catalog index so adjacent catalogs do
    // not share their LCG trajectory.
    const KNUTH_U64: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut catalogs: Vec<Vec<TraceEvent>> = Vec::with_capacity(n_catalogs as usize);
    for c in 0..u64::from(n_catalogs) {
        let derived_seed = seed ^ c.wrapping_mul(KNUTH_U64);
        catalogs.push(synthesize_scaled(
            derived_seed,
            n_entities,
            n_windows,
            events_per_cell,
        ));
    }
    catalogs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_is_deterministic_from_seed() {
        let mut a = Lcg::new(0xD5FB);
        let mut b = Lcg::new(0xD5FB);
        for _ in 0..256 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn lcg_different_seeds_produce_different_streams() {
        let mut a = Lcg::new(1);
        let mut b = Lcg::new(2);
        // Same prefix would be a strong sign the LCG is broken.
        let stream_a: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
        let stream_b: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
        assert_ne!(stream_a, stream_b);
    }

    #[test]
    fn synthesize_produces_exactly_n_events() {
        let events = synthesize(DEFAULT_SEED);
        assert_eq!(events.len(), N_EVENTS);
    }

    #[test]
    fn synthesize_is_deterministic_from_seed() {
        let a = synthesize(DEFAULT_SEED);
        let b = synthesize(DEFAULT_SEED);
        assert_eq!(a, b, "same seed must yield same events");
    }

    #[test]
    fn synthesize_distinct_seeds_distinct_fixtures() {
        let a = synthesize(DEFAULT_SEED);
        let b = synthesize(DEFAULT_SEED.wrapping_add(1));
        // The two fixtures share their first event (jitter has not fired yet
        // for the first call's effect to differ from the bare baseline), but
        // they must differ overall.
        assert_ne!(a, b);
    }

    #[test]
    fn synthesize_spans_the_expected_window_range() {
        let events = synthesize(DEFAULT_SEED);
        let min_window = events
            .iter()
            .map(|e| e.window_index(WINDOW_SIZE_NS))
            .min()
            .unwrap();
        let max_window = events
            .iter()
            .map(|e| e.window_index(WINDOW_SIZE_NS))
            .max()
            .unwrap();
        assert_eq!(min_window, 0);
        assert!(
            max_window == N_WINDOWS - 1 || max_window == N_WINDOWS,
            "max window observed was {max_window}, expected {} or {}",
            N_WINDOWS - 1,
            N_WINDOWS
        );
    }

    #[test]
    fn synthesize_uses_every_entity() {
        let events = synthesize(DEFAULT_SEED);
        let mut seen = [false; N_ENTITIES as usize];
        for event in &events {
            seen[event.entity_id as usize] = true;
        }
        assert!(
            seen.iter().all(|s| *s),
            "every entity_id 0..N_ENTITIES must appear"
        );
    }

    #[test]
    fn synthesize_injects_latency_ramp_episode() {
        let events = synthesize(DEFAULT_SEED);
        // Latency at the start of the ramp must be markedly above baseline
        // by the end of the ramp window range.
        let near_end_window = episode::RAMP_WINDOWS.end - 1;
        let ramp_events: Vec<_> = events
            .iter()
            .filter(|e| {
                e.entity_id == episode::RAMP_ENTITY
                    && e.window_index(WINDOW_SIZE_NS) == near_end_window
            })
            .collect();
        assert!(
            !ramp_events.is_empty(),
            "expected at least one ramp event near the end of the range"
        );
        for event in ramp_events {
            assert!(
                event.latency_us > BASELINE_LATENCY_US + LATENCY_JITTER_US + 30_000,
                "ramp event latency {} is not elevated beyond baseline+jitter+ramp",
                event.latency_us
            );
        }
    }

    #[test]
    fn synthesize_injects_error_burst_episode() {
        let events = synthesize(DEFAULT_SEED);
        let burst_events: Vec<_> = events
            .iter()
            .filter(|e| {
                e.entity_id == episode::BURST_ENTITY
                    && episode::BURST_WINDOWS.contains(&e.window_index(WINDOW_SIZE_NS))
            })
            .collect();
        assert!(
            !burst_events.is_empty(),
            "expected events in the burst window range"
        );
        for event in burst_events {
            assert_eq!(event.error_code, episode::BURST_ERROR_CODE);
            assert_eq!(event.status_code, episode::BURST_STATUS_CODE);
        }
    }

    #[test]
    fn synthesize_injects_slew_shock_episode() {
        let events = synthesize(DEFAULT_SEED);
        let shock_events: Vec<_> = events
            .iter()
            .filter(|e| {
                e.entity_id == episode::SHOCK_ENTITY
                    && e.window_index(WINDOW_SIZE_NS) == episode::SHOCK_WINDOW
            })
            .collect();
        assert!(
            !shock_events.is_empty(),
            "expected events in the shock window"
        );
        for event in shock_events {
            assert_eq!(event.latency_us, episode::SHOCK_LATENCY_US);
        }
    }

    #[test]
    fn synthesize_round_trips_through_canonical_bytes() {
        use crate::serialize::{read_fixture, write_fixture};
        let events = synthesize(DEFAULT_SEED);
        let bytes = write_fixture(&events);
        let parsed = read_fixture(&bytes).expect("synthesized fixture parses");
        assert_eq!(parsed, events);
    }

    #[test]
    fn synthesize_canonical_bytes_are_stable_across_calls() {
        // The load-bearing property: two calls produce byte-identical
        // canonical fixtures. This is the spec's test #1 — CPU replay byte
        // identical — applied to the fixture stage of the pipeline.
        use crate::hash::sha256;
        use crate::serialize::write_fixture;
        let a = write_fixture(&synthesize(DEFAULT_SEED));
        let b = write_fixture(&synthesize(DEFAULT_SEED));
        assert_eq!(a, b);
        assert_eq!(sha256(&a), sha256(&b));
    }

    #[test]
    fn courthouse_factory_is_deterministic_from_seed() {
        // The R.2 courthouse-factory generator must be byte-identical
        // across two calls with the same arguments. This is the
        // load-bearing replay property at the multi-catalog level —
        // every benchmark that batches K catalogs depends on this.
        let a = synthesize_courthouse_factory(DEFAULT_SEED, 4, 16, 32, 2);
        let b = synthesize_courthouse_factory(DEFAULT_SEED, 4, 16, 32, 2);
        assert_eq!(a, b, "courthouse-factory must replay byte-identically");
    }

    #[test]
    fn courthouse_factory_per_catalog_independence() {
        // Adjacent catalogs must be uncorrelated: catalog 0 and catalog 1
        // share the same dimensions but a different derived seed, so
        // their event streams should differ. If they were equal the
        // batched bench would over-count repeated work and lie about
        // per-catalog independence.
        let factory = synthesize_courthouse_factory(DEFAULT_SEED, 4, 16, 32, 2);
        assert_eq!(factory.len(), 4);
        assert_ne!(
            factory[0], factory[1],
            "catalog 0 and 1 must have different event streams (R.2 per-catalog independence)"
        );
        assert_ne!(factory[1], factory[2]);
    }

    #[test]
    fn courthouse_factory_catalog0_matches_synthesize_scaled() {
        // Documented invariant: with `seed ^ 0 = seed`, catalog 0 from
        // the factory equals `synthesize_scaled(seed, ...)`. This makes
        // it easy to swap a single-catalog bench into the K=1 factory
        // path without changing the bytes.
        let factory = synthesize_courthouse_factory(DEFAULT_SEED, 1, 16, 32, 2);
        let scaled = synthesize_scaled(DEFAULT_SEED, 16, 32, 2);
        assert_eq!(factory[0], scaled);
    }
}
