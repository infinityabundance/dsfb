// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — CLI demo binary
//
// Runs the full DSFB pipeline on NASA PCoE B0005 data and exports
// all artifacts to a timestamped output folder: CSV, JSON, 12 figures
// (SVG + PNG), and a ZIP archive.

use clap::Parser;
use dsfb_battery::{
    build_dsfb_detection, build_stage2_audit_trace, build_threshold_detection,
    export_audit_trace_json, export_trajectory_csv, export_zip, generate_all_figures,
    load_b0005_csv, run_dsfb_pipeline, verify_theorem1, AuditTraceBuildContext, FigureContext,
    PipelineConfig, Stage2Results,
};
use std::path::PathBuf;

/// DSFB Battery Health Monitoring — full pipeline CLI.
///
/// Produces semiotic trajectory CSV, detection results JSON,
/// 12 publication figures (SVG and PNG), and a ZIP archive.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to the NASA B0005 capacity CSV file
    #[arg(short, long)]
    data: Option<PathBuf>,

    /// Output directory (default: outputs/dsfb_battery_<timestamp>)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Healthy window size in cycles (N_h)
    #[arg(long)]
    healthy_window: Option<usize>,

    /// Drift averaging window (W)
    #[arg(long)]
    drift_window: Option<usize>,

    /// Drift persistence length (L_d)
    #[arg(long)]
    drift_persistence: Option<usize>,

    /// Slew persistence length (L_s)
    #[arg(long)]
    slew_persistence: Option<usize>,

    /// Drift threshold (θ_d)
    #[arg(long)]
    drift_threshold: Option<f64>,

    /// Slew threshold (θ_s)
    #[arg(long)]
    slew_threshold: Option<f64>,

    /// End-of-life fraction of initial capacity
    #[arg(long)]
    eol_fraction: Option<f64>,

    /// Boundary fraction for grammar classification
    #[arg(long)]
    boundary_fraction: Option<f64>,

    /// Skip figure generation
    #[arg(long, default_value_t = false)]
    no_figures: bool,

    /// Skip ZIP packaging
    #[arg(long, default_value_t = false)]
    no_zip: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_path = cli
        .data
        .unwrap_or_else(|| crate_dir.join("data").join("nasa_b0005_capacity.csv"));

    if !data_path.exists() {
        eprintln!("ERROR: {} not found.", data_path.display());
        eprintln!("Run the extraction script first:");
        eprintln!("  python3 tools/extract_nasa_b0005.py");
        std::process::exit(1);
    }

    println!(
        "Loading NASA PCoE B0005 capacity data from {}",
        data_path.display()
    );
    let raw_data = load_b0005_csv(&data_path)?;
    let capacities: Vec<f64> = raw_data.iter().map(|(_, c)| *c).collect();

    println!("  Cycles: {}", capacities.len());
    println!("  Initial capacity: {:.4} Ah", capacities[0]);
    println!(
        "  Final capacity:   {:.4} Ah",
        capacities[capacities.len() - 1]
    );

    // Pipeline configuration — override any field the user supplies
    let mut config = PipelineConfig::default();
    if let Some(v) = cli.healthy_window {
        config.healthy_window = v;
    }
    if let Some(v) = cli.drift_window {
        config.drift_window = v;
    }
    if let Some(v) = cli.drift_persistence {
        config.drift_persistence = v;
    }
    if let Some(v) = cli.slew_persistence {
        config.slew_persistence = v;
    }
    if let Some(v) = cli.drift_threshold {
        config.drift_threshold = v;
    }
    if let Some(v) = cli.slew_threshold {
        config.slew_threshold = v;
    }
    if let Some(v) = cli.eol_fraction {
        config.eol_fraction = v;
    }
    if let Some(v) = cli.boundary_fraction {
        config.boundary_fraction = v;
    }

    let eol_capacity = config.eol_fraction * capacities[0];
    println!(
        "  EOL threshold ({}% of initial): {:.4} Ah",
        (config.eol_fraction * 100.0) as u32,
        eol_capacity
    );

    // Run DSFB pipeline
    println!("\nRunning DSFB pipeline...");
    let (envelope, trajectory) = run_dsfb_pipeline(&capacities, &config)?;
    println!(
        "  Envelope: μ = {:.4} Ah, σ = {:.4} Ah, ρ = {:.4} Ah",
        envelope.mu, envelope.sigma, envelope.rho
    );

    // Detection comparison
    let dsfb_det = build_dsfb_detection(&trajectory, &capacities, eol_capacity);
    let threshold_det = build_threshold_detection(&capacities, 0.85, eol_capacity);

    println!("\n=== Detection Results ===");
    print_detection(&dsfb_det);
    print_detection(&threshold_det);

    // Theorem 1 verification
    let thm1 = verify_theorem1(&envelope, &trajectory, &config);
    println!("\n=== Theorem 1 Verification ===");
    println!("  ρ (envelope radius): {:.6} Ah", thm1.rho);
    println!("  α (observed drift rate): {:.6} Ah/cycle", thm1.alpha);
    println!("  κ (envelope expansion): {:.6} Ah/cycle", thm1.kappa);
    println!("  t* = ⌈ρ/(α−κ)⌉ = {} cycles", thm1.t_star);
    println!(
        "  Actual detection cycle: {:?}",
        thm1.actual_detection_cycle
    );
    println!("  Bound satisfied: {:?}", thm1.bound_satisfied);

    // Create output folder
    let output_dir = cli.output.unwrap_or_else(|| {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        crate_dir
            .join("outputs")
            .join(format!("dsfb_battery_{}", timestamp))
    });
    std::fs::create_dir_all(&output_dir)?;

    let provenance = "NASA PCoE Battery Dataset, Cell B0005 (18650 Li-ion, constant-current discharge at 2A, 24°C ambient)";

    // CSV + JSON
    export_trajectory_csv(&trajectory, &output_dir.join("semiotic_trajectory.csv"))?;
    println!("\nExported semiotic_trajectory.csv");

    let results = Stage2Results {
        data_provenance: provenance.to_string(),
        config: config.clone(),
        envelope,
        dsfb_detection: dsfb_det.clone(),
        threshold_detection: threshold_det.clone(),
        theorem1: thm1.clone(),
    };
    let supporting_figures = if cli.no_figures {
        Vec::new()
    } else {
        expected_figure_files()
    };
    let supporting_tables = vec!["semiotic_trajectory.csv".to_string()];
    let audit_trace = build_stage2_audit_trace(AuditTraceBuildContext {
        results: &results,
        raw_input: &raw_data,
        trajectory: &trajectory,
        source_artifact: Some(&data_path),
        supporting_figures: &supporting_figures,
        supporting_tables: &supporting_tables,
    })?;
    export_audit_trace_json(
        &audit_trace,
        &output_dir.join("stage2_detection_results.json"),
    )?;
    println!("Exported stage2_detection_results.json");

    // Figures
    if !cli.no_figures {
        println!("\nGenerating 12 figures (SVG + PNG)...");
        let fig_ctx = FigureContext {
            capacities: &capacities,
            trajectory: &trajectory,
            envelope: &envelope,
            config: &config,
            dsfb_detection: &dsfb_det,
            threshold_detection: &threshold_det,
            theorem1: &thm1,
            data_provenance: provenance,
        };
        generate_all_figures(&fig_ctx, &output_dir)?;
        println!("Exported 24 figure files (12 SVG + 12 PNG)");
    }

    // ZIP
    if !cli.no_zip {
        let zip_path = output_dir.join("dsfb_battery_results.zip");
        export_zip(&output_dir, &zip_path)?;
        println!("Exported {}", zip_path.display());
    }

    println!("\nAll artifacts written to: {}", output_dir.display());
    Ok(())
}

fn print_detection(det: &dsfb_battery::DetectionResult) {
    println!("  {}", det.method);
    println!("    Alarm cycle: {:?}", det.alarm_cycle);
    println!("    EOL cycle:   {:?}", det.eol_cycle);
    println!("    Lead time:   {:?} cycles", det.lead_time_cycles);
}

fn expected_figure_files() -> Vec<String> {
    [
        "fig01_capacity_fade.svg",
        "fig02_residual_trajectory.svg",
        "fig03_drift_trajectory.svg",
        "fig04_slew_trajectory.svg",
        "fig05_admissibility_envelope.svg",
        "fig06_grammar_state_timeline.svg",
        "fig07_detection_comparison.svg",
        "fig08_theorem1_verification.svg",
        "fig09_semiotic_projection.svg",
        "fig10_cumulative_drift.svg",
        "fig11_lead_time_comparison.svg",
        "fig12_heuristics_bank_entry.svg",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
