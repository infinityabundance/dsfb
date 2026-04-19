#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use dsfb_database::adapters::DatasetAdapter;
use dsfb_database::adapters::{
    ceb::Ceb, generic_csv, job::Job, snowset::Snowset, sqlshare::SqlShare,
    sqlshare_text::SqlShareText, tpcds::TpcDs,
};
use dsfb_database::grammar::{replay, MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::metrics::{cross_signal_agreement, evaluate, stability_under_perturbation};
use dsfb_database::non_claims;
use dsfb_database::perturbation::{tpcds_with_perturbations, tpcds_with_perturbations_scaled};
use dsfb_database::report::{
    plots, write_episodes_csv, write_json, write_metrics_csv, write_provenance,
};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "dsfb-database",
    about = "DSFB-Database: deterministic, read-only structural observer for SQL telemetry residuals.",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand)]
enum Cmd {
    /// Print the non-claim charter.
    NonClaims,
    /// Run the controlled-perturbation pipeline (TPC-DS-shaped exemplar) end-to-end.
    Reproduce {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Run a single dataset exemplar (no perturbation harness).
    Exemplar {
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Load a real dataset from a CSV path and emit motif episodes.
    Run {
        #[arg(long)]
        dataset: String,
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Re-run the grammar twice and check the SHA256 fingerprint matches.
    ReplayCheck {
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    /// Sweep thresholds at ±20% and emit the elasticity table.
    Elasticity {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Sweep perturbation magnitudes and emit the per-motif degradation
    /// envelope. Reports F1 per (motif, scale) so the operating envelope
    /// of each motif is visible — not a uniform "F1=1.0" headline.
    StressSweep {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Ingest a per-engine telemetry export (CSV) and emit the residual +
    /// episode streams. The only `--engine` currently supported is
    /// `postgres` (`pg_stat_statements`); see `src/adapters/postgres.rs`
    /// for the expected CSV schema.
    Ingest {
        #[arg(long)]
        engine: String,
        #[arg(long)]
        csv: PathBuf,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Run every bundled reproducible artefact end-to-end and bundle the
    /// result into `out/dsfb_database_artifacts.zip`. Composes the existing
    /// `reproduce`, `exemplar`, and stress / funnel / comparison / refusal
    /// paths into a single offline invocation. Byte-stable across runs.
    ReproduceAll {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Apply the motif grammar to a residual stream constructed from an
    /// operator-supplied CSV. See `src/adapters/generic_csv.rs` for the
    /// contract; the grammar is loaded from the optional `--grammar`
    /// JSON (default: the crate's pinned grammar). This is a worked
    /// example, not a universality claim — the operator is responsible
    /// for confirming the grammar is appropriate for the input signal.
    Generic {
        #[arg(long)]
        csv: PathBuf,
        #[arg(long)]
        grammar: Option<PathBuf>,
        #[arg(long)]
        time_col: Option<String>,
        #[arg(long)]
        value_col: Option<String>,
        #[arg(long)]
        channel_col: Option<String>,
        #[arg(long, default_value_t = false)]
        pre_residualized: bool,
        #[arg(long, default_value = "out")]
        out: PathBuf,
    },
    /// Live read-only PostgreSQL telemetry adapter
    /// (feature = `live-postgres`). Polls `pg_stat_statements`,
    /// `pg_stat_activity`, `pg_stat_io`, `pg_stat_database` at a
    /// configurable cadence, writes a SHA-256-finalised *tape*, and
    /// emits motif episodes incrementally. Determinism holds only
    /// given the tape (§10 non-claim #7).
    #[cfg(feature = "live-postgres")]
    Live {
        /// libpq-style connection string
        /// (e.g. `"host=/tmp user=dsfb_observer"`). Required unless
        /// `--print-permissions-manifest` is set.
        #[arg(long)]
        conn: Option<String>,
        /// Nominal polling interval in milliseconds.
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,
        /// Rolling-CPU-ratio ceiling (0.0–1.0). When exceeded, the
        /// next inter-poll sleep doubles.
        #[arg(long, default_value_t = 0.1)]
        cpu_budget_pct: f64,
        /// Upper bound on per-poll wall-clock duration in
        /// milliseconds; exceeding this doubles the inter-poll sleep.
        #[arg(long, default_value_t = 500)]
        max_poll_ms: u64,
        /// Maximum number of seconds the live loop runs before a
        /// graceful shutdown. Absent = run until SIGINT.
        #[arg(long)]
        max_duration_sec: Option<u64>,
        /// Tape file path. Absent = tape disabled (residuals are
        /// distilled and episodes emitted but not persisted; the
        /// determinism guarantee requires a tape).
        #[arg(long)]
        tape: Option<PathBuf>,
        /// In-memory retention window for the rescan buffer.
        #[arg(long, default_value_t = 3600.0)]
        retention_window_sec: f64,
        /// Output directory for episodes CSV and poll telemetry.
        #[arg(long, default_value = "out/live")]
        out: PathBuf,
        /// Dump the permission manifest (`spec/permissions.postgres.sql`)
        /// to stdout and exit; no connection attempted.
        #[arg(long, default_value_t = false)]
        print_permissions_manifest: bool,
        /// Optional motif-grammar override (YAML).
        #[arg(long)]
        grammar: Option<PathBuf>,
    },
    /// Replay a persisted tape (JSONL + `.hash` manifest) through the
    /// batch motif engine and emit a deterministic episodes CSV.
    /// Usable without the live-postgres feature.
    #[cfg(feature = "live-postgres")]
    ReplayTape {
        #[arg(long)]
        tape: PathBuf,
        #[arg(long, default_value = "out/replay")]
        out: PathBuf,
    },
}

fn adapter_for(name: &str) -> Result<Box<dyn DatasetAdapter>> {
    Ok(match name {
        "snowset" => Box::new(Snowset),
        "sqlshare" => Box::new(SqlShare),
        "sqlshare-text" => Box::new(SqlShareText),
        "ceb" => Box::new(Ceb),
        "job" => Box::new(Job),
        "tpcds" => Box::new(TpcDs),
        other => anyhow::bail!("unknown dataset {other}"),
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();
    match cli.cmd {
        Cmd::NonClaims => Ok(()),
        Cmd::Reproduce { seed, out } => reproduce(seed, out),
        Cmd::Exemplar { dataset, seed, out } => exemplar(&dataset, seed, out),
        Cmd::Run { dataset, path, out } => run_real(&dataset, path, out),
        Cmd::ReplayCheck { seed } => replay_check(seed),
        Cmd::Elasticity { seed, out } => elasticity(seed, out),
        Cmd::StressSweep { seed, out } => stress_sweep(seed, out),
        Cmd::Ingest { engine, csv, out } => run_ingest(&engine, csv, out),
        Cmd::ReproduceAll { seed, out } => reproduce_all(seed, out),
        Cmd::Generic {
            csv,
            grammar,
            time_col,
            value_col,
            channel_col,
            pre_residualized,
            out,
        } => run_generic(
            csv,
            grammar,
            time_col,
            value_col,
            channel_col,
            pre_residualized,
            out,
        ),
        #[cfg(feature = "live-postgres")]
        Cmd::Live {
            conn,
            interval_ms,
            cpu_budget_pct,
            max_poll_ms,
            max_duration_sec,
            tape,
            retention_window_sec,
            out,
            print_permissions_manifest,
            grammar,
        } => run_live(
            conn,
            interval_ms,
            cpu_budget_pct,
            max_poll_ms,
            max_duration_sec,
            tape,
            retention_window_sec,
            out,
            print_permissions_manifest,
            grammar,
        ),
        #[cfg(feature = "live-postgres")]
        Cmd::ReplayTape { tape, out } => run_replay_tape(tape, out),
    }
}

fn run_ingest(engine: &str, csv: PathBuf, out: PathBuf) -> Result<()> {
    let stream = match engine {
        "postgres" => dsfb_database::adapters::postgres::load_pg_stat_statements(&csv)?,
        other => anyhow::bail!("unknown --engine {other}; supported engines: postgres"),
    };
    let grammar = MotifGrammar::default();
    let engine_run = MotifEngine::new(grammar);
    let episodes = engine_run.run(&stream);
    fs::create_dir_all(&out)?;
    write_provenance(
        &out.join(format!("{engine}.provenance.txt")),
        &stream.source,
    )?;
    write_episodes_csv(&out.join(format!("{engine}.episodes.csv")), &episodes)?;
    eprintln!(
        "ingest({engine}): {} episodes from {}",
        episodes.len(),
        stream.source
    );
    eprintln!("stream_fingerprint = {}", hex(&stream.fingerprint()));
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );
    eprintln!("(no metrics written: real-engine ingest does not have ground-truth windows.)");
    Ok(())
}

fn samples_per_motif(
    stream: &dsfb_database::residual::ResidualStream,
) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        let count = stream.iter_class(m.residual_class()).count();
        h.insert(m, count);
    }
    h
}

fn reproduce(seed: u64, out: PathBuf) -> Result<()> {
    let (stream, windows) = tpcds_with_perturbations(seed);
    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar.clone()).run(&stream);
    let samples_h = samples_per_motif(&stream);
    let metrics = evaluate(&episodes, &windows, &samples_h, stream.duration());
    debug_assert_eq!(
        metrics.len(),
        MotifClass::ALL.len(),
        "one metrics row per motif"
    );

    write_reproduce_csvs(&stream, &episodes, &windows, &grammar, &metrics, &out)?;
    log_reproduce_fingerprints(&stream, &episodes);
    emit_per_motif_plots(&stream, &episodes, &grammar, &out)?;
    emit_summary_and_funnel(&stream, &episodes, &grammar, &out)?;

    run_stress_sweep(seed, &out)?;
    write_drift_slew_anatomy(&stream, &grammar, &episodes, &out)?;
    write_throughput_report(seed, &out)?;
    Ok(())
}

/// Writes provenance + all machine-readable CSV/JSON artefacts for the
/// canonical reproducible run. Separated from the plotting phase so
/// that downstream tooling can consume the CSVs without requiring the
/// PNG pipeline to succeed.
fn write_reproduce_csvs(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
    windows: &[dsfb_database::perturbation::PerturbationWindow],
    grammar: &MotifGrammar,
    metrics: &[dsfb_database::metrics::PerMotifMetrics],
    out: &Path,
) -> Result<()> {
    debug_assert!(
        out.exists() || out.parent().map(Path::exists).unwrap_or(true),
        "out path resolvable"
    );
    debug_assert!(
        metrics.len() == MotifClass::ALL.len(),
        "metrics row count invariant"
    );
    write_provenance(&out.join("provenance.txt"), &stream.source)?;
    write_episodes_csv(&out.join("tpcds.episodes.csv"), episodes)?;
    write_metrics_csv(&out.join("tpcds.metrics.csv"), metrics)?;
    write_json(&out.join("tpcds.windows.json"), windows)?;
    write_json(&out.join("tpcds.grammar.json"), grammar)?;
    Ok(())
}

fn log_reproduce_fingerprints(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
) {
    eprintln!("stream_fingerprint = {}", hex(&stream.fingerprint()));
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(episodes)
    );
}

fn emit_per_motif_plots(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
    grammar: &MotifGrammar,
    out: &Path,
) -> Result<()> {
    for m in MotifClass::ALL {
        emit_single_motif_plots(stream, episodes, grammar, m, out)?;
    }
    Ok(())
}

fn emit_single_motif_plots(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
    grammar: &MotifGrammar,
    m: MotifClass,
    out: &Path,
) -> Result<()> {
    let title = format!("TPC-DS perturbed: {} residuals + episodes", m.name());
    let path = out.join(format!("tpcds.{}.png", m.name()));
    let p = grammar.params(m);
    debug_assert!(
        p.slew_threshold >= p.drift_threshold,
        "slew >= drift by construction"
    );
    plots::plot_residual_overlay(
        &path,
        &title,
        stream,
        m.residual_class(),
        episodes,
        m,
        p.slew_threshold,
        p.drift_threshold,
    )?;
    if stream.iter_class(m.residual_class()).next().is_some() {
        let ch_title = format!(
            "TPC-DS perturbed: per-channel residual strips ({})",
            m.name()
        );
        let _emitted: bool = plots::plot_channel_small_multiples(
            &out.join(format!("tpcds.{}.channels.png", m.name())),
            &ch_title,
            stream,
            m.residual_class(),
            episodes,
            m,
            8,
        )?;
    }
    if episodes.iter().any(|e| e.motif == m) {
        let emitted = plots::plot_episode_distribution(
            &out.join(format!("tpcds.{}.distribution.png", m.name())),
            &format!(
                "TPC-DS perturbed: episode peak + duration distribution ({})",
                m.name()
            ),
            episodes,
            m,
        )?;
        if !emitted {
            plots::plot_episode_table(
                &out.join(format!("tpcds.{}.table.png", m.name())),
                &format!("TPC-DS perturbed: episode listing ({})", m.name()),
                episodes,
                m,
            )?;
        }
    }
    Ok(())
}

fn emit_summary_and_funnel(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
    grammar: &MotifGrammar,
    out: &Path,
) -> Result<()> {
    plots::plot_episode_summary_table(
        &out.join("tpcds.summary_table.png"),
        "TPC-DS perturbed: motif × channel episode count",
        episodes,
        6,
    )?;
    let funnel_rows = compute_funnel_rows(stream, episodes, grammar);
    debug_assert_eq!(
        funnel_rows.len(),
        MotifClass::ALL.len(),
        "one funnel row per motif"
    );
    plots::plot_pipeline_funnel(
        &out.join("tpcds.funnel.png"),
        "TPC-DS perturbed: noise-reduction funnel per motif (log scale)",
        &funnel_rows,
    )?;
    write_funnel_csv(&out.join("tpcds.funnel.csv"), &funnel_rows)?;
    Ok(())
}

fn compute_funnel_rows(
    stream: &dsfb_database::residual::ResidualStream,
    episodes: &[dsfb_database::grammar::Episode],
    grammar: &MotifGrammar,
) -> Vec<(String, u64, u64, u64)> {
    MotifClass::ALL
        .iter()
        .map(|m| {
            let raw = stream.iter_class(m.residual_class()).count() as u64;
            let slew = grammar.params(*m).slew_threshold;
            let naive = stream
                .iter_class(m.residual_class())
                .filter(|s| s.value.abs() >= slew)
                .count() as u64;
            let eps = episodes.iter().filter(|e| e.motif == *m).count() as u64;
            debug_assert!(naive <= raw, "naive cannot exceed raw samples");
            (m.name().to_string(), raw, naive, eps)
        })
        .collect()
}

fn write_funnel_csv(path: &Path, funnel_rows: &[(String, u64, u64, u64)]) -> Result<()> {
    let mut funnel_csv = csv::Writer::from_path(path)?;
    funnel_csv.write_record([
        "motif",
        "raw_samples",
        "naive_above_slew",
        "dsfb_episodes",
        "noise_reduction_factor",
    ])?;
    for (name, raw, naive, eps) in funnel_rows.iter() {
        let nrf = if *eps == 0 {
            0.0
        } else {
            *naive as f64 / *eps as f64
        };
        debug_assert!(nrf.is_finite(), "noise-reduction factor finite");
        funnel_csv.write_record([
            name,
            &raw.to_string(),
            &naive.to_string(),
            &eps.to_string(),
            &format!("{:.2}", nrf),
        ])?;
    }
    funnel_csv.flush()?;
    Ok(())
}

fn write_drift_slew_anatomy(
    stream: &dsfb_database::residual::ResidualStream,
    grammar: &MotifGrammar,
    episodes: &[dsfb_database::grammar::Episode],
    out: &Path,
) -> Result<()> {
    let p = grammar.params(MotifClass::CacheCollapse);
    // Reconstruct EMA inline using the same recurrence
    // s_k = ρ · s_{k-1} + (1 − ρ) · |r_k| with the same ρ the motif
    // engine uses. Channel "tpcds" is the only cache_collapse channel
    // in the controlled harness.
    let raw: Vec<(f64, f64)> = stream
        .iter_class(dsfb_database::residual::ResidualClass::CacheIo)
        .filter(|s| s.channel.as_deref() == Some("tpcds"))
        .map(|s| (s.t, s.value))
        .collect();
    if raw.is_empty() {
        return Ok(());
    }
    let mut ema_state = 0.0_f64;
    let ema: Vec<(f64, f64)> = raw
        .iter()
        .map(|(t, v)| {
            ema_state = p.rho * ema_state + (1.0 - p.rho) * v.abs();
            (*t, ema_state)
        })
        .collect();
    let episode = episodes
        .iter()
        .find(|e| e.motif == MotifClass::CacheCollapse)
        .map(|e| (e.t_start, e.t_end));

    plots::plot_drift_slew_anatomy(
        &out.join("tpcds.anatomy.png"),
        "Drift vs. slew anatomy: cache_collapse on channel `tpcds`",
        &raw,
        &ema,
        p.slew_threshold,
        p.drift_threshold,
        episode,
    )?;
    plots::plot_phase_portrait(
        &out.join("tpcds.phase_portrait.png"),
        "Drift–slew phase portrait: cache_collapse on channel `tpcds`",
        &raw,
        &ema,
        p.slew_threshold,
        p.drift_threshold,
    )?;
    Ok(())
}

fn write_throughput_report(seed: u64, out: &Path) -> Result<()> {
    use std::time::Instant;
    // Single-thread, release build, this hardware. We report µs per
    // residual sample on the *full motif pipeline* (residual stream
    // construction → all five state machines → episode emission). The
    // number is what a vendor licensing the sidecar would observe on
    // similar hardware processing a residual stream of the same shape.
    // We do NOT extrapolate to TPS — residual rate is not transaction
    // rate; see §9 limitation #21.
    let mut grammar_runs = Vec::new();
    let mut stream_runs = Vec::new();
    let mut samples_total = 0usize;
    for _ in 0..5 {
        let t0 = Instant::now();
        let (s, _) = tpcds_with_perturbations(seed);
        let stream_dt = t0.elapsed();
        samples_total = s.samples.len();
        let grammar = MotifGrammar::default();
        let t1 = Instant::now();
        let _eps = MotifEngine::new(grammar).run(&s);
        let grammar_dt = t1.elapsed();
        stream_runs.push(stream_dt.as_secs_f64());
        grammar_runs.push(grammar_dt.as_secs_f64());
    }
    fn median(mut v: Vec<f64>) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v[v.len() / 2]
    }
    let stream_med = median(stream_runs);
    let grammar_med = median(grammar_runs);
    let total_med = stream_med + grammar_med;
    let us_per_sample = total_med * 1e6 / samples_total as f64;
    let samples_per_sec = samples_total as f64 / total_med;

    let mut f = File::create(out.join("tpcds.throughput.txt"))?;
    writeln!(f, "DSFB-Database throughput report")?;
    writeln!(f, "================================")?;
    writeln!(f, "harness        = tpcds_with_perturbations(seed={seed})")?;
    writeln!(f, "samples_total  = {}", samples_total)?;
    writeln!(f, "stream_build   = {:.4} s (median of 5 runs)", stream_med)?;
    writeln!(
        f,
        "grammar_eval   = {:.4} s (median of 5 runs)",
        grammar_med
    )?;
    writeln!(f, "total_pipeline = {:.4} s", total_med)?;
    writeln!(f, "us_per_sample  = {:.3} µs", us_per_sample)?;
    writeln!(f, "samples_per_s  = {:.0}", samples_per_sec)?;
    writeln!(f)?;
    writeln!(
        f,
        "Reported as a single-thread, release-build measurement on the"
    )?;
    writeln!(
        f,
        "host that ran reproduce_paper.sh. We do NOT extrapolate this to"
    )?;
    writeln!(
        f,
        "queries-per-second or transactions-per-second — residual sample"
    )?;
    writeln!(
        f,
        "rate is set by the engine's telemetry polling cadence, not by"
    )?;
    writeln!(
        f,
        "the workload's TPS. See §9 limitation #21 for the corresponding"
    )?;
    writeln!(f, "non-claim.")?;
    eprintln!(
        "throughput: {:.3} µs/sample, {:.0} samples/s on this host (single thread)",
        us_per_sample, samples_per_sec
    );
    Ok(())
}

const STRESS_SCALES: &[f64] = &[0.05, 0.10, 0.20, 0.35, 0.50, 0.75, 1.00, 1.50, 2.00];

fn run_stress_sweep(seed: u64, out: &Path) -> Result<()> {
    let grammar = MotifGrammar::default();
    let mut series: Vec<(String, Vec<f64>)> = MotifClass::ALL
        .iter()
        .map(|m| {
            (
                m.name().to_string(),
                Vec::with_capacity(STRESS_SCALES.len()),
            )
        })
        .collect();
    // Per-(motif, scale) median time-to-detection. NaN means no
    // detection at this scale, which is structurally different from
    // "detected immediately" (TTD = 0) and we keep them separate.
    let mut ttd_series: Vec<(String, Vec<f64>)> = MotifClass::ALL
        .iter()
        .map(|m| {
            (
                m.name().to_string(),
                Vec::with_capacity(STRESS_SCALES.len()),
            )
        })
        .collect();

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(out)?;
    let csv_path = out.join("tpcds.stress.csv");
    let mut csv = csv::Writer::from_path(&csv_path)?;
    let mut header = vec!["scale".to_string()];
    for m in MotifClass::ALL {
        header.push(format!("{}_f1", m.name()));
    }
    csv.write_record(&header)?;

    for &scale in STRESS_SCALES {
        let (stream, windows) = tpcds_with_perturbations_scaled(seed, scale);
        let engine = MotifEngine::new(grammar.clone());
        let episodes = engine.run(&stream);
        let samples_h = samples_per_motif(&stream);
        let metrics = evaluate(&episodes, &windows, &samples_h, stream.duration());

        let mut row = vec![format!("{:.3}", scale)];
        for (i, motif) in MotifClass::ALL.iter().enumerate() {
            let m_metric = metrics.iter().find(|m| m.motif == motif.name());
            let f1 = m_metric.map(|m| m.f1).unwrap_or(0.0);
            // TTD is only defined when there was at least one TP;
            // otherwise we record NaN so the chart can show a gap
            // rather than a misleading zero.
            let ttd = match m_metric {
                Some(m) if m.tp > 0 => m.time_to_detection_median_s,
                // Metric recorded but no true positives — TTD undefined.
                Some(_) => f64::NAN,
                // Motif produced no metric row at this scale — TTD undefined.
                None => f64::NAN,
            };
            series[i].1.push(f1);
            ttd_series[i].1.push(ttd);
            row.push(format!("{:.4}", f1));
        }
        csv.write_record(&row)?;
    }
    csv.flush()?;

    // Companion TTD CSV — one row per scale, NaN where motif failed
    // to detect at that scale.
    let ttd_csv_path = out.join("tpcds.ttd.csv");
    let mut ttd_csv = csv::Writer::from_path(&ttd_csv_path)?;
    let mut ttd_header = vec!["scale".to_string()];
    for m in MotifClass::ALL {
        ttd_header.push(format!("{}_ttd_median_s", m.name()));
    }
    ttd_csv.write_record(&ttd_header)?;
    for (si, &scale) in STRESS_SCALES.iter().enumerate() {
        let mut row = vec![format!("{:.3}", scale)];
        for s in &ttd_series {
            let v = s.1[si];
            row.push(if v.is_nan() {
                "NaN".to_string()
            } else {
                format!("{:.2}", v)
            });
        }
        ttd_csv.write_record(&row)?;
    }
    ttd_csv.flush()?;

    plots::plot_stress_curves(
        &out.join("tpcds.stress.png"),
        "TPC-DS perturbed: per-motif F1 vs. perturbation magnitude",
        STRESS_SCALES,
        &series,
    )?;

    // Companion text summary so the reviewer can read the envelope
    // without opening the PNG.
    let mut summary = File::create(out.join("tpcds.stress.txt"))?;
    writeln!(
        summary,
        "Per-motif F1 across perturbation-magnitude scales (seed={seed})"
    )?;
    writeln!(summary, "scale=1.0 reproduces the published baseline.")?;
    writeln!(summary)?;
    write!(summary, "{:>8}", "scale")?;
    for m in MotifClass::ALL {
        write!(summary, "  {:>32}", m.name())?;
    }
    writeln!(summary)?;
    for (si, &scale) in STRESS_SCALES.iter().enumerate() {
        write!(summary, "{:>8.3}", scale)?;
        for s in &series {
            write!(summary, "  {:>32.4}", s.1[si])?;
        }
        writeln!(summary)?;
    }
    Ok(())
}

fn stress_sweep(seed: u64, out: PathBuf) -> Result<()> {
    run_stress_sweep(seed, &out)?;
    eprintln!(
        "stress sweep written to {} (tpcds.stress.csv, tpcds.stress.png, tpcds.stress.txt)",
        out.display()
    );
    Ok(())
}

fn exemplar(dataset: &str, seed: u64, out: PathBuf) -> Result<()> {
    let adapter = adapter_for(dataset)?;
    let stream = adapter.exemplar(seed);
    let grammar = MotifGrammar::default();
    let engine = MotifEngine::new(grammar.clone());
    let episodes = engine.run(&stream);
    write_provenance(
        &out.join(format!("{dataset}.provenance.txt")),
        &stream.source,
    )?;
    write_episodes_csv(
        &out.join(format!("{dataset}.exemplar.episodes.csv")),
        &episodes,
    )?;
    eprintln!(
        "{} exemplar fingerprint = {}",
        dataset,
        replay::fingerprint_hex(&episodes)
    );
    for m in MotifClass::ALL {
        let count = stream.iter_class(m.residual_class()).count();
        if count == 0 {
            continue;
        }
        let title = format!("{} exemplar: {} residuals + episodes", dataset, m.name());
        let path = out.join(format!("{dataset}.exemplar.{}.png", m.name()));
        let p = grammar.params(m);
        plots::plot_residual_overlay(
            &path,
            &title,
            &stream,
            m.residual_class(),
            &episodes,
            m,
            p.slew_threshold,
            p.drift_threshold,
        )?;
    }
    Ok(())
}

fn run_real(dataset: &str, path: PathBuf, out: PathBuf) -> Result<()> {
    let adapter = adapter_for(dataset)?;
    let stream = adapter.load(&path)?;
    let grammar = MotifGrammar::default();
    let engine = MotifEngine::new(grammar.clone());
    let episodes = engine.run(&stream);
    fs::create_dir_all(&out)?;
    write_provenance(
        &out.join(format!("{dataset}.provenance.txt")),
        &stream.source,
    )?;
    write_episodes_csv(&out.join(format!("{dataset}.episodes.csv")), &episodes)?;

    // Emit per-motif PNG residual overlays + companion per-channel
    // small multiples + per-motif episode distribution figures. Captions
    // identify the source as real (no `[exemplar]` tag).
    for m in MotifClass::ALL {
        let count = stream.iter_class(m.residual_class()).count();
        if count == 0 {
            continue;
        }
        let p = grammar.params(m);
        let title = format!("{dataset} (real shard): {} residuals + episodes", m.name());
        let overlay_path = out.join(format!("{dataset}.{}.png", m.name()));
        plots::plot_residual_overlay(
            &overlay_path,
            &title,
            &stream,
            m.residual_class(),
            &episodes,
            m,
            p.slew_threshold,
            p.drift_threshold,
        )?;

        // Per-channel strip figure — emission decision is delegated to
        // the plotter: it returns false (no file written) when no
        // channel survives the >=3 samples / std>0 filters. A
        // single-channel overlay-complete stream (e.g. sqlshare-text)
        // naturally lands in that skip branch.
        let ch_title = format!(
            "{dataset} (real shard): per-channel residual strips ({})",
            m.name()
        );
        // Same pattern as the TPC-DS call site: the plotter decides
        // whether to emit a file; we inspect the result via a named
        // binding rather than `let _ =`.
        let _emitted: bool = plots::plot_channel_small_multiples(
            &out.join(format!("{dataset}.{}.channels.png", m.name())),
            &ch_title,
            &stream,
            m.residual_class(),
            &episodes,
            m,
            8,
        )?;

        // Distribution figure — only when there is at least one episode
        // for this motif. If the histogram would collapse to a single
        // bar (N<5 or zero peak/duration variance) the plotter returns
        // false and we emit a compact tabular listing instead.
        if episodes.iter().any(|e| e.motif == m) {
            let emitted = plots::plot_episode_distribution(
                &out.join(format!("{dataset}.{}.distribution.png", m.name())),
                &format!(
                    "{dataset} (real shard): episode peak + duration distribution ({})",
                    m.name()
                ),
                &episodes,
                m,
            )?;
            if !emitted {
                plots::plot_episode_table(
                    &out.join(format!("{dataset}.{}.table.png", m.name())),
                    &format!("{dataset} (real shard): episode listing ({})", m.name()),
                    &episodes,
                    m,
                )?;
            }
        }
    }
    if !episodes.is_empty() {
        plots::plot_episode_summary_table(
            &out.join(format!("{dataset}.summary_table.png")),
            &format!("{dataset} (real shard): motif × channel episode count"),
            &episodes,
            6,
        )?;
    }

    eprintln!(
        "real-data run: {} episodes from {}",
        episodes.len(),
        stream.source
    );
    eprintln!("stream_fingerprint = {}", hex(&stream.fingerprint()));
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );
    eprintln!("(no metrics written: real-data runs do not have ground-truth windows.)");
    Ok(())
}

fn replay_check(seed: u64) -> Result<()> {
    let (stream1, _) = tpcds_with_perturbations(seed);
    let (stream2, _) = tpcds_with_perturbations(seed);
    if stream1.fingerprint() != stream2.fingerprint() {
        anyhow::bail!("residual stream is not deterministic under fixed seed");
    }
    let g = MotifGrammar::default();
    let eps1 = MotifEngine::new(g.clone()).run(&stream1);
    let eps2 = MotifEngine::new(g).run(&stream2);
    if replay::fingerprint(&eps1) != replay::fingerprint(&eps2) {
        anyhow::bail!("episode stream is not bytewise deterministic");
    }
    eprintln!("OK: stream + episode fingerprints match across two runs.");
    Ok(())
}

fn elasticity(seed: u64, out: PathBuf) -> Result<()> {
    let (stream, windows) = tpcds_with_perturbations(seed);
    let baseline = MotifGrammar::default();
    let baseline_eps = MotifEngine::new(baseline.clone()).run(&stream);
    let samples_h = samples_per_motif(&stream);
    let baseline_metrics = evaluate(&baseline_eps, &windows, &samples_h, stream.duration());

    let scaled_up = scale_grammar(&baseline, 1.20);
    let scaled_down = scale_grammar(&baseline, 0.80);
    let m_up = evaluate(
        &MotifEngine::new(scaled_up).run(&stream),
        &windows,
        &samples_h,
        stream.duration(),
    );
    let m_down = evaluate(
        &MotifEngine::new(scaled_down).run(&stream),
        &windows,
        &samples_h,
        stream.duration(),
    );

    write_metrics_csv(&out.join("elasticity_baseline.csv"), &baseline_metrics)?;
    write_metrics_csv(&out.join("elasticity_plus20.csv"), &m_up)?;
    write_metrics_csv(&out.join("elasticity_minus20.csv"), &m_down)?;
    eprintln!(
        "elasticity csvs written to {} (compare F1 columns by motif).",
        out.display()
    );
    Ok(())
}

fn scale_grammar(g: &MotifGrammar, factor: f64) -> MotifGrammar {
    let mut g = g.clone();
    for p in [
        &mut g.plan_regression_onset,
        &mut g.cardinality_mismatch_regime,
        &mut g.contention_ramp,
        &mut g.cache_collapse,
        &mut g.workload_phase_transition,
    ] {
        p.drift_threshold *= factor;
        p.slew_threshold *= factor;
    }
    g
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Load an operator-supplied grammar JSON. The grammar JSON schema is
/// exactly what `reproduce` emits to `out/tpcds.grammar.json`, so a
/// user's easiest path is to run `reproduce` once, copy the file, and
/// tweak the numbers.
fn load_grammar(path: Option<PathBuf>) -> Result<MotifGrammar> {
    use anyhow::Context;
    match path {
        None => Ok(MotifGrammar::default()),
        Some(p) => {
            let s = std::fs::read_to_string(&p)
                .with_context(|| format!("reading grammar JSON at {}", p.display()))?;
            let g: MotifGrammar = serde_json::from_str(&s)
                .with_context(|| format!("parsing grammar JSON at {}", p.display()))?;
            Ok(g)
        }
    }
}

fn run_generic(
    csv: PathBuf,
    grammar_path: Option<PathBuf>,
    time_col: Option<String>,
    value_col: Option<String>,
    channel_col: Option<String>,
    pre_residualized: bool,
    out: PathBuf,
) -> Result<()> {
    let opts = generic_csv::GenericCsvOptions {
        time_col,
        value_col,
        channel_col,
        pre_residualized,
    };
    let stream = generic_csv::load_generic_csv(&csv, &opts)?;
    let grammar = load_grammar(grammar_path)?;
    let engine = MotifEngine::new(grammar.clone());
    let episodes = engine.run(&stream);
    fs::create_dir_all(&out)?;
    write_provenance(&out.join("generic.provenance.txt"), &stream.source)?;
    write_episodes_csv(&out.join("generic.episodes.csv"), &episodes)?;

    for m in MotifClass::ALL {
        let count = stream.iter_class(m.residual_class()).count();
        if count == 0 {
            continue;
        }
        let p = grammar.params(m);
        let title = format!("generic CSV: {} residuals + episodes", m.name());
        plots::plot_residual_overlay(
            &out.join(format!("generic.{}.png", m.name())),
            &title,
            &stream,
            m.residual_class(),
            &episodes,
            m,
            p.slew_threshold,
            p.drift_threshold,
        )?;
    }

    let funnel_rows = compute_funnel_rows(&stream, &episodes, &grammar);
    plots::plot_pipeline_funnel(
        &out.join("generic.funnel.png"),
        "Generic CSV: noise-reduction funnel per motif (log scale)",
        &funnel_rows,
    )?;
    write_funnel_csv(&out.join("generic.funnel.csv"), &funnel_rows)?;

    eprintln!(
        "generic: {} episodes from {}",
        episodes.len(),
        stream.source
    );
    eprintln!("stream_fingerprint = {}", hex(&stream.fingerprint()));
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );
    Ok(())
}

/// Orchestrates every bundled, offline artefact the crate can produce
/// at a single seed, writes a MANIFEST.md, and packs everything into a
/// deterministic zip. See `tests/reproduce_all_zip_is_deterministic.rs`
/// for the byte-stability guarantee.
fn reproduce_all(seed: u64, out: PathBuf) -> Result<()> {
    fs::create_dir_all(&out)?;

    // 1. Canonical TPC-DS controlled-perturbation pipeline.
    reproduce(seed, out.clone())?;

    // 2. Bundled exemplars for every non-TPC-DS dataset.
    for ds in &["snowset", "sqlshare-text", "ceb", "job"] {
        exemplar(ds, seed, out.clone())?;
    }

    // 3. Comparison + refusal figures.
    emit_comparison_figure(seed, &out)?;
    emit_refusal_figure(seed, &out)?;

    // 4. Extra metrics: cross-signal agreement + stability AUC.
    emit_cross_signal_agreement(seed, &out)?;
    emit_stability_auc(&out)?;

    // 5. MANIFEST.md
    write_manifest(&out)?;

    // 6. Deterministic zip.
    let zip_path = out.join("dsfb_database_artifacts.zip");
    build_deterministic_zip(&out, &zip_path)?;
    eprintln!("artifact bundle: {}", zip_path.display());
    Ok(())
}

/// Emits `out/comparison.png` and `out/comparison.csv` contrasting
/// PELT / BOCPD change-points with a DSFB episode on the TPC-DS
/// perturbed `plan_regression` channel. Baselines are intentionally
/// scored charitably (each point wrapped as an episode of duration
/// `min_dwell_seconds`) — the purpose is *structural contrast*, not
/// detection-quality ranking.
fn emit_comparison_figure(seed: u64, out: &Path) -> Result<()> {
    use dsfb_database::baselines::{bocpd::Bocpd, pelt::Pelt, ChangePointDetector};
    let (stream, _windows) = tpcds_with_perturbations(seed);
    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar.clone()).run(&stream);

    let channel = "q42";
    let samples_on_channel: Vec<(f64, f64)> = stream
        .iter_class(dsfb_database::residual::ResidualClass::PlanRegression)
        .filter(|s| s.channel.as_deref() == Some(channel))
        .map(|s| (s.t, s.value))
        .collect();
    if samples_on_channel.is_empty() {
        return Ok(());
    }
    let pelt_events: Vec<f64> = Pelt::default().detect(&samples_on_channel);
    let bocpd_events: Vec<f64> = Bocpd::default().detect(&samples_on_channel);

    let t_lo = 150.0;
    let t_hi = 330.0;
    plots::plot_detector_contrast(
        &out.join("comparison.png"),
        "TPC-DS q42: PELT / BOCPD points vs DSFB episode (structural contrast)",
        &stream,
        dsfb_database::residual::ResidualClass::PlanRegression,
        Some(channel),
        &episodes,
        &pelt_events,
        &bocpd_events,
        t_lo,
        t_hi,
    )?;

    let mut w = csv::Writer::from_path(out.join("comparison.csv"))?;
    w.write_record(["method", "event_type", "t_start", "t_end"])?;
    for ep in episodes
        .iter()
        .filter(|e| e.channel.as_deref() == Some(channel))
        .filter(|e| e.motif == MotifClass::PlanRegressionOnset)
    {
        w.write_record([
            "dsfb",
            "episode",
            &format!("{:.3}", ep.t_start),
            &format!("{:.3}", ep.t_end),
        ])?;
    }
    for t in pelt_events.iter() {
        w.write_record([
            "pelt",
            "change_point",
            &format!("{:.3}", t),
            &format!("{:.3}", t),
        ])?;
    }
    for t in bocpd_events.iter() {
        w.write_record([
            "bocpd",
            "change_point",
            &format!("{:.3}", t),
            &format!("{:.3}", t),
        ])?;
    }
    w.flush()?;
    Ok(())
}

/// Emits `out/refusal.png` and `out/refusal.csv` on a pure-noise
/// Gaussian null trace. DSFB is expected to emit zero episodes; the
/// baselines fire at a non-zero rate under the charitable point-wrap
/// scoring. The figure anchors the "Cases Where Interpretation Is Not
/// Justified" paper section.
fn emit_refusal_figure(seed: u64, out: &Path) -> Result<()> {
    use dsfb_database::baselines::{bocpd::Bocpd, pelt::Pelt, ChangePointDetector};
    use dsfb_database::residual::{ResidualClass, ResidualSample, ResidualStream};
    use rand::{Rng, SeedableRng};

    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    let n = 3600_usize;
    let mut stream = ResidualStream::new(format!("null-trace@seed{seed}"));
    let mut null_trace: Vec<(f64, f64)> = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64;
        let u1: f64 = rng.gen_range(1e-12..1.0);
        let u2: f64 = rng.gen_range(0.0..1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let v = z * 0.05;
        null_trace.push((t, v));
        stream.push(
            ResidualSample::new(t, ResidualClass::PlanRegression, v).with_channel("null"),
        );
    }
    stream.sort();

    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar).run(&stream);

    let pelt_times: Vec<f64> = Pelt::default().detect(&null_trace);
    let bocpd_times: Vec<f64> = Bocpd::default().detect(&null_trace);

    let mut baseline_events: Vec<(f64, &'static str)> = pelt_times
        .iter()
        .map(|t| (*t, "pelt"))
        .chain(bocpd_times.iter().map(|t| (*t, "bocpd")))
        .collect();
    baseline_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    plots::plot_refusal_contrast(
        &out.join("refusal.png"),
        "Null-trace refusal: baselines false-alarm; DSFB refuses",
        &null_trace,
        &episodes,
        &baseline_events,
    )?;

    let dur_hours = null_trace.last().map(|p| p.0).unwrap_or(1.0) / 3600.0;
    let mut w = csv::Writer::from_path(out.join("refusal.csv"))?;
    w.write_record(["method", "false_alarms_per_hour", "episodes"])?;
    w.write_record([
        "pelt",
        &format!("{:.3}", pelt_times.len() as f64 / dur_hours.max(1e-9)),
        &pelt_times.len().to_string(),
    ])?;
    w.write_record([
        "bocpd",
        &format!("{:.3}", bocpd_times.len() as f64 / dur_hours.max(1e-9)),
        &bocpd_times.len().to_string(),
    ])?;
    w.write_record([
        "dsfb",
        &format!("{:.3}", episodes.len() as f64 / dur_hours.max(1e-9)),
        &episodes.len().to_string(),
    ])?;
    w.flush()?;
    Ok(())
}

fn emit_cross_signal_agreement(seed: u64, out: &Path) -> Result<()> {
    let (stream, _windows) = tpcds_with_perturbations(seed);
    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar).run(&stream);
    let rows = cross_signal_agreement(&episodes);
    let mut w = csv::Writer::from_path(out.join("tpcds.cross_signal.csv"))?;
    w.write_record(["motif", "cross_signal_agreement_mean"])?;
    for (m, v) in rows {
        w.write_record([m.name().to_string(), format!("{:.4}", v)])?;
    }
    w.flush()?;
    Ok(())
}

fn emit_stability_auc(out: &Path) -> Result<()> {
    let stress_path = out.join("tpcds.stress.csv");
    if !stress_path.exists() {
        return Ok(());
    }
    let mut rdr = csv::Reader::from_path(&stress_path)?;
    let headers = rdr.headers()?.clone();
    let motif_cols: Vec<(usize, String)> = headers
        .iter()
        .enumerate()
        .filter_map(|(i, h)| h.strip_suffix("_f1").map(|m| (i, m.to_string())))
        .collect();
    let mut rows: Vec<(f64, String, f64)> = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        let scale: f64 = rec.get(0).unwrap_or("nan").parse().unwrap_or(f64::NAN);
        for (col_idx, motif) in &motif_cols {
            let f1: f64 = rec.get(*col_idx).unwrap_or("nan").parse().unwrap_or(f64::NAN);
            rows.push((scale, motif.clone(), f1));
        }
    }
    let aucs = stability_under_perturbation(&rows);
    let mut w = csv::Writer::from_path(out.join("tpcds.stability.csv"))?;
    w.write_record(["motif", "auc_0p5_1p5"])?;
    let mut motifs: Vec<String> = aucs.keys().cloned().collect();
    motifs.sort();
    for m in motifs {
        w.write_record([m.clone(), format!("{:.4}", aucs[&m])])?;
    }
    w.flush()?;
    Ok(())
}

