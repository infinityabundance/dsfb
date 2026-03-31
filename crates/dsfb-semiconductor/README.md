# dsfb-semiconductor

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

`dsfb-semiconductor` is the empirical software companion for the DSFB semiconductor paper. It instantiates the paper's bounded claim: DSFB is a deterministic augmentation layer over existing semiconductor monitoring signals, not a replacement for incumbent SPC/APC/FDC infrastructure.

The current crate turns real semiconductor datasets into inspectable DSFB artifacts:

- nominal reference summaries
- residual traces
- drift traces
- slew traces
- admissibility-envelope / grammar-state traces
- DSA structural traces, scores, consistency flags, and persistence-gated alerts
- provenance-aware heuristics-bank entries with operational fields
- lead-time, sliding-window density, and pass-run nuisance proxy metrics
- calibration grid artifacts over the fixed DSFB parameter surface
- a bounded DSA calibration grid over fixed deterministic threshold and persistence settings
- a deterministic residual stateflow chart (DRSC) plus aligned trace CSV for the top boundary-activity feature
- all notebook-parity PNG figures directly from the crate
- an engineering report in Markdown, LaTeX, and PDF when `pdflatex` is available; the PDF includes the generated figures and an artifact inventory
- a ZIP bundle of the full run directory

## What math from the paper is instantiated

This crate implements the paper's core operator-facing objects with explicit saved parameters:

- nominal reference from an initial healthy passing window
- residuals `r(k) = x(k) - x_hat(k)`
- drift from a finite window difference on residual norms
- slew as the first difference of drift
- admissibility envelope radius `rho = sigma_multiplier * healthy_std`
- grammar states `Admissible`, `Boundary`, and `Violation`
- hysteretic state confirmation together with persistent boundary / violation traces
- a separate deterministic DSA layer built from rolling raw-boundary density, outward drift persistence, slew density, normalized EWMA occupancy, motif recurrence, and a directional-consistency gate
- persistence-gated DSA alerts `dsa_score >= tau` for at least `K` consecutive runs when the structural consistency constraint also holds
- run-level DSA aggregation as `any_feature_dsa_alert(k)` together with `feature_count_dsa_alert(k)`
- a provenance-aware heuristics bank built from observed grammar motifs, severity tags, action notes, and limitations
- two explicit scalar comparators: a raw residual-magnitude threshold and a univariate EWMA residual-norm comparator
- per-failure-run earliest-signal tracking and lead-time deltas against the scalar comparators for both the DSFB state logic and the DSA layer
- sliding-window density summaries for boundary / violation / threshold / EWMA occupancy
- pass-run nuisance proxies derived from SECOM pass labels
- a deterministic SECOM calibration workflow over explicit parameter grids
- a bounded DSA calibration workflow over `W`, `K`, and `tau`
- a deterministic residual stateflow chart (DRSC) that synchronizes residual/drift/slew structure, confirmation-filtered grammar state, and admissibility overlay for one emitted feature trace

The implementation is intentionally simple and deterministic. It is designed for auditability and reproducibility, not for inflated benchmark claims.

## Supported datasets

### SECOM

Implemented and verified as the first real-data benchmark path.

- Source: UCI Machine Learning Repository SECOM dataset
- Access mode: automated download via `fetch-secom`, or manual placement of `secom.zip`
- Role in this crate: real-data benchmark for residual structure, drift, slew, grammar, and motif extraction
- Non-claim: SECOM is anonymized and instance-level; it does not by itself validate chamber-mechanism attribution or full run-to-failure prognostics

Archive-layout note:

- The current distributed `secom.data` parses as `590` numeric columns.
- The UCI metadata text in `secom.names` states `591` attributes.
- This crate uses the `590` numeric columns actually present in `secom.data` and reads pass/fail labels plus timestamps separately from `secom_labels.data`.
- The exact resolved note is emitted to `secom_archive_layout.json` in every run bundle.

Expected raw-data location after fetch or manual placement:

```text
crates/dsfb-semiconductor/data/raw/secom/
  secom.zip
  secom.data
  secom_labels.data
  secom.names
```

### PHM 2018 ion mill etch

Not fully implemented in this version.

- Official benchmark page is exposed through the crate
- The crate provides a manual archive contract and a real archive probe
- Full ingestion is intentionally not claimed unless the real archive is present and verified

Expected manual archive path:

```text
crates/dsfb-semiconductor/data/raw/phm2018/phm_data_challenge_2018.tar.gz
```

