# dsfb-computer-graphics

`dsfb-computer-graphics` is a self-contained Rust research crate that implements a minimal DSFB-supervised temporal accumulation artifact for computer graphics. The crate focuses on a deterministic synthetic scene that exposes thin-geometry disocclusion, baseline fixed-alpha TAA failure, and DSFB trust-gated correction. The experiment is intentionally bounded and reproducible.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope Boundary

This crate is intentionally standalone inside `crates/dsfb-computer-graphics`.

- It does not require editing any workspace-root files.
- It does not modify or depend on changes to sibling crates.
- It declares its own nested `[workspace]` so it can be built from this directory alone.

## What It Implements

- A deterministic canonical 2D scene with a moving opaque object, thin geometry, and disocclusion.
- A baseline fixed-alpha TAA pipeline.
- A DSFB-gated TAA pipeline that only changes the blending-control path.
- Metric computation for temporal error, trust evolution, residual magnitude, and ghost persistence.
- Deterministic figure generation and markdown report assembly.
- Crate-local documentation for GPU realization considerations without fabricated timings.

## Quickstart

Run the full demo and generate artifacts:

```bash
cd crates/dsfb-computer-graphics
cargo run -- run-demo-a
```

Generate only the canonical scene frames:

```bash
cd crates/dsfb-computer-graphics
cargo run -- generate-scene
```

Regenerate figures and report from the deterministic pipeline:

```bash
cd crates/dsfb-computer-graphics
cargo run -- make-figures
cargo run -- make-report
```

## Generated Artifacts

The crate writes deterministic outputs under `generated/`:

- `generated/frames/gt/`: ground-truth scene sequence
- `generated/frames/baseline/`: baseline fixed-alpha TAA outputs
- `generated/frames/dsfb/`: DSFB-gated TAA outputs
- `generated/figures/fig_system_diagram.svg`
- `generated/figures/fig_trust_map.svg`
- `generated/figures/fig_before_after.svg`
- `generated/figures/fig_trust_vs_error.svg`
- `generated/metrics.json`
- `generated/report.md`

## GPU Implementation Considerations

See [`docs/gpu_implementation.md`](docs/gpu_implementation.md). That document uses architectural reasoning only. It does not report fabricated timings or unsupported production claims.

## What This Crate Does Not Claim

- It does not claim optimal TAA parameters.
- It does not claim production integration completeness.
- It does not claim universal superiority over modern temporal reconstruction pipelines.
- It does not claim measured GPU timing data.
- It does not claim that all ghosting artifacts disappear.

## Limitations

- The renderer is a bounded synthetic 2D study rather than a full 3D engine integration.
- The DSFB supervisory grammar is hand-specified for interpretability, not learned or tuned for optimality.
- The baseline is a serious but minimal fixed-alpha TAA reference, not a full production anti-ghosting stack.
- Demo B adaptive sampling is documented as future work unless explicitly implemented by the crate artifacts.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Future Extensions

- Add a rigorously evaluated adaptive-sampling demo surface that uses the same trust signal at fixed sample budget.
- Replace nearest-neighbor reprojection with subpixel jitter and resolve filters.
- Add comparative baselines such as variance clipping or neighborhood clamping.
- Export richer per-frame diagnostics for external plotting or notebook analysis.

## Completion Note

See [`docs/completion_note.md`](docs/completion_note.md) for the bounded completion report and crate-local self-check list.
