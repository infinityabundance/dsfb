# dsfb-chemical-engineering-core

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-69.5%25-yellowgreen)](../../audit/dsfb-gray/core/)
[![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)
[![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md)
[![Miri](https://img.shields.io/badge/Miri-no%20UB-brightgreen)](../../audit/miri/README.md)
[![no_std](https://img.shields.io/badge/no__std-yes-brightgreen)](#what-this-crate-is)
[![heap](https://img.shields.io/badge/heap-none-brightgreen)](#embedded-profile)

`dsfb-chemical-engineering-core` is the tiny embedded grammar core of **DSFB-Chemical-Engineering**.

It is a `no_std`, no-heap, fixed-point Rust crate that implements the deterministic residual-state grammar used by DSFB-Chemical-Engineering in a form suitable for microcontrollers, WASM substrates, and edge-adjacent evidence devices.

It does **not** perform chemometrics. It does **not** run PCA/PLS/MSPC. It does **not** control a plant. It is the small deterministic grammar kernel:

```text
scaled residual sample
→ fixed residual triple (r, δ, σ)
→ admissibility-envelope classification
→ DSFB grammar token + reason code
```

## What this crate is

This crate provides:

```text
Fixed-point residual triples
Const-generic ring buffer
Windowed drift estimate
First-difference slew estimate
Admissibility-envelope classification
DSFB grammar-state classifier
Stable grammar tokens
Reason codes
Dependency-free no_std execution
```

It is intended for situations where the full edge/CUDA DSFB court is too large, but the deterministic residual grammar is still useful as an embedded witness.

Typical uses:

```text
microcontroller-side residual witness
edge-device preclassification
browser/WASM demonstration substrate
fixed-point grammar regression target
DPU / densor-runtime design seed
```

## What this crate is not

This crate is deliberately narrow.

It is **not**:

```text
a controller
a safety-instrumented function
a plant model
a soft sensor
a root-cause engine
a replacement for PCA/PLS/MSPC/ML fault detection
a bit-identical replacement for the floating edge pipeline
```

It emits advisory grammar tokens over residuals. It writes no setpoint, alarm limit, historian tag, actuator value, or control variable.

Removing it restores the upstream system exactly.

## Embedded profile

Design constraints:

```text
std:        no
heap:       no
unsafe:     forbidden
deps:       none
arithmetic: fixed-point i64
scale:      1_000_000
memory:     fixed capacity, const-generic
panic path: intended for panic=abort embedded builds
```

The core is written as:

```rust
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
```

The crate has no dependencies. It uses only `core`.

The same grammar core is designed to build for:

```text
thumbv7m-none-eabi
wasm32-unknown-unknown
host test harness
```

The broader DSFB-Chemical-Engineering verification surface includes QEMU Cortex-M smoke testing and a documented embedded memory budget.

## Core model

Each residual sample is represented as a fixed-point value:

```rust
pub const SCALE: i64 = 1_000_000;
```

A sample stream is converted into a residual triple:

```text
r     = current scaled residual
δ     = windowed mean of r over a fixed ring buffer
σ     = first difference of r
```

The triple is evaluated against a fixed-point admissibility envelope:

```text
r_min     ≤ r     ≤ r_max
delta_min ≤ δ     ≤ delta_max
sigma_min ≤ σ     ≤ sigma_max
```

Each axis is classified as:

```text
Interior
Grazing
Outside
```

The grammar then emits one DSFB state token per sample.

## Grammar states

The embedded grammar states are:

| State             | Token | Meaning                                                     |
| ----------------- | ----: | ----------------------------------------------------------- |
| `Nominal`         | `NOM` | Interior on every axis                                      |
| `DriftAccum`      |  `DA` | Sustained drift breached the drift envelope                 |
| `SlewSpike`       |  `SS` | Rapid first-difference transient breached the slew envelope |
| `EnvViolation`    |  `EV` | Raw residual breached its envelope                          |
| `BoundaryGrazing` |  `BG` | Near-boundary evidence without breach                       |
| `Recovery`        |  `RC` | First interior sample after a disturbance                   |
| `Compound`        |  `CP` | Drift and slew breach together                              |
| `SensorFault`     |  `SF` | Invalid reading; internal memory is preserved               |

Reason codes additionally preserve direction where relevant:

```text
DriftPositive
DriftNegative
SlewPositive
SlewNegative
Violation
Grazing
Recovery
Compound
OobSensor
```

## Minimal example

```rust
use dsfb_chemical_engineering_core::{
    DsfbCore,
    FixedEnvelope,
    GrammarState,
    SCALE,
};

fn fx(x: f64) -> i64 {
    (x * SCALE as f64).round() as i64
}

fn main() {
    // Symmetric residual envelope:
    // r:     ±3.0
    // delta: ±1.8
    // sigma: ±6.0
    // grazing band: 10%
    let env = FixedEnvelope::symmetric(fx(3.0), fx(0.1));

    // One channel, fixed window of 8 samples.
    let mut core = DsfbCore::<8>::new(env);

    let samples = [
        0.0, 0.1, -0.1, 0.0,
        2.5, 2.5, 2.5, 2.5,
    ];

    for x in samples {
        let (state, reason) = core.step(fx(x), true);
        println!("{} {:?}", state.token(), reason);

        if state == GrammarState::DriftAccum {
            println!("sustained drift witness formed");
        }
    }
}
```

## Invalid readings

Use the `valid` flag to preserve deterministic handling of invalid sensor readings.

```rust
let (state, reason) = core.step(0, false);
```

When `valid = false`, the core emits:

```text
SensorFault / OobSensor
```

and does not push into the ring buffer, does not update the previous sample, and does not corrupt the grammar memory.

## Relationship to the full DSFB-Chemical-Engineering stack

This crate is the embedded grammar sibling of the larger DSFB-Chemical-Engineering system.

The full stack includes:

```text
dsfb-chemical-engineering-edge     CPU / edge execution over process residuals
dsfb-chemical-engineering-cuda     GPU evidence factory + forensic court
dsfb-chemical-engineering-atlas    detector / heuristic / signature authority
dsfb-chemical-engineering-corpus   soft-sensor dataset authority catalogue
dsfb-densor-runtime                deterministic densor execution substrate
```

This core crate intentionally stays smaller:

```text
one channel
one fixed ring
one grammar token per sample
no heap
no dependencies
no plant-control authority
```

It is the right crate to inspect first if you want to understand the deterministic DSFB residual grammar without the full paper, CUDA path, dataset bundle, or operator-reporting layer.

## Determinism boundary

This crate is deterministic because:

```text
all arithmetic is fixed-point integer arithmetic
memory is fixed capacity
classification uses exact comparisons
there is no allocation
there is no randomness
there is no probability model
there is no likelihood
there is no learned state
there is no hidden runtime dependency
```

It is not claimed to be bit-identical to the floating-point edge pipeline. Treat it as the same grammar expressed in a fixed-point embedded profile, calibrated independently.

## Safety boundary

This crate is advisory only.

It must not be used as:

```text
a safety shutdown mechanism
a SIL/SIS decision function
a controller
an alarm-limit writer
a regulatory compliance verdict
a root-cause proof
```

It may be used as a read-only witness over residual streams.

## Installation

```toml
[dependencies]
dsfb-chemical-engineering-core = "0.1"
```

## Build checks

Host tests:

```bash
cargo test -p dsfb-chemical-engineering-core
```

Embedded target build:

```bash
rustup target add thumbv7m-none-eabi
cargo build -p dsfb-chemical-engineering-core --target thumbv7m-none-eabi
```

WASM target build:

```bash
rustup target add wasm32-unknown-unknown
cargo build -p dsfb-chemical-engineering-core --target wasm32-unknown-unknown
```

QEMU smoke testing is provided in the repository under the `qemu-smoke` harness.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (v1.0). Zenodo. https://doi.org/10.5281/zenodo.20443279

Machine-readable citation metadata is provided in `CITATION.cff`.
