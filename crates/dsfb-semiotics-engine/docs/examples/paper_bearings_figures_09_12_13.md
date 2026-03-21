# Paper Bearings Figures 9 / 12 / 13

NASA Bearings is the primary paper dataset for Figures 9, 12, and 13 because it gives the clearest
single-run structural progression without forcing a cross-dataset comparison.

These figures are still generated through the normal crate artifact pipeline and keep the same
paper-facing filenames:

- `figure_09_detectability_bound_comparison.png`
- `figure_12_semantic_retrieval_heuristics_bank.png`
- `figure_13_internal_baseline_comparators.png`

No LaTeX edits are required. Replace the files in the paper `figures/` folder with the regenerated
PNGs and recompile.

## What Figure 9 Argues

Figure 9 is no longer a generic detectability bar chart for the Bearings run.

It now makes one explicit argument:

- two within-run windows show very similar primary residual magnitude
- their meta-residual slew structure diverges
- the downstream grammar outcome diverges

The figure is therefore about structural insufficiency of first-order magnitude alone, not about
cross-dataset comparison.

## What Figure 12 Argues

Figure 12 shows semantic interpretation as a timeline:

- top-candidate score and score margin evolve with time
- candidate counts narrow through admissibility and scope
- the semantic disposition evolves rather than appearing only as a final label

The figure is about semantic process, not bank existence.

## What Figure 13 Argues

Figure 13 shows the interpretability delta:

- Panel A: what the internal deterministic comparators see first
- Panel B: what the DSFB grammar layer adds as typed structural context
- Panel C: what the DSFB semantic layer adds as temporal interpretation

It is framed as a complementary interpretability comparison, not a performance benchmark.

## Regeneration

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --dataset nasa_bearings --phase all
```

Regenerated outputs appear under:

- `artifacts/public_dataset_demo/nasa_bearings/latest/figures/`
- `artifacts/public_dataset_demo/nasa_bearings/latest/csv/`
- `artifacts/public_dataset_demo/nasa_bearings/latest/json/`

The machine-readable figure sources are regenerated automatically with the normal artifact pipeline.
