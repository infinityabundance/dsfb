//! `paper-lock` — headline-metric enforcement binary for the
//! `dsfb-robotics` companion paper.
//!
//! ## Usage
//!
//! ```text
//! paper-lock <dataset> [--fixture]
//! paper-lock --help
//! paper-lock --list
//!
//! <dataset>:
//!   cwru | ims | cmapss | kuka_lwr | femto_st |
//!   panda_gaz | dlr_justin | ur10_kufieta |
//!   cheetah3 | icub_pushrecovery
//!
//! --fixture  Run the in-crate micro-fixture (deterministic smoke test).
//!            Does NOT claim empirical results. Use real-data mode for
//!            the companion paper's §10 numbers.
//! --list     Print the set of supported dataset slugs (one per line).
//! --help     Print this usage text.
//! ```
//!
//! ## Exit codes
//!
//! - `0`  — report emitted successfully to stdout (JSON).
//! - `64` (EX_USAGE) — malformed arguments, unknown dataset slug, **or
//!   real-data corpus absent** at the expected path. The latter case
//!   is deliberate: paper-lock never silently substitutes a synthetic
//!   fixture. See `docs/<dataset>_oracle_protocol.md` for the fetch
//!   instructions.
//!
//! ## Determinism
//!
//! Three consecutive invocations with the same arguments produce
//! byte-identical stdout output. Verified by the
//! `paper_lock::tests::serialized_report_is_byte_identical_across_runs`
//! unit test.

use std::process::ExitCode;

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::{self, build_explain, serialize_report, PaperLockReport};

/// Per-invocation options assembled from CLI flags.
#[derive(Debug, Clone)]
struct Options {
    fixture: bool,
    emit_episodes: bool,
    /// Override for the residual CSV path. Used by the bootstrap and
    /// sensitivity-grid scripts to feed resampled / parameter-swept
    /// streams into the same Rust engine without modifying
    /// `data/processed/<slug>.csv`.
    csv_path: Option<String>,
    /// Emit a per-Boundary/Violation review-log CSV alongside the
    /// JSON aggregate. Operator-facing triage artefact: one row per
    /// non-Admissible episode with `index, residual_norm, drift,
    /// grammar, reason_code` columns.
    emit_review_csv: bool,
    /// Override for the review-CSV output path. Defaults to
    /// `<slug>_review_log.csv` in the cwd.
    review_csv_path: Option<String>,
    /// Add an `explain[]` array to the JSON output: per-episode
    /// reason codes and human-readable narratives explaining why each
    /// non-Admissible episode fired.
    explain: bool,
}

const EX_OK: u8 = 0;
const EX_USAGE: u8 = 64;

fn main() -> ExitCode {
    // ITER-UNB: std::env::args() is bounded externally by the OS argv
    // limit (ARG_MAX, typically 128–2048 KiB on POSIX), but we apply an
    // explicit `.take(256)` here to make the bound visible at the call
    // site for static review. The debug_assert below additionally
    // catches deeply-pathological inputs in test builds.
    let argv: Vec<String> = std::env::args().take(256).collect();
    debug_assert!(!argv.is_empty(), "argv[0] must exist");
    debug_assert!(argv.len() <= 256, "argv bounded by .take(256) above");
    match parse_args(&argv[1..]) {
        ParsedArgs::ShowHelp => {
            print!("{}", usage_text());
            ExitCode::from(EX_OK)
        }
        ParsedArgs::ShowList => {
            for slug in SUPPORTED_SLUGS {
                println!("{slug}");
            }
            ExitCode::from(EX_OK)
        }
        ParsedArgs::Run { id, options } => {
            let mode = if options.fixture { Mode::Fixture } else { Mode::RealData };
            match run_and_emit(id, mode, options) {
                Ok(()) => ExitCode::from(EX_OK),
                Err(code) => ExitCode::from(code),
            }
        }
        ParsedArgs::Invalid(msg) => {
            eprintln!("paper-lock: {msg}");
            eprint!("{}", usage_text());
            ExitCode::from(EX_USAGE)
        }
    }
}

