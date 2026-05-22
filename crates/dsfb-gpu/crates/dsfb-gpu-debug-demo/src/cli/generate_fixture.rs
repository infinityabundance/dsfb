//! `dsfb-gpu-debug generate-fixture` — synthesize the canonical 10 000-event
//! trace fixture and write it to disk in canonical-JSON form.
//!
//! Flags:
//!
//! * `--out PATH` (required): destination JSON path.
//! * `--seed HEX_OR_DEC` (optional, default `0xD5FBD5FBD5FBD5FB`): the LCG
//!   seed. Same seed always produces the same fixture bytes.
//!
//! Exit codes:
//! * 0 — wrote file successfully.
//! * 1 — usage error.
//! * 5 — I/O failure writing the output.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::serialize::write_fixture;

use super::{parse_flags, require_flag, usage_error};

/// Parse argv tail and run the generate-fixture command.
pub fn parse_and_run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args) {
        Ok(f) => f,
        Err(message) => return usage_error(&message),
    };

    let out_path = match require_flag(&flags, "out") {
        Ok(s) => s.to_string(),
        Err(message) => return usage_error(&message),
    };

    let seed = match flags.get("seed") {
        None => DEFAULT_SEED,
        Some(raw) => match parse_seed(raw) {
            Ok(s) => s,
            Err(message) => return usage_error(&message),
        },
    };

    let events = synthesize(seed);
    let bytes = write_fixture(&events);

    if let Some(parent) = Path::new(&out_path).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!(
                    "dsfb-gpu-debug: could not create {}: {error}",
                    parent.display()
                );
                return ExitCode::from(5);
            }
        }
    }

    if let Err(error) = fs::write(&out_path, &bytes) {
        eprintln!("dsfb-gpu-debug: failed to write {out_path}: {error}");
        return ExitCode::from(5);
    }

    eprintln!(
        "dsfb-gpu-debug: wrote {n} events ({bytes} canonical bytes) to {out_path}",
        n = events.len(),
        bytes = bytes.len()
    );
    ExitCode::SUCCESS
}

/// Parse a seed literal. Accepts plain decimal (e.g. `12345`) and the
/// `0x`/`0X` hexadecimal prefix (e.g. `0xD5FBD5FBD5FBD5FB`).
fn parse_seed(raw: &str) -> Result<u64, String> {
    if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| format!("invalid hex seed {raw}: {e}"))
    } else {
        raw.parse::<u64>()
            .map_err(|e| format!("invalid decimal seed {raw}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seed_accepts_decimal_and_hex() {
        assert_eq!(parse_seed("0").unwrap(), 0);
        assert_eq!(parse_seed("42").unwrap(), 42);
        assert_eq!(parse_seed("0xff").unwrap(), 0xff);
        assert_eq!(parse_seed("0XFF").unwrap(), 0xff);
        assert_eq!(
            parse_seed("0xD5FBD5FBD5FBD5FB").unwrap(),
            0xD5FB_D5FB_D5FB_D5FB
        );
    }

    #[test]
    fn parse_seed_rejects_garbage() {
        assert!(parse_seed("not a number").is_err());
        assert!(parse_seed("0xZZZZ").is_err());
    }
}
