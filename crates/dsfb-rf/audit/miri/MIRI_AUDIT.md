# `dsfb-rf` Miri Audit

[![Miri: clean](https://img.shields.io/badge/Miri-clean-brightgreen)](./RUN_MANIFEST.json)

Dynamic undefined-behaviour audit artefacts produced by
[Miri](https://github.com/rust-lang/miri), the MIR interpreter maintained
by the Rust project.

Audit scope: `dsfb-rf` crate library test surface (v1.0.0).
Host: `x86_64-unknown-linux-gnu`.

---

## Posture

This folder is **not a certification**. It is dynamic UB-detection evidence
for reviewers. DSFB does not certify compliance with IEC, ISO, RTCA, MIL,
NIST, or any other standard. The audit confirms that the library test
surface of `dsfb-rf@1.0.0` executes without Miri-detectable undefined
behaviour under the configurations listed below.

---

## Headline Result

| Configuration | Aliasing model | Tests | Result |
|---|---|---|---|
| `no_std` + strict-provenance | stacked-borrows | 351 / 351 | **CLEAN** |
| `std` + `serde` + stacked-borrows | stacked-borrows | 360 / 360 | **CLEAN** |
| `std` + `serde` + tree-borrows | tree-borrows (stricter) | 360 / 360 | **CLEAN** |

**Zero undefined-behaviour findings across three orthogonal configurations.**

Crate-wide invariant supporting this result: `#![forbid(unsafe_code)]` at
`src/lib.rs` — the compiler refuses to build any `unsafe` block anywhere
in the crate tree, which eliminates by construction the categories of UB
that Miri primarily catches.

---

## What Each Pass Validates

### Pass 1 — pure `no_std`, strict provenance

```
MIRIFLAGS="-Zmiri-strict-provenance" \
  cargo +nightly miri test --no-default-features --lib
```

- No allocator available. Exercises the zero-allocation deployment path
  that targets Cortex-M4F, RISC-V 32-bit, and other bare-metal DSP
  hardware.
- `-Zmiri-strict-provenance` forbids integer-to-pointer casts that would
  not survive a CHERI capability-machine target. Passing this means the
  crate is forward-compatible with strict-provenance runtimes.

Output: [`miri_nostd_strict_provenance.txt`](./miri_nostd_strict_provenance.txt).

### Pass 2 — `std` + `serde`, stacked-borrows

```
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
  cargo +nightly miri test --features std,serde --lib
```

- Exercises the full library surface including artefact writers, JSON
  traceability emission, and the allocator-backed collections used in the
  paper-lock pipeline.
- `-Zmiri-disable-isolation` allows filesystem syscalls that a subset of
  writer tests issue (e.g. `create_dir_all`). This is a sandbox policy
  switch, not a UB assertion; the tests themselves exercise normal
  safe-Rust filesystem APIs.
- Aliasing model: stacked-borrows (the Miri default).

Output: [`miri_std_serde_stacked_borrows.txt`](./miri_std_serde_stacked_borrows.txt).

### Pass 3 — `std` + `serde`, tree-borrows

```
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation -Zmiri-tree-borrows" \
  cargo +nightly miri test --features std,serde --lib
```

- Re-runs the full library surface under the **tree-borrows** aliasing
  model — an experimental stricter alternative to stacked-borrows
  (<https://perso.crans.org/vanille/treebor/>). Tree-borrows rejects some
  patterns that stacked-borrows allows, so passing both models gives
  stronger evidence than either alone.

Output: [`miri_std_serde_tree_borrows.txt`](./miri_std_serde_tree_borrows.txt).

---

## Scope Exclusions (and Why)

### Integration tests (`tests/*.rs`)

Not run under Miri. The four integration tests
(`bit_exactness.rs`, `concurrency_observer.rs`, `long_duration_stability.rs`,
`proptest_invariants.rs`) run 10⁶-sample end-to-end simulations. Native
runtime is seconds; under Miri MIR interpretation they would take an
estimated tens of hours to a full day per binary. They exercise the same
module code paths the lib tests already cover, so the exclusion is about
runtime feasibility, not coverage gaps.

To run them yourself (budget several hours):

```
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
  cargo +nightly miri test --features std,serde --tests
```

### Doc tests

Doc tests build and run code fragments from rustdoc comments.
`cargo test --doc` exercises them under native execution; no Miri gate
because `#![forbid(unsafe_code)]` means doc-test snippets cannot
introduce UB either.

### Release-mode arithmetic overflow

Rust deliberately does not trap on integer overflow in `--release`
builds, so Miri does not flag it either. Overflow semantics in this
crate are validated by the Kani proofs
[`proof_quantize_q16_16_no_panic`](../../src/kani_proofs.rs) and
[`proof_fixedpoint_resync_drift_bounded`](../../src/kani_proofs.rs),
which are stronger (they range over all inputs formally) rather than
weaker than Miri's dynamic sampling.

---

## What Miri Validates Here

- Integer-overflow UB in debug builds (panics in release; formalised
  separately in Kani).
- Out-of-bounds slicing / indexing.
- Use-after-free in `alloc`-backed data structures (when the `std` or
  `alloc` feature is on).
- Aliasing-model violations (both stacked-borrows and tree-borrows).
- Thread-local storage misuse.
- Strict-provenance compatibility (forward-compatible with CHERI
  capability-machine targets).
- `Drop` soundness across panic-unwinding boundaries.

## What Miri Does Not Validate

- Real-time timing bounds (see `.github/workflows/qemu_timing.yml`).
- Stack-depth bounds on bare-metal targets (exercised by
  `cargo check --target thumbv7em-none-eabihf`).
- Hardware-specific SIMD codegen correctness (Miri interprets MIR, not
  target assembly).
- Release-mode arithmetic overflow (Rust does not trap; Kani does).

---

## Reproducibility

Required toolchain:

```
rustup toolchain install nightly
rustup component add --toolchain nightly miri
cargo +nightly miri setup            # one-time sysroot build, ~15 min
```

Then from the crate root:

```
# Pass 1 — no_std
MIRIFLAGS="-Zmiri-strict-provenance" \
  cargo +nightly miri test --no-default-features --lib

# Pass 2 — std + serde, stacked-borrows
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
  cargo +nightly miri test --features std,serde --lib

# Pass 3 — std + serde, tree-borrows
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation -Zmiri-tree-borrows" \
  cargo +nightly miri test --features std,serde --lib
```

Expected wall-clock on an 8-core x86-64 host: ~5 minutes total (50 s +
90 s + 135 s). Nightly rustc and Miri versions under which this audit
was produced are recorded in [`RUN_MANIFEST.json`](./RUN_MANIFEST.json).

---

## Artefact Index

| File | Contents |
|---|---|
| `MIRI_AUDIT.md` | This human-readable report |
| `RUN_MANIFEST.json` | Machine-readable manifest: toolchain versions, flags, pass status, SHA-256 of each `.txt` |
| `miri_nostd_strict_provenance.txt` | Pass 1 full cargo-miri stdout/stderr |
| `miri_std_serde_stacked_borrows.txt` | Pass 2 full cargo-miri stdout/stderr |
| `miri_std_serde_tree_borrows.txt` | Pass 3 full cargo-miri stdout/stderr |

---

## Non-Certification Notice

This report is evidence for reviewers, not evidence for a certifier.
DSFB does not certify compliance with any safety or security standard.
Re-run the commands above on your own hardware if you need to confirm
the result. The `.txt` files in this folder are the literal console
output of the Miri passes they record — line 1 is a cargo progress
banner, the final line is the test-result summary.
