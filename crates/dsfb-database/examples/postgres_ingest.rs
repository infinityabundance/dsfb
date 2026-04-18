//! End-to-end PostgreSQL `pg_stat_statements` ingest demo.
//!
//! What this example does:
//!   1. Loads `examples/data/pg_stat_statements_sample.csv` — a 50-row,
//!      synthetic-but-realistic snapshot file that mirrors the schema a
//!      real `\copy` from `pg_stat_statements` would produce. Five
//!      `query_id`s (md5-hashed; no real query text) sampled across ten
//!      60-second snapshots, with one of the five (`e2fc714c…`) showing
//!      a 10× mean-exec-time regression after snapshot 6.
//!   2. Runs the [`postgres`] adapter to convert it into a typed
//!      [`ResidualStream`] (plan_regression + workload_phase channels).
//!   3. Runs the motif grammar (default thresholds from §7) and prints
//!      the episode stream.
//!
//! What this example does NOT do:
//!   * Connect to a live database. The DSFB-Database crate is a
//!     residual-stream observer; collection is the operator's
//!     responsibility (see the `\copy` recipe in `src/adapters/postgres.rs`).
//!   * Emit cardinality, contention, or cache_io residuals — those
//!     require additional PostgreSQL views (`EXPLAIN ANALYZE`,
//!     `pg_stat_activity`, `pg_stat_io`) and per-view adapters that
//!     are not yet shipped. The §11 deployability matrix records the gap.
//!
//! Expected output (for the bundled sample):
//!   * stream fingerprint: 64-hex pinned by the example assertion below,
//!     so a future change to the adapter or sample CSV is caught
//!     immediately;
//!   * episode count ≥ 1 (the q3 regression should produce one
//!     `plan_regression_onset` episode).

use anyhow::Result;
use dsfb_database::adapters::postgres::load_pg_stat_statements;
use dsfb_database::grammar::{replay, MotifClass, MotifEngine, MotifGrammar};
use std::path::Path;

fn main() -> Result<()> {
    let csv_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/data/pg_stat_statements_sample.csv");
    let stream = load_pg_stat_statements(&csv_path)?;
    println!("loaded residual stream: {}", stream.source);
    println!("samples: {}", stream.samples.len());
    // `.take(SHA256_HEX_BYTES)` is an explicit finite-source bound: the
    // fingerprint is exactly 32 bytes (SHA-256 output). The bound also
    // satisfies dsfb-gray's ITER-UNB audit without silently truncating.
    const SHA256_HEX_BYTES: usize = 32;
    let stream_fp = stream
        .fingerprint()
        .iter()
        .take(SHA256_HEX_BYTES)
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    println!("stream fingerprint = {}", stream_fp);

    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar).run(&stream);
    println!("episodes: {}", episodes.len());
    let eps_fp = replay::fingerprint_hex(&episodes);
    println!("episode fingerprint = {}", eps_fp);

    for ep in &episodes {
        println!(
            "  motif={:?} channel={:?} t=[{:.1}, {:.1}] peak={:.3}",
            ep.motif, ep.channel, ep.t_start, ep.t_end, ep.peak
        );
    }

    let regressions = episodes
        .iter()
        .filter(|e| e.motif == MotifClass::PlanRegressionOnset)
        .count();
    assert!(
        regressions >= 1,
        "expected at least one plan_regression_onset episode from the q3 regression in the bundled sample CSV; got {regressions}"
    );

    // Pinned fingerprints for the bundled sample. If you change the
    // adapter, the sample CSV, or the residual serialisation, both
    // values change and you must (a) re-derive them, (b) update the
    // README quickstart if the episode count moves.
    const EXPECTED_STREAM_FP: &str =
        "ca3630150f102c2d7f9cfcc7db58e9ed23d81dbc501c8bf5896336728e6853a2";
    const EXPECTED_EPISODE_FP: &str =
        "36f75f6e495dff8ef551164e605deb24df365f03f1f6f5f8cb7ea8b6102637ce";
    assert_eq!(
        stream_fp, EXPECTED_STREAM_FP,
        "stream fingerprint drift: adapter or sample CSV changed"
    );
    assert_eq!(
        eps_fp, EXPECTED_EPISODE_FP,
        "episode fingerprint drift: motif state machine or adapter changed"
    );

    println!(
        "OK: {} plan_regression_onset episode(s) detected.",
        regressions
    );
    println!("OK: stream + episode fingerprints match pinned values.");
    Ok(())
}
