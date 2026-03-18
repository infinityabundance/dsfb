[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-lattice/dsfb_lattice_colab.ipynb)

# dsfb-lattice

`dsfb-lattice` is a standalone nested Rust crate under `crates/dsfb-lattice`. It exists to provide a bounded, reproducible, and inspectable empirical demonstrator for selected mathematical ideas from the paper *Deterministic Structural Inference in Solid-State Systems: A DSFB Engine for Crystal Lattices, Phonons, and Structural Forensics*.

The crate is intentionally modest. It does not attempt ab initio crystal simulation, material calibration, or universal defect identification. Instead, it implements a fixed-end harmonic 1D lattice, applies controlled perturbations, compares nominal and perturbed spectra, simulates deterministic observations, computes raw and normalized residual / drift / slew statistics, builds explicit baseline-derived residual envelopes, runs a bounded synthetic stress-test suite with additive noise and predictor mismatch, locks a canonical evaluation layer for run-to-run comparison, executes a small heuristic-bank retrieval pass with explicit descriptor weights, generates a controlled failure/degradation map, and writes timestamped artifacts suitable for paper support and notebook replay.

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
   A residual-norm envelope is estimated from several deterministic nominal baseline runs with small forcing variations. The point-defect example is then checked pointwise in time against that envelope, so first crossing is determined by the same-time condition `||r(t)|| > E(t)` rather than by comparing global peaks. The envelope is reported explicitly as baseline-derived, regime-specific, and non-universal.

7. Normalized residual metrics
   Alongside raw residual norms, the crate reports a normalized residual norm
   `||r(t)||_2 / (||y_pred(t)||_2 + epsilon)`
   and an energy-relative ratio
   `sum_t ||r(t)||_2^2 / (sum_t ||y_pred(t)||_2^2 + epsilon)`.
   These are intended only as transparent within-crate comparison aids.

8. Group-mode correlation
   Residual covariance is computed across observed modal channels so the grouped perturbation can be compared against the more localized point-defect case.

9. Controlled pressure test
   The point-defect detectability path is also rerun under four synthetic cases: clean, noise only, predictor mismatch only, and noise plus mismatch. An additional optional ambiguity case mixes a weak localized defect with a weak smooth gradient so descriptor-space retrieval can become near-tied. Each case uses its own baseline-derived envelope under the same settings, and the outputs distinguish clean structural detectability from stressed early low-margin crossings.

10. Canonical evaluation quantities
   A fixed set of spectral, residual, temporal, detectability, correlation, and envelope-provenance quantities is exported as the comparison backbone for this synthetic benchmark. The crate treats them as canonical for run-to-run comparability inside `dsfb-lattice`, while still allowing auxiliary metrics to be added later.

11. Minimal heuristic bank
   Existing experiment descriptors are stored as simple heuristic entries with admissibility tags. Observed cases are ranked against them with a weighted L1 distance, and the retrieval layer now exposes `unambiguous`, `near_tie`, and `ambiguous` tiers rather than forcing brittle yes/no ambiguity decisions.

12. Controlled failure map
   A small synthetic grid over noise and predictor mismatch makes explicit where the method is cleanly detected, where detection is degraded or ambiguity-dominated, and where it stops detecting in this toy setting. The map is not presented as a universal operating boundary, and it is used in part to show that detectability is not monotone in raw residual size alone.

13. Softening precursor toy example
   A global spring-softening sweep pushes the smallest eigenvalue toward zero and tracks the resulting residual / drift / slew growth. This is a toy precursor study only.

## What The Crate Demonstrates

- a clean assembly of a harmonic lattice operator
- controlled perturbation of that operator
- numerical spectrum and modal comparison
- residual, drift, and slew generation from deterministic simulations
- normalized residual metrics for more interpretable cross-regime comparison inside this toy crate
- envelope crossing logic with finite-time detectability in a toy setting
- explicit baseline-derived envelope provenance and parameters
- a controlled synthetic stress-test suite under additive observation noise, predictor mismatch, and an optional ambiguity case
- canonical evaluation quantities that form the crate's comparison backbone across runs
- a minimally algorithmic heuristic bank with explicit descriptor fields, explicit weighted-L1 coefficients, admissibility filtering, and ambiguity signaling
- a controlled synthetic failure/degradation map over noise and predictor mismatch
- a covariance contrast between localized and grouped perturbations
- a softening sweep consistent with an approaching-instability interpretation

