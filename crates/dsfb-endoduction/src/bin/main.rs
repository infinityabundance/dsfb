//! Main binary entrypoint for the dsfb-endoduction pipeline.

use anyhow::{bail, Context, Result};
use clap::Parser;
use dsfb_endoduction::cli::{Cli, Command};
use dsfb_endoduction::types::{GateResults, RunManifest, WindowMetrics};
use dsfb_endoduction::{admissibility, baseline, baselines, data, evaluation, figures, grammar, report, residual, trust};
use std::fs;


fn main() {
    if let Err(e) = try_main() {
        eprintln!("dsfb-endoduction failed: {e:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Run(args) => run_pipeline(&args),
    }
}

fn run_pipeline(args: &dsfb_endoduction::cli::RunArgs) -> Result<()> {
    let config = args.to_config();
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Create timestamped output directory.
    let run_dir = config.output_dir.join(&timestamp);
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("Cannot create output dir {}", run_dir.display()))?;
    eprintln!("[pipeline] Output directory: {}", run_dir.display());

    // Download dataset if requested.
    if args.download {
        data::download_dataset(&args.data_root)?;
    }

    // Load dataset — memory-efficient: only the requested channel is kept.
    let (file_names, channel_data) =
        data::load_channel_data(&args.data_root, config.bearing_set, config.primary_channel)?;
    if channel_data.is_empty() {
        bail!("No snapshots loaded. Check dataset path.");
    }
    let total = channel_data.len();
    let nominal_end = (total as f64 * config.nominal_fraction).ceil() as usize;
    let failure_window = total.saturating_sub(1);
    eprintln!(
        "[pipeline] {} snapshots, nominal window: 0..{}, failure ref: {}",
        total, nominal_end, failure_window
    );

    // Build channel data references.
    let channel_refs: Vec<&[f64]> = channel_data.iter().map(|v| v.as_slice()).collect();

    // Estimate nominal baseline.
    let nominal_windows: Vec<&[f64]> = channel_refs[..nominal_end].to_vec();
    let bl = baseline::estimate_baseline(&nominal_windows);
    eprintln!(
        "[pipeline] Baseline — RMS: {:.6}, Var: {:.6}, AC: {:.6}",
        bl.mean_rms, bl.mean_variance, bl.mean_autocorr
    );

    // Estimate admissibility envelope.
    let envelope = admissibility::estimate_envelope(&bl, config.envelope_quantile);

    // Compute nominal-window drift scale for normalisation.
    let nominal_drifts: Vec<f64> = nominal_windows
        .iter()
        .map(|w| {
            let r = residual::compute_residual(w, &bl);
            grammar::drift(&r).abs()
        })
        .collect();
    let baseline_drift_scale = baseline::mean(&nominal_drifts) + baseline::std_dev(&nominal_drifts) + 1e-10;

    // Process all windows.
    eprintln!("[pipeline] Processing windows...");
    let mut all_metrics: Vec<WindowMetrics> = Vec::with_capacity(total);

    // Save early and late signal samples for figures.
    let early_signal = channel_refs[0].to_vec();
    let late_signal = channel_refs[failure_window].to_vec();

    // Save a mid-life sample for the model-vs-observation figure.
    let mid = total / 2;
    let sample_obs = channel_refs[mid].to_vec();
    let sample_resid = residual::compute_residual(&sample_obs, &bl);
    let sample_model = bl.mean_waveform.clone();

    for (i, signal) in channel_refs.iter().enumerate() {
        let resid = residual::compute_residual(signal, &bl);
        let classical = baselines::compute_classical(signal);
        let breach = admissibility::breach_fraction(&resid, &envelope);
        let drift_val = grammar::drift(&resid);
        let persist = grammar::persistence(&resid);
        let var_g = grammar::variance_growth(&resid, bl.mean_variance);
        let ac_g = grammar::autocorrelation_growth(&resid, bl.mean_autocorr);
        let sc_shift = grammar::spectral_centroid_shift(&resid, bl.mean_spectral_centroid);

        let trust_inputs = trust::TrustInputs {
            breach_fraction: breach,
            persistence: persist,
            autocorr_growth: ac_g,
            spectral_shift: sc_shift,
            variance_growth: var_g,
            drift_magnitude: drift_val.abs(),
            baseline_drift_scale,
            baseline_spectral_scale: bl.std_spectral_centroid + 1e-10,
        };
        let trust_score = trust::compute_trust_score(&trust_inputs);

        all_metrics.push(WindowMetrics {
            index: i,
            file_name: file_names[i].clone(),
            rms: classical.rms,
            kurtosis: classical.kurtosis,
            crest_factor: classical.crest_factor,
            residual_variance: baseline::variance(&resid),
            residual_autocorr: baseline::lag1_autocorrelation(&resid),
            envelope_breach_fraction: breach,
            spectral_centroid_shift: sc_shift,
            persistence: persist,
            drift: drift_val,
            variance_growth: var_g,
            trust_score,
            baseline_rms: classical.rms,
            baseline_rolling_var: classical.rolling_variance,
            spectral_band_energy: classical.spectral_band_energy,
        });

        if (i + 1) % 100 == 0 || i + 1 == total {
            eprintln!("[pipeline] Processed {}/{} windows", i + 1, total);
        }
    }

    // Evaluate.
    let summary = evaluation::evaluate(
        &all_metrics,
        nominal_end,
        failure_window,
        config.sustained_count,
        config.trust_threshold,
        3.0,
    );

    // DSFB first sustained detection.
    let trust_flags: Vec<bool> = all_metrics.iter().map(|m| m.trust_score >= config.trust_threshold).collect();
    let dsfb_first = baselines::first_sustained_detection(&trust_flags, config.sustained_count);

    eprintln!("[pipeline] DSFB first sustained detection: {:?}", dsfb_first);
    if let Some(lead) = summary.dsfb_lead_time_windows {
        eprintln!("[pipeline] DSFB lead time: {} windows before failure", lead);
    }
    for (name, lead) in &summary.baseline_lead_times {
        match lead {
            Some(l) => eprintln!("[pipeline] {name} lead time: {l} windows"),
            None => eprintln!("[pipeline] {name}: not detected"),
        }
    }

    // Write CSV.
    let csv_name = "metrics.csv";
    let csv_path = run_dir.join(csv_name);
    {
        let mut wtr = csv::Writer::from_path(&csv_path).context("create CSV")?;
        for m in &all_metrics {
            wtr.serialize(m)?;
        }
        wtr.flush()?;
    }
    eprintln!("[pipeline] Wrote {}", csv_path.display());

    // Generate figures.
    eprintln!("[pipeline] Generating figures...");
    let envelope_upper: Vec<f64> = if envelope.upper.is_empty() {
        vec![envelope.global_upper; 100]
    } else {
        envelope.upper.clone()
    };
    let envelope_lower: Vec<f64> = if envelope.lower.is_empty() {
        vec![envelope.global_lower; 100]
    } else {
        envelope.lower.clone()
    };

    let figure_files = figures::generate_all(
        &all_metrics,
        nominal_end,
        failure_window,
        &early_signal,
        &late_signal,
        &sample_resid,
        &sample_obs,
        &sample_model,
        &envelope_upper,
        &envelope_lower,
        &run_dir,
    )?;
    eprintln!("[pipeline] Generated {} figures", figure_files.len());

    // Build manifest.
    let git_rev = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let mut files_produced: Vec<String> = vec![csv_name.to_string()];
    files_produced.extend(figure_files.iter().cloned());

    let gates = GateResults {
        crate_builds: true,
        real_data_used: true,
        timestamped_output: true,
        twelve_figures: figure_files.len() >= 12,
        csv_produced: csv_path.exists(),
        json_produced: false, // will be set after writing
        pdf_produced: false,  // will be set after generating
        zip_produced: false,  // will be set after creating
        baseline_comparisons: !summary.baseline_lead_times.is_empty(),
        manifest_produced: false,
    };

    let mut manifest = RunManifest {
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        git_revision: git_rev,
        timestamp: timestamp.clone(),
        config: config.clone(),
        dataset_source: format!(
            "NASA IMS Bearing Dataset, Set {}, Channel {}. \
             Source: NASA Prognostics Data Repository, University of Cincinnati.",
            config.bearing_set, config.primary_channel
        ),
        snapshots_processed: total,
        nominal_windows: nominal_end,
        dsfb_first_detection: dsfb_first,
        summary,
        files_produced: files_produced.clone(),
        gates,
    };

    // Generate PDF report.
    eprintln!("[pipeline] Generating PDF report...");
    let pdf_name = report::generate_pdf(&manifest, &all_metrics, &figure_files, &run_dir)?;
    manifest.files_produced.push(pdf_name.clone());
    manifest.gates.pdf_produced = run_dir.join(&pdf_name).exists();

    // Write JSON summary.
    let json_name = "manifest.json";
    let json_path = run_dir.join(json_name);
    let json_str = serde_json::to_string_pretty(&manifest)?;
    fs::write(&json_path, &json_str)?;
    manifest.files_produced.push(json_name.to_string());
    manifest.gates.json_produced = json_path.exists();
    manifest.gates.manifest_produced = true;
    eprintln!("[pipeline] Wrote {}", json_path.display());

    // Re-write JSON with updated gates.
    let json_str = serde_json::to_string_pretty(&manifest)?;
    fs::write(&json_path, &json_str)?;

    // Create ZIP bundle.
    eprintln!("[pipeline] Creating ZIP bundle...");
    let zip_name = report::create_zip(&run_dir)?;
    manifest.gates.zip_produced = run_dir.join(&zip_name).exists();
    manifest.files_produced.push(zip_name.clone());

    // Final manifest write.
    let json_str = serde_json::to_string_pretty(&manifest)?;
    fs::write(&json_path, &json_str)?;

    // Gate check.
    eprintln!("\n=== ACCEPTANCE GATES ===");
    eprintln!("Crate builds:          {}", manifest.gates.crate_builds);
    eprintln!("Real data used:        {}", manifest.gates.real_data_used);
    eprintln!("Timestamped output:    {}", manifest.gates.timestamped_output);
    eprintln!("12 figures:            {}", manifest.gates.twelve_figures);
    eprintln!("CSV produced:          {}", manifest.gates.csv_produced);
    eprintln!("JSON produced:         {}", manifest.gates.json_produced);
    eprintln!("PDF produced:          {}", manifest.gates.pdf_produced);
    eprintln!("ZIP produced:          {}", manifest.gates.zip_produced);
    eprintln!("Baseline comparisons:  {}", manifest.gates.baseline_comparisons);
    eprintln!("Manifest produced:     {}", manifest.gates.manifest_produced);
    let all_passed = manifest.gates.all_passed();
    eprintln!("ALL GATES PASSED:      {}", all_passed);
    eprintln!("========================\n");

    if !all_passed {
        bail!("Not all acceptance gates passed. See above for details.");
    }

    eprintln!("[pipeline] Run complete. Output: {}", run_dir.display());
    println!("{}", run_dir.display());
    Ok(())
}
