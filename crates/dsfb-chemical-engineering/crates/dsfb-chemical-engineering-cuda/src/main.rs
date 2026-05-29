//! `dsfb-chem-cuda` — CUDA evidence factory + forensic court entry point.
//!
//! Parses the first positional argument as a command name and dispatches to the appropriate
//! function in [`dsfb_chemical_engineering_cuda::cli`]. All commands return an integer exit code
//! (0 = success, 1 = hard failure, 2 = evidence produced but verification failed).
//!
//! Usage:
//!   dsfb-chem-cuda demo            forensic court over all datasets -> case files + replay + zip
//!   dsfb-chem-cuda bench           GB/s throughput benchmarks (multiple runs) -> reports/
//!   dsfb-chem-cuda verify-replay   byte-exact evidence-root replay over all datasets
//!   dsfb-chem-cuda device          print backend + device name
//!   dsfb-chem-cuda profile         single kernel launch for clean Nsight capture
//!   dsfb-chem-cuda atlas           print the atlas authority summary + frozen atlas_hash_v1
//!   dsfb-chem-cuda corpus          print the soft-sensor data-corpus authority summary
//!
//! The crate directory resolves from $DSFB_CHEM_CUDA_DIR (for runtime flexibility) or falls back
//! to the compile-time CARGO_MANIFEST_DIR (suitable for `cargo run`).

use dsfb_chemical_engineering_cuda::cli;
use std::path::PathBuf;

/// Determine the directory that anchors dataset discovery and output paths.
///
/// Allows the binary to be run from any working directory by overriding the path with
/// `DSFB_CHEM_CUDA_DIR`. Without the override, `env!("CARGO_MANIFEST_DIR")` points to the
/// crate root at compile time.
fn crate_dir() -> PathBuf {
    if let Ok(d) = std::env::var("DSFB_CHEM_CUDA_DIR") {
        return PathBuf::from(d);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Default to "demo" if no command is provided.
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("demo");
    let dir = crate_dir();
    let code = match cmd {
        "demo" => cli::run_demo(&dir),
        "bench" => cli::run_bench(&dir),
        "profile" => cli::run_profile(),
        "profile-batched" => cli::run_profile_batched(),
        "profile-v2-segmented" => cli::run_profile_v2_segmented(),
        "verify-replay" => cli::run_verify_replay(&dir),
        "device" => cli::run_device(),
        "atlas" => cli::run_atlas(),
        "corpus" => cli::run_corpus(),
        "-h" | "--help" | "help" => {
            println!("commands: demo | bench | verify-replay | device | profile | profile-batched | profile-v2-segmented | atlas | corpus");
            0
        }
        other => {
            eprintln!("unknown command '{other}'. try: demo | bench | verify-replay | device | profile | profile-batched | profile-v2-segmented | atlas | corpus");
            1
        }
    };
    std::process::exit(code);
}
