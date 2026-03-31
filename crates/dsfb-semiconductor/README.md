# dsfb-semiconductor

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

`dsfb-semiconductor` is the empirical software companion for the DSFB semiconductor paper. It instantiates the paper's bounded claim: DSFB is a deterministic augmentation layer over existing semiconductor monitoring signals, not a replacement for incumbent SPC/APC/FDC infrastructure.

The current crate turns real semiconductor datasets into inspectable DSFB artifacts:

- nominal reference summaries
- residual traces
- drift traces
- slew traces
- admissibility-envelope / grammar-state traces
- provenance-aware heuristics-bank entries
- figures
- an engineering report in Markdown, LaTeX, and PDF when `pdflatex` is available
- a ZIP bundle of the full run directory

## What math from the paper is instantiated

This crate implements the paper's core operator-facing objects with explicit saved parameters:

- nominal reference from an initial healthy passing window
- residuals `r(k) = x(k) - x_hat(k)`
- drift from a finite window difference on residual norms
- slew as the first difference of drift
- admissibility envelope radius `rho = sigma_multiplier * healthy_std`
- grammar states `Admissible`, `Boundary`, and `Violation`
- a provenance-aware heuristics bank built from observed grammar motifs
- two explicit scalar comparators: a raw residual-magnitude threshold and a univariate EWMA residual-norm comparator

The implementation is intentionally simple and deterministic. It is designed for auditability and reproducibility, not for inflated benchmark claims.

## Supported datasets

### SECOM

Implemented and verified as the first real-data benchmark path.

- Source: UCI Machine Learning Repository SECOM dataset
- Access mode: automated download via `fetch-secom`, or manual placement of `secom.zip`
- Role in this crate: real-data benchmark for residual structure, drift, slew, grammar, and motif extraction
- Non-claim: SECOM is anonymized and instance-level; it does not by itself validate chamber-mechanism attribution or full run-to-failure prognostics

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

Current implemented baselines:

- residual-threshold baseline: `|r(k)| > rho`
- EWMA baseline: univariate EWMA on residual norms with explicit `alpha` and healthy-window thresholding

Current baseline classes not implemented:

- CUSUM drift baseline
- PCA / Hotelling `T^2` / SPE-style multivariate FDC baseline
- lightweight ML anomaly baselines

## Output structure

All benchmark runs write to a repo-level timestamped directory and do not reuse an existing run folder:

```text
output-dsfb-semiconductor/<timestamped-run-folder>/
```

The current SECOM pipeline writes:

```text
artifact_manifest.json
benchmark_metrics.json
dataset_summary.json
drifts.csv
ewma_baseline.csv
engineering_report.md
engineering_report.tex
engineering_report.pdf          # when pdflatex is available
feature_metrics.csv
figures/
grammar_states.csv
heuristics_bank.json
parameter_manifest.json
phm2018_support_status.json
residuals.csv
run_bundle.zip
run_configuration.json
slews.csv
```

## Reproducibility discipline

- Every meaningful threshold and window is saved to `parameter_manifest.json`
- Dataset source and output root are saved to `run_configuration.json`
- Missing values are preserved at load time and then deterministically imputed with the healthy-window nominal mean before residual construction
- Repeated runs with the same inputs and parameters produce the same metrics and traces, modulo different timestamped output directories

## Caveats and non-claims

- This crate does not claim SEMI standards compliance or completed qualification.
- This crate does not claim universal superiority over SPC, EWMA/CUSUM, multivariate FDC, or ML baselines.
- The current comparator set is still narrow: a univariate residual-magnitude threshold plus a univariate EWMA residual-norm comparator.
- SECOM is real semiconductor data, but it is not a deployment validation dataset.
- PHM 2018 support is not claimed beyond the manual-placement contract and archive probe unless the real archive is present and verified.
- PDF generation depends on `pdflatex` being installed in the runtime.
- The notebook file is wired to the current CLI and output paths, but this README does not claim that a live Colab execution was performed in this environment.

## Notebook

The Colab notebook lives at:

[`crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb`](/home/one/dsfb/crates/dsfb-semiconductor/notebooks/dsfb_semiconductor_secom_colab.ipynb)

It is wired to:

- bootstrap a Rust environment in Colab
- fetch or reuse the real SECOM dataset
- run the crate end to end
- display the generated figures inline
- surface the PDF report and ZIP bundle for download