## What It Does Not Demonstrate

- first-principles phonon prediction for real materials
- universal defect identifiability
- calibrated sensor physics or measurement noise models
- universal thresholds for detectability
- any claim that the synthetic pressure test constitutes statistical validation
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
Simulates deterministic lattice responses, computes raw and normalized residual metrics, and can inject controlled additive observation noise.

`src/detectability.rs`
Builds explicit baseline-derived envelopes with provenance metadata, reports first / sustained crossings, and adds post-crossing persistence plus an interpretive layer for structural vs stress-confounded crossings.

`src/canonical.rs`
Defines the canonical evaluation quantities and flattens them into CSV-friendly rows.

`src/heuristics.rs`
Builds the transparent heuristic-bank descriptors, admissibility tags, weighted-L1 similarity scores, and tiered ambiguity signaling.

`src/failure_map.rs`
Runs the controlled stress grid used for the failure/degradation map and exports structured rows and summaries with explicit detected / degraded / ambiguous / not-detected labels.

`src/io.rs`
Creates timestamped run folders, writes JSON / CSV outputs, and zips completed runs.

`src/report.rs`
Generates PNG figures, a Markdown report, and a paginated PDF report with fixed margins, an artifact inventory, and embedded figure pages.

`src/utils.rs`
Small shared helpers for ranges, deterministic synthetic noise, PDF escaping, path formatting, and covariance summaries.

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

Key optional controls for the research extension include:

```bash
cargo run --release --manifest-path crates/dsfb-lattice/Cargo.toml -- \
  --example all \
  --normalization-epsilon 1e-6 \
  --pressure-test-enabled true \
  --pressure-test-noise-std 0.018 \
  --pressure-test-predictor-spring-scale 0.97 \
  --pressure-test-seed 20260318 \
  --pressure-test-include-ambiguity-case true \
  --pressure-test-ambiguity-point-mass-scale 1.08 \
  --pressure-test-ambiguity-point-spring-scale 0.96 \
  --pressure-test-ambiguity-strain-strength 0.14 \
  --failure-map-enabled true \
  --heuristics-enabled true \
  --heuristics-ambiguity-tolerance 0.18
```

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

5. Controlled stress-test suite
   Replays the point-defect detectability pipeline in clean, noise-only, mismatch-only, and combined synthetic settings with case-specific baseline-derived envelopes, and optionally adds a mixed-signature ambiguity case for descriptor-space ranking.

6. Canonical metrics and heuristic ranking
   Exports a fixed set of canonical evaluation quantities and uses them to drive a small admissibility-aware heuristic-bank ranking step.

7. Failure / degradation map
   Sweeps a weak localized defect and a mixed-signature scenario over noise and predictor mismatch to show where the method stays structurally legible, where it degrades or becomes ambiguity-dominated, and where it fails to detect in this synthetic setting.

8. Softening sweep
   Reduces global spring scale over a grid and tracks the smallest eigenvalue together with residual / drift / slew maxima.

## Artifacts Produced

Every run writes at least the following files into a new timestamped run directory:

- `config.json`
- `summary.json`
- `canonical_metrics.csv`
- `canonical_metrics.json`
- `canonical_metrics_summary.csv`
- `canonical_metrics_summary.json`
- `heuristic_ranking.csv`
- `heuristic_ranking.json`
- `heuristic_rankings.csv`
- `heuristic_rankings.json`
- `metrics.csv`
- `eigenvalues_nominal.csv`
- `eigenvalues_perturbed.csv`
- `residual_timeseries.csv`
- `normalized_residual_norm_timeseries.csv`
- `drift_timeseries.csv`
- `slew_timeseries.csv`
- `covariance.csv`
- `envelope_timeseries.csv`
- `failure_map.csv`
- `failure_map.json`
- `pressure_test_summary.csv`
- `pressure_test_summary.json`
- `softening_sweep.csv`
- `nominal_observations.csv`
- `figure_01_nominal_vs_point_spectrum.png`
- `figure_02_spectral_shift_comparison.png`
- `figure_03_residual_timeseries_point_defect.png`
- `figure_04_drift_slew_timeseries_point_defect.png`
- `figure_05_detectability_envelope.png`
- `figure_06_covariance_heatmap.png`
- `figure_07_softening_precursor.png`
- `figure_08_pressure_test_raw_residual_comparison.png`
- `figure_09_pressure_test_normalized_residual_comparison.png`
- `figure_10_pressure_test_detectability_summary.png`
- `figure_11_failure_map_status.png`
- `report.md`
- `report.pdf`
  This PDF includes fixed-margin text pages, an inventory of the generated run artifacts, and one embedded page for each PNG figure.

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
- fixed RNG seeds recorded for the synthetic noisy pressure-test cases
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

