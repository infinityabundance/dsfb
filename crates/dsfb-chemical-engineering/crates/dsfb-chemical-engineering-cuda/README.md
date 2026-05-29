# dsfb-chemical-engineering-cuda

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-50.1%25-orange)](../../audit/dsfb-gray/cuda/) [![unsafe](https://img.shields.io/badge/unsafe-FFI%20boundary-yellow)](../../audit/cargo-geiger/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

CUDA-accelerated DSFB-Chemical-Engineering **evidence factory** + **forensic court**, with
Nsight-measured throughput and **byte-exact, replayable** evidence.

*Riaan de Beer — Invariant Forge LLC — ORCID 0009-0006-1155-027X.*

## Doctrine

> The GPU produces evidence; the court decides what that evidence is allowed to mean.

Residual streams (and the usually-discarded sub-threshold noise) are quantised to fixed point and
sealed, per lane, into an **on-GPU SHA-256** digest under a locked deterministic contract
(`--fmad=false`, integer evidence arithmetic). Because the arithmetic is associativity-independent
and the device SHA-256 reproduces the host `sha2` digest, the GPU evidence root is **cross-verified
byte-for-byte against a CPU reference**. The court then assembles a hash-linked **case file**
(passport, per-lane evidence, Merkle + evidence root, challenge docket, precedent chain) and an
**execution attestation** (backend, device, timing, throughput) sealed over the reproducible root.

## Build & run

```bash
bash scripts/build_cuda.sh            # builds --features cuda if nvcc present, else CPU reference
cargo run --release -p dsfb-chemical-engineering-cuda --features cuda -- demo     # forensic court
cargo run --release -p dsfb-chemical-engineering-cuda --features cuda -- bench    # GB/s benchmarks
bash scripts/run_bench.sh 3           # repeat the GB/s sweep 3x (5 internal runs each)
bash scripts/run_nsight.sh 5          # nsys + ncu, 5 runs x 3 size variants -> reports/
cargo run --release -p dsfb-chemical-engineering-cuda --features cuda -- verify-replay
```

Without `--features cuda` (or without a GPU) the crate runs a **CPU reference path that produces the
identical evidence root**, so it always builds and runs; the GPU path is an acceleration + an
independent cross-check, never a correctness dependency.

## What the numbers mean

- **Memory roofline kernel** measures achievable DRAM bandwidth (≈88% of the device peak on an
  RTX 4080 SUPER in local runs).
- **Evidence-factory kernel** is **SHA-256-compute-bound**, not bandwidth-bound: each lane sequentially
  hashes a 40-byte record per sample. Throughput therefore scales with lane parallelism and sits far
  below the memory roofline — the honest cost of cryptographically sealing every residual and its
  noise. The Nsight reports (`reports/ncu_*.csv`, `reports/nsys_*.txt`) quantify this across runs and
  size variants; the paper cites those committed files directly.

## Evidence-format versions (how an optimisation is admitted)

A performance change must **never silently mutate forensic identity**. Every candidate kernel is classified against
the V1 sealed reference by [`EquivalenceReport::optimization_status()`](src/digest_equivalence.rs) into exactly one
`CudaOptimizationStatus`. The decisive check is `lanes_match` — whether the *data each lane sealed* is byte-identical;
a differing root with identical lanes can only be a deliberate re-sealing **format**, never a data regression.

| Kernel variant | `evidence_root` | Per-lane evidence | Digest-identical? | Court status (`CudaOptimizationStatus`) |
|---|---|---|---|---|
| **V1** — sealed reference | canonical Merkle `evidence_root` | baseline | — (the reference) | everything is measured against it |
| **V2-A** — throughput optimisation | identical to V1 | byte-identical | **yes** (lanes + Merkle + root + replay all match) | `digest-identical-optimization` — **admitted** ✅ |
| **V2-B** — segmented re-seal | `evidence_root_v2` (differs *by design*) | byte-identical | no (root *construction* changed) | `new-evidence-format-version` — **admitted as a declared format** ✅ |
| *any kernel that perturbs lane data* | differs | **diverged** | no | `rejected-performance-regression` — **must not ship** ❌ |

The first two columns are what a downstream verifier compares; the last column is the GPU harness's verdict (a
review gate, **not** part of the sealed `evidence_root`). CPU-only tests in `src/digest_equivalence.rs` pin each
mapping; the GPU-gated parity paths run under `--features cuda` on an NVIDIA host.

## Determinism contract

`evidence-contract/v1`: SCALE = 1e6, drift window = 16, per-sample hashed record =
`raw_bits ‖ q ‖ exceedance ‖ drift ‖ slew` (little-endian), quantisation = single IEEE-754 double
multiply + round-half-away-from-zero, `--fmad=false`. See `src/evidence.rs` and `cuda/common.cuh`.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
