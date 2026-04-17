use anyhow::Result;
use clap::Parser;
use dsfb_database::adapters::DatasetAdapter;
use dsfb_database::adapters::{ceb::Ceb, job::Job, snowset::Snowset, sqlshare::SqlShare, tpcds::TpcDs};
use dsfb_database::grammar::{replay, MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::metrics::evaluate;
use dsfb_database::non_claims;
use dsfb_database::perturbation::{tpcds_with_perturbations, tpcds_with_perturbations_scaled};
use dsfb_database::report::{
    plots, write_episodes_csv, write_json, write_metrics_csv, write_provenance,
};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
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
}

fn adapter_for(name: &str) -> Result<Box<dyn DatasetAdapter>> {
    Ok(match name {
        "snowset" => Box::new(Snowset),
        "sqlshare" => Box::new(SqlShare),
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
    }
}

fn run_ingest(engine: &str, csv: PathBuf, out: PathBuf) -> Result<()> {
    let stream = match engine {
        "postgres" => dsfb_database::adapters::postgres::load_pg_stat_statements(&csv)?,
        other => anyhow::bail!(
            "unknown --engine {other}; supported engines: postgres"
        ),
    };
    let grammar = MotifGrammar::default();
    let engine_run = MotifEngine::new(grammar);
    let episodes = engine_run.run(&stream);
    fs::create_dir_all(&out)?;
    write_provenance(&out.join(format!("{engine}.provenance.txt")), &stream.source)?;
    write_episodes_csv(&out.join(format!("{engine}.episodes.csv")), &episodes)?;
    eprintln!(
        "ingest({engine}): {} episodes from {}",
        episodes.len(),
        stream.source
    );
    eprintln!(
        "stream_fingerprint = {}",
        hex(&stream.fingerprint())
    );
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );
    eprintln!(
        "(no metrics written: real-engine ingest does not have ground-truth windows.)"
    );
    Ok(())
}

