# Rust Safety Subset Audit

Status: Partial

This file summarizes the safe-Rust subset position of the current crate. It is not a Ferrocene qualification or ISO 26262 approval artifact.

Relevant crate components:
- `src/types.rs`
- `src/math.rs`
- `src/detection.rs`
- `src/ffi.rs`
- `src/compliance.rs::scan_safe_rust_subset`

Traceability matrix:

| Audit concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Unsafe in core logic | `src/types.rs`, `src/math.rs`, `src/detection.rs` | Heuristic scan reports no core-logic `unsafe` blocks | Partial | The FFI boundary still uses `unsafe` |
| Unsafe boundary handling | `src/ffi.rs` | Explicit `unsafe extern "C"` and pointer handling is localized | Partial | Boundary is visible and auditable, not eliminated |
| Dynamic allocation in core path | `src/math.rs`, `src/detection.rs`, `src/types.rs` | `Vec`, `String`, `format!` usage remains | Partial | Core path is `no_std + alloc`, not heapless |
| Recursion | `src/compliance.rs` scan over `src/` | Heuristic direct-recursion scan | Partial | A helper scan is provided, not a proof system |

Generated runtime evidence is emitted by the compliance helper as `misra_equivalent_report.txt` and `safe_rust_audit.json`.
