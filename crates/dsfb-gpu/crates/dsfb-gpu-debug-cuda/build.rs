// build.rs for `dsfb-gpu-debug-cuda`.
//
// Build scripts panic on configuration errors by design; suppressing the
// pedantic-clippy lints that would force every panic site to be rewritten
// as an `assert!` or a `Result`. The build script has no callers other
// than cargo, and a clean panic with a descriptive message is the
// idiomatic way to report a missing toolchain.

#![allow(clippy::expect_used)]
#![allow(clippy::manual_assert)]
#![allow(clippy::similar_names)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unwrap_used)]

//
// Without the `cuda` feature: no-op, no host requirements.
// With the `cuda` feature: discover `nvcc` (CUDA_HOME → PATH → common
// install prefixes), compile `cuda/kernels.cu` with the locked nvcc flags
// from the deterministic execution contract, archive the object into a
// static library, and emit the cargo link directives for the archive plus
// `cudart`. The locked flags are pinned: changing any of them is a contract
// breach.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "cuda") {
        cuda_build();
    }
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn cuda_build() {
    // ---- 1. Locate nvcc and the CUDA toolkit root. ----
    let nvcc = locate_nvcc().unwrap_or_else(|| {
        panic!(
            "nvcc not found. Set CUDA_HOME, or install the CUDA toolkit in one of: \
             /opt/cuda, /usr/local/cuda. Or build without --features cuda."
        );
    });
    let cuda_root = nvcc
        .parent()
        .and_then(Path::parent)
        .expect("nvcc has a parent directory")
        .to_path_buf();
    let cuda_include = cuda_root.join("include");
    let cuda_lib64 = cuda_root.join("lib64");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cuda_src_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("cuda");
    let kernels_cu = cuda_src_dir.join("kernels.cu");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj_path = out_dir.join("kernels.o");
    let archive_path = out_dir.join("libdsfb_gpu_kernels.a");

    println!("cargo:rerun-if-changed={}", kernels_cu.display());
    println!(
        "cargo:rerun-if-changed={}",
        cuda_src_dir.join("common.cuh").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        cuda_src_dir.join("layout.cuh").display()
    );
    println!("cargo:rerun-if-env-changed=CUDA_HOME");

    // ---- 2. Compile the kernel translation unit. ----
    //
    // Locked nvcc flags from the deterministic execution contract. Two
    // architectures are listed: sm_70 as the baseline so Colab T4 (sm_75)
    // and A100 (sm_80) can PTX-JIT, and sm_89 native so the local RTX 4080
    // SUPER runs without JIT overhead. Any change to this list is a
    // contract breach.
    // Locked nvcc flags. `--use_fast_math` is omitted intentionally: it
    // is a flag (presence = enable), and fast-math is off by default. The
    // `--fmad=false` switch forbids fused-multiply-add even when other
    // optimizations are on.
    //
    // Architectures:
    //   * compute_75 / sm_75 — T4 baseline. Also emit forward-compatible
    //     PTX so newer GPUs (A100, H100, RTX 4080 SUPER) can JIT.
    //   * sm_80 / sm_89 — native cubin for the most common Colab and
    //     local-dev cards so they skip JIT.
    //
    // CUDA 13.x dropped sm_70 (Volta). The compute_75 baseline keeps the
    // Colab T4 covered. If a future contract needs an older device the
    // toolkit minor version must change in lockstep.
    let nvcc_flags: &[&str] = &[
        "--std=c++17",
        "-gencode=arch=compute_75,code=sm_75",
        "-gencode=arch=compute_80,code=sm_80",
        "-gencode=arch=compute_89,code=sm_89",
        "-gencode=arch=compute_89,code=compute_89",
        "--fmad=false",
        "-O2",
        "-Xcompiler",
        "-fPIC",
        "-Xptxas",
        "-O2",
        "-DDSFB_GPU_FIXED_POINT",
    ];

    let mut cmd = Command::new(&nvcc);
    cmd.args(nvcc_flags)
        .arg("-c")
        .arg(&kernels_cu)
        .arg("-o")
        .arg(&obj_path)
        .arg("-I")
        .arg(&cuda_src_dir)
        .arg("-I")
        .arg(&cuda_include);
    let status = cmd.status().expect("failed to invoke nvcc");
    if !status.success() {
        panic!(
            "nvcc compilation failed for {}: {status}",
            kernels_cu.display()
        );
    }

    // ---- 3. Archive the object into a static library. ----
    let ar = env::var("AR").unwrap_or_else(|_| "ar".to_string());
    let status = Command::new(&ar)
        .arg("rcs")
        .arg(&archive_path)
        .arg(&obj_path)
        .status()
        .expect("failed to invoke ar");
    if !status.success() {
        panic!("ar failed building {}: {status}", archive_path.display());
    }

    // ---- 4. Emit cargo link directives. ----
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dsfb_gpu_kernels");
    println!("cargo:rustc-link-search=native={}", cuda_lib64.display());
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn locate_nvcc() -> Option<PathBuf> {
    // 1. CUDA_HOME env var (the canonical convention used by NVIDIA samples).
    if let Ok(home) = env::var("CUDA_HOME") {
        let p = Path::new(&home).join("bin").join("nvcc");
        if p.is_file() {
            return Some(p);
        }
    }
    // 2. PATH.
    if let Some(path) = env::var_os("PATH") {
        for dir in env::split_paths(&path) {
            let p = dir.join("nvcc");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    // 3. Common install prefixes — checked last so user-set state wins.
    for prefix in ["/opt/cuda", "/usr/local/cuda"] {
        let p = Path::new(prefix).join("bin").join("nvcc");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
