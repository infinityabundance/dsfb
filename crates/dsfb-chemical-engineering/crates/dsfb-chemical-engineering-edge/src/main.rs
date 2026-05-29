//! `dsfb-chem-edge` — command-line entry point for the edge crate.
//!
//! Usage:
//!   dsfb-chem-edge demo                 run the full demo over all datasets → timestamped artifacts
//!   dsfb-chem-edge analyze <dataset>    analyse one dataset by name
//!   dsfb-chem-edge atlas                print the detector-atlas + atlas authority-crate summary
//!   dsfb-chem-edge corpus               print the soft-sensor data-corpus authority summary
//!   dsfb-chem-edge verify-replay        confirm deterministic replay on the synthetic suite
//!   dsfb-chem-edge completeness-court   machine-checked artifact-graph completeness + consistency gate
//!   dsfb-chem-edge generate-index       deterministic reports/index.{html,json} over the whole artifact graph
//!   dsfb-chem-edge verify-index         court that the committed reports/index.json still matches live artifacts
//!   dsfb-chem-edge narration-context <dataset>  emit the citable-anchor narration context for a Court Record
//!   dsfb-chem-edge densor-runtime-demo  (feature densor-runtime-demo) carry edge episodes through the runtime substrate
//!   dsfb-chem-edge narration-heuristic-demo  (feature narration-heuristic-demo) exhibit the H7–H12 narration-failure detector
//!   dsfb-chem-edge release-scrub        public-release hygiene gate (placeholder DOI / private-file leak / controlled rows)
//!   dsfb-chem-edge unit-consistency     unit/dimension court over every documented balance (°C↔K, bar↔Pa, …)
//!   dsfb-chem-edge data-readiness <csv> grade a real historian CSV before analysis (Ready / Caveats / NotReady)
//!   dsfb-chem-edge authority-diff <old.json> <new.json>   governed authority-drift receipts (semantic diff + compat)
//!   dsfb-chem-edge figures              render the 60-figure gallery + verbose log + deterministic ZIP
//!   dsfb-chem-edge regime-eval          regime-conditioned admissibility-envelope evaluation
//!   dsfb-chem-edge historian <csv>      replay a plant-historian CSV in batch-record mode
//!   dsfb-chem-edge balance-witness <n>  mass/energy-balance witness on an instrumented dataset
//!   dsfb-chem-edge control-action <n>   control-action context for a dataset
//!   dsfb-chem-edge casefile <dataset>   write the Chemical Court Record v1 evidence bundle
//!
//! The crate directory (for `data/slices/` and the corpus) resolves from `$DSFB_CHEM_DIR`, else the
//! compile-time manifest dir, else the current directory.

#![forbid(unsafe_code)]

