//! GPU↔CPU byte-exact parity gate across SHA-256 message-length residue classes.
//!
//! This is the regression gate for the device-SHA-256 padding bug fixed in `cuda/sha256.cuh`: the
//! on-device `final()` previously inflated the encoded message length by 512 bits whenever the padded
//! message needed a second block — i.e. for any lane whose `40 * n_samples` byte stream is ≡ 56 (mod
//! 64), which is `n_samples ≡ 3 (mod 8)`. On those inputs the GPU digest diverged from the host `sha2`
//! crate, so `gpu_cross_verified` came back false and the court silently fell back to the CPU reference.
//! The 20 evaluation datasets all happen to dodge that residue class, so the headline "GPU ≡ CPU on all
//! 20" was true only by luck of the sample counts. This test exercises the residue classes directly —
//! especially `n_samples ≡ 3 (mod 8)` — so byte-exactness is gate-enforced, not corpus-dependent.
//!
//! The whole test is `#[cfg(feature = "cuda")]`: it only compiles when the CUDA backend is built. Even
//! then, if no CUDA device is present at runtime `dispatch::run` returns the CPU path, and the test
//! skips its assertions (it cannot test a GPU that is not there). On a real CUDA host it asserts the GPU
//! produced byte-identical evidence to the CPU reference for every n_samples below.

#![cfg(feature = "cuda")]

use dsfb_chemical_engineering_cuda::dispatch::{self, Backend, Lanes};
use dsfb_chemical_engineering_cuda::evidence::merkle_root;
use sha2::{Digest, Sha256};

/// Build a deterministic, bit-portable `Lanes` (values are exact multiples of 1/8) with the given
/// shape, so the test input is identical on every platform.
fn make_lanes(n_lanes: u32, n_samples: u32) -> Lanes {
    let nl = n_lanes as usize;
    let ns = n_samples as usize;
    let mut data = vec![0.0f64; nl * ns];
    for i in 0..ns {
        for l in 0..nl {
            data[i * nl + l] = (((i * 5 + l * 11) % 64) as f64) / 8.0 - 4.0;
        }
    }
    let mut h = Sha256::new();
    for &x in &data {
        h.update(x.to_bits().to_le_bytes());
    }
    let input_sha256 = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    Lanes {
        data,
        n_lanes,
        n_samples,
        input_sha256,
    }
}

#[test]
fn gpu_matches_cpu_across_sha_residue_classes() {
    // n_samples spanning every residue mod 8, with the previously-broken `≡ 3 (mod 8)` class
    // (3, 11, 27, 35) over-represented, plus the block-boundary cases (56-bit/64-byte alignment).
    let sample_counts: &[u32] = &[3, 8, 11, 16, 27, 35, 56, 64, 96, 120];
    let mut ran_on_gpu = false;
    for &ns in sample_counts {
        let lanes = make_lanes(3, ns);
        let run = dispatch::run(&lanes);
        match run.backend {
            Backend::Cuda => {
                ran_on_gpu = true;
                // The GPU ran: it must have matched the CPU reference (this is exactly what the SHA
                // padding bug broke for n_samples ≡ 3 (mod 8)).
                assert!(
                    run.gpu_cross_verified,
                    "GPU evidence diverged from CPU reference at n_samples={ns} (n mod 8 = {}). \
                     This is the device-SHA-256 padding bug — check cuda/sha256.cuh::final().",
                    ns % 8
                );
                // And the sealed lanes must reproduce the CPU reference Merkle root byte-for-byte.
                let cpu = dispatch::evidence_cpu(&lanes);
                assert_eq!(
                    merkle_root(&run.lanes),
                    merkle_root(&cpu),
                    "GPU/CPU Merkle root mismatch at n_samples={ns}"
                );
            }
            Backend::Cpu => {
                // CUDA feature compiled but no device available at runtime — nothing to cross-check.
                eprintln!(
                    "no CUDA device at runtime; skipping GPU parity assertion for n_samples={ns}"
                );
            }
        }
    }
    if !ran_on_gpu {
        eprintln!("gpu_cpu_parity: no CUDA device exercised (CPU-only run) — assertions skipped.");
    }
}
