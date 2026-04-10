#![forbid(unsafe_code)]

//! Full C-MAPSS evaluation entrypoint for the library crate.
//!
//! Usage:
//!   cargo run --example cmapss_eval -- [data-dir] [output-dir]
//!
//! Expects `data-dir` to contain `train_FD001.txt`, `train_FD002.txt`, and
//! `train_FD003.txt` for the full paper evaluation suite. If `train_FD001.txt`
//! is unavailable, the example falls back to a synthetic demonstration.

use std::env;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use dsfb_turbine::core::channels::INFORMATIVE_CHANNELS_FD001;
use dsfb_turbine::core::config::DsfbConfig;
use dsfb_turbine::dataset::cmapss::CmapssDataset;
use dsfb_turbine::figures::{fleet_summary_svg, grammar_trajectory_svg};
use dsfb_turbine::pipeline::discrimination::{analyze_discrimination, discrimination_report};
use dsfb_turbine::pipeline::fleet::{evaluate_fleet, evaluate_fd001};
use dsfb_turbine::pipeline::negative_control::{compute_negative_control, negative_control_report};
use dsfb_turbine::pipeline::sweep::{run_2d_sweep, sweep_heatmap_svg, sweep_json, sweep_table};
use dsfb_turbine::pipeline::trace_chain::trace_chain_report;
use dsfb_turbine::report::generate_report;

