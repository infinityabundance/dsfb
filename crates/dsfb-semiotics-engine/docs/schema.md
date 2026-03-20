# Artifact Schema Overview

`dsfb-semiotics-engine` emits additive machine-readable artifacts under the schema marker:

`dsfb-semiotics-engine/v1`

This marker appears in:

- `run_metadata.json`
- `manifest.json`
- evaluation JSON artifacts
- artifact completeness records
- selected CSV summaries where a schema column is appropriate

## Stability Discipline

- The crate prefers additive schema evolution over breaking rewrites.
- New CSV or JSON files may be added to the artifact bundle when they expose new deterministic outputs.
- Existing field names should be preserved unless a documented correction is required for scientific honesty or parseability.
- Snapshot tests protect a small canonical subset of outputs against unintentional drift.

## Core JSON Artifacts

- `run_metadata.json`
  Run provenance, crate version, Rust version when available, input mode, CLI args, and deterministic engine settings.
- `scenario_catalog.json`
  Scenario metadata for synthetic, CSV-driven, or sweep members.
- `scenario_outputs.json`
  Full layered outputs for each executed scenario.
- `reproducibility_checks.json`
  Per-scenario deterministic materialization hashes over full layered outputs.
- `reproducibility_summary.json`
  Run-level reproducibility summary.
- `evaluation_summary.json`
  Run-level deterministic evaluation summary.
- `scenario_evaluations.json`
  Scenario-level evaluation summaries.
- `baseline_comparators.json`
  Internal deterministic comparator results.
- `heuristic_bank_validation.json`
  Built-in heuristic bank governance and validation summary.
- `artifact_completeness.json`
  Export completeness check recorded after artifact generation.
- `sweep_results.json`
  Present only for synthetic sweep runs.
- `sweep_summary.json`
  Present only for synthetic sweep runs.

## Core CSV Artifacts

- `scenario_catalog.csv`
- `semantic_matches.csv`
- `grammar_events.csv`
- `pipeline_summary.csv`
- `reproducibility_check.csv`
- `reproducibility_summary.csv`
- `evaluation_summary.csv`
- `scenario_evaluations.csv`
- `baseline_comparators.csv`
- `heuristic_bank_validation.csv`
- `artifact_completeness.csv`

Scenario-specific CSV files are also emitted for time series, residual, drift, slew, sign, envelope, grammar, and coordinated group structure when present.

## Interpretation Notes

- Schema stability is computational and contractual, not a claim of field validity.
- Evaluation outputs summarize deterministic engine behavior and internal deterministic comparators only.
- Semantic outputs remain typed retrieval summaries, not unique latent-cause recovery.
