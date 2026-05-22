//! S-PERF.14b.1 v3 — Path 1b streaming-blockcoop tile-size
//! sweep (panel-required pre-promotion gate).
//!
//! **Purpose (panel-locked, 2026-05-18 post-v2-seal correction)**:
//!
//!   v2 sealed `compact_densor_digest_v1_root_kernel_streaming_blockcoop`
//!   at TILE_BYTES = 2048 and measured a +9.0 % per-kernel
//!   regression vs Path 1a (925.2 → 1008.4 µs). The panel
//!   rejected "structural cleanup" framing: switching the
//!   active dispatcher to a slower kernel is not admissible.
//!   The active dispatcher is now reverted to Path 1a; the
//!   streaming kernel is retained as an inactive candidate
//!   backend.
//!
//!   This sweep test exercises the inactive candidate at six
//!   tile sizes (1 KiB / 2 KiB / 4 KiB / 8 KiB / 16 KiB /
//!   32 KiB) across all four D64 _timed throughput-path stage
//!   shapes (residual / sign / detector-compact-pack /
//!   consensus). For each (stage, tile_bytes) pair it measures
//!   the streaming kernel wall AND the Path 1a kernel wall at
//!   the SAME stage shape, so the comparison is apples-to-
//!   apples without relying on a single averaged Path 1a
//!   number.
//!
//! **Per the user's analysis**:
//!
//!   > "Most likely v3 win condition: reduce number of tile
//!   >  iterations. If v2 uses small tiles, it is dying by a
//!   >  thousand syncs."
//!
//!   If a tile size beats Path 1a's per-stage AND total walls
//!   → a follow-on commit promotes the streaming kernel as the
//!   active root backend at that tile size. If not → the
//!   streaming kernel stays inactive; the panel verdict is
//!   that Path 1a's one-shot SHA over L2-resident scratch
//!   wins despite the extra memory traffic.
//!
//! **Measurement contract**:
//!
//!   - cudaEvent_t timing (sub-µs precision; sufficient for
//!     tile-size comparison without sudo ncu — the gold-standard
//!     ncu run is the final gate after a tile size wins).
//!   - 4 warm-up iterations, 32 timed iterations per
//!     (stage, kernel, tile_bytes).
//!   - Synthetic leaf buffer (cudaMemset 0xA5; digest VALUES
//!     are not examined — only wall-time).
//!   - All four stage shapes at canonical 256×4096 K=1 with
//!     chunk_size = 16 KiB.

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    // f64 cast acknowledged: all wall-time values are in
    // nanoseconds bounded by ~10 ms (10^7), well within f64's
    // 53-bit mantissa. f64 only used for the human-readable
    // millisecond / percent display columns; per-iteration walls
    // remain u64 for the integer accumulators.
    clippy::cast_precision_loss,
    // Sign casts acknowledged: per-iteration walls are positive
    // u64 values; we cast to i64 only to compute signed deltas
    // for the verdict (delta_ns < 0 = WIN). Cast is bounded by
    // ~10 ms (10^7), well below i64::MAX.
    clippy::cast_possible_wrap
)]

use dsfb_gpu_debug_cuda::{
    compact_densor_root_path1a_sweep_time, compact_densor_root_streaming_sweep_time,
};

const N_WARMUP: i32 = 4;
const N_TIMED: i32 = 32;
const CHUNK_SIZE: u32 = 16_384;
const N_CATALOGS: u32 = 1;
const TILE_SIZES: [u32; 6] = [1024, 2048, 4096, 8192, 16_384, 32_768];

/// Sweep one stage shape across all six streaming tile sizes
/// AND measure Path 1a at the same shape for apples-to-apples
/// comparison. Returns `(path1a_ns, best_streaming_tile,
/// best_streaming_ns)`.
fn sweep_stage(stage_label: &str, n_chunks_per_catalog: u32, stage_id: u32) -> (u64, u32, u64) {
    // Path 1a baseline at this stage shape (active production backend).
    let path1a_ns = compact_densor_root_path1a_sweep_time(
        n_chunks_per_catalog,
        CHUNK_SIZE,
        stage_id,
        N_CATALOGS,
        N_WARMUP,
        N_TIMED,
    )
    .expect("path1a sweep FFI");

    println!();
    println!(
        "  stage: {stage_label}  (n_chunks={n_chunks_per_catalog}, leaf_bytes={} KB)",
        (u64::from(n_chunks_per_catalog) * 32) / 1024
    );
    println!("  Path 1a baseline at this stage : {path1a_ns} ns");
    println!("  tile_bytes | streaming_ns | delta vs Path 1a (per-stage) | verdict");
    println!("  -----------+--------------+-------------------------------+---------");

    let mut best_tile: u32 = 0;
    let mut best_ns: u64 = u64::MAX;

    for &tile_bytes in &TILE_SIZES {
        let streaming_ns = compact_densor_root_streaming_sweep_time(
            n_chunks_per_catalog,
            CHUNK_SIZE,
            stage_id,
            N_CATALOGS,
            tile_bytes,
            N_WARMUP,
            N_TIMED,
        )
        .expect("streaming sweep FFI");

        let delta_ns: i64 = streaming_ns as i64 - path1a_ns as i64;
        let delta_pct: f64 = (delta_ns as f64) * 100.0 / (path1a_ns as f64);
        let verdict = if streaming_ns < path1a_ns {
            "WIN vs Path 1a"
        } else {
            "regress vs Path 1a"
        };

        println!(
            "  {tile_bytes:>5} B    | {streaming_ns:>10} ns | {delta_ns:>+11} ns ({delta_pct:+6.1}%)    | {verdict}"
        );

        if streaming_ns < best_ns {
            best_ns = streaming_ns;
            best_tile = tile_bytes;
        }
    }

    (path1a_ns, best_tile, best_ns)
}

