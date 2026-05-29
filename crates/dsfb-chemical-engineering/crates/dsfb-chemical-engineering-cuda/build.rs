// build.rs for dsfb-chemical-engineering-cuda.
//
// Without the `cuda` feature: no-op (the crate builds a CPU reference path that produces the
// identical evidence root, so it always compiles and runs).
// With `cuda`: discover nvcc (CUDA_HOME -> PATH -> /opt/cuda -> /usr/local/cuda), compile
// cuda/kernels.cu with the LOCKED deterministic flags (--fmad=false is part of the evidence
// contract), archive into a static lib, and emit link directives for it plus cudart.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cuda/kernels.cu");
    println!("cargo:rerun-if-changed=cuda/common.cuh");
    println!("cargo:rerun-if-changed=cuda/sha256.cuh");
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    if cfg!(feature = "cuda") {
        cuda_build();
    }
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn cuda_build() {
    let nvcc = locate_nvcc().unwrap_or_else(|| {
        panic!("nvcc not found. Set CUDA_HOME or install the CUDA toolkit (/opt/cuda, /usr/local/cuda), or build without --features cuda.");
    });
    let cuda_root = nvcc
        .parent()
        .and_then(Path::parent)
        .expect("nvcc parent")
        .to_path_buf();
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let kernels = manifest.join("cuda").join("kernels.cu");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out.join("dsfb_chem_kernels.o");
    let archive = out.join("libdsfb_chem_kernels.a");

    // LOCKED flags — changing any of these is an evidence-contract breach.
    let status = Command::new(&nvcc)
        .args([
            "-O3",
            "--fmad=false",
            "-std=c++14",
            "-Xcompiler",
            "-fPIC",
            "-gencode",
            "arch=compute_75,code=compute_75", // PTX baseline (Colab T4/A100 JIT; CUDA 13 min arch)
            "-gencode",
            "arch=compute_89,code=sm_89", // native RTX 4080 SUPER
            "-I",
        ])
        .arg(manifest.join("cuda"))
        .arg("-c")
        .arg(&kernels)
        .arg("-o")
        .arg(&obj)
        .status()
        .expect("failed to invoke nvcc");
    assert!(status.success(), "nvcc failed to compile kernels.cu");

    let ar = Command::new("ar")
        .arg("crus")
        .arg(&archive)
        .arg(&obj)
        .status()
        .expect("ar failed");
    assert!(ar.success(), "ar failed to archive kernels");

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=dsfb_chem_kernels");
    for libdir in ["lib64", "lib"] {
        let p = cuda_root.join(libdir);
        if p.is_dir() {
            println!("cargo:rustc-link-search=native={}", p.display());
        }
    }
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn locate_nvcc() -> Option<PathBuf> {
    if let Ok(home) = env::var("CUDA_HOME") {
        let p = PathBuf::from(home).join("bin").join("nvcc");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(path) = env::var("PATH") {
        for d in env::split_paths(&path) {
            let p = d.join("nvcc");
            if p.exists() {
                return Some(p);
            }
        }
    }
    for root in ["/opt/cuda", "/usr/local/cuda"] {
        let p = PathBuf::from(root).join("bin").join("nvcc");
        if p.exists() {
            return Some(p);
        }
    }
    None
}
