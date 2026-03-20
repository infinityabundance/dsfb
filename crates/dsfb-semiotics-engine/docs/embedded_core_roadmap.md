# Embedded-Core Roadmap

This crate is currently a `std` research-software package. It includes CLI, filesystem export, plotting, PDF generation, and zip packaging. Those capabilities are intentionally convenient for review, but they are not a direct embedded deployment path.

## Current Separation

The codebase is already split conceptually into:

- core mathematical primitives in `src/math/`
- deterministic engine-layer transformations in `src/engine/`
- runtime and artifact concerns in `src/io/`, `src/figures/`, `src/report/`, and `src/cli/`

The first two groups are the best candidates for future extraction into a smaller embedded-friendly core crate. The latter groups are intentionally `std`-bound.

## Likely Future Extraction Boundary

A future `no_std`-oriented extraction would most naturally isolate:

- residual construction
- finite-difference helpers
- envelope evaluation
- syntax metric computation
- grammar-state evaluation
- typed semantic retrieval over prevalidated bank entries

The following concerns would remain outside that core:

- filesystem traversal and timestamped output layout
- CSV parsing and validation
- JSON/CSV artifact writing
- plotting and figure rendering
- PDF report generation
- zip packaging
- CLI argument parsing

## Design Guidance Followed Today

The current crate already tries to make future extraction easier by:

- keeping mathematical reductions deterministic and explicit
- avoiding hidden global state in the engine path
- preserving typed engine outputs separate from artifact/export logic
- keeping the heuristic bank registry typed rather than stringly-typed
- exporting configuration and bank provenance separately from the core math

## What This Does Not Mean

- The current crate is not `no_std`.
- The current crate is not ready for direct avionics or embedded deployment.
- Computational reproducibility in this crate is not a deployment or certification claim.

## Practical Next Step

If embedded extraction becomes a real requirement, the next disciplined move would be to create a new crate for the pure deterministic core and move only math-, syntax-, grammar-, and semantic-retrieval primitives there, leaving CLI, CSV, plotting, reporting, and zip logic in this crate.
