//! Drift-Impulse Simulation Example
//!
//! Runs a simulation comparing DSFB against baseline methods with an impulse disturbance

use dsfb::{
    sim::{peak_error_during_impulse, recovery_time, rms_error, run_simulation, SimConfig},
    DsfbParams,
};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

fn utc_timestamp() -> io::Result<String> {
    let output = Command::new("date")
        .arg("-u")
        .arg("+%Y%m%d_%H%M%S")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other("failed to execute date command"));
    }

    let stamp = String::from_utf8(output.stdout)
        .map_err(|err| io::Error::other(format!("invalid UTF-8 from date command: {err}")))?;
    let stamp = stamp.trim().to_string();
    if stamp.is_empty() {
        return Err(io::Error::other("empty timestamp from date command"));
    }

    Ok(stamp)
}

fn create_run_output_dir(base_dir: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(base_dir)?;

    let stamp = utc_timestamp()?;

    let mut run_dir = PathBuf::from(base_dir).join(&stamp);
    let mut suffix = 1usize;
    while run_dir.exists() {
        if suffix > 999 {
            return Err(io::Error::other(format!(
                "failed to allocate unique run output directory under {base_dir}"
            )));
        }
        run_dir = PathBuf::from(base_dir).join(format!("{stamp}_{suffix:03}"));
        suffix += 1;
    }

    fs::create_dir_all(&run_dir)?;
    Ok(run_dir)
}

fn main() -> std::io::Result<()> {
    println!("Running DSFB Drift-Impulse Simulation...\n");

    let base_outdir =
        std::env::var("DSFB_OUTPUT_BASE").unwrap_or_else(|_| "output-dsfb".to_string());
    let run_outdir = create_run_output_dir(&base_outdir)?;

    // Configure simulation
    let config = SimConfig {
        dt: 0.01,
        steps: 1000,
        sigma_noise: 0.05,
        sigma_alpha: 0.01,
        drift_beta: 0.1,
        impulse_start: 300,
        impulse_duration: 100,
        impulse_amplitude: 1.0,
        seed: 42,
    };

    // Configure DSFB parameters
    let dsfb_params = DsfbParams::new(
        0.5,  // k_phi
        0.1,  // k_omega
        0.01, // k_alpha
        0.95, // rho
        0.1,  // sigma0
    );

    // Run simulation
    println!("Configuration:");
    println!("  Time step: {}", config.dt);
    println!("  Total steps: {}", config.steps);
    println!("  Noise sigma: {}", config.sigma_noise);
    println!(
        "  Impulse start: {} (t={:.2})",
        config.impulse_start,
        config.impulse_start as f64 * config.dt
    );
    println!("  Impulse duration: {} steps", config.impulse_duration);
    println!("  Impulse amplitude: {}", config.impulse_amplitude);
    println!("  Output directory: {}", run_outdir.display());
    println!();

    let results = run_simulation(config.clone(), dsfb_params);

    // Calculate metrics
    let errors_mean: Vec<f64> = results.iter().map(|r| r.err_mean).collect();
    let errors_freqonly: Vec<f64> = results.iter().map(|r| r.err_freqonly).collect();
    let errors_dsfb: Vec<f64> = results.iter().map(|r| r.err_dsfb).collect();

    let rms_mean = rms_error(&errors_mean);
    let rms_freqonly = rms_error(&errors_freqonly);
    let rms_dsfb = rms_error(&errors_dsfb);

    let peak_mean = peak_error_during_impulse(
        &results,
        config.impulse_start,
        config.impulse_duration,
        |s| s.err_mean,
    );
    let peak_freqonly = peak_error_during_impulse(
        &results,
        config.impulse_start,
        config.impulse_duration,
        |s| s.err_freqonly,
    );
    let peak_dsfb = peak_error_during_impulse(
        &results,
        config.impulse_start,
        config.impulse_duration,
        |s| s.err_dsfb,
    );

    let impulse_end = config.impulse_start + config.impulse_duration;
    let recovery_threshold = 0.05;
    let recovery_mean = recovery_time(&results, impulse_end, recovery_threshold, |s| s.err_mean);
    let recovery_freqonly = recovery_time(&results, impulse_end, recovery_threshold, |s| {
        s.err_freqonly
    });
    let recovery_dsfb = recovery_time(&results, impulse_end, recovery_threshold, |s| s.err_dsfb);

    // Print metrics
    println!("METRICS SUMMARY");
    println!("===============");
    println!("\nRMS Errors:");
    println!("  Mean Fusion:    {:.6}", rms_mean);
    println!("  Freq-Only:      {:.6}", rms_freqonly);
    println!("  DSFB:           {:.6}", rms_dsfb);

    println!("\nPeak Error During Impulse:");
    println!("  Mean Fusion:    {:.6}", peak_mean);
    println!("  Freq-Only:      {:.6}", peak_freqonly);
    println!("  DSFB:           {:.6}", peak_dsfb);

    println!(
        "\nRecovery Time (steps after impulse, threshold={}):",
        recovery_threshold
    );
    println!("  Mean Fusion:    {}", recovery_mean);
    println!("  Freq-Only:      {}", recovery_freqonly);
    println!("  DSFB:           {}", recovery_dsfb);

    // Write CSV
    let csv_path = run_outdir.join("sim-dsfb.csv");
    let mut file = File::create(&csv_path)?;

    writeln!(
        file,
        "t,phi_true,phi_mean,phi_freqonly,phi_dsfb,err_mean,err_freqonly,err_dsfb,w2,s2"
    )?;

    for step in &results {
        writeln!(
            file,
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            step.t,
            step.phi_true,
            step.phi_mean,
            step.phi_freqonly,
            step.phi_dsfb,
            step.err_mean,
            step.err_freqonly,
            step.err_dsfb,
            step.w2,
            step.s2
        )?;
    }

    println!("\nCSV output written to: {}", csv_path.display());
    println!("Done!");

    Ok(())
}
