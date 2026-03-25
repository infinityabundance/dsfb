# dsfb-endoduction

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-endoduction/dsfb_endoduction_nasa_bearings_colab.ipynb)

**Empirical evaluation of the Thermodynamic Precursor Visibility Principle on NASA IMS bearing run-to-failure data using DSFB structural residual analysis.**

---

## Overview

This crate implements a complete residual-structure analysis pipeline that tests whether DSFB-style structured residual analysis reveals degradation precursors *earlier* or *more clearly* than conventional scalar diagnostics on real run-to-failure bearing data.

It operationalises the paper *"The Thermodynamic Precursor Visibility Principle: A Formal Connection Between Non-Equilibrium Thermodynamics, Critical Phenomena, and Structural Semiotic Inference"* and its endoduction boundary framing.

**This is an empirical evaluation, not a proof.** The implementation preserves the paper's epistemic posture: the principle is a falsifiable, physically motivated hypothesis, and the crate tests it against real data, reporting results honestly regardless of outcome.

---

## Scientific Motivation

Physical systems approaching critical transitions exhibit characteristic precursor signatures: increased variance, autocorrelation growth, spectral redistribution, and altered fluctuation structure. The Thermodynamic Precursor Visibility Principle hypothesises that these signatures are not merely present but are *computationally accessible* to regime-aware structural inference mechanisms.

This crate tests that hypothesis by:
1. Constructing a nominal-regime model from early-life bearing data
2. Computing structured residuals against that model
3. Applying a grammar of structural motifs to characterise residual evolution
4. Aggregating motif evidence into a bounded precursor score
5. Comparing the resulting detection lead time against conventional scalar diagnostics

---

## Paper-to-Code Mapping

| Paper Concept | Code Module | Implementation |
|---|---|---|
| Observation x_obs(t) | `data` | Parsed NASA IMS accelerometer snapshots |
| Nominal model x_model(t) | `baseline` | Per-sample mean waveform from early-life windows |
| Residual r(t) = x_obs - x_model | `residual` | Point-wise subtraction |
| Admissibility envelope E_R | `admissibility` | Per-sample tolerance band from nominal variance |
| Structural grammar | `grammar` | Drift, slew, persistence, variance growth, autocorrelation growth, spectral shift |
| Trust / precursor score | `trust` | Bounded weighted sigmoid aggregate of structural indicators |
| Lead-time evaluation | `evaluation` | First sustained detection vs failure reference |
| Classical diagnostics | `baselines` | RMS, kurtosis, crest factor, spectral energy, variance, autocorrelation |

### Key Formulas

**Residual construction:**
```
r(t) = x_obs(t) - x_model(t)
```

**Admissibility envelope:**
```
E_R = { r : |r_i| ≤ k · σ_i  for all sample indices i }
```
where k is determined from the configured quantile level via the probit function, and σ_i is the per-sample standard deviation estimated from the nominal regime.

**Breach fraction:**
```
B = |{ i : r_i ∉ E_R }| / N
```

**Trust score:**
```
T = Σ_j  w_j · σ(f_j(indicators))
```
where σ is the logistic sigmoid, f_j are normalisation functions, and w_j are fixed weights summing to 1. The score is bounded to [0, 1].

---

## Dataset

**NASA IMS Bearing Run-to-Failure Dataset**

- **Source:** NASA Prognostics Data Repository, Intelligent Maintenance Systems (IMS) Center, University of Cincinnati
- **Reference:** J. Lee, H. Qiu, G. Yu, J. Lin, "Rexnord Technical Services, IMS, University of Cincinnati. Bearing Data Set", NASA Prognostics Data Repository, 2007.
- **URL:** https://www.nasa.gov/content/prognostics-center-of-excellence-data-set-repository
- **Structure:** Three test sets, each with 4 bearings instrumented with accelerometers, run to failure under constant radial load at 2156 RPM.
- **Sampling:** 20 kHz, 1-second snapshots at ~10-minute intervals.

### Set 1 (Default)
- 4 bearings, 2 accelerometers each (8 channels)
- ~2156 snapshots over 35 days
- Inner-race fault develops on bearing 3 or 4

No synthetic data is used. All results come exclusively from real sensor measurements.

---

## Method

1. **Data loading:** Parse all snapshot files chronologically.
2. **Nominal baseline:** Estimate per-sample mean waveform and variance from the first 15% of snapshots (configurable).
3. **Residual construction:** For each window, compute r(t) = x_obs(t) - x_model(t).
4. **Admissibility envelope:** Estimate per-sample bounds from the nominal regime; compute breach fraction for each window.
5. **Structural grammar:** Compute drift, persistence, variance growth, autocorrelation growth, and spectral centroid shift for each residual window.
6. **Trust score:** Aggregate structural indicators into a bounded [0, 1] precursor score.
7. **Classical baselines:** Compute RMS, kurtosis, crest factor, rolling variance, lag-1 autocorrelation, and spectral band energy.
8. **Evaluation:** Determine first sustained detection for DSFB and each baseline; compute lead times relative to the failure reference.
9. **Artifact generation:** Produce 12 figures, CSV, JSON manifest, PDF report, and ZIP bundle.