fn write_manifest(out: &Path) -> Result<()> {
    let mut f = File::create(out.join("MANIFEST.md"))?;
    writeln!(f, "# DSFB-Database Artefact Manifest")?;
    writeln!(f)?;
    writeln!(
        f,
        "Every file below is produced by `dsfb-database reproduce-all --seed 42 --out out`"
    )?;
    writeln!(f, "at crate version `{}`.", dsfb_database::CRATE_VERSION)?;
    writeln!(f)?;
    writeln!(f, "## Phase A — TPC-DS controlled perturbation")?;
    writeln!(f, "- `tpcds.episodes.csv` — episode stream")?;
    writeln!(f, "- `tpcds.metrics.csv` — per-motif precision/recall/F1/TTD")?;
    writeln!(f, "- `tpcds.windows.json` — planted perturbation windows")?;
    writeln!(f, "- `tpcds.grammar.json` — pinned grammar parameters")?;
    writeln!(f, "- `tpcds.*.png` — per-motif residual overlays")?;
    writeln!(f, "- `tpcds.summary_table.png` — motif × channel episode count")?;
    writeln!(f, "- `tpcds.funnel.png` + `tpcds.funnel.csv` — noise-reduction funnel")?;
    writeln!(f, "- `tpcds.anatomy.png` + `tpcds.phase_portrait.png` — drift/slew anatomy with Lyapunov overlay")?;
    writeln!(f, "- `tpcds.stress.csv` + `.png` + `.txt` — per-motif F1 vs. perturbation scale")?;
    writeln!(f, "- `tpcds.ttd.csv` — per-(motif, scale) median time-to-detection")?;
    writeln!(f, "- `tpcds.throughput.txt` — single-thread µs/sample")?;
    writeln!(f, "- `tpcds.cross_signal.csv` — per-motif cross-signal agreement mean")?;
    writeln!(f, "- `tpcds.stability.csv` — per-motif F1-AUC over scales [0.5, 1.5]")?;
    writeln!(f)?;
    writeln!(f, "## Phase B — bundled dataset exemplars")?;
    writeln!(f, "Deterministic synthetic exemplars shaped like the real corpora.")?;
    writeln!(f, "- `snowset.exemplar.episodes.csv` + `snowset.exemplar.*.png`")?;
    writeln!(
        f,
        "- `sqlshare-text.exemplar.episodes.csv` + `sqlshare-text.exemplar.*.png`"
    )?;
    writeln!(f, "- `ceb.exemplar.episodes.csv` + `ceb.exemplar.*.png`")?;
    writeln!(f, "- `job.exemplar.episodes.csv` + `job.exemplar.*.png`")?;
    writeln!(f)?;
    writeln!(f, "## Phase C — contrast and refusal")?;
    writeln!(
        f,
        "- `comparison.png` + `comparison.csv` — DSFB episode vs PELT/BOCPD points on TPC-DS q42"
    )?;
    writeln!(
        f,
        "- `refusal.png` + `refusal.csv` — null-trace DSFB refuses while baselines false-alarm"
    )?;
    writeln!(f)?;
    writeln!(f, "## Provenance")?;
    writeln!(f, "- `provenance.txt` — crate version, source labels, non-claim charter")?;
    writeln!(f, "- `dsfb_database_artifacts.zip` — byte-stable bundle of everything above")?;
    Ok(())
}

