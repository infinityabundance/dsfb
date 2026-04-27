# `dsfb-atlas` Safety Audit

This document records the audit posture of the `dsfb-atlas` crate against
four tools: **dsfb-gray** (paperstack-defined Rust threat-surface scan),
**Miri** (Rust UB checker), **Kani** (Rust bounded model checker), and
**cargo-fuzz** (libfuzzer-driven fuzzer). The crate is ~600 LOC of
pure-data Rust: no `unsafe`, no FFI, no concurrency, no network, no
`Cell`/`UnsafeCell`. The audit's load-bearing claim is the *negative*
result — that this attack surface is genuinely empty — and that the one
non-trivial invariant (`Dedup::record` correctly reports collisions)
holds.

## 1. dsfb-gray (DSFB Structural Semiotics Engine for Rust crate auditing)

**Role.** Static structural scan that produces an advisory assurance
score across six subscores (correctness, maintainability, concurrency /
async, resource discipline, verification / reviewability, assurance /
provenance). Backed by a checkpoint rubric (Safety 15%, Verification 15%,
Build/Tooling 10%, Lifecycle/Governance 10%, NASA/JPL Power of Ten 25%,
Advanced Structural Checks 25%). Emits SARIF, in-toto, and DSSE
attestations alongside the human-readable text report. **dsfb-gray
deliberately does not certify compliance** — it is a guideline for
review readiness.

**Invocation.**

```bash
cd /home/one/dsfb
cargo build --release -p dsfb-gray --bin dsfb-scan-crate
./target/release/dsfb-scan-crate \
    --out-dir crates/dsfb-atlas/audit/reports/dsfb-gray-runs \
    ./crates/dsfb-atlas
```

The wrapper script `audit/scripts/dsfb_gray.sh` does the same and falls
back to a portable grep-based threat-surface scan when the dsfb-gray
binary is unavailable.

**Outputs.** Per-run subdirectory at
`audit/reports/dsfb-gray-runs/dsfb-gray-<UTC-timestamp>/` containing:
- `dsfb_atlas_scan.txt` (human-readable report)
- `dsfb_atlas_scan.sarif.json` (SARIF v2.1 for code-scanning consumers)
- `dsfb_atlas_scan.intoto.json` (in-toto attestation envelope)
- `dsfb_atlas_scan.dsse.json` (DSSE; unsigned unless `DSFB_SCAN_SIGNING_KEY` is set)

The canonical machine-readable summary lives at
`audit/reports/dsfb_gray.json` (overall score, subscores, threat-surface
counters, source SHA-256, scan timestamp).

**Pass criterion.** Overall score reported with non-empty subscore
breakdown; threat-surface counters all zero (`unsafe`, FFI, net, threads,
shell, `-sys`); no `license_violations`. The current run reports
**Overall: 68.7%** with `Advanced Structural Checks: 95.7%` and
`NASA/JPL Power of Ten: 55.0%` — typical mixed-assurance posture for a
pure-data-pipeline crate.

**Expected runtime.** ~5 seconds.

**Honest limit.** dsfb-gray scores are advisory and structural. They
highlight where review effort has the highest yield (Power of Ten
bounded-loop / bounded-allocation prompts; serde-related design-review
prompts) but they do not constitute a compliance or certification badge.

## 2. Miri (undefined-behaviour checker)

**Role.** Interprets the program against the Rust abstract machine and
flags any UB: out-of-bounds access, use-after-free, invalid `Box`,
aliasing violations under Stacked / Tree Borrows, uninitialised reads.

**Invocation (full pass on the binary).**

```bash
cd /home/one/dsfb
rustup +nightly component add miri
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows -Zmiri-disable-isolation" \
cargo +nightly miri run --release --bin dsfb-atlas -- \
    --spec-dir crates/dsfb-bank/spec/atlas \
    --bank-spec-dir crates/dsfb-bank/spec \
    --out /tmp/dsfb_atlas_miri_out \
    --git-hash miri-run
```

`-Zmiri-disable-isolation` is required because the binary does
filesystem I/O.

**CI-tractable invocation (unit tests of dedup + schema only).**

```bash
cargo +nightly miri test -p dsfb-atlas --release
```

**Pass criterion.** Process exits 0; final stdout line is
`OK: 10,000 atlas theorems generated with structurally unique proofs.`;
Miri prints no `Undefined Behavior:` diagnostics.

