//! Render the paper's three-panel pulsed-scrape figure from a
//! deterministically-synthesised fixture.
//!
//! The fixture is a 600-second, 1 Hz `pg_stat_statements`-shaped
//! snapshot trajectory for a single pinned `query_id = "q1"`:
//!
//!   - **t ∈ [0, 250)**  — baseline: 50 calls/s at 10 ms/call.
//!   - **t ∈ [250, 400)** — planted plan regression: still 50
//!     calls/s but 30 ms/call (normalised residual = 2.0, well above
//!     the default `slew_threshold = 0.50`).
//!   - **t ∈ [400, 600)** — recovery: back to 10 ms/call.
//!
//! A synthetic throttle-factor trace is overlaid on the bottom panel
//! to illustrate the measured-not-guaranteed backpressure signal: a
//! smooth bump to ~3× nominal sleep between `t ∈ [200, 260)` that
//! decays back to 1.0 outside the window. The bump is hand-authored,
//! not produced by the scraper — its purpose is to show what a
//! backpressure event *looks like* on the figure, not to claim that
//! the scraper will behave this way on every engine.
//!
//! The binary writes three artefacts:
//!
//!   1. `paper/fixtures/live_pg/pg_stat_statements.csv` — the
//!      synthesised trajectory in the exact schema the batch adapter
//!      reads (`snapshot_t, query_id, calls, total_exec_time_ms`).
//!   2. `paper/fixtures/live_pg/pg_stat_activity.csv` — a placeholder
//!      (only the header row), since the figure does not exercise the
//!      contention channel. Committed so the fixture directory is
//!      complete.
//!   3. `paper/figs/live_pulsed_scrape.png` — the three-panel figure.
//!
//! The distillation path runs the trajectory through
//! [`dsfb_database::live::DistillerState::ingest`] — the *same*
//! function the live binary calls on every poll — so the figure is
//! a byte-deterministic function of committed fixture data. The
//! emitted `plan_regression_onset` episode window (first such
//! episode, if any) is highlighted on the middle and bottom panels.

use anyhow::Result;
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::live::distiller::{DistillerState, PgssRow, Snapshot};
use dsfb_database::report::plots_live::plot_live_pulsed_scrape;
use dsfb_database::residual::ResidualStream;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const SNAPSHOTS: usize = 600;
const CALLS_PER_SEC: u64 = 50;
const BASELINE_MS_PER_CALL: f64 = 10.0;
const PERTURBED_MS_PER_CALL: f64 = 30.0;
const PERTURB_START: usize = 250;
const PERTURB_END: usize = 400;
const THROTTLE_START: f64 = 200.0;
const THROTTLE_END: f64 = 260.0;

fn ms_per_call_at(t_s: usize) -> f64 {
    if t_s >= PERTURB_START && t_s < PERTURB_END {
        PERTURBED_MS_PER_CALL
    } else {
        BASELINE_MS_PER_CALL
    }
}

fn throttle_factor_at(t_s: f64) -> f64 {
    if t_s >= THROTTLE_START && t_s < THROTTLE_END {
        let center = (THROTTLE_START + THROTTLE_END) / 2.0;
        let half_span = (THROTTLE_END - THROTTLE_START) / 2.0;
        let u = (t_s - center) / half_span;
        1.0 + 2.0 * (1.0 - u * u).max(0.0)
    } else {
        1.0
    }
}

