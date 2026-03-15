# dsfb-bank

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-bank/colab/dsfb_bank_repro.ipynb)

`dsfb-bank` is the executable empirical companion crate for the DSFB paper and theorem banks. It turns the theorem banks into machine-readable YAML specifications, loads them into a Rust theorem registry, runs deterministic witness demonstrations, and writes fresh timestamped CSV, manifest, summary, and notebook-ready figure artifacts for every run.

## Citation

If you use `dsfb-bank` or cite the associated paper artifact, cite:

de Beer, R. (2026). *Alternative Deterministic Structural Inference: The DSFB Stack for Reconstruction, Causal Architecture, Trust Recursion, and Historical Replay* (v1.0). Zenodo. https://doi.org/10.5281/zenodo.19028440

## Purpose

The crate exists to make the theorem banks executable and reproducible:

- theorem statements, assumptions, expected behavior, witness cases, and output intent live in YAML under `spec/`
- the Rust registry loads those YAML files as the source of truth
- deterministic bank-specific runners emit theorem-level CSV artifacts for `core`, `dsfb`, `dscd`, `tmtr`, `add`, `srd`, and `hret`
- realization-space YAML drives realization CSV exports under `realizations/`
- every run creates a new timestamped output directory so previous empirical figures are never overwritten

## How a Run Works

The current code path is intentionally simple and auditable:

1. load the theorem-bank and realization-space specifications from `spec/`
2. resolve the CLI selection for `--all`, `--core`, `--bank <component>`, or `--list`
3. create a fresh timestamped run directory under `output-dsfb-bank/YYYY-MM-DD_HH-MM-SS/`
4. execute deterministic witness generators for the selected theorems
5. write one theorem CSV per theorem into the matching component directory
6. write realization-space CSVs under `realizations/`
7. derive `component_summary.csv` from the emitted theorem CSV rows
8. derive `run_summary.md` from the same theorem-row aggregation
9. collect the generated file inventory and write `manifest.json`

This order matters. The summary layers are not hand-authored and they are not maintained separately from the theorem outputs. The code writes the witness rows first and only then derives the component summary, run summary, and manifest from those actual rows.

## Why the Artifact Is Structured This Way

The crate is opinionated because the paper artifact has to be reviewable:

- theorem rows are the primary empirical record
- `component_summary.csv`, `manifest.json`, and `run_summary.md` are secondary summaries derived from those rows
- passing, boundary, and violating witnesses are all kept visible so the artifact does not look curated to always succeed
- fresh timestamped output directories prevent accidental reuse of prior runs
- the Colab notebook is designed to prove that its figures come from the current run rather than from cached outputs

The result is an artifact that is deterministic, assumption-sensitive, and easy to audit end to end.

## Theorem Bank Architecture

The crate is organized around four layers:

1. `spec/*.yaml`
   The authoritative theorem-bank and realization-space specifications.
2. `src/registry.rs`
   The loader and registry for theorem specs and realization specs.
3. `src/runners/`
   Deterministic empirical witness generators for each theorem bank and the core layer.
4. `src/output.rs`, `src/run_summary.rs`, `src/timestamp.rs`
   Output layout, manifests, run summaries, logs, and timestamped run-directory creation.

Supporting deterministic witness generators live in `src/sim/` for signals, trust dynamics, causal graphs, regimes, anomalies, and trace replay.

## Theorem CSV Contract

Every theorem CSV row now carries the same empirical case metadata:

- `theorem_id`
- `theorem_name`
- `component`
- `case_id`
- `case_class` with values such as `passing`, `boundary`, and `violating`
- `assumption_satisfied`
- `expected_outcome`
- `observed_outcome`
- `pass`
- `notes`

Bank-specific plotting fields are retained and extended per component. The artifact intentionally includes explicit assumption-violating witness families for:

- core cross-layer theorem non-applicability witnesses
- non-injective DSFB observation maps
- cycle-creating DSCD edge insertions
- trust-increasing TMTR updates
- ADD threshold misconfiguration
- invalid SRD regime labels or coarsening semantics
- non-injective HRET observation maps

These rows are not presented as theorem counterexamples. They are marked as non-admissible or assumption-violating witnesses to show why the theorem hypotheses matter.

