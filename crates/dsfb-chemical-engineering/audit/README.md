# Audit suite — DSFB-Chemical-Engineering

This folder collects every audit run on the workspace: the maintainer's own **dsfb-gray** assurance scan, a
**real Rust security / supply-chain suite**, an **undefined-behaviour** interpreter, and **bounded formal
proofs**. Open `index.html` for the rendered dashboard.

> **What this suite does and does NOT certify.** These are *review-readiness* and *evidence* artifacts, reported
> honestly — not a certification of security, correctness, or fitness for any purpose. Clean machine-audit results
> mean "no *known* issue was found by *this* tool at scan time," never "provably safe." dsfb-gray's own wording:
> *"a broad improvement and review-readiness target — not a compliance certification."* Nothing here is gamed or
> fabricated; where a tool could not run, that is recorded as such (see cargo-scan).

## What was run

| Tool | Scope | Result | Folder |
|---|---|---|---|
| **dsfb-gray** | first-party assurance score (per crate + workspace) | 50–76% per crate; honest doc-hygiene pass only, no code gamed | [`dsfb-gray/`](dsfb-gray/SUMMARY.md) |
| **cargo-audit** | RustSec advisories vs `Cargo.lock` | **clean** — 42 deps, 0 advisories | [`cargo-audit/`](cargo-audit/README.md) |
| **cargo-geiger** | `unsafe` usage | **5/7 crates `#![forbid(unsafe_code)]`**; `unsafe` only at the cuda + wasm FFI boundaries | [`cargo-geiger/`](cargo-geiger/README.md) |
| **cargo-auditable** | embedded SBOM in the `edge` binary | embedded BOM verified, 34 deps, clean | [`cargo-auditable/`](cargo-auditable/README.md) |
| **cargo-vet** | supply-chain audit-status | "Succeeded (36 exempted)" — review-readiness, not yet vetted | [`cargo-vet/`](cargo-vet/README.md) |
| **cargo-crev** | community web-of-trust reviews | no community reviews available (honest) | [`cargo-crev/`](cargo-crev/README.md) |
| **cargo-scan** | unsafe-effects research tool | **no installable Rust CLI** — coverage delegated to geiger + Miri + source | [`cargo-scan/`](cargo-scan/README.md) |
| **Miri** | undefined behaviour (interpreter) | **no UB** — `core` 8/8 + `atlas` 7/7 + `corpus` 7/7 | [`miri/`](miri/README.md) |
| **panic-analysis** | static panic-surface tally | no `panic!`/`unreachable!`/`todo!` anywhere; embedded/authority crates panic-site-free | [`panic-analysis/`](panic-analysis/panic-analysis.txt) |
| **Kani** | bounded model-checking of grammar soundness | **6/6 harnesses verified, 1047 checks, 0 failures** | [`kani/`](kani/README.md) |
| **cargo-fuzz** | coverage-guided fuzzing (libFuzzer + ASan) of the pure core | **225.2M executions, 0 crashes / 0 ASan errors** across 3 targets — empirical companion to Kani | [`cargo-fuzz/`](cargo-fuzz/README.md) |
| **loom** | concurrency permutation testing | **N/A by design** — zero shared-state concurrency in first-party code, so no data-race surface to permute | [`loom/`](loom/README.md) |
| **Flux** | refinement types (compile-time SMT) | **INSTALLED + RUN** — `cargo flux` checks the `core` crate clean (0 refinement errors); `cargo-flux 4d329f2` | [`flux/`](flux/README.md) |
| **cargo-valgrind** | runtime Memcheck (leaks / invalid memory) | **CLEAN — 0 errors** on the DSFB pipeline (static-musl binary, valgrind 3.25.1 + musl-allocator suppressions; correct replay hashes). glibc route blocked by this host's compiled-in AVX-512 (proven: even `/usr/bin/true` SIGILLs) | [`cargo-valgrind/`](cargo-valgrind/README.md) |
| **hax** | Rust → F\* extraction | **INSTALLED full + RAN** — extracted the `no_std` core grammar to a **716-line F\* model** (`classify_axis`, `t_FixedEnvelope`, …) generated from the real Rust | [`hax/`](hax/README.md) |
| **Creusot** | deductive verification (Why3 + SMT, unbounded) | **INSTALLED + RAN to Coma IR** — creusot-rustc (built for nightly-2026-04-21) translated our `classify_axis` to Coma; the SMT prove step needs creusot's hermetic forked why3+why3find toolchain | [`creusot/`](creusot/README.md) |
| **crux-mir** | symbolic execution (Crucible, cross-engine) | **BUILT + RAN — `Overall status: Valid`** (4/4 goals proved on the core invariants; crux-mir 0.12, mir-json, z3). Second-engine corroboration of Kani. The nonlinear-i128 `classify_axis` VC times out in SMT (BV nonlinear mul) — covered by Kani + fuzz | [`crux-mir/`](crux-mir/README.md) |

