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
- `semantic_matches.json`
  Standalone semantic retrieval results, including explicit stage-wise heuristic filtering counts.
- `heuristic_bank_validation.json`
  Built-in heuristic bank governance and validation summary.
- `artifact_completeness.json`
  Export completeness check recorded after artifact generation.
- `<figure-id>_source.json`
  One machine-readable source table per rendered publication-style figure. Each table contains figure metadata plus row-wise panel/series/value records for the exact plotted content.
- `figure_09_detectability_source.json`
  Legacy additive detectability summary rows retained for compatibility.
- `figure_12_semantic_retrieval_source.json`
  Legacy additive semantic retrieval summary rows retained for compatibility.
- `figure_13_internal_baseline_comparators_source.json`
  Legacy additive internal deterministic comparator summary rows retained for compatibility.
- `figure_integrity_checks.json`
  Integrity records linking summary figures back to their exported source rows.
- `sweep_results.json`
  Present only for synthetic sweep runs.
- `sweep_summary.json`
  Present only for synthetic sweep runs.
- `figure_14_sweep_stability_source.json`
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
- `<figure-id>_source.csv`
- `figure_09_detectability_source.csv`
- `figure_12_semantic_retrieval_source.csv`
- `figure_13_internal_baseline_comparators_source.csv`
- `figure_integrity_checks.csv`

For synthetic sweep runs the crate also emits:

- `figure_14_sweep_stability_summary_source.csv`
- `figure_14_sweep_stability_source.csv`

Scenario-specific CSV files are also emitted for time series, residual, drift, slew, sign, envelope, grammar, and coordinated group structure when present.

## Figure-Source Discipline

- Every publication-style figure is paired with a machine-readable source table.
- `figure_12_semantic_retrieval_source.*` exports explicit typed-bank counts including post-admissibility, post-regime, pre-scope, post-scope, rejected-stage counts, and final selected count.
- Generic `<figure-id>_source.*` tables export panel ids, panel titles, series ids, series labels, plot coordinates, figure metadata, and additive notes for the rendered plot.
- `figure_integrity_checks.*` records panel counts, source row counts, emitted image presence, count-like-panel integer checks, source-file locations, and simple consistency checks for the exported figure inputs.
- The semantic retrieval figure's middle panel now plots admissibility-qualified heuristic counts directly, not grammar boundary counts or other proxies.

## Interpretation Notes

- Schema stability is computational and contractual, not a claim of field validity.
- Evaluation outputs summarize deterministic engine behavior and internal deterministic comparators only.
- Semantic outputs remain typed retrieval summaries, not unique latent-cause recovery.