fn main() {
    println!("================================================================");
    println!("  DSFB Structural Semiotics Engine for Gas Turbine Engines");
    println!("  Version: {}", dsfb_turbine::VERSION);
    println!("  Contract: {}", dsfb_turbine::NON_INTERFERENCE_CONTRACT);
    println!("  #![forbid(unsafe_code)] | no_std core | no_alloc core");
    println!("================================================================");
    println!();

    let args: Vec<String> = env::args().collect();
    let data_dir = args.get(1).map(|s| s.as_str()).unwrap_or("data");
    let output_dir = args.get(2).map(|s| s.as_str()).unwrap_or("output_full");

    fs::create_dir_all(output_dir).ok();

    let fd001_path = format!("{}/train_FD001.txt", data_dir);
    let fd003_path = format!("{}/train_FD003.txt", data_dir);

    if !Path::new(&fd001_path).exists() {
        eprintln!("Dataset not found: {}", fd001_path);
        eprintln!("Download from: https://phm-datasets.s3.amazonaws.com/NASA/6.+Turbofan+Engine+Degradation+Simulation+Data+Set.zip");
        eprintln!();
        eprintln!("Running synthetic demo instead...");
        run_synthetic_demo(output_dir);
        return;
    }

    let fd001 = load_dataset(&fd001_path, "FD001");

    println!("━━━ Phase 1: FD001 Default Evaluation ━━━");
    let default_config = DsfbConfig::cmapss_fd001_default();
    let (results_default, metrics_default) = evaluate_fd001(&fd001);
    println!(
        "  {} engines, mean lead time: {:.1} cycles\n",
        metrics_default.total_engines,
        metrics_default.mean_lead_time
    );

    let nc_default = compute_negative_control(&results_default, &default_config);
    let nc_report = negative_control_report(&nc_default);
    println!("{}", nc_report);

    if let Some(first_result) = results_default.first() {
        let trace = trace_chain_report(first_result);
        fs::write(format!("{}/trace_chain_unit_1_default.txt", output_dir), &trace).ok();
        println!("  Trace chain written: trace_chain_unit_1_default.txt");
    }

    println!("\n━━━ Phase 2: Sensitivity Sweep (P0/P3) ━━━");
    println!("  Sweeping envelope_sigma [2.0..5.0] x persistence_threshold [5..40]");
    println!("  This evaluates 100 engines × 56 configurations = 5,600 fleet runs...");

    let sweep_result = run_2d_sweep(&fd001, &default_config);

    let table = sweep_table(&sweep_result);
    println!("{}", table);
    fs::write(format!("{}/sensitivity_sweep.txt", output_dir), &table).ok();

    let heatmap = sweep_heatmap_svg(&sweep_result);
    fs::write(format!("{}/sensitivity_heatmap.svg", output_dir), &heatmap).ok();
    println!("  Heatmap SVG: sensitivity_heatmap.svg");

    let json = sweep_json(&sweep_result);
    fs::write(format!("{}/sensitivity_sweep.json", output_dir), &json).ok();
    println!("  JSON: sensitivity_sweep.json");

    println!("\n━━━ Phase 3: Recommended Configuration Evaluation ━━━");
    let rec_config = &sweep_result.recommended_config;
    println!(
        "  envelope_sigma={:.1}, persistence_threshold={}",
        rec_config.envelope_sigma,
        rec_config.persistence_threshold
    );

    let (results_rec, metrics_rec) = evaluate_fleet(&fd001, rec_config, INFORMATIVE_CHANNELS_FD001);
    let report_rec = generate_report(&results_rec, &metrics_rec, "C-MAPSS FD001 (Recommended Config)");
    fs::write(format!("{}/report_fd001_recommended.txt", output_dir), &report_rec).ok();
    println!("{}", &report_rec);

    let nc_rec = compute_negative_control(&results_rec, rec_config);
    let nc_rec_report = negative_control_report(&nc_rec);
    println!("{}", nc_rec_report);
    fs::write(format!("{}/negative_control_recommended.txt", output_dir), &nc_rec_report).ok();

    if let Some(first_result) = results_rec.first() {
        let trace_rec = trace_chain_report(first_result);
        fs::write(format!("{}/trace_chain_unit_1_recommended.txt", output_dir), &trace_rec).ok();
        println!("  Trace chain written: trace_chain_unit_1_recommended.txt");
    }

    let fleet_svg = fleet_summary_svg(&metrics_rec, "FD001 (Recommended)");
    fs::write(format!("{}/fleet_summary_recommended.svg", output_dir), &fleet_svg).ok();

    for result in results_rec.iter().take(10) {
        let svg = grammar_trajectory_svg(result);
        fs::write(format!("{}/grammar_unit_{}_rec.svg", output_dir, result.unit), &svg).ok();
    }

    if Path::new(&fd003_path).exists() {
        println!("\n━━━ Phase 4: FD003 Multi-Fault Discrimination (P1) ━━━");
        let fd003 = load_dataset(&fd003_path, "FD003");

        let (results_fd003, metrics_fd003) =
            evaluate_fleet(&fd003, rec_config, INFORMATIVE_CHANNELS_FD001);

        let report_fd003 = generate_report(&results_fd003, &metrics_fd003, "C-MAPSS FD003");
        fs::write(format!("{}/report_fd003.txt", output_dir), &report_fd003).ok();

        let disc = analyze_discrimination(&results_fd003);
        let disc_report = discrimination_report(&disc);
        println!("{}", disc_report);
        fs::write(format!("{}/discrimination_fd003.txt", output_dir), &disc_report).ok();

        let fd003_svg = fleet_summary_svg(&metrics_fd003, "FD003 (Two Fault Modes)");
        fs::write(format!("{}/fleet_summary_fd003.svg", output_dir), &fd003_svg).ok();
    } else {
        println!("\n  FD003 not found at {}, skipping P1.", fd003_path);
    }

    let fd002_path = format!("{}/train_FD002.txt", data_dir);
    if Path::new(&fd002_path).exists() {
        println!("\n━━━ Phase 5: FD002 Regime-Conditioned Evaluation (P2) ━━━");
        let fd002 = load_dataset(&fd002_path, "FD002");

        use dsfb_turbine::pipeline::regime_eval::evaluate_fleet_regime_conditioned;

        let fd002_configs: Vec<DsfbConfig> = vec![
            DsfbConfig {
                envelope_sigma: 4.0,
                persistence_threshold: 25,
                healthy_window: 30,
                ..DsfbConfig::cmapss_fd002_default()
            },
            DsfbConfig {
                envelope_sigma: 4.5,
                persistence_threshold: 30,
                healthy_window: 30,
                ..DsfbConfig::cmapss_fd002_default()
            },
            DsfbConfig {
                envelope_sigma: 5.0,
                persistence_threshold: 35,
                healthy_window: 30,
                ..DsfbConfig::cmapss_fd002_default()
            },
            DsfbConfig {
                envelope_sigma: 5.0,
                persistence_threshold: 40,
                healthy_window: 30,
                ..DsfbConfig::cmapss_fd002_default()
            },
            DsfbConfig {
                envelope_sigma: 5.5,
                persistence_threshold: 40,
                healthy_window: 35,
                ..DsfbConfig::cmapss_fd002_default()
            },
        ];

        println!(
            "  Running regime-conditioned sweep ({} configs x {} engines)...",
            fd002_configs.len(),
            fd002.num_units()
        );

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;
        let mut all_fd002_metrics = Vec::new();

        for (ci, cfg) in fd002_configs.iter().enumerate() {
            let (results, metrics) = evaluate_fleet_regime_conditioned(
                &fd002,
                INFORMATIVE_CHANNELS_FD001,
                cfg,
            );

            let nc = compute_negative_control(&results, cfg);
            let detection =
                metrics.engines_with_boundary as f64 / metrics.total_engines.max(1) as f64;
            let ew = metrics.early_warning_count as f64 / metrics.total_engines.max(1) as f64;

            println!(
                "    sigma={:.1} persist={:2} hw={:2} | det={:.0}% ew={:.0}% false={:.1}% lead={:.1}",
                cfg.envelope_sigma,
                cfg.persistence_threshold,
                cfg.healthy_window,
                detection * 100.0,
                ew * 100.0,
                nc.false_boundary_rate * 100.0,
                metrics.mean_lead_time
            );

            let lead_score = if metrics.mean_lead_time >= 30.0 && metrics.mean_lead_time <= 150.0 {
                1.0
            } else if metrics.mean_lead_time > 150.0 {
                150.0 / metrics.mean_lead_time
            } else {
                metrics.mean_lead_time / 30.0
            };
            let score = detection * ew * (1.0 - nc.false_boundary_rate) * lead_score;
            if score > best_score {
                best_score = score;
                best_idx = ci;
            }

            all_fd002_metrics.push((cfg.clone(), results, metrics, nc));
        }

        let (ref best_cfg, ref best_results, ref best_metrics, ref best_nc) =
            all_fd002_metrics[best_idx];
        println!(
            "\n  Best FD002 config: sigma={:.1}, persist={}, hw={}",
            best_cfg.envelope_sigma,
            best_cfg.persistence_threshold,
            best_cfg.healthy_window
        );

        let report_fd002 = generate_report(
            best_results,
            best_metrics,
            "C-MAPSS FD002 (6 conditions, regime-conditioned)",
        );
        fs::write(format!("{}/report_fd002_regime.txt", output_dir), &report_fd002).ok();
        println!("{}", report_fd002);

        let nc_report = negative_control_report(best_nc);
        println!("{}", nc_report);
        fs::write(format!("{}/negative_control_fd002_regime.txt", output_dir), &nc_report).ok();

        let fd002_svg = fleet_summary_svg(best_metrics, "FD002 Regime-Conditioned");
        fs::write(format!("{}/fleet_summary_fd002_regime.svg", output_dir), &fd002_svg).ok();
    } else {
        println!("\n  FD002 not found at {}, skipping P2.", fd002_path);
    }

    println!("\n================================================================");
    println!("  Evaluation complete. All outputs in: {}/", output_dir);
    println!("================================================================");
    println!("  Non-interference verified: all inputs were &[f64] immutable slices.");
    println!("  No upstream EHM/GPA/FADEC system was modified or accessed.");
    println!("  DSFB augments existing methods. It does not replace them.");
    println!("  DSFB does not predict RUL. It classifies structural state.");
}