/// Canonical slug list, in paper §10 order.
const SUPPORTED_SLUGS: &[&str] = &[
    "cwru",
    "ims",
    "kuka_lwr",
    "femto_st",
    "panda_gaz",
    "dlr_justin",
    "ur10_kufieta",
    "cheetah3",
    "icub_pushrecovery",
    "droid",
    "openx",
    "anymal_parkour",
    "unitree_g1",
    "aloha_static",
    "icub3_sorrentino",
    "mobile_aloha",
    "so100",
    "aloha_static_tape",
    "aloha_static_screw_driver",
    "aloha_static_pingpong_test",
];

enum Mode {
    Fixture,
    RealData,
}

enum ParsedArgs {
    Run { id: DatasetId, options: Options },
    ShowHelp,
    ShowList,
    Invalid(String),
}

/// Mutable state collected while walking argv. Extracted so
/// `parse_args` stays under the NASA SWE-220 cyclomatic-complexity
/// threshold of 15.
struct ArgWalker<'a> {
    positional: Option<&'a str>,
    fixture_mode: bool,
    emit_episodes: bool,
    csv_path: Option<String>,
    emit_review_csv: bool,
    review_csv_path: Option<String>,
    explain: bool,
}

impl<'a> ArgWalker<'a> {
    fn new() -> Self {
        Self {
            positional: None,
            fixture_mode: false,
            emit_episodes: false,
            csv_path: None,
            emit_review_csv: false,
            review_csv_path: None,
            explain: false,
        }
    }

    /// Consume one token (and its arg if it's a flag-with-value);
    /// return Err(ParsedArgs::Invalid) on malformed input. Dispatches
    /// to small helpers so cyclomatic complexity stays under the NASA
    /// SWE-220 threshold of 15.
    fn step(&mut self, args: &'a [String], i: &mut usize) -> Result<(), ParsedArgs> {
        let a = &args[*i];
        if let Some(()) = self.step_boolean_flag(a) {
            return Ok(());
        }
        if let Some(result) = self.step_value_flag(a, args, i) {
            return result;
        }
        self.step_positional_or_unknown(a)
    }

    fn step_boolean_flag(&mut self, a: &str) -> Option<()> {
        // SAFE-STATE: the explicitly-named `unhandled` arm is the
        // documented fallback when `a` is not one of the four known
        // boolean flags. Returning `None` lets the caller fall through
        // to the value-flag matcher and then the positional/unknown
        // matcher; there is no silent acceptance.
        match a {
            "--fixture" => self.fixture_mode = true,
            "--emit-episodes" => self.emit_episodes = true,
            "--emit-review-csv" => self.emit_review_csv = true,
            "--explain" => self.explain = true,
            unhandled => {
                debug_assert!(!unhandled.is_empty(), "argv tokens are non-empty");
                return None;
            }
        }
        Some(())
    }

    fn step_value_flag(
        &mut self,
        a: &str,
        args: &'a [String],
        i: &mut usize,
    ) -> Option<Result<(), ParsedArgs>> {
        match a {
            "--csv-path" => Some(
                self.consume_value("--csv-path", args, i, |w, v| w.csv_path = Some(v)),
            ),
            "--review-csv-path" => Some(self.consume_value(
                "--review-csv-path",
                args,
                i,
                |w, v| {
                    w.review_csv_path = Some(v);
                    w.emit_review_csv = true;
                },
            )),
            // SAFE-STATE: explicitly-named `unhandled` is the documented
            // fallback when `a` is not one of the two known value-flags.
            // None signals "not a value flag" to the caller — there is
            // no silent acceptance path.
            unhandled => {
                debug_assert!(!unhandled.is_empty(), "argv tokens are non-empty");
                None
            }
        }
    }

    fn step_positional_or_unknown(&mut self, a: &'a str) -> Result<(), ParsedArgs> {
        if a.starts_with("--") {
            return Err(ParsedArgs::Invalid(alloc_format(&["unknown flag: ", a])));
        }
        if self.positional.is_some() {
            return Err(ParsedArgs::Invalid(alloc_format(&[
                "unexpected positional argument: ",
                a,
                "; exactly one dataset slug is accepted",
            ])));
        }
        self.positional = Some(a);
        Ok(())
    }

