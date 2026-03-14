# dsfb-bank

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-bank/colab/dsfb_bank_repro.ipynb)

`dsfb-bank` is the executable empirical companion crate for the DSFB paper and theorem banks. It turns the theorem banks into machine-readable YAML specifications, loads them into a Rust theorem registry, runs deterministic witness demonstrations, and writes plotting-friendly CSV outputs into fresh timestamped run directories.

## Purpose

The crate exists to make the theorem banks executable and reproducible:

- theorem statements, assumptions, expected behavior, witness cases, and output intent live in YAML under `spec/`
- the Rust registry loads those YAML files as the source of truth
- deterministic bank-specific runners emit theorem-level CSV artifacts for `core`, `dsfb`, `dscd`, `tmtr`, `add`, `srd`, and `hret`
- realization-space YAML drives realization CSV exports under `realizations/`
- every run creates a new timestamped output directory so previous empirical figures are never overwritten

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

## Reproducibility Model

Reproducibility is enforced by construction:

- theorem banks are checked into the crate as YAML
- witness generators are deterministic
- outputs are timestamped and never overwritten
- `manifest.json` records the crate version, git hash when available, command line, theorem IDs, generated files, and component counts
- `run_summary.md` records theorem counts, pass/fail totals, generated CSVs, and realization-space outputs

## Google Colab Notebook

`colab/dsfb_bank_repro.ipynb` is the reproducible figure factory for the theorem banks. It:

- installs or verifies Rust in the current Colab session
- operates relative to the repository root
- runs a clean build of `dsfb-bank`
- runs `cargo run -p dsfb-bank -- --all`
- captures the fresh timestamped output directory from the current notebook session
- validates the output inventory before plotting
- loads only CSVs from that fresh run
- generates empirical figures and summary tables directly from those outputs
- ends with an explicit reproducibility confirmation stating that the build, run, and figures all came from the current session

## Figure Generation

The notebook generates:

- theorem coverage figures
- per-bank behavior figures for DSFB, DSCD, TMTR, ADD, SRD, and HRET
- a full DSFB pipeline trace figure
- a core-theorem dashboard

Every figure is sourced from fresh CSV outputs generated during the current Colab execution.