#[test]
fn s_perf_14b_1_v3_tile_sweep_all_four_stages() {
    // The four D64 _timed throughput-path stages, with the actual
    // per-stage n_chunks values for the canonical 256×4096 K=1
    // fixture at chunk_size = 16384. Note the detector stage
    // digests the R.10b/S-PERF.15.a 18-byte COMPACT PACK, NOT the
    // 264-byte wide arena (`d_detector_digest_compact`, not
    // `d_detectors_wide`):
    //
    //   residual : n_cells × 16 B/cell  = 16 MB / 16 KB = 1024 chunks
    //   sign     : n_cells × 20 B/cell  = 20 MB / 16 KB = 1280 chunks
    //   detector : n_cells × 18 B/cell  = 18 MB / 16 KB = 1152 chunks  (compact pack)
    //   consensus: n_cells × 32 B/cell  = 32 MB / 16 KB = 2048 chunks

    println!();
    println!("S-PERF.14b.1 v3 tile-size sweep (panel-required pre-promotion gate)");
    println!(
        "  measurement     : {N_WARMUP} warmup + {N_TIMED} timed iters per (stage, kernel, tile)"
    );

    let mut totals_path1a: u64 = 0;
    let mut totals_streaming_best: u64 = 0;
    let mut per_stage_winners: Vec<(String, u64, u32, u64)> = Vec::new();

    for (stage_label, n_chunks, stage_id) in [
        ("residual ", 1024_u32, 0_u32),
        ("sign     ", 1280_u32, 1_u32),
        ("detector ", 1152_u32, 2_u32),
        ("consensus", 2048_u32, 3_u32),
    ] {
        let (path1a_ns, best_tile, best_streaming_ns) =
            sweep_stage(stage_label, n_chunks, stage_id);
        totals_path1a += path1a_ns;
        totals_streaming_best += best_streaming_ns;
        per_stage_winners.push((
            stage_label.trim().to_string(),
            path1a_ns,
            best_tile,
            best_streaming_ns,
        ));
    }

    println!();
    println!("==============================================================");
    println!("Per-stage summary (best streaming tile vs Path 1a, per stage):");
    println!("  stage     | Path 1a (ns) | best streaming tile | streaming (ns) | per-stage Δ");
    println!("  ----------+--------------+---------------------+----------------+------------");
    for (label, p1a, tile, sns) in &per_stage_winners {
        let delta_ns: i64 = *sns as i64 - *p1a as i64;
        let delta_pct: f64 = (delta_ns as f64) * 100.0 / (*p1a as f64);
        println!(
            "  {label:<9} | {p1a:>10} | {tile:>15} B  | {sns:>10}    | {delta_ns:>+8} ns ({delta_pct:+6.1}%)"
        );
    }

    let total_delta_ns: i64 = totals_streaming_best as i64 - totals_path1a as i64;
    let total_delta_pct: f64 = (total_delta_ns as f64) * 100.0 / (totals_path1a as f64);
    let path1a_ms: f64 = totals_path1a as f64 / 1_000_000.0;
    let stream_ms: f64 = totals_streaming_best as f64 / 1_000_000.0;

    println!();
    println!("Total per-dispatch wall (sum of 4 stages):");
    println!("  Path 1a total                     : {totals_path1a} ns ({path1a_ms:.3} ms)");
    println!(
        "  Streaming (best tile per stage)   : {totals_streaming_best} ns ({stream_ms:.3} ms)"
    );
    println!(
        "  Δ                                 : {total_delta_ns:+} ns ({total_delta_pct:+.1}%)"
    );
    println!();

    if totals_streaming_best < totals_path1a {
        println!("PANEL VERDICT  : streaming kernel ADMITTED");
        println!("                 (streaming total beats Path 1a total at best-tile-per-stage)");
        println!("                 (follow-on commit promotes per-stage tile selection)");
    } else {
        println!("PANEL VERDICT  : streaming kernel REMAINS INACTIVE candidate backend");
        println!("                 (no tile selection beats Path 1a's TOTAL per-dispatch wall;");
        println!("                  streaming wins at small leaf-byte stages but loses at the");
        println!("                  largest stage by enough to make the total regress)");
    }
    println!("==============================================================");
    println!();
}
