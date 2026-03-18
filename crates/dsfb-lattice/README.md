[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/REPLACE_WITH_REPO_OWNER/REPLACE_WITH_REPO_NAME/blob/main/crates/dsfb-lattice/dsfb_lattice_colab.ipynb)



# dsfb-lattice

`dsfb-lattice` is a standalone nested Rust crate under `crates/dsfb-lattice`. It exists to provide a bounded, reproducible, and inspectable empirical demonstrator for selected mathematical ideas from the paper *Deterministic Structural Inference in Solid-State Systems: A DSFB Engine for Crystal Lattices, Phonons, and Structural Forensics*.

The crate is intentionally modest. It does not attempt ab initio crystal simulation, material calibration, or universal defect identification. Instead, it implements a fixed-end harmonic 1D lattice, applies controlled perturbations, compares nominal and perturbed spectra, simulates deterministic observations, computes residual / drift / slew statistics, builds a simple residual envelope from nominal baseline variability, and writes timestamped artifacts suitable for paper support and notebook replay.

## Why This Crate Exists

The repository request was to create an isolated research prototype inside `crates/dsfb-lattice` without touching the root workspace or any production crate. This crate therefore:

- builds as its own nested workspace via `[workspace]` in `Cargo.toml`
- is invoked with `--manifest-path` from the repo root
- writes all runtime artifacts under `crates/dsfb-lattice/output-dsfb-lattice/`
- does not require edits to the root `Cargo.toml`, CI, scripts, or shared repository settings

## Mathematical Scope

The implementation covers a bounded subset of the paper's ideas.

1. Lattice as structured operator
   A fixed-end monatomic mass-spring chain is assembled into a stiffness matrix `K` and a symmetric dynamical matrix `D = M^{-1/2} K M^{-1/2}`.

2. Perturbed operator
   Controlled perturbations create `D' = D + Delta` through mass and spring changes, a smooth strain-like spring gradient, a grouped multi-site perturbation, and a softening sweep.

3. Phonon-like modal structure
   The code computes eigenvalues and eigenvectors of `D`. Frequencies are reported as `omega_i = sqrt(lambda_i)` for non-negative `lambda_i`.

4. Spectral perturbation illustration
   For each finite symmetric toy system, the code checks numerically that the sorted spectral shifts satisfy `|lambda_i' - lambda_i| <= ||Delta||_2`. This is an empirical illustration on controlled matrices, not a proof of the theory.

5. Observation model and residual hierarchy
   The simulated observation is a deterministic modal projection of the state:
   `y_pred = f(x, D)` from the nominal chain and `y_meas = f(x', D')` from the perturbed chain.
   Residual, drift, and slew are then computed as
   `r_k = y_meas,k - y_pred,k`,
   `d_k = r_k - r_{k-1}`,
   `s_k = d_k - d_{k-1}`.

6. Envelope-based detectability
   A residual-norm envelope is estimated from several deterministic nominal baseline runs with small forcing variations. The point-defect example is then checked against that envelope for first crossing and sustained crossing.

7. Group-mode correlation
   Residual covariance is computed across observed modal channels so the grouped perturbation can be compared against the more localized point-defect case.

8. Softening precursor toy example
   A global spring-softening sweep pushes the smallest eigenvalue toward zero and tracks the resulting residual / drift / slew growth. This is a toy precursor study only.

## What The Crate Demonstrates

- a clean assembly of a harmonic lattice operator
- controlled perturbation of that operator
- numerical spectrum and modal comparison
- residual, drift, and slew generation from deterministic simulations
- envelope crossing logic with finite-time detectability in a toy setting
- a covariance contrast between localized and grouped perturbations
- a softening sweep consistent with an approaching-instability interpretation

## What It Does Not Demonstrate

- first-principles phonon prediction for real materials
- universal defect identifiability
- calibrated sensor physics or measurement noise models
- anharmonic dynamics or temperature-dependent effects
- general transition forecasting across solid-state systems
- any claim that the notebook or crate validates the full paper

## Code Structure

`src/lib.rs`
Coordinates the full pipeline, writes structured outputs, and assembles the run summary.

`src/main.rs`
CLI entry point.

`src/lattice.rs`
Defines the lattice representation and constructs the stiffness and dynamical matrices.

`src/perturbation.rs`
Creates the point defect, distributed strain, grouped perturbation, and softening variants.

`src/spectra.rs`
Runs symmetric eigendecomposition and computes spectral-shift / norm comparisons.

`src/residuals.rs`
Simulates deterministic lattice responses and computes residual, drift, slew, and covariance data.

`src/detectability.rs`
Builds the baseline envelope and reports first / sustained crossings.

`src/io.rs`
Creates timestamped run folders, writes JSON / CSV outputs, and zips completed runs.

`src/report.rs`
Generates PNG figures, a Markdown report, and a simple text PDF report.

`src/utils.rs`
Small shared helpers for ranges, PDF escaping, path formatting, and covariance summaries.

`dsfb_lattice_colab.ipynb`
Colab notebook that rebuilds the crate from scratch, runs the demo, displays figures inline, and confirms the PDF and zip artifacts.

`Cargo.lock`
Pinned dependency resolution for reproducible builds.

## Build And Run Locally

From the repo root:

```bash
cargo run --release --manifest-path crates/dsfb-lattice/Cargo.toml -- --example all
```

The binary prints:

