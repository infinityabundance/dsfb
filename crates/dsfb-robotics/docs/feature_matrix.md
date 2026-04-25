# `dsfb-robotics` Feature-Flag Matrix

This document satisfies NASA/JPL Power-of-Ten Rule 8 (*conditional
compilation and metaprogramming stay minimal*) for the `dsfb-robotics`
crate by enumerating every `#[cfg(feature = ...)]` site in the source
tree, the feature it gates, and the justification for the fork.

**Scope.** Only `feature = ...` cfg sites in [`src/`](../src/) are
listed; the crate ships zero `macro_rules!`, zero proc-macros, and no
`#[cfg(target_os ...)]` or `#[cfg(target_arch ...)]` forks.

## Feature inventory

| Feature | Default? | Purpose | Activates |
|---|---|---|---|
| `default = []` | n/a | Empty default — `no_std` + no_alloc surface is the canonical core | nothing |
| `alloc` | no | Pulls in `extern crate alloc;` — needed by anything that allocates `String`, `Vec`, etc. | `extern crate alloc` in [`src/lib.rs:117`](../src/lib.rs#L117) |
| `std` | no | Adds `std`, transitively enables `alloc`. Required by `paper_lock` and `real_figures` features. | `extern crate std` in [`src/lib.rs:120`](../src/lib.rs#L120) |
| `serde` | no | Optional serde derive on public types (`DatasetId`, `DatasetFamily`, `PaperLockReport`, `EpisodeRecord`). Required by `paper_lock`. | `serialize_report` in [`src/paper_lock.rs:459`](../src/paper_lock.rs#L459); test in [`src/paper_lock.rs:605`](../src/paper_lock.rs#L605) |
| `paper_lock` | no | Compiles the `paper-lock` binary (feature-gates the whole `crate::paper_lock` module + the `bin/paper-lock` target). Implies `std + serde + serde_json + csv`. | `pub mod paper_lock` in [`src/lib.rs:180`](../src/lib.rs#L180) |
| `real_figures` | no | Enables the `--emit-episodes` JSON-trace output and CSV ingestion path. Implies `std + serde + csv`. | (no direct cfg site — feature is solely a Cargo dep activator) |

## Per-site `#[cfg(feature = ...)]` ledger

| File | Line | Feature gate | Item gated | Why this fork is necessary |
|---|---|---|---|---|
| [`src/lib.rs`](../src/lib.rs#L116) | 116 | `alloc` | `extern crate alloc;` | The `no_std` core never allocates; `alloc` is opt-in via this feature so adapters needing `Vec`/`String` (e.g. CSV ingestion) compose. |
| [`src/lib.rs`](../src/lib.rs#L119) | 119 | `std` | `extern crate std;` | Same rationale at the higher level: `std` is gated so the bare-metal target keeps zero std symbols. |
| [`src/lib.rs`](../src/lib.rs#L180) | 180 | `paper_lock` | `pub mod paper_lock;` | The `paper_lock` module is the CLI implementation; it pulls in serde-json and the `Vec`-based report type, so it stays out of the no_std core. |
| [`src/paper_lock.rs`](../src/paper_lock.rs#L459) | 459 | `serde` | `pub fn serialize_report` | Serialisation lives behind the optional `serde` feature so the report type (which is itself derive-gated) only emits a public serialiser when the derive is present. |
| [`src/paper_lock.rs`](../src/paper_lock.rs#L605) | 605 | `serde` | unit-test `serialize_report_round_trips` | Symmetric: the test is meaningful only when the derive and the serialiser are both compiled in. |

## Composite-cfg attestation

This crate uses **no** `#[cfg(any(...))]`, `#[cfg(all(...))]`, or
`#[cfg(not(...))]` over `feature = ...` predicates anywhere in `src/`
or `tests/`. Verify with:

```sh
grep -rn '#\[cfg(any\|#\[cfg(all\|#\[cfg(not' crates/dsfb-robotics/src/ \
                                              crates/dsfb-robotics/tests/
```

## Macro attestation

This crate defines **zero** `macro_rules!`, zero `#[proc_macro]`, zero
`#[proc_macro_derive]`, and zero `#[proc_macro_attribute]`. Verify
with:

```sh
grep -rn 'macro_rules!\|#\[proc_macro' crates/dsfb-robotics/src/ \
                                       crates/dsfb-robotics/tests/
```

## Reviewer guidance

1. The `default = []` empty-default policy is intentional. Bare `cargo
   build -p dsfb-robotics` builds the no_std + no_alloc surface — the
   minimum-trust kernel that the engine, observe, math, and per-dataset
   adapter modules are designed to live in.
2. The `paper_lock` and `real_figures` flags layer the std-only,
   alloc-friendly surface on top. They never gate behaviour inside
   the no_std core; they only gate the existence of additional surface
   (the CLI binary, the CSV ingestion path, the JSON serialiser).
3. There is no third state. Either the no_std core is built standalone
   (zero conditional compilation paths active), or one of the std
   layers is added (which monotonically extends — never overrides —
   the core surface).

## Verification

The cfg ledger above is mechanically reproducible:

```sh
grep -rn '#\[cfg(feature' crates/dsfb-robotics/src/
```

If the count of lines differs from the table above, the table is the
source of truth — update the table or remove the new cfg site.