Probe command:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- probe-phm2018
```

## Exact run instructions

Fetch SECOM into the crate-local raw-data directory:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- fetch-secom
```

Run the full SECOM benchmark with default parameters:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- run-secom --fetch-if-missing
```

Run the deterministic SECOM calibration grid:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- calibrate-secom \
  --healthy-pass-runs-grid 80,100,120 \
  --drift-window-grid 3,5 \
  --boundary-fraction-of-rho-grid 0.4,0.5 \
  --state-confirmation-steps-grid 1,2 \
  --persistent-state-steps-grid 1,2 \
  --density-window-grid 10 \
  --pre-failure-lookback-runs-grid 10,20
```

Run the bounded deterministic DSA calibration grid:

```bash
cargo run --manifest-path crates/dsfb-semiconductor/Cargo.toml -- calibrate-secom-dsa --fetch-if-missing
```

The bounded DSA grid is fixed at:

- `W ∈ {5, 10, 15}`
- `K ∈ {2, 3, 4}`
- `tau ∈ {2.0, 2.5, 3.0}`

Key configurable parameters:

- `--healthy-pass-runs`
- `--drift-window`
- `--envelope-sigma`
- `--boundary-fraction-of-rho`
- `--ewma-alpha`
- `--ewma-sigma-multiplier`
- `--drift-sigma-multiplier`
- `--slew-sigma-multiplier`
- `--grazing-window`
- `--grazing-min-hits`
- `--pre-failure-lookback-runs`
- `--state-confirmation-steps`
- `--persistent-state-steps`
- `--density-window`
- `--dsa-window`
- `--dsa-persistence-runs`
- `--dsa-alert-tau`

Calibration-grid arguments:

- `--healthy-pass-runs-grid`
- `--drift-window-grid`
- `--envelope-sigma-grid`
- `--boundary-fraction-of-rho-grid`
- `--ewma-alpha-grid`
- `--ewma-sigma-multiplier-grid`
- `--drift-sigma-multiplier-grid`
- `--slew-sigma-multiplier-grid`
- `--grazing-window-grid`
- `--grazing-min-hits-grid`
- `--pre-failure-lookback-runs-grid`
- `--state-confirmation-steps-grid`
- `--persistent-state-steps-grid`
- `--density-window-grid`

Current implemented baselines:

- residual-threshold baseline: `|r(k)| > rho`
- EWMA baseline: univariate EWMA on residual norms with explicit `alpha` and healthy-window thresholding

DSFB state-layer distinction:

- `DSFB Violation`: hard envelope exit `|r(k)| > rho`
- `DSA`: persistence-constrained structural accumulation from rolling structural features

Current baseline classes not implemented:

- CUSUM drift baseline
- PCA / Hotelling `T^2` / SPE-style multivariate FDC baseline
- lightweight ML anomaly baselines

## Output structure