fn build_deterministic_zip(out: &Path, zip_path: &Path) -> Result<()> {
    use zip::{write::FileOptions, CompressionMethod, ZipWriter};
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_files(out, out, &mut paths)?;
    let zip_rel = zip_path.strip_prefix(out).unwrap_or(zip_path).to_path_buf();
    // Exclude the zip itself and any file whose contents are
    // wall-clock-dependent by construction (throughput numbers).
    paths.retain(|p| {
        let s = p.to_string_lossy();
        p != &zip_rel && !s.ends_with("throughput.txt")
    });
    paths.sort();

    let file = File::create(zip_path)?;
    let mut zw = ZipWriter::new(file);
    let pinned_dt = zip::DateTime::from_date_and_time(2026, 1, 1, 0, 0, 0)
        .unwrap_or_else(|_| zip::DateTime::default());
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644)
        .last_modified_time(pinned_dt);

    for rel in &paths {
        let abs = out.join(rel);
        if !abs.is_file() {
            continue;
        }
        let name = rel.to_string_lossy().replace('\\', "/");
        zw.start_file(name, options)?;
        let mut buf = Vec::new();
        File::open(&abs)?.read_to_end(&mut buf)?;
        zw.write_all(&buf)?;
    }
    zw.finish()?;
    Ok(())
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(root)?.to_path_buf();
            out.push(rel);
        }
    }
    Ok(())
}

