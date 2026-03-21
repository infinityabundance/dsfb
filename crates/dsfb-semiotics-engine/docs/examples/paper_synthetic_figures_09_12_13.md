# Synthetic Paper Figures 09 / 12 / 13

These synthetic paper figures are run-specific and are intended to be the clearest controlled explanation of the DSFB structural argument.

## Why Synthetic Is Useful

Synthetic is useful here because it lets the figure source stay deterministic while isolating structural differences cleanly:

- Figure 9 uses the magnitude-matched admissible and detectable synthetic cases, so the primary residual magnitude stays similar while higher-order structure and outcome diverge.
- Figure 12 uses a transition-rich synthetic scenario to show semantic evolution through time rather than a static final label.
- Figure 13 uses the same synthetic transition to show what baseline/internal comparators see first and what DSFB adds structurally.

## Figure 9 Argument

Figure 9 argues:

- the two synthetic cases can look similar at the primary residual-magnitude level,
- higher-order / meta-residual structure still differs,
- and the grammar outcome differs accordingly.

This is the textbook controlled version of the "primary behavior alone is insufficient" argument.

## Figure 12 Argument

Figure 12 argues:

- semantic interpretation is a process through time,
- candidate scores and candidate-set size evolve,
- and semantic disposition can narrow or stabilize as more structure accumulates.

The figure is about semantic evolution, not bank existence.

## Figure 13 Argument

Figure 13 argues:

- baseline/internal comparators provide alarm-like trigger timing,
- DSFB adds syntax / grammar / semantic structure on top of those triggers,
- and the value is interpretability delta rather than benchmark superiority.

## Regeneration

Regenerate the synthetic paper figures with the normal synthetic artifact pipeline:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- --output-root crates/dsfb-semiotics-engine/output-dsfb-semiotics-engine
```

The upgraded figures will be written under the generated run directory as:

- `figures/figure_09_detectability_bound_comparison.png`
- `figures/figure_12_semantic_retrieval_heuristics_bank.png`
- `figures/figure_13_internal_baseline_comparators.png`

## Paper Drop-In Workflow

No LaTeX edits are required.

Take the regenerated PNGs with the same basenames, copy them into the paper `figures/` folder, and recompile the paper unchanged.
