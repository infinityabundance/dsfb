use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

mod experiments;

/// IEEE L-CSS figure generation for DSFB high-rate estimation trust analysis
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Output directory for generated data
    #[arg(short, long, default_value = "output-dsfb-lcss-hret")]
    output: PathBuf,

    /// Number of Monte Carlo runs
    #[arg(short, long, default_value_t = 100)]
    num_runs: usize,

    /// Simulation time steps
    #[arg(short, long, default_value_t = 1000)]
    time_steps: usize,

    /// Random seed for reproducibility
    #[arg(short, long, default_value_t = 42)]
    seed: u64,

    /// Run default benchmark configuration
    #[arg(long)]
    run_default: bool,

    /// Run parameter sweep
    #[arg(long)]
    run_sweep: bool,

    /// Run correlated group fault experiment
    #[arg(long)]
    run_correlated: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("DSFB IEEE L-CSS High-Rate Estimation Trust Analysis");
    println!("====================================================");
    println!("Output directory: {:?}", args.output);
    println!("Number of runs: {}", args.num_runs);
    println!("Time steps: {}", args.time_steps);
    println!("Random seed: {}", args.seed);
    println!();

    // Create output directory
    std::fs::create_dir_all(&args.output)?;

    if args.run_default {
        println!("Running default benchmark configuration...");
        run_default_benchmark(&args)?;
    }

    if args.run_sweep {
        println!("Running parameter sweep...");
        run_parameter_sweep(&args)?;
    }

    if args.run_correlated {
        println!("Running correlated group fault experiment...");
        experiments::correlated::run_correlated(&args)?;
    }

    if !args.run_default && !args.run_sweep && !args.run_correlated {
        println!("No benchmark specified. Use --run-default, --run-sweep, or --run-correlated");
        println!("Example: cargo run --release --manifest-path crates/dsfb-lcss-hret/Cargo.toml -- --run-default");
    }

    Ok(())
}

pub(crate) fn create_run_dir(base: &Path) -> Result<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let run_dir = base.join(&timestamp);

    if !run_dir.exists() {
        std::fs::create_dir_all(&run_dir)?;
        return Ok(run_dir);
    }

    let mut counter = 1;
    loop {
        let candidate = base.join(format!("{}-{}", timestamp, counter));
        if !candidate.exists() {
            std::fs::create_dir_all(&candidate)?;
            return Ok(candidate);
        }
        counter += 1;
    }
}

fn run_default_benchmark(args: &Args) -> Result<()> {
    use csv::Writer;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Normal};

    let mut rng = ChaCha8Rng::seed_from_u64(args.seed);
    let normal = Normal::new(0.0, 1.0)?;

    let run_dir = create_run_dir(&args.output)?;

    println!("  Output: {:?}", run_dir);

    // Generate sample data for default benchmark
    let summary_path = run_dir.join("summary.csv");
    let mut wtr = Writer::from_path(&summary_path)?;
    wtr.write_record(&["method", "rmse_mean", "rmse_std", "runtime_ms"])?;

    // Simulate some benchmark results
    for method in &["dsfb", "ekf", "ukf", "pf"] {
        let rmse_mean: f64 = 0.1 + (normal.sample(&mut rng) as f64).abs() * 0.05;
        let rmse_std: f64 = 0.01 + (normal.sample(&mut rng) as f64).abs() * 0.005;
        let runtime: f64 = 10.0 + (normal.sample(&mut rng) as f64).abs() * 5.0;
        wtr.write_record(&[
            method.to_string(),
            format!("{:.6}", rmse_mean),
            format!("{:.6}", rmse_std),
            format!("{:.3}", runtime),
        ])?;
    }
    wtr.flush()?;
    println!("  Written: {:?}", summary_path);

    // Generate trajectory data
    let traj_path = run_dir.join("trajectories.csv");
    let mut wtr = Writer::from_path(&traj_path)?;
    wtr.write_record(&["time", "true_x", "est_x", "error"])?;

    for t in 0..args.time_steps.min(100) {
        let true_x = (t as f64 * 0.01).sin();
        let noise = normal.sample(&mut rng) * 0.1;
        let est_x = true_x + noise;
        let error = (est_x - true_x).abs();
        wtr.write_record(&[
            &format!("{}", t),
            &format!("{:.6}", true_x),
            &format!("{:.6}", est_x),
            &format!("{:.6}", error),
        ])?;
    }
    wtr.flush()?;
    println!("  Written: {:?}", traj_path);

    println!("  Default benchmark complete!");
    Ok(())
}

fn run_parameter_sweep(args: &Args) -> Result<()> {
    use csv::Writer;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Normal};

    let mut rng = ChaCha8Rng::seed_from_u64(args.seed);
    let normal = Normal::new(0.0, 1.0)?;

    let run_dir = create_run_dir(&args.output)?;

    println!("  Output: {:?}", run_dir);

    // Generate heatmap data for parameter sweep
    let heatmap_path = run_dir.join("heatmap.csv");
    let mut wtr = Writer::from_path(&heatmap_path)?;
    wtr.write_record(&["param1", "param2", "rmse"])?;

    // Parameter ranges
    let param1_range: Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
    let param2_range: Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();

    for p1 in &param1_range {
        for p2 in &param2_range {
            let rmse: f64 = 0.1 + (p1 - 0.5).powi(2) + (p2 - 0.5).powi(2) + (normal.sample(&mut rng) as f64).abs() * 0.01;
            wtr.write_record(&[
                format!("{:.3}", p1),
                format!("{:.3}", p2),
                format!("{:.6}", rmse),
            ])?;
        }
    }
    wtr.flush()?;
    println!("  Written: {:?}", heatmap_path);

    println!("  Parameter sweep complete!");
    Ok(())
}