## Passing, Boundary, and Violating

Every theorem-row witness is classified into one of three explicit case classes:

- `passing`
  The theorem assumptions are satisfied and the deterministic witness exhibits the expected admissible behavior.
- `boundary`
  The witness sits on an admissible edge case, equality case, or threshold case where the theorem still applies but the behavior is intentionally close to the limit of the stated hypotheses.
- `violating`
  The witness intentionally breaks one or more assumptions or admissibility conditions. These rows are theorem-non-applicable witnesses, not claims that the theorem is false.

The `pass` column should be read with that framing:

- `pass = true` means the emitted witness behaved as expected for its admissibility class.
- `pass = false` means the row is an intentional assumption-violating or non-admissible witness whose theorem-level guarantee is not expected to hold.

This is especially important for the DSFB and core layers:

- DSFB violating rows illustrate ambiguity under non-injective or non-image observations, where exact recovery is not admissible.
- Core violating rows illustrate the same assumption sensitivity for the 11 cross-layer theorems, so the core layer is not curated to look artificially all-clean.

TMTR is treated the same way: the bank includes multiple deterministic monotonicity-breaking or trust-gap-breaking witnesses so the stability layer visibly shows what happens when its hypotheses are not met.

## Machine-Readable Spec System

Each theorem spec in `spec/` contains:

- `id`
- `component`
- `ordinal`
- `title`
- `statement_summary`
- `assumptions`
- `variables`
- `expected_behavior`
- `witness_cases`
- `runner`
- `outputs`

The Rust code does not reduce the theorem banks to names only. The explicit statements, assumptions, and expected behavior are loaded from YAML and used to build the registry and the run inventory.

## CLI

Run the full empirical artifact:

```bash
cargo run -p dsfb-bank -- --all
```

Run only the core theorem layer:

```bash
cargo run -p dsfb-bank -- --core
```

Run a single theorem bank:

```bash
cargo run -p dsfb-bank -- --bank dsfb
```

List available theorem demos and realization outputs:

```bash
cargo run -p dsfb-bank -- --list
```

Override the output root:

```bash
cargo run -p dsfb-bank -- --all --output /tmp/output-dsfb-bank
```

Provide a deterministic seed:

```bash
cargo run -p dsfb-bank -- --all --seed 7
```

## Output Directory Structure

The default output root is the repository root:

```text
output-dsfb-bank/
  YYYY-MM-DD_HH-MM-SS/
    component_summary.csv
    manifest.json
    run_summary.md
    logs.txt
    core/
    dsfb/
    dscd/
    tmtr/
    add/
    srd/
    hret/
    realizations/
```

Each theorem is written to its own CSV file inside the corresponding component directory. Realization-space exports are written to:

- `realizations/dsfb_realizations.csv`
- `realizations/dscd_realizations.csv`
- `realizations/tmtr_realizations.csv`
- `realizations/add_realizations.csv`
- `realizations/srd_realizations.csv`
- `realizations/hret_realizations.csv`
- `realizations/all_realizations.csv`

## Summary Artifacts

Each fresh run also emits summary layers derived from the actual theorem rows:

- `component_summary.csv`
  One row per component (`core`, `dsfb`, `dscd`, `tmtr`, `add`, `srd`, `hret`) with:
  - `theorem_count`
  - `cases`
  - `pass`
  - `fail`
  - `boundary`
  - `violating`
  - `passing`
  - `assumption_satisfied_count`
  - `assumption_violated_count`
- `manifest.json`
  Records crate version, git hash when available, command line, theorem IDs, generated files, per-component counts, and `case_class_counts` both globally and by component.
- `run_summary.md`
  Human-readable run narrative including theorem counts, global case-class counts, by-component case-class counts, pass/fail and assumption summaries, and a reminder that violating rows are non-admissible witnesses rather than theorem falsifications.

These three summary layers should agree exactly because they are all derived from the same emitted theorem CSV rows.

## Reproducibility Model

Reproducibility is enforced by construction:

