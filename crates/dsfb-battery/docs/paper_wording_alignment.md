# Artifact-to-Paper Wording Alignment

This note is crate-local only. It does not edit the paper. It records wording that the current `dsfb-battery` implementation and emitted Stage II artifact support today.

## Wording the Current Artifact Supports

- The Stage II artifact emits a deterministic audit trace in `stage2_detection_results.json`.
- The emitted artifact is advisory and read-only in its declared interface contract.
- The current interface contract declares `deployment_mode: offline`, `read_only: true`, `protocol_independent: true`, `requires_cloud_connectivity: false`, `requires_model_retraining: false`, and `advisory_only: true`.
- The audit trace records per-cycle classifications, explicit state-transition events, thresholds in force, persistence counters, residual/drift/slew evidence, benchmark lead-time summaries, and the theorem-derived `t_star` value already computed by the crate.
- State transitions are explicit and auditable because each transition event records the cycle index, previous state, current state, reason code when present, evidence, thresholds, persistence counters, and audit fields.
- The current artifact supports wording that classifications and interpretations are reported, rather than decisions.

## Observed Artifact Facts Relevant to Manuscript Wording

- In the current emitted Stage II artifact path, `interface_contract.fail_silent_on_invalid_stream` is `false`.
- The current audit-trace builder emits `stream_valid: true` for recorded events and does not emit invalid-stream gap events in the default B0005 path.
- The current B0005 audit trace records these observed state-transition pairs: `Admissible -> Boundary`, `Boundary -> Violation`, and `Violation -> Admissible`.
- A current default B0005 emission includes a `Violation -> Admissible` transition at cycle `48`.
- Transition events are emitted whenever the per-cycle grammar classification changes; the current implementation does not impose monotone-only progression in the artifact exporter.
- Theorem 1 output is reported in the artifact summary and legacy theorem section, but the current detection path still raises the DSFB alarm from the grammar trajectory rather than from `t_star`.

## Wording to Avoid Unless Implementation Changes

- Avoid claiming fail-silent behavior on invalid streams. The current emitted interface contract does not assert it.
- Avoid claiming monotone-only or irreversible grammar progression. The current implementation permits return transitions, including `Violation -> Admissible`, when the observed residual and persistence conditions return to admissible conditions.
- Avoid claiming that Theorem 1 drives the classifier or triggers the DSFB alarm. The current artifact reports the theorem-derived bound separately from the grammar-based alarm path.
- Avoid stronger claims such as formally verified, safety-preserving by proof, certifiable, or guaranteed protocol independence beyond what the emitted interface contract explicitly states.