---

## Outputs

Each run creates a timestamped subfolder under `output-dsfb-endoduction/`:

```
output-dsfb-endoduction/
  2026-03-24_14-33-12/
    fig01_dataset_overview.png
    fig02_raw_signal_snapshots.png
    fig03_conventional_diagnostics.png
    fig04_nominal_vs_observation.png
    fig05_residual_evolution.png
    fig06_admissibility_envelope.png
    fig07_structural_grammar_panel.png
    fig08_trust_score.png
    fig09_baseline_comparison.png
    fig10_lead_time_comparison.png
    fig11_robustness.png
    fig12_summary_synthesis.png
    metrics.csv
    manifest.json
    report.pdf
    bundle.zip
```

Nothing overwrites previous runs. The JSON manifest records all parameters, gate results, file inventory, and summary metrics.

---

## Reproducibility

- All analysis is deterministic (no stochasticity in the pipeline).
- Fixed random seed (42) is recorded for any future extensions.
- All parameters are recorded in the JSON manifest.
- Git revision is captured when available.
- The Colab notebook clones and builds from scratch every run.

---

## Limitations and Non-Claims

### What this does not claim

- **Does not prove** the Thermodynamic Precursor Visibility Principle. It tests it.
- **Does not establish universality.** Results apply to the specific dataset and configuration tested.
- **Does not measure thermodynamic entropy production.** Residual structure is used as a departure-from-nominal proxy, not a direct thermodynamic measurement.
- **Does not replace** validated condition monitoring systems. This is a research artifact.
- **Does not guarantee** that DSFB will outperform every classical diagnostic on every metric or dataset.

### Known limitations

- Spectral analysis uses a simplified DFT (not a full periodogram or Welch method).
- The nominal model is a simple per-sample mean; more sophisticated models might perform differently.
- Trust score weights are fixed; they have not been optimised and are not claimed to be optimal.
- Only one channel is used as the primary analysis channel per run.
- Lead-time estimates depend on the threshold and sustained-count parameters.

---

## Code Architecture

```
src/
  lib.rs              — Crate root, module declarations
  types.rs            — Config, WindowMetrics, RunManifest, GateResults
  data.rs             — Dataset download, verification, parsing
  baseline.rs         — Nominal regime estimation, math helpers
  residual.rs         — Residual construction: r(t) = x_obs(t) - x_model(t)
  admissibility.rs    — Envelope estimation and breach detection
  grammar.rs          — Structural motif detectors
  trust.rs            — Precursor score aggregation
  baselines.rs        — Classical diagnostic metrics
  evaluation.rs       — Lead-time and comparison evaluation
  figures.rs          — 12 publication-grade PNG figures
  report.rs           — PDF report assembly and ZIP archiving
  cli.rs              — CLI argument definitions
  bin/main.rs         — Pipeline orchestration
tests/
  integration.rs      — Unit and integration tests
```

---

## Running Locally

### Prerequisites
- Rust ≥ 1.74
- The NASA IMS bearing dataset, extracted to a local directory

### Download the dataset
Download from the NASA Prognostics Data Repository and extract so the structure is:
```
data/IMS/1st_test/
data/IMS/2nd_test/
data/IMS/3rd_test/
```

Or use the `--download` flag to attempt automatic download.

### Build and run
```bash
# From the workspace root:
cargo build -p dsfb-endoduction --release

# Run the full pipeline:
cargo run -p dsfb-endoduction --release -- run \
  --data-root data \
  --bearing-set 1 \
  --channel 0 \
  --download

# With custom parameters:
cargo run -p dsfb-endoduction --release -- run \
  --data-root /path/to/data \
  --nominal-fraction 0.20 \
  --trust-threshold 0.4 \
  --sustained 3
```

### Run tests
```bash
cargo test -p dsfb-endoduction
```

---

## Running in Colab

Click the badge at the top of this README or open the notebook directly:

[`dsfb_endoduction_nasa_bearings_colab.ipynb`](dsfb_endoduction_nasa_bearings_colab.ipynb)

The notebook:
1. Installs the Rust toolchain
2. Clones the repository
3. Builds the crate
4. Downloads the NASA IMS bearing dataset
5. Runs the full analysis pipeline
6. Displays all 12 figures inline
7. Prints summary metrics
8. Generates a PDF report and ZIP bundle
9. Provides download buttons for the PDF and ZIP

---

## License

Apache-2.0. See [LICENSE](../../LICENSE).