- theorem banks are checked into the crate as YAML
- witness generators are deterministic
- outputs are timestamped and never overwritten
- `component_summary.csv`, `manifest.json`, and `run_summary.md` are all derived from the emitted theorem rows rather than hardcoded totals
- `manifest.json` records the crate version, git hash when available, command line, theorem IDs, generated files, component counts, and case-class counts
- `run_summary.md` records theorem counts, global and per-component case-class counts, pass/fail totals, generated CSVs, and realization-space outputs
- theorem CSVs expose explicit `passing` / `boundary` / `violating` case classes for downstream analysis and plotting

## Google Colab Notebook

`colab/dsfb_bank_repro.ipynb` is the reproducible figure factory for the theorem banks. It:

- installs or verifies Rust in the current Colab session
- operates relative to the repository root
- runs `cargo clean -p dsfb-bank` followed by `cargo build -p dsfb-bank`
- runs `cargo run -p dsfb-bank -- --all`
- proves that the selected `output-dsfb-bank/YYYY-MM-DD_HH-MM-SS/` directory was created in the current notebook session
- validates `manifest.json`, `run_summary.md`, `component_summary.csv`, `logs.txt`, theorem CSV counts, realization CSVs, and theorem-row schemas before plotting
- validates that `component_summary.csv`, `manifest.json`, `run_summary.md`, and notebook tables are mutually consistent with the fresh theorem CSV aggregation
- loads only CSVs from that fresh run
- generates the paper-critical figures:
  - Full DSFB Stack Timeline
  - Theorem Bank Coverage Heatmap
  - TMTR Trust Convergence
- generates additional paper-critical figures:
  - DSFB Structural Inference Flow
  - DSFB–TMTR Convergence Phase Plot
- generates a dedicated core theorem dashboard and an assumption-violating witness summary section
- retains additional per-bank behavior figures for DSFB, DSCD, ADD, SRD, and HRET
- saves every figure as an individual PNG under `output-dsfb-bank/YYYY-MM-DD_HH-MM-SS/figures/`
- writes `figure_manifest.csv` recording figure id, title, source CSVs, theorem families, notebook section, and output PNG path
- creates `dsfb-bank-YYYY-MM-DD_HH-MM-SS.zip` in the same run directory without removing the original CSV or PNG files
- ends with summary tables and an explicit reproducibility confirmation stating that the build, run, figures, and zip all came from the current session

In other words, the notebook is not just a plotting notebook. It is a current-session provenance check over the crate outputs.

For every major figure section, the notebook explicitly states:

- the exact fresh source CSV path(s)
- that the figure is generated from the current-session run
- which theorem family the figure supports
- where the saved PNG lives after export

## Figure Generation

The notebook generates:

- the full DSFB stack timeline hero figure
- the theorem-bank coverage heatmap
- TMTR trust convergence
- the DSFB structural inference flow diagram
- the DSFB–TMTR convergence phase plot
- the core theorem dashboard
- per-bank behavior figures for DSFB, DSCD, ADD, SRD, and HRET

Every figure is sourced from fresh CSV outputs generated during the current Colab execution, and every figure PNG remains available individually after the notebook finishes.

Figure PNGs are written to:

```text
output-dsfb-bank/YYYY-MM-DD_HH-MM-SS/figures/
```

The notebook also writes `figure_manifest.csv` beside the fresh run outputs so reviewers can map each PNG back to its source CSVs and notebook section.

## Zip Archive Contents

At notebook completion, the fresh run directory also contains:

```text
output-dsfb-bank/YYYY-MM-DD_HH-MM-SS/dsfb-bank-YYYY-MM-DD_HH-MM-SS.zip
```

The zip is a convenience artifact and does not replace the original files. It contains:

- all theorem CSVs
- all realization CSVs
- all saved figure PNGs under `figures/`
- `manifest.json`
- `run_summary.md`
- `logs.txt` when present
- `component_summary.csv`
- `figure_manifest.csv`

## Local Reproduction

Local end-to-end execution from the repository root:

```bash
cargo clean -p dsfb-bank
cargo build -p dsfb-bank
cargo run -p dsfb-bank -- --all
```

Run the crate tests:

```bash
cargo test -p dsfb-bank
```

After the run completes, inspect the fresh timestamped directory under `output-dsfb-bank/` or open the Colab notebook to reproduce the figure-generation and archive steps from scratch.
