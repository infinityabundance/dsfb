use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

use dsfb_fusion_bench::io::{
    ensure_outdir, write_heatmap_csv, write_manifest_json, write_summary_csv,
    write_trajectories_csv, HeatmapRow, Manifest, SummaryRow, TrajectoryRow, OUTPUT_SCHEMA_VERSION,
};
use dsfb_fusion_bench::methods::cov_inflate::CovInflateMethod;
use dsfb_fusion_bench::methods::dsfb::DsfbAdaptiveMethod;
use dsfb_fusion_bench::methods::equal::EqualMethod;
use dsfb_fusion_bench::methods::irls_huber::IrlsHuberMethod;
use dsfb_fusion_bench::methods::nis_gating::{NisGatingMethod, NisMode};
use dsfb_fusion_bench::methods::{
    canonical_method_list, solve_group_weighted_wls, ReconstructionMethod, METHOD_ORDER,
};
use dsfb_fusion_bench::metrics::{MethodMetrics, MetricsAccumulator};
use dsfb_fusion_bench::sim::diagnostics::{build_diagnostic_model, DiagnosticModel};
use dsfb_fusion_bench::sim::state::{generate_simulation_data, BenchConfig, SimulationData};
use dsfb_fusion_bench::timing::TimingAccumulator;

#[derive(Debug, Parser)]
#[command(name = "dsfb-fusion-bench")]
#[command(about = "Deterministic synthetic benchmarking for DSFB fusion diagnostics")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long, default_value = "output-dsfb-fusion-bench")]
    outdir: PathBuf,

    #[arg(long)]
    seed: Option<u64>,

    #[arg(long, default_value_t = false)]
    run_default: bool,

    #[arg(long, default_value_t = false)]
    run_sweep: bool,

    #[arg(long)]
    methods: Option<String>,
}

#[derive(Debug, Clone)]
struct MethodRunResult {
    summary: SummaryRow,
    metrics: MethodMetrics,
    trajectories: Vec<TrajectoryRow>,
}