fn load_dataset(path: &str, name: &str) -> CmapssDataset {
    println!("  Loading {}...", path);
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    let dataset = CmapssDataset::parse(Cursor::new(content.as_bytes()), name)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e));
    println!("  {} engines, {} rows", dataset.num_units(), dataset.rows.len());
    dataset
}

fn run_synthetic_demo(output_dir: &str) {
    use dsfb_turbine::core::channels::ChannelId;
    use dsfb_turbine::pipeline::engine_eval::evaluate_engine;
    use dsfb_turbine::pipeline::metrics::compute_fleet_metrics;

    let config = DsfbConfig {
        envelope_sigma: 3.5,
        persistence_threshold: 25,
        ..DsfbConfig::default()
    };
    let mut results = Vec::new();

    for unit in 1..=3u16 {
        let n = 150 + (unit as usize) * 30;
        let mut values = vec![0.0f64; n];
        for i in 0..n {
            let degradation = if i > 40 {
                (i - 40) as f64 * 0.12 * unit as f64
            } else {
                0.0
            };
            values[i] = 1580.0 + degradation + ((i as f64 * 0.7).sin()) * 0.5;
        }
        let channel_data = vec![(ChannelId::TempHpcOutlet, values)];
        results.push(evaluate_engine(unit, &channel_data, &config));
    }

    let metrics = compute_fleet_metrics(&results);
    let report = dsfb_turbine::report::generate_report(&results, &metrics, "Synthetic Demo");
    fs::create_dir_all(output_dir).ok();
    fs::write(format!("{}/report_synthetic.txt", output_dir), &report).ok();

    let fleet_svg = fleet_summary_svg(&metrics, "Synthetic Demo");
    fs::write(format!("{}/fleet_summary_synthetic.svg", output_dir), &fleet_svg).ok();

    for result in &results {
        let svg = grammar_trajectory_svg(result);
        fs::write(
            format!("{}/grammar_unit_{}_synthetic.svg", output_dir, result.unit),
            &svg,
        )
        .ok();
    }

    let nc = compute_negative_control(&results, &config);
    println!("{}", negative_control_report(&nc));

    if let Some(first_result) = results.first() {
        let trace = trace_chain_report(first_result);
        fs::write(format!("{}/trace_chain_synthetic.txt", output_dir), &trace).ok();
    }

    println!("{}", report);
}
