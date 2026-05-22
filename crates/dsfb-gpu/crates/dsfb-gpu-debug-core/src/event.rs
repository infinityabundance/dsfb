//! The canonical trace-event record.
//!
//! `TraceEvent` is the input atom of the DSFB-GPU-Debug pipeline. Each
//! record represents one observable point in the debug catalog — a span
//! emitted by a tracing system, a latency sample, an error stamp. The shape
//! is intentionally narrow: we keep only the fields the downstream pipeline
//! actually reads, so the canonical byte form is small and the hash chain
//! stays cheap to compute.
//!
//! Layout decisions:
//!
//! * `#[repr(C)]` so the struct can cross the FFI boundary into CUDA
//!   without per-field marshaling. The CUDA-side mirror in
//!   `cuda/layout.cuh` (introduced in Section H) lays the fields out
//!   identically.
//! * Field order is part of the in-crate canonical trace contract. Changing
//!   this order would change the catalog hash and therefore the case-file
//!   hash, so the order is not to be reshuffled.
//! * Widths chosen for the bounded v0 demo: `latency_us: u32` covers up to
//!   ~71 minutes of latency (way beyond the contract clamp of 32 767 ms),
//!   `entity_id` and `route_id` are `u32` to leave room without paying for
//!   `u64`, and the 16-bit status/error/kind/flags fields keep the total
//!   record size small.
//!
//! Stability: this type is deserialized from the canonical JSON fixture and
//! re-serialized into the case file's `input_catalog_hash`. The
//! serialization order is fixed by `serialize::write_event` (canonical key
//! ordering); modifying field names or order requires bumping the contract
//! version.

/// A single observable trace point.
///
/// All fields are unsigned; the deserializer rejects negative literals.
/// The constructors offered here are intentionally limited — callers should
/// build events via the fixture synthesizer or by parsing canonical JSON,
/// not by direct field assignment, so the bit-stability rules around field
/// widths are honored automatically.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct TraceEvent {
    /// Wall-clock timestamp, nanoseconds since the synthetic epoch (event
    /// 0). The pipeline derives the window index from this field via
    /// integer division by the contract's `window_size_ms * 1_000_000`.
    pub ts_ns: u64,
    /// The entity (service, process, sensor) the event belongs to.
    /// Bounded by `n_entities` in the contract.
    pub entity_id: u32,
    /// The route within the entity (e.g. an HTTP endpoint or a sensor
    /// channel). Not used by the v0 pipeline for windowing but carried
    /// into the canonical bytes so the hash chain reflects the full
    /// fixture shape.
    pub route_id: u32,
    /// Tracing span identifier. Carried for fidelity of the catalog hash;
    /// not consumed by the residual/sign stages.
    pub span_id: u64,
    /// Parent span identifier, or `0` for a root.
    pub parent_span_id: u64,
    /// Observed latency in microseconds. Clamped to
    /// `contract.numeric.latency_clamp_ms * 1000` at the residual boundary
    /// to keep Q16.16 quantization in range.
    pub latency_us: u32,
    /// HTTP-style status code (or a domain-specific equivalent). Carried
    /// through to the canonical bytes; the v0 pipeline checks for the
    /// error class via `error_code` not `status_code`.
    pub status_code: u16,
    /// Domain error code, with `0` meaning no error. Drives the error-rate
    /// derivative of the residual.
    pub error_code: u16,
    /// Free-form event-type discriminator (request, response, log,
    /// sample). Not consumed in v0 but preserved in the catalog hash.
    pub event_kind: u16,
    /// Bit-field for miscellaneous flags (sampled, replayed, simulated).
    /// Preserved verbatim for the catalog hash.
    pub flags: u16,
}