fn samples_per_motif(stream: &dsfb_database::residual::ResidualStream) -> HashMap<MotifClass, usize> {
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
    let engine = MotifEngine::new(grammar.clone());
    let episodes = engine.run(&stream);
    let samples_h = samples_per_motif(&stream);
    let metrics = evaluate(&episodes, &windows, &samples_h, stream.duration());

    write_provenance(&out.join("provenance.txt"), &stream.source)?;
    write_episodes_csv(&out.join("tpcds.episodes.csv"), &episodes)?;
    write_metrics_csv(&out.join("tpcds.metrics.csv"), &metrics)?;
    write_json(&out.join("tpcds.windows.json"), &windows)?;
    write_json(&out.join("tpcds.grammar.json"), &grammar)?;

    eprintln!(
        "stream_fingerprint = {}",
        hex(&stream.fingerprint())
    );
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );

    // Per-motif plots
    for m in MotifClass::ALL {
        let title = format!("TPC-DS perturbed: {} residuals + episodes", m.name());
        let path = out.join(format!("tpcds.{}.png", m.name()));
        plots::plot_residual_overlay(&path, &title, &stream, m.residual_class(), &episodes, m)?;
    }
    // Pipeline funnel: raw samples -> naive slew-threshold crossings ->
    // DSFB episodes. Replaces the compression-by-motif bar chart, which
    // was just sample-density in disguise. The funnel exposes the
    // *differential* contribution of the motif state machine on top of
    // a flat threshold — that's the SBIR-relevant claim.
    let funnel_rows: Vec<(String, u64, u64, u64)> = MotifClass::ALL
        .iter()
        .map(|m| {
            let raw = stream.iter_class(m.residual_class()).count() as u64;
            let slew = grammar.params(*m).slew_threshold;
            let naive = stream
                .iter_class(m.residual_class())
                .filter(|s| s.value.abs() >= slew)
                .count() as u64;
            let eps = episodes.iter().filter(|e| e.motif == *m).count() as u64;
            (m.name().to_string(), raw, naive, eps)
        })
        .collect();
    plots::plot_pipeline_funnel(
        &out.join("tpcds.funnel.png"),
        "TPC-DS perturbed: noise-reduction funnel per motif (log scale)",
        &funnel_rows,
    )?;
    {
        let mut funnel_csv = csv::Writer::from_path(out.join("tpcds.funnel.csv"))?;
        funnel_csv.write_record([
            "motif",
            "raw_samples",
            "naive_above_slew",
            "dsfb_episodes",
            "noise_reduction_factor",
        ])?;
        for (name, raw, naive, eps) in &funnel_rows {
            let nrf = if *eps == 0 {
                0.0
            } else {
                *naive as f64 / *eps as f64
            };
            funnel_csv.write_record([
                name,
                &raw.to_string(),
                &naive.to_string(),
                &eps.to_string(),
                &format!("{:.2}", nrf),
            ])?;
        }
        funnel_csv.flush()?;
    }

    // Stress sweep: where each motif breaks down. This replaces the
    // uninformative uniform-F1 bar chart — five equal columns at F1=1.0
    // told the reviewer nothing about the operating envelope. The sweep
    // reports F1 across a range of perturbation magnitudes so the
    // degradation curve per motif is visible.
    run_stress_sweep(seed, &out)?;

    // Drift-vs-slew anatomy figure on the cache_collapse channel.
    // Pedagogical: shows how EMA lag and the drift envelope differ
    // from instantaneous slew breaches. Uses the same DSFB params the
    // motif loop uses, so the figure literally renders what the state
    // machine sees.
    write_drift_slew_anatomy(&stream, &grammar, &episodes, &out)?;

    // Throughput measurement (recommendation D from the panel
    // mentorship pass): measure µs per residual sample on the actual
    // pipeline, on this hardware, with this build. Refuse to
    // extrapolate to TPS — see §9 limitations.
    write_throughput_report(seed, &out)?;

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
    writeln!(
        f,
        "harness        = tpcds_with_perturbations(seed={seed})"
    )?;
    writeln!(f, "samples_total  = {}", samples_total)?;
    writeln!(
        f,
        "stream_build   = {:.4} s (median of 5 runs)",
        stream_med
    )?;
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
        .map(|m| (m.name().to_string(), Vec::with_capacity(STRESS_SCALES.len())))
        .collect();
    // Per-(motif, scale) median time-to-detection. NaN means no
    // detection at this scale, which is structurally different from
    // "detected immediately" (TTD = 0) and we keep them separate.
    let mut ttd_series: Vec<(String, Vec<f64>)> = MotifClass::ALL
        .iter()
        .map(|m| (m.name().to_string(), Vec::with_capacity(STRESS_SCALES.len())))
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
                _ => f64::NAN,
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
    write_provenance(&out.join(format!("{dataset}.provenance.txt")), &stream.source)?;
    write_episodes_csv(&out.join(format!("{dataset}.exemplar.episodes.csv")), &episodes)?;
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
        plots::plot_residual_overlay(&path, &title, &stream, m.residual_class(), &episodes, m)?;
    }
    Ok(())
}

fn run_real(dataset: &str, path: PathBuf, out: PathBuf) -> Result<()> {
    let adapter = adapter_for(dataset)?;
    let stream = adapter.load(&path)?;
    let grammar = MotifGrammar::default();
    let engine = MotifEngine::new(grammar);
    let episodes = engine.run(&stream);
    fs::create_dir_all(&out)?;
    write_provenance(&out.join(format!("{dataset}.provenance.txt")), &stream.source)?;
    write_episodes_csv(&out.join(format!("{dataset}.episodes.csv")), &episodes)?;

    // Emit per-motif PNG residual overlays for any class with samples,
    // so a real-data run produces the same figure shape the paper uses
    // for the controlled tier. Captions identify the source as real
    // (no `[exemplar]` tag) so a reader cannot confuse the two.
    for m in MotifClass::ALL {
        let count = stream.iter_class(m.residual_class()).count();
        if count == 0 {
            continue;
        }
        let title = format!("{dataset} (real shard): {} residuals + episodes", m.name());
        let path = out.join(format!("{dataset}.{}.png", m.name()));
        plots::plot_residual_overlay(&path, &title, &stream, m.residual_class(), &episodes, m)?;
    }

    eprintln!("real-data run: {} episodes from {}", episodes.len(), stream.source);
    eprintln!(
        "stream_fingerprint = {}",
        hex(&stream.fingerprint())
    );
    eprintln!(
        "episodes_fingerprint = {}",
        replay::fingerprint_hex(&episodes)
    );
    eprintln!(
        "(no metrics written: real-data runs do not have ground-truth windows.)"
    );
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