    fn consume_value<F>(
        &mut self,
        name: &str,
        args: &'a [String],
        i: &mut usize,
        apply: F,
    ) -> Result<(), ParsedArgs>
    where
        F: FnOnce(&mut Self, String),
    {
        if *i + 1 >= args.len() {
            return Err(ParsedArgs::Invalid(alloc_format(&[
                name,
                " requires a value argument",
            ])));
        }
        *i += 1;
        apply(self, args[*i].clone());
        Ok(())
    }
}

fn parse_args(args: &[String]) -> ParsedArgs {
    debug_assert!(args.len() <= 256, "argv unreasonably long");
    if args.is_empty() {
        return ParsedArgs::ShowHelp;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return ParsedArgs::ShowHelp;
    }
    if args.iter().any(|a| a == "--list") {
        return ParsedArgs::ShowList;
    }
    let mut walker = ArgWalker::new();
    let mut i = 0_usize;
    while i < args.len() {
        if let Err(err) = walker.step(args, &mut i) {
            return err;
        }
        i += 1;
    }
    let Some(slug) = walker.positional else {
        return ParsedArgs::Invalid("missing dataset slug".to_string());
    };
    let Some(id) = DatasetId::from_slug(slug) else {
        return ParsedArgs::Invalid(alloc_format(&[
            "unknown dataset: ",
            slug,
            "; run `paper-lock --list` for supported slugs",
        ]));
    };
    ParsedArgs::Run {
        id,
        options: Options {
            fixture: walker.fixture_mode,
            emit_episodes: walker.emit_episodes,
            csv_path: walker.csv_path,
            emit_review_csv: walker.emit_review_csv,
            review_csv_path: walker.review_csv_path,
            explain: walker.explain,
        },
    }
}

fn run_and_emit(id: DatasetId, mode: Mode, options: Options) -> Result<(), u8> {
    debug_assert!(!id.slug().is_empty(), "dataset slug must be non-empty");
    debug_assert!(matches!(mode, Mode::Fixture | Mode::RealData));
    match mode {
        Mode::Fixture => run_fixture_path(id, &options),
        Mode::RealData => run_real_data_path(id, &options),
    }
}

fn run_fixture_path(id: DatasetId, options: &Options) -> Result<(), u8> {
    let report = if options.emit_episodes {
        paper_lock::run_fixture_with_trace(id)
    } else {
        paper_lock::run_fixture(id)
    };
    emit_fixture_banner(&report);
    emit_report(&report).map_err(|_| EX_USAGE)
}

fn run_real_data_path(id: DatasetId, options: &Options) -> Result<(), u8> {
    // Either --explain or --emit-review-csv requires the per-episode
    // trace; force trace generation when either operator-facing flag
    // is on.
    let need_trace =
        options.emit_episodes || options.explain || options.emit_review_csv;
    let result = match options.csv_path.as_ref() {
        Some(p) => paper_lock::run_real_data_with_csv_path(id, need_trace, std::path::Path::new(p)),
        None => paper_lock::run_real_data_with_trace(id, need_trace),
    };
    let mut report = match result {
        Ok(r) => r,
        Err(unavailable) => return real_data_unavailable(unavailable),
    };
    if options.emit_review_csv {
        write_review_csv(&report, options)?;
    }
    if options.explain {
        report.explain = Some(build_explain(&report));
    }
    // Drop the bulky trace from the emitted JSON unless the caller
    // explicitly asked for it via --emit-episodes. The explain[] array
    // (if any) is preserved.
    if !options.emit_episodes {
        report.trace = None;
    }
    emit_report(&report).map_err(|_| EX_USAGE)
}

fn write_review_csv(report: &PaperLockReport, options: &Options) -> Result<(), u8> {
    let path = options
        .review_csv_path
        .clone()
        .unwrap_or_else(|| format!("{}_review_log.csv", report.dataset));
    let rows =
        paper_lock::emit_review_csv(report, std::path::Path::new(&path)).map_err(|e| {
            eprintln!("paper-lock: failed to write review CSV: {e}");
            EX_USAGE
        })?;
    eprintln!("paper-lock: wrote {rows} review-log rows to {path}");
    Ok(())
}

fn real_data_unavailable(unavailable: paper_lock::RealDataUnavailable) -> Result<(), u8> {
    eprintln!(
        "paper-lock: real dataset not found for {slug}.\n  expected at: {path}\n  {instructions}",
        slug = unavailable.dataset.slug(),
        path = unavailable.expected_path,
        instructions = unavailable.instructions
    );
    Err(EX_USAGE)
}