#[cfg(feature = "live-postgres")]
const PERMISSIONS_MANIFEST: &str = include_str!("../spec/permissions.postgres.sql");

#[cfg(feature = "live-postgres")]
#[allow(clippy::too_many_arguments)]
fn run_live(
    conn: Option<String>,
    interval_ms: u64,
    cpu_budget_pct: f64,
    max_poll_ms: u64,
    max_duration_sec: Option<u64>,
    tape: Option<PathBuf>,
    retention_window_sec: f64,
    out: PathBuf,
    print_permissions_manifest: bool,
    grammar_path: Option<PathBuf>,
) -> Result<()> {
    if print_permissions_manifest {
        print!("{}", PERMISSIONS_MANIFEST);
        return Ok(());
    }
    let conn = conn.ok_or_else(|| {
        anyhow::anyhow!(
            "--conn is required unless --print-permissions-manifest is set"
        )
    })?;
    fs::create_dir_all(&out)?;
    let grammar = match grammar_path.as_ref() {
        Some(p) => {
            let y = fs::read_to_string(p)?;
            MotifGrammar::from_yaml(&y)?
        }
        None => MotifGrammar::default(),
    };
    let budget = dsfb_database::live::Budget {
        max_poll_ms,
        cpu_pct: cpu_budget_pct,
    };
    let interval = std::time::Duration::from_millis(interval_ms);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        live_loop_async(
            &conn,
            interval,
            budget,
            max_duration_sec,
            tape,
            retention_window_sec,
            out,
            grammar,
        )
        .await
    })
}

