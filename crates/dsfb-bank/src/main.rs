use std::process;

use clap::Parser;
use dsfb_bank::{cli::Cli, execute};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("dsfb-bank failed: {error}");
        process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(run_dir) = execute(&cli)? {
        println!("{}", run_dir.display());
    }
    Ok(())
}