fn emit_fixture_banner(report: &PaperLockReport) {
    debug_assert!(!report.dataset.is_empty(), "banner requires non-empty dataset slug");
    debug_assert_eq!(report.mode, "fixture-smoke-test", "banner only applies to fixture mode");
    eprintln!(
        "paper-lock: FIXTURE MODE — results for {dataset} reflect the in-crate \
         micro-fixture, NOT empirical data from the published dataset. Use \
         real-data mode for the companion paper's §10 numbers.",
        dataset = report.dataset
    );
}

fn emit_report(report: &PaperLockReport) -> Result<(), SerializationError> {
    debug_assert!(!report.dataset.is_empty(), "report dataset slug must be non-empty");
    debug_assert!(report.aggregate.total_samples <= usize::MAX / 2);
    let json = serialize_report(report).map_err(|_| SerializationError)?;
    // Using print! relies on stdout's panic-on-broken-pipe semantics —
    // appropriate for a CLI tool where a severed downstream pipe is a
    // hard failure rather than a silent discard.
    print!("{json}");
    Ok(())
}

struct SerializationError;

fn usage_text() -> &'static str {
    debug_assert!(SUPPORTED_SLUGS.len() >= 14, "expected 14+ supported slugs");
    "Usage: paper-lock <dataset> [--fixture] [--emit-episodes] [--explain] [--emit-review-csv [--review-csv-path PATH]] [--csv-path PATH]\n\
         \n\
         Supported datasets (14 real-world public benchmarks):\n\
           cwru                Case Western Reserve bearing (PHM)\n\
           ims                 NASA/IMS run-to-failure bearing (PHM)\n\
           kuka_lwr            KUKA LWR-IV+ ID, Simionato 7R (kinematics, link-side)\n\
           femto_st            FEMTO-ST PRONOSTIA, IEEE PHM 2012 (PHM)\n\
           panda_gaz           Franka Panda ID, Gaz 2019 (kinematics, motor-side)\n\
           dlr_justin          Panda 7-DoF, Giacomuzzo 2024 (DLR-class, link-side)\n\
           ur10_kufieta        UR10 pick-and-place, Polydoros 2015 (kinematics)\n\
           cheetah3            MIT Mini-Cheetah open logs (balancing)\n\
           icub_pushrecovery   ergoCub push-recovery, ami-iit (balancing)\n\
           droid               DROID 100-episode slice, Khazatsky 2024 (kinematics)\n\
           openx               Open X-Embodiment NYU ROT subset (kinematics)\n\
           anymal_parkour      ANYmal-C GrandTour outdoor locomotion (balancing)\n\
           unitree_g1          Unitree G1 humanoid teleoperation (balancing)\n\
           aloha_static        ALOHA bimanual static coffee, Zhao 2023 (kinematics)\n\
         \n\
         Flags:\n\
           --fixture            Run the in-crate micro-fixture (smoke test).\n\
           --emit-episodes      Include per-sample episode trace in the JSON.\n\
           --explain            Add an `explain[]` array to the JSON: one entry per\n\
                                Boundary/Violation episode with index, grammar,\n\
                                residual_norm_sq, drift, and a human-readable narrative.\n\
           --emit-review-csv    Emit an operator-facing review-log CSV alongside the\n\
                                JSON: one row per non-Admissible episode. Defaults to\n\
                                `<slug>_review_log.csv` in the cwd.\n\
           --review-csv-path P  Override the review-CSV output path (implies\n\
                                --emit-review-csv).\n\
           --csv-path P         Override the residual-stream CSV input path (used\n\
                                by the bootstrap and sensitivity-grid scripts).\n\
           --list               Print supported dataset slugs, one per line.\n\
           --help               Print this usage text.\n\
         \n\
         Exit codes: 0 success; 64 EX_USAGE on bad arguments or missing real data.\n"
}

/// Allocate a `String` by concatenating pieces. A tiny helper so we
/// don't pull in `format!` machinery for trivial concatenations.
fn alloc_format(parts: &[&str]) -> String {
    let len = parts.iter().map(|s| s.len()).sum();
    let mut s = String::with_capacity(len);
    for p in parts {
        s.push_str(p);
    }
    s
}
