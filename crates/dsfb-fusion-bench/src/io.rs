use anyhow::{Context, Result};
use csv::WriterBuilder;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const OUTPUT_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone)]
pub struct SummaryRow {
    pub method: String,
    pub seed: u64,
    pub n: usize,
    pub k: usize,
    pub m: usize,
    pub peak_err: f64,
    pub rms_err: f64,
    pub false_downweight_rate: Option<f64>,
    pub baseline_wls_us: f64,
    pub overhead_us: f64,
    pub total_us: f64,
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct HeatmapRow {
    pub alpha: f64,
    pub beta: f64,
    pub method: String,
    pub peak_err: f64,
    pub rms_err: f64,
    pub false_downweight_rate: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TrajectoryRow {
    pub t: f64,
    pub method: String,
    pub err_norm: f64,
    pub weights: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub schema_version: String,
    pub mode: String,
    pub methods: Vec<String>,
    pub seeds: Vec<u64>,
    pub note: String,
}

fn fmt_f64(v: f64) -> String {
    format!("{v:.10}")
}

fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) => fmt_f64(x),
        None => "NA".to_string(),
    }
}

pub fn ensure_outdir(outdir: &Path) -> Result<()> {
    fs::create_dir_all(outdir)
        .with_context(|| format!("failed to create output directory: {}", outdir.display()))
}

pub fn write_summary_csv(path: &Path, rows: &[SummaryRow]) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_path(path)
        .with_context(|| format!("failed to open summary.csv for writing: {}", path.display()))?;

    wtr.write_record([
        "method",
        "seed",
        "n",
        "K",
        "M",
        "peak_err",
        "rms_err",
        "false_downweight_rate",
        "baseline_wls_us",
        "overhead_us",
        "total_us",
        "alpha",
        "beta",
        "schema_version",
    ])?;

    for row in rows {
        wtr.write_record([
            row.method.as_str(),
            &row.seed.to_string(),
            &row.n.to_string(),
            &row.k.to_string(),
            &row.m.to_string(),
            &fmt_f64(row.peak_err),
            &fmt_f64(row.rms_err),
            &fmt_opt(row.false_downweight_rate),
            &fmt_f64(row.baseline_wls_us),
            &fmt_f64(row.overhead_us),
            &fmt_f64(row.total_us),
            &fmt_opt(row.alpha),
            &fmt_opt(row.beta),
            OUTPUT_SCHEMA_VERSION,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn write_heatmap_csv(path: &Path, rows: &[HeatmapRow]) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_path(path)
        .with_context(|| format!("failed to open heatmap.csv for writing: {}", path.display()))?;

    wtr.write_record([
        "alpha",
        "beta",
        "method",
        "peak_err",
        "rms_err",
        "false_downweight_rate",
        "schema_version",
    ])?;

    for row in rows {
        wtr.write_record([
            &fmt_f64(row.alpha),
            &fmt_f64(row.beta),
            row.method.as_str(),
            &fmt_f64(row.peak_err),
            &fmt_f64(row.rms_err),
            &fmt_opt(row.false_downweight_rate),
            OUTPUT_SCHEMA_VERSION,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn write_trajectories_csv(path: &Path, rows: &[TrajectoryRow], k: usize) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_path(path)
        .with_context(|| {
            format!(
                "failed to open trajectories.csv for writing: {}",
                path.display()
            )
        })?;

    let mut header = vec![
        "t".to_string(),
        "method".to_string(),
        "err_norm".to_string(),
    ];
    for i in 0..k {
        header.push(format!("w_{i}"));
    }
    header.push("schema_version".to_string());
    wtr.write_record(&header)?;

    for row in rows {
        let mut record = vec![fmt_f64(row.t), row.method.clone(), fmt_f64(row.err_norm)];
        if let Some(w) = &row.weights {
            for i in 0..k {
                if i < w.len() {
                    record.push(fmt_f64(w[i]));
                } else {
                    record.push("NA".to_string());
                }
            }
        } else {
            for _ in 0..k {
                record.push("NA".to_string());
            }
        }
        record.push(OUTPUT_SCHEMA_VERSION.to_string());
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn write_manifest_json(outdir: &Path, manifest: &Manifest) -> Result<PathBuf> {
    let path = outdir.join("manifest.json");
    let payload = serde_json::to_string_pretty(manifest).context("failed to serialize manifest")?;
    fs::write(&path, payload)
        .with_context(|| format!("failed to write manifest: {}", path.display()))?;
    Ok(path)
}