**Expected runtime.** Full pass: 25–60 minutes (Miri is ~100× slower
than native; the pipeline does substantial string allocation). CI
unit-test pass: <60 seconds.

**Honest limit.** With zero `unsafe`, zero FFI, zero raw pointers, Miri
is overwhelmingly likely to find nothing. The value is regression
protection: if a later edit adds `unsafe` (e.g. to speed up YAML
parsing), Miri catches the inevitable mistake.

## 3. Kani (model checker)

**Role.** Bounded model checking of Rust functions. The non-trivial
invariant in this crate is in `dedup.rs`: *for any sequence of
`Dedup::record(id_i, body_i)` calls, the finalize report's `collisions`
field contains exactly the pairs `(id_a, id_b)` whose body bytes are
SHA-256-equivalent.* Everything else is straight LaTeX-string
concatenation.

**Harness.** `src/dedup.rs` includes a `#[cfg(kani)]`-gated proof
harness (see `dedup_collision_iff_repeated_body`).

**Invocation.**

```bash
cd /home/one/dsfb/crates/dsfb-atlas
cargo install --locked kani-verifier && cargo kani setup
cargo kani --harness dedup_collision_iff_repeated_body
```

**Pass criterion.** Kani output ends with `VERIFICATION:- SUCCESSFUL`;
no `Failed Checks` lines; no unwind assertions firing within the bound.

**Expected runtime.** ~30 seconds for the bounded proof at unwind 4;
~2 minutes at unwind 8.

**Honest limit.** Kani cannot prove the headline statistical claim
("10,000 real proof bodies do not collide under SHA-256") — that is a
property of the inputs and of SHA-256, not of the crate. Kani proves
the much narrower, but actually-load-bearing, claim that the dedup
*report* faithfully surfaces any collision the SHA-256 oracle reports.
That is the one logical bug that could make the build's "0 collisions"
attestation a false negative.

## 4. cargo-fuzz (libfuzzer-driven fuzzer)

**Role.** Drives randomised byte inputs into a target function looking
for panics, aborts, sanitizer trips, or excessive memory use. The
natural target here is the YAML parser (`schema::Part` deserialisation)
— the only place untrusted-shaped input can reach allocation-heavy code.

**Target.** `fuzz/fuzz_targets/yaml_part.rs` defines the harness;
`fuzz/corpus/yaml_part/` is seeded with the ten real `P01_*.yaml …
P10_*.yaml` files for fast convergence.

**Invocation.**

```bash
cd /home/one/dsfb/crates/dsfb-atlas
cargo install --locked cargo-fuzz
cargo +nightly fuzz run yaml_part -- -max_total_time=1800 -max_len=65536
```

**Pass criterion.** Fuzzer terminates after the time bound with no
`crash-*` files written into `fuzz/artifacts/yaml_part/`.

**Expected runtime.** 30 minutes for the smoke run; 24 hours for a
release-gating run.

**Honest limit.** `serde_yaml` and `serde` are mature, widely-fuzzed
crates; the most likely finding is a panic on absurd numeric coercions
or a stack overflow on deeply nested YAML — neither is a security
boundary because the YAML inputs are committed source, not user input.
The reason to run cargo-fuzz anyway is that a fuzz corpus checked into
the repo is *evidence* that the unique-proof claim is falsifiable, which
materially strengthens the defensive-publication posture.

## Status (latest run)

| Tool          | Status | Notes                                                                         |
|---------------|--------|-------------------------------------------------------------------------------|
| `dsfb-gray`   | PASS   | Overall **68.7 %** (advisory). Subscores: correctness 71.1, maintainability 65.8, concurrency 62.5, resource discipline 58.3, verification 63.3, assurance 66.9. Threat surface empty (0 unsafe, 0 FFI, 0 net, 0 threads, 0 shell, 0 `-sys`). Full report at `audit/reports/dsfb-gray-runs/dsfb-gray-2026-04-26T22-02-36Z/dsfb_atlas_scan.txt`. |
| Miri (tests)  | PASS   | <60 s; no UB diagnostics.                                                     |
| Kani          | PASS   | `dedup_collision_iff_repeated_body` green.                                     |
| cargo-fuzz    | PASS   | 30 min smoke run; 0 crashes; corpus seeded.                                   |
