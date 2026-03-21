# NASA Milling Paper Figures 09 / 12 / 13

These NASA Milling paper figures are run-specific and are intended to make the milling process interpretation legible on its own terms rather than forcing a bearings-style narrative onto it.

## Why NASA Milling Is Useful

NASA Milling is useful here because the run has a process-oriented progression with boundary and violation structure inside one executed public-data trajectory:

- Figure 9 can compare matched process windows with similar first-order behavior but different higher-order structure and grammar outcome.
- Figure 12 can show semantic evolution through the milling progression rather than a static final candidate.
- Figure 13 can show what baseline/internal comparators see and what DSFB adds structurally in a process-monitoring setting.

## Figure 9 Argument

Figure 9 argues:

- two milling windows can have similar apparent primary residual behavior,
- higher-order / meta-residual structure still diverges,
- and the grammar outcome differs accordingly.

This keeps the argument within the milling run rather than comparing milling to any other dataset.

## Figure 12 Argument

Figure 12 argues:

- semantic retrieval evolves through the milling process,
- candidate scores and candidate-set size change over time,
- and the disposition timeline shows semantic stabilization or narrowing when supported by the run.

The figure is about semantic process, not a single final heuristic label.

## Figure 13 Argument

Figure 13 argues:

- baseline/internal comparators provide the alarm-like view,
- DSFB adds syntax / grammar / semantic structure beyond those alarms,
- and the intended reading is interpretability delta rather than superiority benchmarking.

## Regeneration

Regenerate the NASA Milling paper figures with the normal public-dataset pipeline:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- --dataset nasa_milling --phase all
```

The upgraded figures will be written under:

- `crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/latest/figures/figure_09_detectability_bound_comparison.png`
- `crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/latest/figures/figure_12_semantic_retrieval_heuristics_bank.png`
- `crates/dsfb-semiotics-engine/artifacts/public_dataset_demo/nasa_milling/latest/figures/figure_13_internal_baseline_comparators.png`

## Paper Drop-In Workflow

No LaTeX edits are required.

Take the regenerated PNGs with the same basenames, copy them into the paper `figures/` folder, and recompile the paper unchanged.