use dsfb_chemical_engineering_edge::{artifact_index, cli};
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    if let Ok(d) = std::env::var("DSFB_CHEM_DIR") {
        return PathBuf::from(d);
    }
    let manifest = env!("CARGO_MANIFEST_DIR");
    let p = PathBuf::from(manifest);
    if p.is_dir() {
        p
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("demo");
    let dir = crate_dir();
    let code = match cmd {
        "demo" => cli::run_demo(&dir),
        "figures" => cli::run_figures(&dir),
        "atlas" => cli::run_atlas(),
        "corpus" => cli::run_corpus(),
        "verify-replay" => cli::run_verify_replay(),
        "completeness-court" => cli::run_completeness_court(&dir),
        "generate-index" => artifact_index::run_generate_index(&dir),
        "verify-index" => cli::run_verify_index(&dir),
        #[cfg(feature = "densor-runtime-demo")]
        "densor-runtime-demo" => {
            dsfb_chemical_engineering_edge::densor_demo::run_densor_runtime_demo(&dir)
        }
        #[cfg(not(feature = "densor-runtime-demo"))]
        "densor-runtime-demo" => {
            eprintln!("densor-runtime-demo: rebuild with --features densor-runtime-demo");
            1
        }
        #[cfg(feature = "narration-heuristic-demo")]
        "narration-heuristic-demo" => {
            dsfb_chemical_engineering_edge::narration_heuristic_demo::run()
        }
        #[cfg(not(feature = "narration-heuristic-demo"))]
        "narration-heuristic-demo" => {
            eprintln!("narration-heuristic-demo: rebuild with --features narration-heuristic-demo");
            1
        }
        "release-scrub" => match (args.get(2).map(|s| s.as_str()), args.get(3)) {
            (Some("--archive-dir"), Some(path)) => cli::run_release_scrub_archive(path),
            (None, _) => cli::run_release_scrub(&dir),
            _ => {
                eprintln!("usage: dsfb-chem-edge release-scrub [--archive-dir <release_dir>]");
                1
            }
        },
        "unit-consistency" => cli::run_unit_consistency(&dir),
        "data-readiness" => match args.get(2) {
            Some(csv) => cli::run_data_readiness(&dir, csv),
            None => {
                eprintln!("usage: dsfb-chem-edge data-readiness <historian.csv>");
                1
            }
        },
        "authority-diff" => match (args.get(2), args.get(3)) {
            (Some(old), Some(new)) => cli::run_authority_diff(old, new),
            _ => {
                eprintln!("usage: dsfb-chem-edge authority-diff <old.json> <new.json>");
                1
            }
        },
        "regime-eval" => cli::run_regime_eval(&dir),
        "historian" => match args.get(2) {
            Some(csv) => cli::run_historian(&dir, csv),
            None => {
                eprintln!("usage: dsfb-chem-edge historian <historian.csv>");
                1
            }
        },
        "balance-witness" => match args.get(2) {
            Some(name) => cli::run_balance_witness(&dir, name),
            None => {
                eprintln!("usage: dsfb-chem-edge balance-witness <instrumented-dataset-name>");
                1
            }
        },
        "control-action" => match args.get(2) {
            Some(name) => cli::run_control_action(&dir, name),
            None => {
                eprintln!("usage: dsfb-chem-edge control-action <dataset-name>");
                1
            }
        },
        "casefile" => match args.get(2) {
            Some(name) => cli::run_casefile(&dir, name),
            None => {
                eprintln!("usage: dsfb-chem-edge casefile <dataset-name>");
                1
            }
        },
        "analyze" => match args.get(2) {
            Some(name) => cli::run_analyze(&dir, name),
            None => {
                eprintln!("usage: dsfb-chem-edge analyze <dataset>");
                1
            }
        },
        "narration-context" => match args.get(2) {
            Some(name) => cli::run_narration_context(&dir, name),
            None => {
                eprintln!("usage: dsfb-chem-edge narration-context <dataset>");
                1
            }
        },
        "confidential-demo" => match (args.get(2).map(|s| s.as_str()), args.get(3)) {
            // Optional `--fixture <name>` (default: cavitation_instrumented — a gated synthetic incident
            // that yields a clean episode + playbook; any data/instrumented or data/historian fixture works).
            (Some("--fixture"), Some(name)) => cli::run_confidential_demo(&dir, name),
            (None, _) => cli::run_confidential_demo(&dir, "cavitation_instrumented"),
            _ => {
                eprintln!("usage: dsfb-chem-edge confidential-demo [--fixture <name>]");
                1
            }
        },
        "-h" | "--help" | "help" => {
            println!("commands: demo | figures | analyze <dataset> | atlas | corpus | verify-replay | completeness-court | generate-index | verify-index | release-scrub [--archive-dir <dir>] | unit-consistency | data-readiness <csv> | authority-diff <old.json> <new.json> | regime-eval | historian <csv> | balance-witness <name> | control-action <name> | casefile <name> | narration-context <name> | confidential-demo [--fixture <name>] | densor-runtime-demo (--features densor-runtime-demo) | narration-heuristic-demo (--features narration-heuristic-demo)");
            0
        }
        other => {
            eprintln!("unknown command '{other}'. try: demo | analyze <name> | atlas | corpus | verify-replay | regime-eval | historian <csv> | balance-witness <name> | control-action <name> | casefile <name>");
            1
        }
    };
    std::process::exit(code);
}
