//! `dsfb-gpu-debug run-gpu` — run the CUDA-accelerated pipeline.
//!
//! Required flags:
//!
//! * `--fixture PATH` — canonical fixture JSON.
//! * `--out PATH` — destination for the case file.
//!
//! Optional flags:
//!
//! * `--contract PATH` — accepted but the v0 implementation pins
//!   `Contract::canonical()`.
//!
//! Exit codes:
//! * 0 — success.
//! * 1 — usage error.
//! * 2 — GPU unavailable (built without `cuda` feature, or kernel failed).
//! * 5 — I/O failure.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::emit;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_core::serialize::read_fixture;
use dsfb_gpu_debug_cuda::{build_gpu, GpuError};

use super::{parse_flags, require_flag, usage_error};

pub fn parse_and_run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args) {
        Ok(f) => f,
        Err(message) => return usage_error(&message),
    };
    let fixture_path = match require_flag(&flags, "fixture") {
        Ok(s) => s.to_string(),
        Err(message) => return usage_error(&message),
    };
    let out_path = match require_flag(&flags, "out") {
        Ok(s) => s.to_string(),
        Err(message) => return usage_error(&message),
    };
    let _contract_path = flags.get("contract").cloned();

    let raw = match fs::read(&fixture_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("dsfb-gpu-debug: failed to read {fixture_path}: {error}");
            return ExitCode::from(5);
        }
    };
    let events = match read_fixture(&raw) {
        Ok(events) => events,
        Err(error) => {
            eprintln!("dsfb-gpu-debug: fixture parse error: {error:?}");
            return ExitCode::from(5);
        }
    };

    let mut contract = Contract::canonical();
    contract.pin_bank_hash(bank_hash());
    contract.pin_detector_registry_hash(registry_hash());

    let case = match build_gpu(&events, &contract) {
        Ok(case) => case,
        Err(GpuError::CudaUnavailable) => {
            eprintln!("dsfb-gpu-debug: GPU pipeline unavailable (built without --features cuda)");
            return ExitCode::from(2);
        }
        Err(GpuError::KernelFailed(code)) => {
            eprintln!("dsfb-gpu-debug: GPU kernel failed with cuda status {code}");
            return ExitCode::from(2);
        }
        Err(GpuError::InvalidInput(msg)) => {
            eprintln!("dsfb-gpu-debug: GPU dispatcher rejected input: {msg}");
            return ExitCode::from(2);
        }
    };

    let bytes = emit(&case);

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
        "dsfb-gpu-debug: wrote GPU case file ({} bytes, {} episodes, verdict={}) to {out_path}",
        bytes.len(),
        case.episodes.len(),
        case.final_verdict.name(),
    );
    ExitCode::from(case.final_verdict.exit_code())
}
