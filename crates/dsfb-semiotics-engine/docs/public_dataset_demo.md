# Public Dataset Demo

This crate now ships a real, executed NASA public-dataset path.

Primary dataset:

- NASA Milling (`3. Milling.zip`) from the NASA/PHM data repository

Secondary dataset:

- NASA IMS Bearings (`4. Bearings.zip`) from the NASA/PHM data repository

These demos are public-data illustrations of deterministic structural interpretation only. They are
not field validation, diagnosis certification, or benchmark-superiority claims.

## Why These Two

- `nasa_milling` is the primary path because the NASA archive is small enough to fetch quickly and
  still contains real progressive wear structure across repeated machining runs.
- `nasa_bearings` is the secondary path because it is recognizable to condition-monitoring
  reviewers and gives a real replay path from the NASA IMS run-to-failure set without changing the
  crate’s conservative interpretation posture.

## One-Command Workflow

Run the full executed pipeline for both NASA datasets:

```bash
cd crates/dsfb-semiotics-engine
just demo-public-dataset
```

The same path is available directly through the dedicated binary:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --phase all
```

Run only the primary NASA Milling path:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --dataset nasa_milling \
  --phase all
```

Run only the secondary NASA Bearings path:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --dataset nasa_bearings \
  --phase all
```

## Automated Fetch

The authoritative NASA fetch and raw-summary extraction step is scripted in:

- `tools/fetch_public_dataset.py`

That script:

- downloads the NASA archive when it is not already available locally
- verifies archive checksums
- writes deterministic crate-local raw-summary CSV slices under `data/public_dataset/raw/`

For testability and offline reruns, the crate also keeps the deterministic raw-summary CSV slices
checked in after they have been generated once from the real NASA sources.

## Deterministic Preprocessing

The DSFB-compatible observed/predicted CSV generation step is scripted in:

- `tools/preprocess_public_dataset.py`

Outputs are written under:

- `data/processed/nasa_milling/`
- `data/processed/nasa_bearings/`

The preprocessing policy is fixed and documented:

- NASA Milling:
  case 11 only
  one DSFB time step per machining run
  observed channels are normalized per-run RMS summaries of `vib_table`, `AE_spindle`, and
  `smcDC`
  predicted channels are causal trailing means of those normalized features
- NASA Bearings:
  IMS set 1 only
  the last 64 one-second snapshots from `1st_test`
  observed channels are normalized failing-bearing summaries derived from channels 5-8 as defined
  in the IMS readme
  predicted channels are causal trailing means of those normalized features

## Generated Outputs

The executed artifact roots are:

- `artifacts/public_dataset_demo/nasa_milling/latest/`
- `artifacts/public_dataset_demo/nasa_bearings/latest/`

Each run writes:

- PDF report
- PNG figures
- ZIP bundle
- replay event CSV/JSON
- replay ASCII snapshot
- replay input CSV copies

Representative checked-in sample artifact subsets are available at:

- `examples/public_dataset_demo/nasa_milling/`
- `examples/public_dataset_demo/nasa_bearings/`

Those sample subsets include:

- `report/dsfb_semiotics_engine_report.pdf`
- `figures/*.png`
- `manifest.json`
- `replay/replay_events.csv`
- `replay_inputs/observed.csv`
- `replay_inputs/predicted.csv`

Sample-versus-full packaging:

- `examples/public_dataset_demo/...` is the lightweight sample-grade inspection surface checked
  into the repository
- `artifacts/public_dataset_demo/.../latest/` is the full regenerated artifact surface for the
  current local run
- sample artifacts are intended for immediate inspection; full artifacts are the reproducible
  regeneration target

## Dashboard Replay

Replay the primary NASA Milling processed CSV pair through the deterministic dashboard:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv crates/dsfb-semiotics-engine/data/processed/nasa_milling/observed.csv \
  --predicted-csv crates/dsfb-semiotics-engine/data/processed/nasa_milling/predicted.csv \
  --scenario-id nasa_milling_public_demo \
  --time-column time \
  --dashboard-replay-csv \
  --dashboard-scenario nasa_milling_public_demo
```

Replay the secondary NASA Bearings processed CSV pair:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv crates/dsfb-semiotics-engine/data/processed/nasa_bearings/observed.csv \
  --predicted-csv crates/dsfb-semiotics-engine/data/processed/nasa_bearings/predicted.csv \
  --scenario-id nasa_bearings_public_demo \
  --time-column time \
  --dashboard-replay-csv \
  --dashboard-scenario nasa_bearings_public_demo
```

## Source Links

- NASA Milling: `https://phm-datasets.s3.amazonaws.com/NASA/3.+Milling.zip`
- NASA Bearings: `https://phm-datasets.s3.amazonaws.com/NASA/4.+Bearings.zip`

## Interpretation Boundary

- The crate is replaying real NASA data through the same deterministic engine path used for
  synthetic and CSV runs.
- The crate is not claiming that the returned syntax, grammar, or semantics are externally
  validated failure diagnoses.
- The public dataset demos exist to show real-data ingestion, replay, artifact generation, and
  operator-legible structural transitions under a fully reproducible workflow.
