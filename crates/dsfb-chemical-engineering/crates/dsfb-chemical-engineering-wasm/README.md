# dsfb-chemical-engineering-wasm — the interactive Chemical Court "what-if" simulator

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-64.7%25-yellow)](../../audit/dsfb-gray/wasm/) [![unsafe](https://img.shields.io/badge/unsafe-1%20audited%20FFI-yellow)](../../audit/cargo-geiger/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

A browser tool that **replays a chemical-process residual stream through the DSFB grammar under an
operator-amended admissibility envelope**. The residual stream is the *immutable evidence*; the operator drags
the envelope half-width `k`, the grazing-band fraction, and the drift window, and watches how the **same**
evidence is re-classified into grammar tokens (`NOM` / `DriftAccum` / `SlewSpike` / `EnvViolation` /
`BoundaryGrazing` / `Recovery` / `Compound` / `SensorFault`) and how many episodes form.

It is the Wave-7 *"interactive Chemical Court simulator"* item — a HAZOP / training / forensic what-if
instrument: *"if our admissibility band had been this tight, how would the court have read this episode?"* —
over evidence that never changes.

## Why it is built this way

- It compiles to **`wasm32-unknown-unknown`** and depends ONLY on the dependency-free embedded
  [`dsfb-chemical-engineering-core`](../dsfb-chemical-engineering-core) crate — the *same* fixed-point grammar
  that runs on the Cortex-M3 / under QEMU. The exact integer grammar an operator could deploy at the edge is the
  one replaying in their browser; there is no second implementation to keep honest.
- It uses **raw `extern "C"` exports + hand-written JS glue**, not `wasm-bindgen`: the whole tool is one
  `cargo build` plus a static HTML page — no build-tool or extra-dependency supply-chain surface.
- The numeric work is the pure, host-tested `simulate_into` (in `src/lib.rs`); `cargo test` gates the simulator
  logic on the host even though the UI is exercised in a browser.

## Build & run

```sh
# from this crate directory:
sh web/build.sh                         # cargo build --target wasm32 --release + copy .wasm + regen sample
cd web && python3 -m http.server 8000   # a static server (file:// fetch is blocked by browsers)
# open http://localhost:8000/
```

The committed `web/dsfb_court_sim.wasm` (~25 KB) and `web/sample_residuals.json` let the page run with no build
step; `web/build.sh` refreshes both from source. Host-side logic tests:

```sh
cargo test            # 5 simulate_into tests (flat-nominal, sustained-breach, tighter-flags-more, determinism, window/truncation)
```

## Files

| File | Role |
|---|---|
| `src/lib.rs` | `simulate_into` (pure, host-tested) + the `ffi` module (raw `dsfb_sim_*` exports over fixed linear-memory buffers) |
| `web/index.html` | the page: sliders (`k` / grazing band / drift window), the grammar-token timeline, counts, the non-claims banner |
| `web/court_sim.js` | glue: instantiate the `.wasm` (no imports), marshal residuals → `IN_BUF`, call `dsfb_sim_run`, render `OUT_BUF` tokens; computes the residual SHA-256 (shown constant across what-if runs) |
| `web/sample_residuals.json` | a labelled **synthetic** chemical (CSTR/reactor SPE-style) residual stream — **not plant data** |
| `web/gen_sample_residuals.py` | deterministic (SplitMix64) generator for the sample stream |
| `web/dsfb_court_sim.wasm` | the built module (refresh via `web/build.sh`) |

## What this is NOT (standing DSFB boundaries)

- **Not a controller or safety function.** Read-only, advisory; it classifies a residual stream and counts
  episodes — no actuation, no SIS / IEC-61511 authority.
- **Not an amendment to any sealed record.** What-if envelope changes are sandboxed in the browser; the
  evidence stream (and its displayed SHA-256) is never mutated. The simulator cannot re-seal or alter a Court
  Record.
- **Not bit-identical to the edge float pipeline.** It is the embedded fixed-point *sibling* grammar (the edge
  crate + its replay-hash gate remain the numeric reference); the two are the same *grammar*, calibrated
  independently.
- **Not plant data, and not domain-general.** The bundled sample is a synthetic **chemical** residual; this
  project is strictly chemical-engineering-specific and makes no cross-domain claim.

This crate is a standalone workspace (excluded from the host workspace, like the `py` bindings); its build
artifacts go to the shared root `target/wasm` (see `.cargo/config.toml`).

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
