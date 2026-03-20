# Embedded-Core Roadmap

This crate is currently a `std` research-software package. It includes CLI, filesystem export, plotting, PDF generation, and zip packaging. Those capabilities are intentionally convenient for review, but they are not a direct embedded deployment path.

## Current Separation

The codebase is already split conceptually into:

- core mathematical primitives in `src/math/`
- deterministic engine-layer transformations in `src/engine/`
- bounded live replay state in `src/live/`
- runtime and artifact concerns in `src/io/`, `src/figures/`, `src/report/`, `src/cli/`, `src/dashboard/`, and `ffi/`

The first three groups are the best candidates for future extraction into a smaller embedded-friendly core crate. The latter groups are intentionally `std`-bound.

The current live path is already more deployment-oriented than the artifact path:

- online residual history is bounded through an explicit ring buffer
- offline accumulation is optional and separate
- the nested FFI crate wraps the bounded live engine rather than the report/export pipeline

## Likely Future Extraction Boundary

A future `no_std`-oriented extraction would most naturally isolate:

- residual construction
- finite-difference helpers
- envelope evaluation
- syntax metric computation
- grammar-state evaluation
- typed semantic retrieval over prevalidated bank entries
- bounded ring-buffer live state and stepwise update logic

The following concerns would remain outside that core:

- filesystem traversal and timestamped output layout
- CSV parsing and validation
- JSON/CSV artifact writing
- plotting and figure rendering
- terminal dashboard rendering
- PDF report generation
- zip packaging
- CLI argument parsing
- FFI dynamic-library packaging details

## Design Guidance Followed Today

The current crate already tries to make future extraction easier by:

- keeping mathematical reductions deterministic and explicit
- avoiding hidden global state in the engine path
- keeping the live path memory-bounded with explicit overwrite semantics
- preserving typed engine outputs separate from artifact/export logic
- keeping the heuristic bank registry typed rather than stringly-typed
- exporting configuration and bank provenance separately from the core math
- keeping the FFI surface thin and delegating all science to safe Rust engine code

## What This Does Not Mean

- The current crate is not `no_std`.
- The current crate is not ready for direct avionics or embedded deployment.
- Computational reproducibility in this crate is not a deployment or certification claim.

## Practical Next Step

If embedded extraction becomes a real requirement, the next disciplined move would be to create a new crate for the pure deterministic core and move only math-, syntax-, grammar-, and semantic-retrieval primitives there, leaving CLI, CSV, plotting, reporting, and zip logic in this crate.

## Deployment-Focused Roadmap Notes

- Bare-metal microcontroller exploration should start with the bounded `src/live/` stepwise engine rather than the batch artifact pipeline.
- ARM and Jetson-class deployments can keep the current `std` path initially, but should still prefer the bounded live engine and external bank artifacts over report-generation code in any sustained runtime process.
- A future feature matrix should separate:
  - pure deterministic core
  - CLI/runtime/reporting
  - dashboard replay
  - FFI packaging

That split would make `no_std` evaluation materially easier without pretending that the current crate has already crossed that boundary.
