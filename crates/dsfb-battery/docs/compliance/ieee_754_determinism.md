# IEEE 754-2019 Determinism Mapping

Status: Partial

This file describes the crate's deterministic floating-point usage in scope. It does not claim cross-platform bit-identical proof.

Relevant crate components:
- `src/math.rs`
- `src/detection.rs`
- `src/compliance.rs::build_determinism_check`

Traceability matrix:

| Determinism concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Fixed operation ordering | `compute_residual`, `compute_drift`, `compute_slew`, `run_dsfb_pipeline` | Straight-line arithmetic in fixed iteration order | Partial | Current implementation is deterministic on a given toolchain and input |
| Local repeated-run reproducibility | `determinism_check.json` | Two repeated summary hashes compared in the compliance helper | Partial | Same-host check only |
| Cross-target numeric equivalence | Outside crate scope | None | Not supported | No proof across architectures/toolchains is claimed |
