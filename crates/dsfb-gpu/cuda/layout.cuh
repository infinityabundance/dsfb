// cuda/layout.cuh
//
// repr(C) struct mirrors for every record that crosses the FFI boundary.
// Field order and width must match the Rust definitions verbatim — these
// types share their memory representation with the Rust core crate.
//
// Verified sizes (must match `core::mem::size_of` on the Rust side):
//
//   * TraceEvent       : 48 bytes (44 used + 4 trailing pad)
//   * WindowFeature    : 24 bytes
//   * ResidualCell     : 16 bytes
//   * SignCell         : 20 bytes
//   * DetectorCell     : 12 bytes
//   * ConsensusCell    : 32 bytes
//   * CandidateInterval: 64 bytes (8 u32 fields + 8 i32 Q16 fields,
//                                  laid out in canonical order)
//
// The Q16 type is a transparent wrapper around `int32_t` so it crosses the
// FFI as a plain 32-bit signed integer.

#pragma once

#include <cstdint>

namespace dsfb {

struct TraceEvent {
    uint64_t ts_ns;
    uint32_t entity_id;
    uint32_t route_id;
    uint64_t span_id;
    uint64_t parent_span_id;
    uint32_t latency_us;
    uint16_t status_code;
    uint16_t error_code;
    uint16_t event_kind;
    uint16_t flags;
};
static_assert(sizeof(TraceEvent) == 48, "TraceEvent layout mismatch");

// R.11c — compact GPU-ingest projection of TraceEvent. Mirrors
// `dsfb_gpu_debug_core::event::GpuTraceEventCompact` byte-for-byte
// (`#[repr(C)]` on the Rust side, plain `struct` here):
//   offset  field             type     notes
//        0  ts_ns             u64      cell timestamp
//        8  entity_and_error  u32      low 31 bits = entity_id;
//                                      bit 31 = TraceEvent::error_code != 0
//       12  latency_us        u32      full TraceEvent::latency_us
//
// Total 16 bytes, 8-byte aligned by construction. Replaces the
// 48-byte TraceEvent in the throughput H2D path; the audit path
// keeps the full record.
struct GpuTraceEventCompact {
    uint64_t ts_ns;
    uint32_t entity_and_error;
    uint32_t latency_us;
};
static_assert(sizeof(GpuTraceEventCompact) == 16,
              "GpuTraceEventCompact layout mismatch");

struct WindowFeature {
    uint32_t window_idx;
    uint32_t entity_id;
    uint32_t event_count;
    uint32_t error_count;
    uint64_t sum_latency_us;
};
static_assert(sizeof(WindowFeature) == 24, "WindowFeature layout mismatch");

struct ResidualCell {
    uint32_t window_idx;
    uint32_t entity_id;
    int32_t  residual_latency_q;
    int32_t  residual_error_q;
};
static_assert(sizeof(ResidualCell) == 16, "ResidualCell layout mismatch");

struct SignCell {
    uint32_t window_idx;
    uint32_t entity_id;
    int32_t  norm_q;
    int32_t  drift_q;
    int32_t  slew_q;
};
static_assert(sizeof(SignCell) == 20, "SignCell layout mismatch");

struct DetectorCell {
    uint32_t window_idx;
    uint32_t entity_id;
    uint32_t detector_mask;
};
static_assert(sizeof(DetectorCell) == 12, "DetectorCell layout mismatch");

// R.9.b — wide-mask detector cell used by every profile other than
// DetectorProfile::D16. Layout mirrors the Rust DetectorCellWide
// struct in dsfb-gpu-debug-core/src/detector.rs verbatim (repr(C)):
//   offset  field
//        0  window_idx: u32
//        4  entity_id:  u32
//        8  detector_mask: u64[32]   (256 bytes)
// Total 264 bytes, 8-byte aligned by construction.
struct DetectorCellWide {
    uint32_t window_idx;
    uint32_t entity_id;
    uint64_t detector_mask[32];
};
static_assert(sizeof(DetectorCellWide) == 264, "DetectorCellWide layout mismatch");

struct ConsensusCell {
    uint32_t window_idx;
    uint32_t entity_id;
    uint32_t detector_count;
    int32_t  axis1_residual_q;
    int32_t  axis2_drift_q;
    int32_t  axis3_slew_q;
    int32_t  axis4_temporal_q;
    int32_t  axis7_consensus_q;
};
static_assert(sizeof(ConsensusCell) == 32, "ConsensusCell layout mismatch");

struct CandidateInterval {
    uint32_t entity_id;
    uint32_t start_window;
    uint32_t end_window;
    uint32_t length_windows;
    uint32_t union_mask;
    int32_t  peak_residual_q;
    int32_t  peak_drift_q;
    int32_t  peak_slew_q;
    int32_t  peak_temporal_q;
    int32_t  peak_consensus_q;
    // R.5 axis-5 entity-locality and grid-locality averages computed
    // in `candidate_collapse_kernel`. Q16 raw i32, byte-identical to
    // the CPU `prepare_with_detectors` second pass. The bank stage
    // reads these directly instead of summing the consensus grid.
    int32_t  entity_avg_q;
    int32_t  grid_avg_q;
};
static_assert(sizeof(CandidateInterval) == 48, "CandidateInterval layout mismatch");

// Mirror of `DetectorThresholds` from detector.rs. Stored as an i32 array
// so the C ABI is unambiguous regardless of host compiler.
struct DetectorThresholds {
    int32_t spike_q16_raw;
    int32_t sustain_q16_raw;
    int32_t slew_shock_q16_raw;
    int32_t plateau_min_q16_raw;
    int32_t plateau_slew_max_q16_raw;
    uint32_t plateau_windows;
    uint32_t oscillation_window;
    uint32_t oscillation_alternations;
    int32_t deadband_low_q16_raw;
    int32_t deadband_high_q16_raw;
    int32_t error_burst_q16_raw;
    int32_t coupling_lat_q16_raw;
    int32_t coupling_err_q16_raw;
    uint32_t variance_window;
    int32_t variance_threshold_q16_raw;
    uint32_t ramp_window;
    int32_t recovery_min_norm_q16_raw;
    int32_t clean_band_q16_raw;
    int32_t confuser_min_q16_raw;
    int32_t fanout_drift_q16_raw;
    int32_t entity_anomaly_factor_q16_raw;
    uint32_t history_window;
};

// Bit-position constants for the 16 detector motifs. Must match
// `motif::MotifClass::bit_index` on the Rust side.
namespace motif {
    constexpr uint32_t RESIDUAL_SPIKE             = 1u << 0;
    constexpr uint32_t SUSTAINED                  = 1u << 1;
    constexpr uint32_t DRIFT_RAMP                 = 1u << 2;
    constexpr uint32_t SLEW_SHOCK                 = 1u << 3;
    constexpr uint32_t PLATEAU                    = 1u << 4;
    constexpr uint32_t OSCILLATION                = 1u << 5;
    constexpr uint32_t DEADBAND_EXIT              = 1u << 6;
    constexpr uint32_t ERROR_RATE_BURST           = 1u << 7;
    constexpr uint32_t LATENCY_ERROR_COUPLING     = 1u << 8;
    constexpr uint32_t ENTITY_LOCAL_ANOMALY       = 1u << 9;
    constexpr uint32_t ROUTE_LOCAL_ANOMALY        = 1u << 10;
    constexpr uint32_t FANOUT_PRECURSOR           = 1u << 11;
    constexpr uint32_t VARIANCE_EXPANSION         = 1u << 12;
    constexpr uint32_t RECOVERY_EDGE              = 1u << 13;
    constexpr uint32_t CLEAN_WINDOW_STABILITY     = 1u << 14;
    constexpr uint32_t CONFUSER_LIKE_TRANSIENT    = 1u << 15;
}

constexpr int MOTIF_COUNT = 16;

// Consensus-stage constants. Must match `consensus::*` on Rust side.
constexpr uint32_t TEMPORAL_WINDOW = 5;
constexpr uint32_t MAX_DETECTORS   = (uint32_t)MOTIF_COUNT;
constexpr uint32_t MAX_TEMPORAL    = MAX_DETECTORS * TEMPORAL_WINDOW;

}  // namespace dsfb