All benchmark runs write to a repo-level timestamped directory and do not reuse an existing run folder:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_<dataset>/
```

The current SECOM pipeline writes:

```text
artifact_manifest.json
baseline_comparison_summary.json
benchmark_metrics.json
dataset_summary.json
density_metrics.csv
drsc_top_feature.csv
drifts.csv
ewma_baseline.csv
engineering_report.md
engineering_report.tex
engineering_report.pdf          # when pdflatex is available; includes figures and artifact inventory
feature_metrics.csv
figures/
grammar_states.csv
heuristics_bank.json
lead_time_metrics.csv
parameter_manifest.json
dsa_parameter_manifest.json
per_failure_run_signals.csv
per_failure_run_dsa_signals.csv
phm2018_support_status.json
dsa_metrics.csv
dsa_run_signals.csv
dsa_vs_baselines.json
residuals.csv
run_bundle.zip
run_configuration.json
secom_archive_layout.json
slews.csv
```

The current figure set includes:

- `figures/missingness_top20.png`
- `figures/drsc_top_feature.png`
- `figures/benchmark_comparison.png`
- `figures/grammar_timeline.png`
- `figures/top_feature_residual_norms.png`
- `figures/top_feature_drift.png`
- `figures/top_feature_ewma.png`
- `figures/top_feature_slew.png`

The DRSC figure is an operator-facing synchronized chart for the top boundary-activity feature in the run. Its layers are:

- normalized residual / drift / slew structure
- confirmation-filtered persistent grammar state band
- normalized admissibility-envelope occupancy together with normalized EWMA occupancy

The emitted DRSC also annotates the first persistent boundary, the first persistent violation when present in the selected window, and the failure-labeled run. This crate does not currently implement a trust scalar, so the DRSC lower layer is an admissibility overlay rather than a trust plot.

The crate emits these PNG figures directly; the notebook simply renders the contents of `run_dir/figures/*.png` inline. No notebook-only plotting logic is required to obtain them.

The calibration pipeline writes:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_secom_calibration/
  calibration_best_by_metric.json
  calibration_grid_results.csv
  calibration_report.md
  calibration_run_configuration.json
  parameter_grid_manifest.json
```

The bounded DSA calibration pipeline writes:

```text
output-dsfb-semiconductor/<timestamp>_dsfb-semiconductor_secom_dsa_calibration/
  dsa_grid_results.csv
  dsa_calibration_run_configuration.json
  dsa_parameter_grid_manifest.json
```

## Reproducibility discipline

- Every meaningful threshold and window is saved to `parameter_manifest.json`
- Fixed DSA weights, run-level aggregation choice, and the consistency rule are saved to `dsa_parameter_manifest.json`
- Dataset source and output root are saved to `run_configuration.json`
- Calibration grids are saved verbatim to `parameter_grid_manifest.json`
- The bounded DSA calibration grid is saved verbatim to `dsa_parameter_grid_manifest.json`
- Missing values are preserved at load time and then deterministically imputed with the healthy-window nominal mean before residual construction
- Repeated runs with the same inputs and parameters produce the same metrics, traces, and calibration rows, modulo different timestamped output directories

## Current empirical boundary

The crate establishes deterministic structural artifact generation on real semiconductor data, not a blanket superiority claim over scalar baselines.

- `Violation` and `DSA` are intentionally different signals: violation is a hard envelope exit, DSA is a persistence-constrained structural early warning.
- The authoritative comparison artifact for the DSA layer is `dsa_vs_baselines.json`.
- Improvement should only be claimed when that saved summary shows positive DSA lead deltas together with non-lower failure-run recall, lower nuisance than raw DSFB boundary nuisance, and no failed validation gate.
- The current default SECOM run under `output-dsfb-semiconductor/20260331_212951_308_dsfb-semiconductor_secom/` reports DSA recall `10/104`, mean lead `11.10`, pass-run nuisance `0.0273`, mean lead deltas `-8.90` vs threshold and EWMA, and compression ratio `2594.82`.
- On that saved run, nuisance improved and compression improved, but lead time did not improve and recall was not preserved. No superiority claim is made.
- The current bounded DSA grid under `output-dsfb-semiconductor/20260331_213043_869_dsfb-semiconductor_secom_dsa_calibration/` has `0/27` validation-passing configurations. The best-recall point is `W=5, K=2, tau=2.0` with recall `92/104`, mean lead `17.04`, nuisance `0.3554`, and still failed recall preservation against threshold/EWMA.
- The lead-time, density, and nuisance values remain proxy metrics on SECOM labels, not fab-qualified false-alarm or economic metrics.
- The DRSC figure is deterministic and replayable from saved traces, but it is an operator-facing visualization of current rule-based state evolution, not a probabilistic explanation layer.

## Caveats and non-claims

- This crate does not claim SEMI standards compliance or completed qualification.
- This crate does not claim universal superiority over SPC, EWMA/CUSUM, multivariate FDC, or ML baselines.
- The current comparator set is still narrow: a univariate residual-magnitude threshold plus a univariate EWMA residual-norm comparator.
- The current nuisance analysis is a pass-run proxy on SECOM labels, not a fab-qualified false-alarm-rate study.
- The current lead-time analysis is bounded to fixed lookback windows on the available labels.
- SECOM is real semiconductor data, but it is not a deployment validation dataset.
- PHM 2018 support is not claimed beyond the manual-placement contract and archive probe unless the real archive is present and verified.
- This crate does not claim Kani verification for `dsfb-semiconductor`.
- This crate does not claim `no_alloc`, `no_std`, SIMD, rayon, or other parallel-acceleration support.
- This crate does not claim SEMI E125 compatibility.
- PDF generation depends on `pdflatex` being installed in the runtime.
- The notebook file is wired to the current CLI and output paths, but this README does not claim that a live Colab execution was performed in this environment.

## Notebook

The Colab notebook lives at:

[`crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb`](/home/one/dsfb/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

It is wired to:

- bootstrap a Rust environment in Colab
- fetch or reuse the real SECOM dataset
- run the crate end to end
- inspect archive-layout, lead-time, nuisance, and PHM-support summary artifacts
- display the generated figures inline
- optionally run the bounded calibration grid
- surface the PDF report and ZIP bundle for download