fn resolve_default_config_path(run_default: bool) -> PathBuf {
    let file = if run_default {
        "default.toml"
    } else {
        "sweep.toml"
    };

    let local = PathBuf::from("configs").join(file);
    if local.exists() {
        return local;
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join(file)
}

fn resolve_run_output_dir(base_outdir: &Path) -> Result<PathBuf> {
    ensure_outdir(base_outdir)?;

    let output = Command::new("date")
        .arg("-u")
        .arg("+%Y%m%d_%H%M%S")
        .output()
        .context("failed to execute date command for run timestamp")?;

    if !output.status.success() {
        bail!("date command failed while creating output run directory");
    }

    let stamp = String::from_utf8(output.stdout)
        .context("date command produced non-UTF8 output")?
        .trim()
        .to_string();
    if stamp.is_empty() {
        bail!("date command returned empty timestamp");
    }

    let mut candidate = base_outdir.join(&stamp);
    let mut suffix = 1usize;
    while candidate.exists() {
        if suffix > 999 {
            bail!(
                "failed to allocate unique run output directory under {}",
                base_outdir.display()
            );
        }
        candidate = base_outdir.join(format!("{stamp}_{suffix:03}"));
        suffix += 1;
    }

    ensure_outdir(&candidate)?;
    Ok(candidate)
}

fn parse_methods(cli_methods: Option<&str>, cfg: &BenchConfig) -> Result<Vec<String>> {
    let requested: Vec<String> = if let Some(raw) = cli_methods {
        raw.split(',')
            .map(|m| m.trim().to_lowercase())
            .filter(|m| !m.is_empty())
            .collect()
    } else if !cfg.methods.is_empty() {
        cfg.methods.iter().map(|m| m.to_lowercase()).collect()
    } else {
        METHOD_ORDER.iter().map(|m| m.to_string()).collect()
    };

    if requested.is_empty() {
        bail!("methods list cannot be empty");
    }

    for m in &requested {
        if !METHOD_ORDER.contains(&m.as_str()) {
            bail!(
                "unknown method '{m}'. valid methods: {}",
                METHOD_ORDER.join(",")
            );
        }
    }

    Ok(canonical_method_list(&requested))
}

fn build_method(name: &str) -> Result<Box<dyn ReconstructionMethod>> {
    let method: Box<dyn ReconstructionMethod> = match name {
        "equal" => Box::new(EqualMethod),
        "cov_inflate" => Box::new(CovInflateMethod::new()),
        "irls_huber" => Box::new(IrlsHuberMethod::new()),
        "nis_hard" => Box::new(NisGatingMethod::new(NisMode::Hard)),
        "nis_soft" => Box::new(NisGatingMethod::new(NisMode::Soft)),
        "dsfb" => Box::new(DsfbAdaptiveMethod::new()),
        _ => bail!("unsupported method: {name}"),
    };
    Ok(method)
}

fn baseline_wls_us(model: &DiagnosticModel, data: &SimulationData) -> f64 {
    let mut acc = TimingAccumulator::default();
    let weights = vec![1.0; model.groups.len()];

    for frame in &data.measurements {
        let (_x, solve_time) = solve_group_weighted_wls(model, &frame.y_groups, &weights);
        acc.observe(solve_time, solve_time);
    }

    acc.avg_solve_us()
}

fn run_method(
    method_name: &str,
    cfg: &BenchConfig,
    model: &DiagnosticModel,
    data: &SimulationData,
    seed: u64,
    baseline_us: f64,
    alpha_beta: Option<(f64, f64)>,
    keep_trajectories: bool,
) -> Result<MethodRunResult> {
    let mut method = build_method(method_name)?;
    method.reset(cfg, model);

    let mut metrics_acc = MetricsAccumulator::new(method.has_weights());
    let mut timing_acc = TimingAccumulator::default();
    let mut trajectories = Vec::with_capacity(data.t.len());

    for step in 0..data.t.len() {
        let out = method.estimate(model, &data.measurements[step].y_groups);
        let err_norm = (&out.x_hat - &data.x_true[step]).norm();

        metrics_acc.observe(
            err_norm,
            out.group_weights.as_deref(),
            data.corruption_active[step],
        );
        timing_acc.observe(out.solve_time, out.total_time);

        if keep_trajectories {
            trajectories.push(TrajectoryRow {
                t: data.t[step],
                method: method.name().to_string(),
                err_norm,
                weights: out.group_weights,
            });
        }
    }

    let metrics = metrics_acc.finalize();
    let total_us = timing_acc.avg_total_us();
    let overhead_us = (total_us - baseline_us).max(0.0);

    let summary = SummaryRow {
        method: method.name().to_string(),
        seed,
        n: cfg.n,
        k: cfg.group_count(),
        m: cfg.total_measurements(),
        peak_err: metrics.peak_err,
        rms_err: metrics.rms_err,
        false_downweight_rate: metrics.false_downweight_rate,
        baseline_wls_us: baseline_us,
        overhead_us,
        total_us,
        alpha: alpha_beta.map(|v| v.0),
        beta: alpha_beta.map(|v| v.1),
    };

    Ok(MethodRunResult {
        summary,
        metrics,
        trajectories,
    })
}

fn run_default(cfg: &BenchConfig, methods: &[String], outdir: &Path) -> Result<()> {
    let model = build_diagnostic_model(cfg)?;

    let mut summary_rows = Vec::<SummaryRow>::new();
    let mut trajectory_rows = Vec::<TrajectoryRow>::new();

    let mut seeds = cfg.seeds.clone();
    seeds.sort_unstable();

    for seed in seeds {
        let data = generate_simulation_data(cfg, &model, seed)?;
        let baseline_us = baseline_wls_us(&model, &data);

        for method_name in methods {
            let result = run_method(
                method_name,
                cfg,
                &model,
                &data,
                seed,
                baseline_us,
                Some((cfg.dsfb_alpha, cfg.dsfb_beta)),
                true,
            )?;
            summary_rows.push(result.summary);
            trajectory_rows.extend(result.trajectories);
        }
    }

    let summary_path = outdir.join("summary.csv");
    let heatmap_path = outdir.join("heatmap.csv");
    let traj_path = outdir.join("trajectories.csv");
    let sim_path = outdir.join("sim-dsfb-fusion-bench.csv");

    write_summary_csv(&summary_path, &summary_rows)?;
    write_heatmap_csv(&heatmap_path, &[])?;
    write_trajectories_csv(&traj_path, &trajectory_rows, cfg.group_count())?;
    write_trajectories_csv(&sim_path, &trajectory_rows, cfg.group_count())?;

    write_manifest_json(
        outdir,
        &Manifest {
            schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
            mode: "default".to_string(),
            methods: methods.to_vec(),
            seeds: cfg.seeds.clone(),
            note: "Deterministic synthetic benchmark outputs".to_string(),
        },
    )?;

    Ok(())
}

#[derive(Debug, Default, Clone)]
struct HeatAgg {
    peak_sum: f64,
    rms_sum: f64,
    false_sum: f64,
    false_count: usize,
    count: usize,
}

fn run_sweep(cfg: &BenchConfig, methods: &[String], outdir: &Path) -> Result<()> {
    let alpha_values = cfg
        .alpha_values
        .clone()
        .context("sweep requires alpha_values in config")?;
    let beta_values = cfg
        .beta_values
        .clone()
        .context("sweep requires beta_values in config")?;

    if alpha_values.is_empty() || beta_values.is_empty() {
        bail!("alpha_values and beta_values must be non-empty for sweep");
    }

    let mut alphas = alpha_values;
    let mut betas = beta_values;
    alphas.sort_by(|a, b| a.total_cmp(b));
    betas.sort_by(|a, b| a.total_cmp(b));

    let mut seeds = cfg.seeds.clone();
    seeds.sort_unstable();

    let mut summary_rows = Vec::<SummaryRow>::new();
    let mut heatmap_rows = Vec::<HeatmapRow>::new();

    for alpha in &alphas {
        for beta in &betas {
            let mut cfg_ab = cfg.clone();
            cfg_ab.dsfb_alpha = *alpha;
            cfg_ab.dsfb_beta = *beta;

            let model = build_diagnostic_model(&cfg_ab)?;
            let mut aggs = vec![HeatAgg::default(); methods.len()];

            for seed in &seeds {
                let data = generate_simulation_data(&cfg_ab, &model, *seed)?;
                let baseline_us = baseline_wls_us(&model, &data);

                for (idx, method_name) in methods.iter().enumerate() {
                    let result = run_method(
                        method_name,
                        &cfg_ab,
                        &model,
                        &data,
                        *seed,
                        baseline_us,
                        Some((*alpha, *beta)),
                        false,
                    )?;

                    summary_rows.push(result.summary.clone());

                    aggs[idx].peak_sum += result.metrics.peak_err;
                    aggs[idx].rms_sum += result.metrics.rms_err;
                    if let Some(v) = result.metrics.false_downweight_rate {
                        aggs[idx].false_sum += v;
                        aggs[idx].false_count += 1;
                    }
                    aggs[idx].count += 1;
                }
            }

            for (idx, method_name) in methods.iter().enumerate() {
                let agg = &aggs[idx];
                if agg.count == 0 {
                    continue;
                }
                heatmap_rows.push(HeatmapRow {
                    alpha: *alpha,
                    beta: *beta,
                    method: method_name.clone(),
                    peak_err: agg.peak_sum / agg.count as f64,
                    rms_err: agg.rms_sum / agg.count as f64,
                    false_downweight_rate: if agg.false_count > 0 {
                        Some(agg.false_sum / agg.false_count as f64)
                    } else {
                        None
                    },
                });
            }
        }
    }

    let summary_path = outdir.join("summary_sweep.csv");
    let heatmap_path = outdir.join("heatmap.csv");
    let default_summary_path = outdir.join("summary.csv");
    let traj_path = outdir.join("trajectories.csv");
    let sim_path = outdir.join("sim-dsfb-fusion-bench.csv");

    write_summary_csv(&summary_path, &summary_rows)?;
    if !default_summary_path.exists() {
        write_summary_csv(&default_summary_path, &summary_rows)?;
    }
    write_heatmap_csv(&heatmap_path, &heatmap_rows)?;
    if !traj_path.exists() {
        write_trajectories_csv(&traj_path, &[], cfg.group_count())?;
    }
    if !sim_path.exists() {
        write_trajectories_csv(&sim_path, &[], cfg.group_count())?;
    }

    write_manifest_json(
        outdir,
        &Manifest {
            schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
            mode: "sweep".to_string(),
            methods: methods.to_vec(),
            seeds: cfg.seeds.clone(),
            note: "Deterministic synthetic benchmark outputs with alpha/beta sweep".to_string(),
        },
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.run_default == cli.run_sweep {
        bail!("choose exactly one of --run-default or --run-sweep");
    }

    let config_path = if let Some(path) = cli.config.clone() {
        path
    } else {
        resolve_default_config_path(cli.run_default)
    };

    let mut cfg = BenchConfig::from_toml_file(&config_path)?;
    if cfg.schema_version != OUTPUT_SCHEMA_VERSION {
        bail!(
            "config schema_version {} does not match output schema {}",
            cfg.schema_version,
            OUTPUT_SCHEMA_VERSION
        );
    }

    if let Some(seed) = cli.seed {
        cfg.seeds = vec![seed];
    }

    let methods = parse_methods(cli.methods.as_deref(), &cfg)?;
    let run_outdir = resolve_run_output_dir(&cli.outdir)?;

    if cli.run_default {
        run_default(&cfg, &methods, &run_outdir)?;
    } else {
        run_sweep(&cfg, &methods, &run_outdir)?;
    }

    println!("wrote outputs to {}", run_outdir.display());
    Ok(())
}