impl TraceEvent {
    /// Construct a fully-specified `TraceEvent`. Used by the fixture
    /// synthesizer and tests. There is no builder pattern on purpose —
    /// the type is small enough that positional construction is more
    /// auditable than a fluent builder.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        ts_ns: u64,
        entity_id: u32,
        route_id: u32,
        span_id: u64,
        parent_span_id: u64,
        latency_us: u32,
        status_code: u16,
        error_code: u16,
        event_kind: u16,
        flags: u16,
    ) -> Self {
        Self {
            ts_ns,
            entity_id,
            route_id,
            span_id,
            parent_span_id,
            latency_us,
            status_code,
            error_code,
            event_kind,
            flags,
        }
    }

    /// The window index this event falls into, given a window size in
    /// nanoseconds. Integer division; ties go to the lower window.
    #[must_use]
    pub const fn window_index(&self, window_size_ns: u64) -> u32 {
        // Saturate at u32::MAX rather than wrapping if the synthetic clock
        // ever exceeds 2^32 windows; the contract's bounded fixtures stay
        // well under that.
        let raw = self.ts_ns / window_size_ns;
        if raw > u32::MAX as u64 {
            u32::MAX
        } else {
            raw as u32
        }
    }
}

/// R.11c — compact GPU-ingest projection of `TraceEvent`. Carries
/// only the four fields the `window_feature_kernel_structured`
/// kernel actually reads (`ts_ns`, `entity_id`, `latency_us`, and
/// the `error_code != 0` flag), packed into 16 bytes. Throughput
/// dispatches H2D this projection instead of the 48-byte audit-
/// grade `TraceEvent`, cutting PCIe payload ~3× at full scale
/// (192 MB → 64 MB at K=128 256×4096).
///
/// **Audit invariance**: the 48-byte `TraceEvent` byte form, the
/// Audit-mode FFI, and every D16 audit-chain golden hash are
/// untouched. The projection is opt-in to the D64 throughput path.
///
/// **Provenance**: the projection is a deterministic function of
/// `TraceEvent[]`. An auditor can re-pack the events via
/// `GpuTraceEventCompact::from_trace_event` and verify the recorded
/// `compact_event_projection_hash` (the SHA-256 over the packed
/// byte stream) matches. The hash is surfaced through the dispatch
/// diagnostic so a verifier can confirm the compact bytes weren't
/// silently substituted between catalog ingest and the kernel.
///
/// **Byte layout (16 bytes, 8-byte aligned, repr(C))**:
///   offset  field                  type
///        0  `ts_ns`                u64
///        8  `entity_and_error`     u32   (low 31 bits = entity_id,
///                                         high bit = error_code != 0)
///       12  `latency_us`           u32
///
/// `entity_id` is bounded to 31 bits (max 2^31 − 1). All fixtures
/// the v0 plan supports cap `n_entities` at < 2^15.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct GpuTraceEventCompact {
    /// Wall-clock timestamp, nanoseconds since the synthetic epoch.
    pub ts_ns: u64,
    /// Bit-packed `(entity_id, error_flag)`. Low 31 bits carry
    /// `entity_id`; bit 31 is `1` iff the original event's
    /// `error_code` was non-zero.
    pub entity_and_error: u32,
    /// Observed latency in microseconds. Carried as the full
    /// `u32` from `TraceEvent` so any clamp behaviour upstream
    /// is preserved byte-for-byte.
    pub latency_us: u32,
}

impl GpuTraceEventCompact {
    /// Fixed byte width of one compact event. Mirrored on the GPU
    /// side as `cuda/layout.cuh::GpuTraceEventCompact`.
    pub const SIZE: usize = 16;
    /// Bit-31 flag in `entity_and_error` indicating
    /// `TraceEvent::error_code != 0`.
    pub const ERROR_BIT: u32 = 1u32 << 31;
    /// Bit-mask for the `entity_id` field within
    /// `entity_and_error`.
    pub const ENTITY_MASK: u32 = 0x7FFF_FFFF;

