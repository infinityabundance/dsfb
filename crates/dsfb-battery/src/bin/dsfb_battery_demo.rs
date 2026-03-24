// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — CLI demo binary
//
// Runs the full DSFB pipeline on NASA PCoE B0005 data and exports
// all artifacts to a timestamped output folder.

use dsfb_battery::{
    build_dsfb_detection, build_threshold_detection, export_results_json, export_trajectory_csv,
    load_b0005_csv, run_dsfb_pipeline, verify_theorem1, PipelineConfig, Stage2Results,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_path = crate_dir.join("data").join("nasa_b0005_capacity.csv");

    if !data_path.exists() {
        eprintln!("ERROR: {} not found.", data_path.display());
        eprintln!("Run the extraction script first:");
        eprintln!("  python3 tools/extract_nasa_b0005.py");
        std::process::exit(1);
    }

    println!("Loading NASA PCoE B0005 capacity data from {}", data_path.display());
    let raw_data = load_b0005_csv(&data_path)?;
    let capacities: Vec<f64> = raw_data.iter().map(|(_, c)| *c).collect();

    println!("  Cycles: {}", capacities.len());
    println!("  Initial capacity: {:.4} Ah", capacities[0]);
    println!(
        "  Final capacity:   {:.4} Ah",
        capacities[capacities.len() - 1]
    );

    // Pipeline configuration (Stage II benchmark, Section 8)
    let config = PipelineConfig::default();
    let eol_capacity = config.eol_fraction * capacities[0];
    println!("  EOL threshold (80% of initial): {:.4} Ah", eol_capacity);

    // Run DSFB pipeline
    println!("\nRunning DSFB pipeline...");
    let (envelope, trajectory) = run_dsfb_pipeline(&capacities, &config)?;
    println!("  Envelope: μ = {:.4} Ah, σ = {:.4} Ah, ρ = {:.4} Ah",
        envelope.mu, envelope.sigma, envelope.rho);

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
    println!("  Actual detection cycle: {:?}", thm1.actual_detection_cycle);
    println!("  Bound satisfied: {:?}", thm1.bound_satisfied);

    // Create timestamped output folder
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = crate_dir
        .join("outputs")
        .join(format!("dsfb_battery_{}", timestamp));
    std::fs::create_dir_all(&output_dir)?;

    // Export artifacts
    export_trajectory_csv(&trajectory, &output_dir.join("semiotic_trajectory.csv"))?;
    println!("\nExported semiotic_trajectory.csv");

    let results = Stage2Results {
        data_provenance: "NASA PCoE Battery Dataset, Cell B0005 (18650 Li-ion, constant-current discharge at 2A, 24°C ambient)".to_string(),
        config,
        envelope,
        dsfb_detection: dsfb_det,
        threshold_detection: threshold_det,
        theorem1: thm1,
    };
    export_results_json(&results, &output_dir.join("stage2_detection_results.json"))?;
    println!("Exported stage2_detection_results.json");

    println!("\nAll artifacts written to: {}", output_dir.display());
    Ok(())
}

fn print_detection(det: &dsfb_battery::DetectionResult) {
    println!("  {}", det.method);
    println!("    Alarm cycle: {:?}", det.alarm_cycle);
    println!("    EOL cycle:   {:?}", det.eol_cycle);
    println!("    Lead time:   {:?} cycles", det.lead_time_cycles);
}
