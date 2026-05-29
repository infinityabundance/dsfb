# Operator data request — what DSFB needs to evaluate on your plant

> The concrete input contract for a DSFB evaluation on real plant data, and — crucially — **how to evaluate
> without your raw data ever leaving your control**. Generic (no agency/program/vendor). This is the
> human-readable precursor to the machine `PlantDataContractV1` (Wave 6). Advisory; read-only.

## The headline: your data does not have to leave your control
**You run the court locally and share only a redacted, hash-linked evidence bundle.** DSFB is a read-only Rust
binary (optionally a container). It ingests your historian export, emits a sealed Court Record, and you choose
to share **only** the redacted evidence summary (episode structure + hash roots + metrics + non-claims — **no
raw time-series, no real tag names, no raw values**). The hash roots let a reviewer confirm reproducibility
without ever seeing your data. (This is the `ConfidentialEvaluationBundleV1` / `PartnerDataEscrowProtocolV1`
path in Wave 6.)

## Minimum (enough for M0–M2)
- **`tags.csv`** — long format: `timestamp, tag, value` (any sample rate; gaps OK — ragged multi-rate is handled).
- **A baseline window** — a span of known-good / normal operation (even an approximate one).

## Recommended (unlocks more witnesses)
| File | Enables |
|---|---|
| `units.csv` (tag → engineering unit) | the unit-consistency court; physical balances |
| `controllers.csv` / `setpoints.csv` (PV/MV/SP, controller mode) | control-loop context; setpoint-vs-process separation; mode-transition guarding |
| `topology.json` (unit graph + residence times) | process-topology + fault-propagation candidates |
| `alarms.csv` | alarm-flood compression vs your existing alarm system |
| `maintenance.csv` | maintenance-window overlays; calibration-event witnesses |
| `batches.csv` (campaign / genealogy) | batch-phase envelopes; genealogy recurrence |
| `lab_samples.csv` (sparse lab/manual results) | manual-sample bridge; soft-sensor witnesses |
| `data_dictionary.toml` | observability map (which fault classes your instrumentation can even see) |

## What we will tell you honestly *before* analysis
DSFB first runs a **data-readiness grade** (`IndustrialDataReadinessCourtV1`) → `Ready` /
`ReadyWithCaveats` / `NotReadyMissingCriticalWitnesses`, and an **instrumentation-coverage map** (which fault
classes are observable given your tags). If a label is not observable from your data, you get an explicit
*"not observable from supplied data"* receipt — never a fabricated answer.

## What we will NOT do
We will not connect to control, will not move a setpoint or valve, will not claim a root cause, and will not
require your raw bytes. Real plant data is never redistributed; only recipes + hashes are committed publicly.