fn main() -> Result<()> {
    let root: PathBuf = std::env::current_dir()?;
    let fixtures_dir = root.join("paper/fixtures/live_pg");
    let figs_dir = root.join("paper/figs");
    fs::create_dir_all(&fixtures_dir)?;
    fs::create_dir_all(&figs_dir)?;

    let pgss_csv = fixtures_dir.join("pg_stat_statements.csv");
    let activity_csv = fixtures_dir.join("pg_stat_activity.csv");
    let fig_path = figs_dir.join("live_pulsed_scrape.png");
    let readme = fixtures_dir.join("README.md");

    let mut snapshots_t: Vec<f64> = Vec::with_capacity(SNAPSHOTS);
    let mut total_exec_ms: Vec<f64> = Vec::with_capacity(SNAPSHOTS);
    let mut calls_cum: Vec<f64> = Vec::with_capacity(SNAPSHOTS);
    let mut residual_t: Vec<f64> = Vec::with_capacity(SNAPSHOTS);
    let mut residual_v: Vec<f64> = Vec::with_capacity(SNAPSHOTS);

    let mut csv_file = fs::File::create(&pgss_csv)?;
    writeln!(csv_file, "snapshot_t,query_id,calls,total_exec_time_ms")?;

    let mut cum_calls: u64 = 0;
    let mut cum_total_ms: f64 = 0.0;
    let mut distiller = DistillerState::new();
    let mut stream = ResidualStream::new("live_pg_fixture");

    let mut prev_calls: Option<u64> = None;
    let mut prev_total_ms: Option<f64> = None;

    for t in 0..SNAPSHOTS {
        let t_f = t as f64;
        let ms_per_call = ms_per_call_at(t);
        cum_calls += CALLS_PER_SEC;
        cum_total_ms += CALLS_PER_SEC as f64 * ms_per_call;

        snapshots_t.push(t_f);
        total_exec_ms.push(cum_total_ms);
        calls_cum.push(cum_calls as f64);

        writeln!(
            csv_file,
            "{},{},{},{}",
            t_f, "q1", cum_calls, cum_total_ms
        )?;

        if let (Some(pc), Some(pt)) = (prev_calls, prev_total_ms) {
            let dc = cum_calls - pc;
            let dt = cum_total_ms - pt;
            if dc > 0 {
                residual_t.push(t_f);
                residual_v.push(dt / dc as f64);
            }
        }
        prev_calls = Some(cum_calls);
        prev_total_ms = Some(cum_total_ms);

        let snap = Snapshot {
            t: t_f,
            pgss: vec![PgssRow {
                query_id: "q1".to_string(),
                calls: cum_calls,
                total_exec_time_ms: cum_total_ms,
            }],
            activity: Vec::new(),
            stat_io: Vec::new(),
            stat_database: Vec::new(),
        };
        let samples = distiller.ingest(&snap);
        for s in samples {
            stream.push(s);
        }
    }
    csv_file.flush()?;
    drop(csv_file);

    let mut activity_file = fs::File::create(&activity_csv)?;
    writeln!(
        activity_file,
        "snapshot_t,pid,wait_event_type,wait_event,state"
    )?;
    activity_file.flush()?;
    drop(activity_file);

    stream.sort();
    let engine = MotifEngine::new(MotifGrammar::default());
    let episodes = engine.run(&stream);
    let episode_window = episodes
        .iter()
        .find(|e| e.motif == MotifClass::PlanRegressionOnset)
        .map(|e| (e.t_start, e.t_end));

    let throttle_t: Vec<f64> = (0..=SNAPSHOTS).map(|i| i as f64).collect();
    let throttle_factor: Vec<f64> = throttle_t
        .iter()
        .copied()
        .map(throttle_factor_at)
        .collect();

    plot_live_pulsed_scrape(
        &fig_path,
        &snapshots_t,
        &total_exec_ms,
        &calls_cum,
        &residual_t,
        &residual_v,
        episode_window,
        &throttle_t,
        &throttle_factor,
    )?;

    let mut r = fs::File::create(&readme)?;
    writeln!(
        r,
        "# Live PG fixtures (deterministically synthesised)\n\n\
This directory contains a pinned, deterministic synthesis of the\n\
`pg_stat_statements` snapshot stream used by the paper's\n\
`live_pulsed_scrape` figure. It is **not** a capture from a real\n\
PostgreSQL engine — the generator is pure and seedless, so two\n\
invocations produce byte-identical files.\n\n\
## Shape\n\n\
- `pg_stat_statements.csv` — 600 snapshots @ 1 Hz, one `query_id` (`q1`):\n\
  - t ∈ [0, 250): baseline 50 calls/s at 10 ms/call.\n\
  - t ∈ [250, 400): planted plan regression — same call rate, 30 ms/call.\n\
  - t ∈ [400, 600): recovery to 10 ms/call.\n\
- `pg_stat_activity.csv` — header only. The figure does not exercise the contention channel.\n\n\
## Regeneration\n\n\
```\n\
cargo run --release --features \"cli report live-postgres\" --bin live_pulsed_scrape_figure\n\
```\n\n\
The generator writes the two CSVs here and renders\n\
`paper/figs/live_pulsed_scrape.png`. Paper §Live cites this fixture\n\
and the figure caption discloses its synthesised origin explicitly.\n",
    )?;
    r.flush()?;
    drop(r);

    if let Some(w) = episode_window {
        eprintln!(
            "wrote {} and {}; plan_regression_onset window = [{:.2}, {:.2}]",
            pgss_csv.display(),
            fig_path.display(),
            w.0,
            w.1
        );
    } else {
        eprintln!(
            "wrote {} and {}; no plan_regression_onset episode emitted (check thresholds)",
            pgss_csv.display(),
            fig_path.display()
        );
    }
    Ok(())
}
