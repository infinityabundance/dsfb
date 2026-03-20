# Heuristic Bank Schema

This crate supports a typed external heuristic-bank artifact under the schema marker
`dsfb-semiotics-engine-bank/v1`.

The external bank is an input artifact for deterministic constrained retrieval. It is
not a model, not a classifier, and not a latent-cause oracle. Different valid bank
versions may produce different semantic outputs because the bank is part of the
configured retrieval surface.
After parse, external banks are normalized deterministically before validation so
ordering differences in entries or link lists do not create hidden runtime variation.

## Top-Level Shape

```json
{
  "metadata": {
    "schema_version": "dsfb-semiotics-engine-bank/v1",
    "bank_version": "heuristic-bank/v3",
    "note": "human-readable registry note"
  },
  "entries": [
    {
      "heuristic_id": "H-EXAMPLE",
      "motif_label": "example motif",
      "short_label": "example",
      "scope_conditions": { "...": "..." },
      "admissibility_requirements": "NoViolation",
      "regime_tags": ["fixed"],
      "provenance": {
        "source": "paper-aligned synthetic reference",
        "note": "review note"
      },
      "applicability_note": "Conservative explanation of when this motif is intended to apply.",
      "retrieval_priority": 10,
      "compatible_with": [],
      "incompatible_with": []
    }
  ]
}
```

## Required Metadata

- `metadata.schema_version`
  Must equal `dsfb-semiotics-engine-bank/v1`.
- `metadata.bank_version`
  Human-managed bank version string. This is exported into run metadata and manifests.
- `metadata.note`
  Brief audit note for the registry revision.

## Required Entry Fields

Each entry must provide:

- `heuristic_id`
- `motif_label`
- `short_label`
- `scope_conditions`
- `admissibility_requirements`
- `regime_tags`
- `provenance`
- `applicability_note`
- `retrieval_priority`
- `compatible_with`
- `incompatible_with`

## Scope Conditions

`scope_conditions` is a typed threshold object over deterministic syntax metrics. The
current schema supports:

- `min_outward_drift_fraction`
- `max_outward_drift_fraction`
- `min_inward_drift_fraction`
- `max_inward_drift_fraction`
- `max_curvature_energy`
- `min_curvature_energy`
- `max_curvature_onset_score`
- `min_curvature_onset_score`
- `min_directional_persistence`
- `min_sign_consistency`
- `min_channel_coherence`
- `min_aggregate_monotonicity`
- `max_aggregate_monotonicity`
- `min_slew_spike_count`
- `max_slew_spike_count`
- `min_slew_spike_strength`
- `max_slew_spike_strength`
- `min_boundary_grazing_episodes`
- `max_boundary_grazing_episodes`
- `min_boundary_recovery_count`
- `min_coordinated_group_breach_fraction`
- `max_coordinated_group_breach_fraction`
- `require_group_breach`

All numeric thresholds are deterministic comparisons against exported syntax metrics.

## Admissibility Requirements

`admissibility_requirements` must be one of:

- `Any`
- `BoundaryInteraction`
- `ViolationRequired`
- `NoViolation`

These are grammar-state requirements, not causal claims.

## Provenance Requirements

Each entry must include:

- `provenance.source`
- `provenance.note`

Missing provenance text is flagged by bank validation.

## Governance Checks

At startup the crate validates external banks for:

- duplicate heuristic IDs
- self-links in compatibility or incompatibility lists
- references to unknown heuristic IDs
- overlap between `compatible_with` and `incompatible_with`
- provenance completeness
- regime-tag sanity
- retrieval-priority sanity
- scope-condition sanity checks
- optional strict reverse-link symmetry checks

Strict mode is enabled with `--strict-bank-validation`. In strict mode, missing reverse
compatibility or incompatibility links fail the run. The runtime exports
`validation_mode` as either `strict` or `permissive`, plus additive `violations` and `warnings`
arrays so permissive-mode runs do not hide graph-governance findings.

## Exported Runtime Metadata

Every run records:

- bank schema version
- bank version
- bank source kind: builtin or external
- source path for external banks
- deterministic content hash
- strict-validation mode

Those fields appear in:

- `json/run_metadata.json`
- `json/loaded_heuristic_bank_descriptor.json`
- `json/heuristic_bank_validation.json`
- `json/bank_validation_report.json`
- `csv/heuristic_bank_validation.csv`
- `csv/bank_validation_report.csv`
- `manifest.json`

## Limitations

- A valid bank artifact is still only a typed heuristic registry.
- Loading an external bank does not imply validation of the bank author’s claims.
- Different valid bank versions may legitimately change retrieval outcomes.