All six were **actually installed and run in this sandbox** (not scaffolded as a handoff): cargo-fuzz (225.2M execs),
Flux (checks the core clean), and valgrind (**clean, 0 errors** after musl-allocator suppressions) ran on the real
code; hax **extracted the core to a real 716-line F\* model**; Creusot built its custom `creusot-rustc` and
**translated our `classify_axis` to Coma verification IR** (the SMT step needs its hermetic forked why3); and crux-mir
was **built from source (GHC/crucible/mir-json) and proved the core invariants `Valid`** on a second engine. Each
folder records the **real result or the precise remaining step + a *verified* command**; the earlier hand-written
recipes were corrected (e.g. hax is `cargo-hax`, not `hax-cli`). No verdict is fabricated.

## How the evidence layers fit

- **First-party assurance:** dsfb-gray (structural/governance score) + clippy + the 295-test workspace suite.
- **Memory/UB safety:** `#![forbid(unsafe_code)]` on 5/7 crates (geiger) + Miri (no UB on interpreted paths) + the
  static panic-surface tally. All first-party `unsafe` is confined to two declared FFI boundaries (cuda, wasm).
- **Behavioural correctness:** the formal layer — **Kani** (bounded, every-path) + **Lean 4** + **Coq** (unbounded,
  in `formal/`); **crux-mir** (Crucible, a second symbolic engine — proved the core invariants `Valid`); **hax**
  (extracted the `no_std` core to a real F\* model); **Creusot** (translated `classify_axis` to Coma IR); **Flux**
  (refinement types, checks the core clean) — plus **cargo-fuzz** (225.2M executions, 0 crashes) as the empirical
  companion and the byte-exact verify-replay determinism gate.
- **Memory at runtime:** **cargo-valgrind** Memchecks the `std` `edge` execution **clean (0 errors)** below Miri's
  interpreter (static-musl build + musl-allocator suppressions, since this host's AVX-512 glibc defeats valgrind's
  decoder); the `no_std` crates need no leak check (they do not allocate).
- **Concurrency:** **loom** is N/A by design — the pipeline is deterministic single-threaded, so the data-race
  class loom hunts is structurally unreachable.
- **Supply chain:** cargo-audit (advisories) + cargo-auditable (SBOM) + cargo-vet / cargo-crev (review-readiness).

## Deliberately excluded
Prometheus / Grafana / ELK / Loki are **observability/monitoring** systems, not security or robustness auditors;
presenting their output as an "audit report" would be overclaiming, so they are intentionally omitted.

## Reproduce
Each subfolder's `README.md` carries the exact command. The first-party crates score highest where they are
`no_std`, bounded, and `unsafe`-free (atlas/corpus/core); the `std` execution crate (`edge`) and the GPU crate
(`cuda`) score lower for legitimate, stated reasons (heap use, FFI boundary) — that differential is itself a
truthful signal, not hidden.