    /// Pack a single `TraceEvent` into the compact projection. The
    /// resulting bytes are a deterministic function of the input;
    /// two events that compare equal under
    /// `PartialEq<TraceEvent>` produce identical compact bytes.
    #[must_use]
    pub const fn from_trace_event(ev: &TraceEvent) -> Self {
        let entity_bits = ev.entity_id & Self::ENTITY_MASK;
        let error_bit = if ev.error_code != 0 {
            Self::ERROR_BIT
        } else {
            0
        };
        Self {
            ts_ns: ev.ts_ns,
            entity_and_error: entity_bits | error_bit,
            latency_us: ev.latency_us,
        }
    }

    /// Recover the entity id stored in the low 31 bits of
    /// `entity_and_error`.
    #[must_use]
    pub const fn entity_id(&self) -> u32 {
        self.entity_and_error & Self::ENTITY_MASK
    }

    /// `true` iff the original `TraceEvent::error_code` was
    /// non-zero. Stored as bit 31 of `entity_and_error` so the
    /// compact projection holds in 16 bytes.
    #[must_use]
    pub const fn error_nonzero(&self) -> bool {
        (self.entity_and_error & Self::ERROR_BIT) != 0
    }
}

/// R.11c — deterministically pack `events` into the compact GPU
/// projection. Throughput-dispatch helper; the audit path uses the
/// full 48-byte `TraceEvent` slice and is unchanged.
#[cfg(feature = "std")]
#[must_use]
pub fn pack_compact_event_projection(events: &[TraceEvent]) -> std::vec::Vec<GpuTraceEventCompact> {
    events
        .iter()
        .map(GpuTraceEventCompact::from_trace_event)
        .collect()
}

/// R.11c — SHA-256 over a compact-event slice's canonical byte
/// form. Used as the throughput-path provenance anchor: a verifier
/// re-packs `TraceEvent[]` via [`pack_compact_event_projection`]
/// and checks the recorded hash matches.
///
/// The canonical byte form is little-endian per field
/// (`ts_ns` u64 → 8 LE bytes, `entity_and_error` u32 → 4 LE bytes,
/// `latency_us` u32 → 4 LE bytes), concatenated in cell order.
/// Serialised explicitly via `to_le_bytes` so the hash is
/// reproducible without relying on the host's in-memory layout
/// and stays compatible with `forbid(unsafe_code)`.
#[cfg(feature = "std")]
#[must_use]
pub fn compact_event_projection_hash(compact: &[GpuTraceEventCompact]) -> [u8; 32] {
    let mut buf: std::vec::Vec<u8> =
        std::vec::Vec::with_capacity(compact.len() * GpuTraceEventCompact::SIZE);
    for ev in compact {
        buf.extend_from_slice(&ev.ts_ns.to_le_bytes());
        buf.extend_from_slice(&ev.entity_and_error.to_le_bytes());
        buf.extend_from_slice(&ev.latency_us.to_le_bytes());
    }
    crate::hash::sha256(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_index_partitions_at_boundaries() {
        let ev = TraceEvent {
            ts_ns: 1_500_000_000,
            ..TraceEvent::default()
        };
        // 1.5 seconds at a 1-second window → index 1.
        assert_eq!(ev.window_index(1_000_000_000), 1);

        let edge = TraceEvent {
            ts_ns: 2_000_000_000,
            ..TraceEvent::default()
        };
        // Exact boundary lands in the next window per integer-division semantics.
        assert_eq!(edge.window_index(1_000_000_000), 2);
    }

    #[test]
    fn window_index_floors_at_zero() {
        let ev = TraceEvent::default();
        assert_eq!(ev.window_index(1_000_000_000), 0);
    }

    #[test]
    fn struct_size_is_stable() {
        // The size of the canonical record is part of the prior-art posture
        // because the CUDA layout in `cuda/layout.cuh` mirrors it. If this
        // assertion changes, the CUDA mirror needs to change with it. The
        // expected size is the sum of declared field widths plus trailing
        // alignment padding to a multiple of 8 (the largest alignment).
        //
        // Field sizes (bytes): 8+4+4+8+8+4+2+2+2+2 = 44, padded to 48.
        assert_eq!(core::mem::size_of::<TraceEvent>(), 48);
    }
}
