//! CLI entry point for `dsfb-gpu-debug-demo`.
//!
//! Subcommands are wired one at a time so each section's tests can run the
//! exact commands documented in the README. As of Section B,
//! `generate-fixture` is live; the remaining commands (`run-cpu`,
//! `run-gpu`, `compare`, `validate-contract`) are added by their
//! respective sections.
//!
//! Exit codes (documented for downstream automation and the breach matrix):
//!
//! | code | meaning                                          |
//! |------|--------------------------------------------------|
//! |   0  | success                                          |
//! |   1  | usage error / unknown subcommand                 |
//! |   2  | GPU unavailable (built without `cuda` feature)   |
//! |   3  | hash-chain mismatch detected by `compare`        |
//! |   4  | semantic-bypass attempt rejected                 |
//! |   5  | I/O failure                                      |

// Tests inside CLI submodules use `.unwrap()` for parse-result assertions;
// the lint is too noisy if applied uniformly to test code.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

// `cli` is now exposed via the crate's library target (src/lib.rs)
// so integration tests under `tests/` can reuse the ingest +
// audit-report modules. The bin consumes the same surface via
// the crate import below.
use dsfb_gpu_debug_demo::cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let mut args = argv.iter().skip(1).cloned();
    let Some(subcommand) = args.next() else {
        print_usage();
        return ExitCode::from(1);
    };
    let rest: Vec<String> = args.collect();

    match subcommand.as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "version" | "--version" | "-V" => {
            println!("dsfb-gpu-debug {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "generate-fixture" => cli::generate_fixture::parse_and_run(&rest),
        "run-cpu" => cli::run_cpu::parse_and_run(&rest),
        "run-gpu" => cli::run_gpu::parse_and_run(&rest),
        "compare" => cli::compare::parse_and_run(&rest),
        "bench" => cli::bench::parse_and_run(&rest),
        "bench-gpu-scale" => cli::bench_gpu_scale::parse_and_run(&rest),
        // The S-REAL audit driver: `s-real-audit` is the canonical
        // name; `s-real-1-audit` is the historical alias from the
        // original 3-dataset S-REAL.1 seal and remains accepted so
        // every committed bundle artifact (perf_profile.txt prose,
        // bundle_manifest.toml comment) continues to invoke a real
        // subcommand without forcing artifact-byte regeneration.
        // The handler now spans all 20 sealed S-REAL.1/2/3 fixtures.
        "s-real-audit" | "s-real-1-audit" => cli::s_real_audit::parse_and_run(&rest),
        other => {
            eprintln!("dsfb-gpu-debug: unknown subcommand '{other}'");
            print_usage();
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    eprintln!(
        "dsfb-gpu-debug {ver}\n\
         \n\
         Subcommands:\n\
           generate-fixture   produce the canonical synthetic trace fixture\n\
           run-cpu            run the CPU reference pipeline                       (section G)\n\
           run-gpu            run the CUDA-accelerated pipeline                    (section H)\n\
           compare            compare two case files and emit a verdict           (section G)\n\
           validate-contract  recompute input/bank/registry hashes inside contract (section G)\n\
           bench              measure CPU/GPU pipeline wall-clock (section O onward)\n\
           bench-gpu-scale    R.7 headline money-table benchmark + reports/money_table.txt\n\
           s-real-audit       S-REAL real-dataset audit gauntlet (20 sealed fixtures across S-REAL.1/2/3)\n\
                              `s-real-1-audit` accepted as historical alias\n",
        ver = env!("CARGO_PKG_VERSION")
    );
}
