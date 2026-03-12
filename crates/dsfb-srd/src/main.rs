use dsfb_srd::{run_simulation, SimulationConfig};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "{}",
            SimulationConfig::usage(
                "cargo run --manifest-path crates/dsfb-srd/Cargo.toml --release --bin dsfb-srd-generate --"
            )
        );
        return Ok(());
    }

    let config = SimulationConfig::from_args(args)
        .map_err(|message| -> Box<dyn std::error::Error> { message.into() })?;
    let generated_run = run_simulation(config)?;

    println!("run_id={}", generated_run.run_id);
    println!("config_hash={}", generated_run.config_hash);
    println!("timestamp={}", generated_run.timestamp);
    println!("output_dir={}", generated_run.output_dir.display());

    Ok(())
}