#[cfg(feature = "live-postgres")]
#[allow(clippy::too_many_arguments)]
async fn live_loop_async(
    conn_str: &str,
    interval: std::time::Duration,
    budget: dsfb_database::live::Budget,
    max_duration_sec: Option<u64>,
    tape_path: Option<PathBuf>,
    retention_window_sec: f64,
    out: PathBuf,
    grammar: MotifGrammar,
) -> Result<()> {
    use dsfb_database::live::{
        distiller::DistillerState, tape::Tape, LiveEmitter, ReadOnlyPgConn, Scraper,
    };
    let conn = ReadOnlyPgConn::connect(conn_str).await?;
    let mut scraper = Scraper::new(conn, interval, budget);
    let mut distiller = DistillerState::new();
    let mut emitter = LiveEmitter::new(grammar, retention_window_sec, 4_000_000);
    let mut tape: Option<Tape> = match &tape_path {
        Some(p) => Some(Tape::create(p, format!("live-postgres:{}", conn_str))?),
        None => None,
    };
    let mut poll_log = fs::File::create(out.join("poll_log.csv"))?;
    writeln!(
        poll_log,
        "t_wall,snapshot_duration_ms,cpu_pct_rolling,throttle_factor,buffer_samples"
    )?;
    let episodes_path = out.join("live.episodes.csv");
    let mut episodes_file = fs::File::create(&episodes_path)?;
    writeln!(
        episodes_file,
        "motif,channel,t_start,t_end,peak,ema_at_boundary,trust_sum"
    )?;

    let start = std::time::Instant::now();
    let deadline = max_duration_sec.map(|s| start + std::time::Duration::from_secs(s));
    let mut last_tick = std::time::Instant::now();
    let mut total_episodes: usize = 0;

    loop {
        let tick_start = std::time::Instant::now();
        let (snapshot, wall) = tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nSIGINT received — flushing and shutting down cleanly.");
                break;
            }
            r = scraper.next_snapshot() => r?,
        };
        let self_time = tick_start.elapsed();
        let interval_since_last = last_tick.elapsed();
        last_tick = std::time::Instant::now();
        let report = scraper.record_and_plan(wall, self_time, interval_since_last);
        let samples = distiller.ingest(&snapshot);
        if let Some(t) = tape.as_mut() {
            t.append(&samples)?;
        }
        let episodes = emitter.push_samples(samples);
        for ep in episodes.iter() {
            writeln!(
                episodes_file,
                "{},{},{},{},{},{},{}",
                ep.motif.name(),
                ep.channel.as_deref().unwrap_or(""),
                ep.t_start,
                ep.t_end,
                ep.peak,
                ep.ema_at_boundary,
                ep.trust_sum,
            )?;
        }
        total_episodes += episodes.len();
        writeln!(
            poll_log,
            "{},{},{:.6},{:.3},{}",
            report.t_wall_start,
            report.snapshot_duration_ms,
            report.cpu_pct_rolling,
            report.throttle_factor,
            emitter.buffer_len(),
        )?;
        poll_log.flush()?;
        episodes_file.flush()?;
        if let Some(d) = deadline {
            if std::time::Instant::now() >= d {
                eprintln!("max-duration reached; flushing and shutting down cleanly.");
                break;
            }
        }
        tokio::time::sleep(scraper.next_sleep()).await;
    }
    if let Some(t) = tape {
        let manifest = t.finalize()?;
        eprintln!(
            "tape sha256 = {} ({} samples)",
            manifest.sha256, manifest.sample_count
        );
        eprintln!(
            "replay with: dsfb-database replay-tape --tape {} --out {}",
            tape_path.as_ref().unwrap().display(),
            out.display()
        );
    }
    eprintln!(
        "live loop shutdown: {} episodes emitted, emitter_total={}",
        total_episodes,
        emitter.emitted_count(),
    );
    Ok(())
}

#[cfg(feature = "live-postgres")]
fn run_replay_tape(tape: PathBuf, out: PathBuf) -> Result<()> {
    use dsfb_database::grammar::replay::fingerprint_hex;
    use dsfb_database::live::tape::load_and_verify;
    fs::create_dir_all(&out)?;
    let (stream, manifest) = load_and_verify(&tape)?;
    eprintln!(
        "tape verified: sha256={} samples={}",
        manifest.sha256, manifest.sample_count
    );
    let engine = MotifEngine::new(MotifGrammar::default());
    let episodes = engine.run(&stream);
    write_episodes_csv(&out.join("replay.episodes.csv"), &episodes)?;
    eprintln!(
        "replay: {} episodes; episodes_fingerprint = {}",
        episodes.len(),
        fingerprint_hex(&episodes)
    );
    Ok(())
}
