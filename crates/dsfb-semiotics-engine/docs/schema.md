# Artifact Schema Overview

`dsfb-semiotics-engine` emits additive machine-readable artifacts under the schema marker:

`dsfb-semiotics-engine/v1`

This marker appears in:

- `run_metadata.json`
- `manifest.json`
- evaluation JSON artifacts
- artifact completeness records
- selected CSV summaries where a schema column is appropriate

Heuristic-bank artifacts use the separate bank schema marker:

`dsfb-semiotics-engine-bank/v1`

The external-bank artifact itself is documented in [bank_schema.md](bank_schema.md).

## Stability Discipline

- The crate prefers additive schema evolution over breaking rewrites.
- New CSV or JSON files may be added to the artifact bundle when they expose new deterministic outputs.
- Existing field names should be preserved unless a documented correction is required for scientific honesty or parseability.
- Snapshot tests protect a small canonical subset of outputs against unintentional drift.

## Core JSON Artifacts

- `run_metadata.json`
  Run provenance, crate version, Rust version when available, input mode, CLI args, deterministic engine settings, bounded online-history buffer capacity, and selected numeric mode.
- `loaded_heuristic_bank_descriptor.json`
  Resolved bank provenance for the run, including bank schema version, bank version, source kind, optional source path, content hash, and validation mode.
- `scenario_catalog.json`
  Scenario metadata for synthetic, CSV-driven, or sweep members.
- `scenario_outputs.json`
  Full layered outputs for each executed scenario, including typed grammar reason codes and reason text.
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
- `comparator_results.json`
  Additive alias export for comparator results with `comparator_name`, `alarm`, `first_alarm_time`, and `config_reference` fields for downstream audit tools.
- `semantic_matches.json`
  Standalone semantic retrieval results, including explicit stage-wise heuristic filtering counts.
- `heuristic_bank_validation.json`
  Heuristic-bank governance and validation summary for the builtin or external bank selected for the run.
- `bank_validation_report.json`
  Additive alias export for the same governed bank-validation surface.
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
- `figure_integrity_report.json`
  Additive alias export for figure-integrity checks with explicit expected/observed panel lists and failure messages.
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
- `grammar_events.csv`
  Includes grammar state, grammar reason code, reason text, and supporting metric summary per step.
- `pipeline_summary.csv`
- `reproducibility_check.csv`
- `reproducibility_summary.csv`
- `evaluation_summary.csv`
- `scenario_evaluations.csv`
- `baseline_comparators.csv`
- `comparator_results.csv`
- `heuristic_bank_validation.csv`
- `bank_validation_report.csv`
- `artifact_completeness.csv`
- `<figure-id>_source.csv`
- `figure_09_detectability_source.csv`
- `figure_12_semantic_retrieval_source.csv`
- `figure_13_internal_baseline_comparators_source.csv`
- `figure_integrity_checks.csv`
- `figure_integrity_report.csv`

For synthetic sweep runs the crate also emits:

- `figure_14_sweep_stability_summary_source.csv`
- `figure_14_sweep_stability_source.csv`

Scenario-specific CSV files are also emitted for time series, residual, drift, slew, sign, envelope, grammar, and coordinated group structure when present.

## Figure-Source Discipline

- Every publication-style figure is paired with a machine-readable source table.
- `figure_12_semantic_retrieval_source.*` exports explicit typed-bank counts including post-admissibility, post-regime, pre-scope, post-scope, rejected-stage counts, and final selected count.
- Generic `<figure-id>_source.*` tables export panel ids, panel titles, series ids, series labels, plot coordinates, generation timestamp, expected panel ids, rendered panel ids, figure metadata, and additive notes for the rendered plot.
- Figure-source tables and evaluation summaries carry additive schema, engine-version, and bank-version fields so the plotted values can be tied back to the exact deterministic runtime surface.
- `run_metadata.json` and `manifest.json` also export a deterministic `run_configuration_hash` so the settings surface can be referenced compactly in downstream audit tooling.
- `run_metadata.json` and `manifest.json` also export `online_history_buffer_capacity` and `numeric_mode` so bounded live-engine settings remain auditable.
- `figure_integrity_checks.*` and `figure_integrity_report.*` record panel counts, expected/observed panel identities, source-table presence, source row counts, emitted image presence, count-like-panel integer checks, source-file locations, and explicit failure messages for the exported figure inputs.
- The semantic retrieval figure's middle panel now plots admissibility-qualified heuristic counts directly, not grammar boundary counts or other proxies.

## Interpretation Notes

- Schema stability is computational and contractual, not a claim of field validity.
- Evaluation outputs summarize deterministic engine behavior and internal deterministic comparators only.
- Semantic outputs remain typed retrieval summaries, not unique latent-cause recovery.
- External-bank loading preserves the typed registry contract. It changes the active bank content only when the caller selects a different validated bank artifact.
