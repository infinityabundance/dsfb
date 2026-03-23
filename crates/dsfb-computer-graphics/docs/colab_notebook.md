# Colab Notebook Design

This note documents the crate-local Google Colab notebook at `colab/dsfb_computer_graphics_demo.ipynb`.

The notebook exists to make the crate evaluable from a clean browser session. It does not reimplement the artifact in Python. Instead, it installs the small set of runtime dependencies needed for Colab, clones the repository, builds `dsfb-computer-graphics`, and drives the Rust CLI so the crate remains the source of truth for Demo A, Demo B, metrics, figures, and reports.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Design Goals

- keep the notebook honest about scope and limits
- make repeated runs safe by default with timestamped output directories
- generate a reviewer-ready bundle without requiring a local graphics toolchain
- display the major figures inline so a reviewer can inspect the artifact before downloading anything
- surface the expanded scenario suite, stronger baselines, and mixed-outcome cases without asking the reviewer to inspect JSON manually
- package the current run as both a PDF and a ZIP for archiving and external review

## Output Organization

The notebook writes under the crate-local root `output-dsfb-computer-graphics/`.

Each execution creates a timestamped subdirectory such as:

```text
output-dsfb-computer-graphics/
  output-dsfb-computer-graphics-YYYYMMDD-HHMMSS/
```

The timestamped run directory holds the generated artifacts for that execution, including:

- `artifact_manifest.json`
- `scenario_suite_manifest.json`
- `figures/`
- `scenarios/`
- `metrics.json`
- `report.md`
- `reviewer_summary.md`
- `five_mentor_audit.md`
- `check_signing_blockers.md`
- `demo_b_decision_report.md`
- `completion_note.md`
- `artifacts_bundle.pdf`
- `demo_b/`

The ZIP archive is written one level above the run directory and uses the same timestamped run name. This layout prevents accidental overwrite and keeps each run self-describing.

## PDF and ZIP Bundling

The notebook calls the crate-local script `colab/build_artifact_bundle.py` after `cargo run -- run-all --output <run-dir>`.

The bundle script:

- reads `artifact_manifest.json` to discover the actual outputs from the run
- rasterizes the SVG figures for a clean PDF export
- assembles a real PDF bundle containing the main metrics, figures, and scope statement
- creates a ZIP archive of the entire timestamped artifact directory

This is intended to support review convenience and archival traceability, not to imply production packaging completeness.

## Assumptions

- the notebook runs in a Colab-like Linux environment with shell access
- the environment can install `librsvg2-bin`, `zip`, and the small Python dependencies used for display and bundling
- the repository URL and branch configured near the top of the notebook may need to be updated by the user for a fork or a different default branch

## What The Notebook Does Not Claim

- It does not claim deployment readiness.
- It does not claim optimal TAA tuning or optimal adaptive sampling.
- It does not claim measured production GPU performance.
- It does not claim superiority over full commercial temporal reconstruction stacks.

The notebook is a polished access path to a bounded synthetic artifact, not a substitute for broader engine integration or field validation.
