# Upgrade Plan

This document records the blocker-oriented implementation plan for upgrading `dsfb-computer-graphics` from a strong bounded artifact to a serious immediate-evaluation candidate.

The scope boundary remains absolute: only files inside `crates/dsfb-computer-graphics` may change.

## Blocker To Fix Mapping

| Blocker | Planned fix |
| --- | --- |
| Demo A depends too much on privileged synthetic cues | Add a host-realistic DSFB mode driven by residual, depth disagreement, normal disagreement, motion disagreement, neighborhood inconsistency, and thin/local-contrast proxies. Preserve the synthetic-visibility mode only as an explicit comparison path. |
| Demo A baseline set is too weak | Add neighborhood-clamped, depth/normal-rejection, reactive-mask-style, and combined strong-heuristic baselines, all on the same host pipeline. |
| Demo A lacks breadth | Replace the single-scene path with a deterministic scenario suite containing at least four distinct failure modes plus at least one neutral / low-action holdout. |
| Demo A lacks ablations | Add an explicit ablation matrix covering visibility removal, host-realistic mode, no-thin, no-motion-edge, no-grammar, residual-only, and trust-without-alpha-control. |
| Demo B looks like redistribution theater | Upgrade Demo B to a multi-scene fixed-budget study with multiple allocators: uniform, edge-guided, residual-guided, contrast-guided, combined heuristic, imported-trust, and hybrid guidance. |
| Attachability is not proven | Add a host-style integration module with typed temporal inputs and supervisory outputs, plus a dedicated integration-surface document. |
| Cost story is too weak | Add a crate-local cost accounting module and generated cost report with explicit buffer counts, memory classes, operation groups, and mode comparisons. |
| Reporting is still general rather than decision-grade | Rewrite generated reporting around scenarios, baselines, ablations, attachability, cost, remaining blockers, and five-reviewer readiness. |
| Regressions could slip through quietly | Add hard validation gates for scenario lists, baseline lists, ablation lists, fixed-budget invariants, report sections, honesty statements, and artifact completeness. |

## Files To Change

- `Cargo.toml`
- `README.md`
- `src/lib.rs`
- `src/main.rs`
- `src/cli.rs`
- `src/config.rs`
- `src/frame.rs`
- `src/scene/mod.rs`
- `src/taa/mod.rs`
- `src/dsfb/mod.rs`
- `src/metrics.rs`
- `src/sampling.rs`
- `src/plots.rs`
- `src/report.rs`
- `src/pipeline.rs`
- `tests/demo_cli.rs`
- `tests/research_artifact.rs`
- `tests/colab_notebook.rs`
- `generated/*` reports and metrics regenerated from code

## New Files / Modules To Add

- `src/host.rs`
- `src/cost.rs`
- `docs/integration_surface.md`
- `docs/cost_model.md`
- `generated/cost_report.md`
- `generated/five_mentor_audit.md`
- `generated/check_signing_blockers.md`
- `generated/ablation_report.md`
- `generated/demo_b_decision_report.md`

If the refactor warrants a dedicated experiment-orchestration helper module, add it crate-locally rather than pushing logic into the workspace root.

## New Experiments To Add

### Demo A scenario suite

Minimum suite:

1. thin-structure disocclusion reveal
2. fast lateral reveal / stronger reprojection stress
3. textured or diagonal thin-structure reveal
4. lighting or contrast change stress case
5. neutral / low-action holdout where DSFB should do little

### Demo A baselines

- fixed-alpha baseline
- residual-threshold baseline
- neighborhood-clamped baseline
- depth/normal disagreement rejection baseline
- reactive-mask-style baseline
- combined strong-heuristic baseline

### Demo A DSFB variants / ablations

- DSFB full synthetic visibility-assisted mode
- DSFB host-realistic mode
- DSFB without visibility cue
- DSFB without thin proxy
- DSFB without motion-edge proxy
- DSFB without grammar/state logic
- DSFB residual-only
- DSFB trust computed but alpha modulation disabled

### Demo B fixed-budget study

- multiple scenarios taken from the scenario suite
- equal total budgets across all allocation policies
- policies: uniform, edge-guided, residual-guided, contrast-guided, combined heuristic, imported Demo A trust, sampling-native guidance, and hybrid guidance where feasible
- budget efficiency sweep across multiple average spp levels

## Acceptance Criteria

### Demo A

- Host-realistic DSFB is implemented as a first-class mode and reported separately from visibility-assisted mode.
- The report clearly distinguishes synthetic-visibility mode from host-realistic mode.
- Strong heuristic baselines are included in metrics and reports even when they are competitive.
- At least one scenario is neutral or bounded rather than a pure DSFB win.
- The ablation report states what cue removals materially hurt and what effect survives host-realistic mode.

### Demo B

- All allocation policies obey identical total budgets.
- At least one hard scenario shows guided improvement over uniform.
- At least one scenario is neutral or weak, and the report surfaces that honestly.
- Imported-trust guidance is separated from sampling-native and hybrid guidance.

### Attachability and cost

- A host-style input/output interface exists in Rust code.
- `docs/integration_surface.md` explains buffers, order of execution, optional inputs, and likely GPU pass decomposition.
- `docs/cost_model.md` and `generated/cost_report.md` contain buffer counts, memory classes, operation groups, and mode comparisons with no fabricated GPU benchmark claims.

### Reporting

- `generated/report.md`, `generated/reviewer_summary.md`, `generated/five_mentor_audit.md`, `generated/check_signing_blockers.md`, `generated/ablation_report.md`, and `generated/demo_b_decision_report.md` all exist and are generated from the real run outputs.
- All reports contain explicit “what is not proven” and “remaining blockers” sections.

## Test Gates

- Scenario suite completeness gate: expected scenario ids must be present.
- Baseline completeness gate: expected baseline ids must be present.
- Ablation completeness gate: expected ablation ids must be present.
- Artifact existence gate: required figures and reports must exist after a run.
- Fixed-budget gate: all Demo B guided policies must exactly match the uniform total budget.
- Host-realistic behavioral gate: host-realistic DSFB must outperform fixed alpha on the canonical reveal scenario by a meaningful margin.
- Neutral-case honesty gate: at least one scenario must report bounded or neutral outcome rather than a universal win.
- Report honesty gate: generated reports must contain “what is not proven” and “remaining blockers”.
- No-fabricated-benchmark gate: generated reports must not claim GPU benchmark wins.
- CLI gate: suite commands, single-scenario commands, and validation command must succeed.

## Execution Order

1. Refactor scene generation into a deterministic scenario suite with richer metadata and host-realistic buffers.
2. Add host-style supervision interfaces and refactor DSFB into profile-driven modes.
3. Add stronger TAA baselines.
4. Replace single-scenario Demo A analysis with suite analysis, ablations, and broader figures.
5. Upgrade Demo B to multi-scene, multi-policy fixed-budget evaluation.
6. Add attachability and cost-model modules and generate their reports.
7. Rewrite generated reporting around blockers removed, blockers remaining, and reviewer readiness.
8. Add strict validation gates and update README/CLI only after the core technical changes are in place.