`src/residuals.rs` advances a deterministic damped lattice response under fixed forcing. Observations are taken in the nominal modal basis, which makes the residual comparison easy to inspect across channels. The same module also computes the normalized residual norm `||r(t)||_2 / (||y_pred(t)||_2 + epsilon)` and the residual energy ratio over the observation window.

### Envelope Detection

`src/detectability.rs` constructs an upper residual envelope from several nominal baseline runs with small forcing variations. The point-defect residual norm is then checked for first and sustained crossings using a same-time comparison, and the summary distinguishes global peaks from the values observed at the first crossing itself. The module also records post-crossing persistence duration, post-crossing fraction, and peak margin after crossing so stressed early low-margin events can be labeled separately from cleaner structural separation. The envelope provenance records the baseline run count, sigma multiplier, additive floor, and the fact that the threshold is not universal.

### Controlled Pressure Test

`src/lib.rs` also runs a bounded synthetic pressure test around the point-defect detectability path. The measurement side can receive additive Gaussian noise from a fixed-seed deterministic RNG, while the predictor can use a slightly mismatched global spring scale. The resulting clean / noise-only / mismatch-only / noise-plus-mismatch cases each get their own baseline-derived envelope, CSV summary, JSON summary, and comparison plots. An optional ambiguity case blends a weak localized defect with a weak smooth gradient so the heuristic ranking can surface near-tied candidate interpretations. The pressure-test outputs explicitly separate pointwise crossing from its interpretation, so a very early small-margin crossing under stress is not silently treated as clean structural detectability.

### Canonical Evaluation Layer

`src/canonical.rs` defines the canonical evaluation quantities by which `dsfb-lattice` runs are compared. These quantities are intended to be the crate's comparison backbone for synthetic benchmark runs: future revisions may add more metrics, but preserving the canonical layer helps maintain run-to-run comparability.

### Heuristic-Bank Retrieval

`src/heuristics.rs` turns the previously conceptual heuristic bank into a small executable retrieval object. It stores compact descriptors for the reference perturbation classes, filters candidates by simple admissibility tags, ranks candidates with an explicit weighted L1 distance, exposes the descriptor fields and weights in the serialized outputs, and now uses a tiered `unambiguous` / `near_tie` / `ambiguous` interpretation instead of a brittle single threshold.

### Failure / Degradation Map

`src/failure_map.rs` runs a controlled grid over additive noise and predictor mismatch for two synthetic scenarios: a weak localized defect and a mixed-signature ambiguous case. Each grid point records canonical metrics, detectability interpretation, heuristic ambiguity tier, and a final semantic status such as `detected`, `degraded`, `ambiguous`, `degraded_ambiguous`, or `not_detected`. The point of the artifact is to make degradation legible, not to claim a universal operating boundary. In particular, the exported grid makes visible that larger residuals do not by themselves guarantee cleaner detectability when envelope construction, stress, and descriptor-space ambiguity change together.

### Report Generation

`src/report.rs` renders all PNG figures with `plotters`, writes a Markdown report, and emits a multi-page PDF report directly from Rust. The PDF uses fixed margins, wraps long text safely, lists the generated artifacts, and embeds every PNG figure on its own page so the report remains self-contained.

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

The notebook defaults to cloning `https://github.com/infinityabundance/dsfb.git`. If you need to run it against a fork, set `DSFB_LATTICE_REPO_URL` before execution or change the `REPO_URL` assignment in the first setup cell.

## Dependency Notes

The crate uses only a small set of stable dependencies:

- `nalgebra` for linear algebra and symmetric eigendecomposition
- `plotters` for PNG figures
- `image` and `flate2` for self-contained PDF image embedding
- `csv`, `serde`, and `serde_json` for machine-readable outputs
- `clap` for the CLI
- `chrono` for timestamped run folders
- `zip` for archive packaging
- `anyhow` for explicit error propagation

The goal is maintainability and reproducibility rather than maximal numerical sophistication.
