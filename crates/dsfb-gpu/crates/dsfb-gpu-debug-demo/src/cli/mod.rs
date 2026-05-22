//! Hand-rolled CLI argument parsing.
//!
//! `clap` would add a multi-crate dependency tree. The demo binary only
//! takes a handful of flags per subcommand, so a small, declarative parser
//! keeps the surface area observable.
//!
//! Each subcommand module exposes `parse_and_run(args)` which receives the
//! tail of the argv (after the subcommand name) and returns an
//! `std::process::ExitCode`. The parser itself is `parse_flags`, which
//! walks `--key value` and `--key=value` forms and populates a tiny
//! HashMap.

use std::collections::BTreeMap;
use std::process::ExitCode;

pub mod audit_report;
pub mod bench;
pub mod bench_gpu_scale;
pub mod compare;
pub mod generate_fixture;
pub mod ingest;
pub mod run_cpu;
pub mod run_gpu;
pub mod s_real_audit;

/// Parse `--key value`, `--key=value`, and bare-`--key` flag forms into
/// a sorted map.
///
/// A flag with no value (either no following arg or the next arg starts
/// with `--`) is recorded with the literal value `"true"`. This is how
/// boolean toggles like `--detail` are surfaced to subcommands without
/// requiring the caller to type `--detail true`.
///
/// Sorted because deterministic flag handling makes the parsed structure
/// independent of argv order, which in turn makes error messages stable
/// across runs.
///
/// # Errors
///
/// Returns an error message string if a flag appears more than once or
/// a positional argument is encountered.
pub fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut flags: BTreeMap<String, String> = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(rest) = arg.strip_prefix("--") {
            let (key, value) = if let Some((k, v)) = rest.split_once('=') {
                (k.to_string(), v.to_string())
            } else {
                // Peek at the next argument. If it exists and is not
                // another `--flag`, consume it as the value. Otherwise
                // treat this as a boolean toggle with value `"true"`.
                let next_is_value = args.get(i + 1).is_some_and(|next| !next.starts_with("--"));
                if next_is_value {
                    let value = args[i + 1].clone();
                    i += 1;
                    (rest.to_string(), value)
                } else {
                    (rest.to_string(), "true".to_string())
                }
            };
            if flags.insert(key.clone(), value).is_some() {
                return Err(format!("flag --{key} given more than once"));
            }
            i += 1;
        } else {
            return Err(format!("unexpected positional argument: {arg}"));
        }
    }
    Ok(flags)
}

/// Look up a required flag; emit a helpful error if it's missing.
///
/// # Errors
///
/// Returns an error string if `name` is not present in `flags`.
pub fn require_flag<'a>(
    flags: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, String> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required flag --{name}"))
}

/// Print a usage error to stderr and return the standard usage exit code.
pub fn usage_error(message: &str) -> ExitCode {
    eprintln!("dsfb-gpu-debug: {message}");
    ExitCode::from(1)
}