- `RUN_DIRECTORY=...`
- `SUMMARY_JSON=...`
- `REPORT_PDF=...`
- `ZIP_ARCHIVE=...`

You can also override the output root explicitly:

```bash
cargo run --release --manifest-path crates/dsfb-lattice/Cargo.toml -- \
  --example all \
  --output-root crates/dsfb-lattice/output-dsfb-lattice
```

The crate defaults to `crates/dsfb-lattice/output-dsfb-lattice`, so it stays within the isolated crate directory even when invoked from the repo root.

## Demonstrations Included

1. Baseline nominal chain
   Builds the nominal fixed-end monatomic chain, computes `D`, the spectrum, and the nominal observation trace.

2. Point defect
   Applies a single-site mass increase and adjacent spring softening to illustrate selective spectral motion, residual growth, and envelope crossing.

3. Distributed strain-like perturbation
   Applies a smooth spring gradient to show coherent spectral drift and broader residual organization.

4. Group-mode perturbation
   Applies a clustered multi-site perturbation to highlight more correlated residual covariance than the localized point defect.

5. Softening sweep
   Reduces global spring scale over a grid and tracks the smallest eigenvalue together with residual / drift / slew maxima.

## Artifacts Produced

Every run writes at least the following files into a new timestamped run directory:

- `config.json`
- `summary.json`
- `metrics.csv`
- `eigenvalues_nominal.csv`
- `eigenvalues_perturbed.csv`
- `residual_timeseries.csv`
- `drift_timeseries.csv`
- `slew_timeseries.csv`
- `covariance.csv`
- `envelope_timeseries.csv`
- `softening_sweep.csv`
- `nominal_observations.csv`
- `figure_01_nominal_vs_point_spectrum.png`
- `figure_02_spectral_shift_comparison.png`
- `figure_03_residual_timeseries_point_defect.png`
- `figure_04_drift_slew_timeseries_point_defect.png`
- `figure_05_detectability_envelope.png`
- `figure_06_covariance_heatmap.png`
- `figure_07_softening_precursor.png`
- `report.md`
- `report.pdf`

After the run completes, a sibling archive is also written:

- `output-dsfb-lattice/<timestamp>.zip`

That zip contains the full `<timestamp>/` directory tree so one file is enough to download the whole experiment package.

## Output Folder Convention

The output root is:

```text
crates/dsfb-lattice/output-dsfb-lattice/
```

Each execution creates a unique timestamped directory:

```text
output-dsfb-lattice/2026-03-18_10-14-51/
```

If a directory already exists for the current second, the crate waits and uses the next timestamp. Previous runs are never overwritten.

## Reproducibility

Reproducibility is handled by:

- deterministic forcing and deterministic baseline variations
- no hidden local input files
- a crate-local `Cargo.lock`
- timestamped non-overwriting output folders
- a standalone nested workspace that does not depend on root manifest edits
- a Colab notebook that can install Rust if needed, rebuild from scratch, run the binary, display the figures, and confirm the PDF / zip artifacts

## Technical Walkthrough

### Lattice Construction

`src/lattice.rs` creates a fixed-end 1D chain with `sites` masses and `sites + 1` springs. The stiffness matrix is assembled explicitly, then converted into the mass-normalized dynamical matrix `D`.

### Perturbation Injection

`src/perturbation.rs` applies:

- a point defect through one mass and one spring
- a smooth strain-like spring gradient
- a grouped perturbation with clustered spring and mass changes
- a global softening sweep

Each perturbation produces a new lattice and therefore a new operator `D'`.

### Spectral Analysis

`src/spectra.rs` uses symmetric eigendecomposition to compute eigenvalues and eigenvectors, sorts the spectrum, computes `||Delta||_2`, and records the largest per-mode eigenvalue shift. The numerical inequality check is reported as a bounded finite-matrix observation.

### Residual Construction

`src/residuals.rs` advances a deterministic damped lattice response under fixed forcing. Observations are taken in the nominal modal basis, which makes the residual comparison easy to inspect across channels.

### Envelope Detection

`src/detectability.rs` constructs an upper residual envelope from several nominal baseline runs with small forcing variations. The point-defect residual norm is then checked for first and sustained crossings.

### Report Generation

`src/report.rs` renders all PNG figures with `plotters`, writes a Markdown report, and emits a simple text PDF report directly from Rust so the run is self-contained.

### Zip Packaging

`src/io.rs` zips the completed run directory into `output-dsfb-lattice/<timestamp>.zip` without overwriting prior runs.

## Colab Workflow

Use the notebook at `crates/dsfb-lattice/dsfb_lattice_colab.ipynb`.

The notebook:

1. resolves or clones the repository
2. installs Rust if needed
3. runs `cargo clean`
4. rebuilds the crate from scratch
5. runs `--example all`
6. loads `summary.json` and the CSV outputs
7. displays all figures inline
8. confirms `report.pdf`
9. confirms the zip archive
10. ends with a careful findings summary

## Dependency Notes

The crate uses only a small set of stable dependencies:

- `nalgebra` for linear algebra and symmetric eigendecomposition
- `plotters` for PNG figures
- `csv`, `serde`, and `serde_json` for machine-readable outputs
- `clap` for the CLI
- `chrono` for timestamped run folders
- `zip` for archive packaging
- `anyhow` for explicit error propagation

The goal is maintainability and reproducibility rather than maximal numerical sophistication.
