# Safety — `dsfb-atlas`

## Memory and concurrency safety

- **`unsafe_code = forbid`** is set in `Cargo.toml` `[lints.rust]`.
  Compilation fails on any `unsafe` block.
- **No FFI, no concurrency primitives, no network access.** See
  [`audit/reports/dsfb_gray.json`](./audit/reports/dsfb_gray.json) for the
  threat-surface scan output.

## Verified invariants

| Invariant | Where | Verification |
|-----------|-------|--------------|
| `Dedup::record` reports collisions iff some pair `(body_i, body_j)` with `i ≠ j` is byte-equal | `src/dedup.rs::dedup_collision_iff_repeated_body` | Kani proof harness, unwind = 5, body alphabet `{α,β,γ}` |
| Generator output is byte-deterministic on identical input | `tests/atlas_end_to_end.rs::generate_part_is_deterministic` | Integration test, two-run byte-equality assertion |
| Each Part emits exactly 1,000 atlas theorems with 1,000 unique SHA-256 proof-body hashes | `tests/atlas_end_to_end.rs::generate_part_emits_one_thousand_unique_proofs` | Integration test |
| Aggregate atlas size is exactly 10,000 theorems | `src/main.rs::print_summary_and_check` | Build-time runtime assertion (`bail!` on mismatch) |
| Loops in the generator are bounded by `MAX_*` constants | `src/generator.rs`, `src/main.rs` | `take(MAX_*)` calls + debug-assertion at entry of every helper |

## NASA/JPL Power-of-Ten posture

| Rule | Status | Notes |
|------|--------|-------|
| P10-1 (no recursion) | applied | No direct recursion; no indirect-recursion motifs flagged. |
| P10-2 (bounded loops) | applied | Every loop has either a `take(N)` cap or a `MAX_*` constant. |
| P10-3 (no dynamic alloc after init) | mitigated | Allocation is initialisation-time only (parsing YAML, building output strings); `Dedup::DEDUP_MAX_RECORDS` bounds the steady-state map. |
| P10-4 (≤ 60 LOC per function) | applied | Each public function and helper is single-page. |
| P10-5 (≥ 2 assertions per function) | applied | `debug_assert!` at entry/exit of each helper. |
| P10-6 (declare smallest scope possible) | applied | Local `let` bindings; no global mutable state. |
| P10-7 (check return values) | applied | All `Result`s flow through `?`; no `unwrap` on fallible code. |
| P10-8 (preprocessor restraint) | n/a | No `cfg`-gated control-flow except the `kani` harness. |
| P10-9 (single-level pointer indirection) | n/a | Rust idiom; no raw pointers. |
| P10-10 (warnings + analyzer gates) | applied | `Cargo.toml` `[lints]` + clippy-pedantic, miri/kani via `audit/`. |

## Reporting

To report a safety regression or a soundness issue, follow
[`SECURITY.md`](./SECURITY.md).
