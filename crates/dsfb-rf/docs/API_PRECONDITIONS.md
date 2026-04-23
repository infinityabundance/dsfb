# `dsfb-rf` — API Preconditions, Exit Codes, and Error-Propagation Style

This document is the single source of truth for:

1. Library-caller preconditions on the observer entry points
   (`run_stage_iii`, `GrammarEvaluator::observe`,
   `AdmissibilityEnvelope::calibrate`).
2. The `paper-lock` binary's exit-code table.
3. The principled disposition of `.unwrap()` / `.expect()` sites in
   the `examples/` bank.
4. Error-propagation style guidance for downstream contributors.

It supplements (does not replace) the rustdoc on each module.
Nothing here narrows the library's public API; it documents the
contract that is already in force.

---

## 1. Library Preconditions

### 1.1  `pipeline::run_stage_iii`

```rust
pub fn run_stage_iii(
    dataset_name: &'static str,
    observations: &[RfObservation],
    ground_truth_events: &[RegimeTransitionEvent],
) -> Stage3Result
```

**Preconditions the caller must satisfy.**

| Precondition | Consequence if violated |
|---|---|
| `observations.len() >= calibration.healthy_window` | Calibration pass panics with descriptive message from `AdmissibilityEnvelope::calibrate` |
| `observations[0..healthy_window]` represents a healthy capture (no regime transition in that window) | Envelope ρ is calibrated to contaminated data; downstream episodes may under- or over-fire without panic, but the Stage~III metrics lose their meaning |
| `ground_truth_events` timestamps must fall within `[0, observations.len())` | No panic; events outside the window are silently excluded from the denominator. Intentional to allow GT sets that span larger captures than the current slice |

**Postcondition.** `Stage3Result::episode_precision` and
`Stage3Result::recall_numerator / recall_denominator` are defined
even on degenerate inputs. `compression_factor` is $\geq 1.0$ by
construction.

### 1.2  `grammar::GrammarEvaluator::observe`

```rust
pub fn observe(&mut self, residual: &ResidualSample) -> GrammarTransition
```

**Preconditions.**

- `residual.norm >= 0.0` (physical invariant of `||r||`).
- `residual.timestamp_k` monotonically non-decreasing across calls
  within a single evaluator instance. Epoch resets are handled by
  `reset_on_calibration_fault()` — not by rewinding `k`.

**Postcondition.** `observe` is pure-total: there is no
configuration of valid `residual` that can induce a panic. The
[Kani] `grammar_panic_freedom` proof locks this invariant.

### 1.3  `envelope::AdmissibilityEnvelope::calibrate`

```rust
pub fn calibrate(&mut self, healthy_window: &[f32]) -> Result<(), EnvelopeFault>
```

**Preconditions.**

- `healthy_window.len() >= 1`.
- All entries finite (no NaN, no ±∞).
- At least one entry must be strictly positive for the 3σ estimator
  to produce a non-degenerate ρ.

**Postcondition.** On `Err(EnvelopeFault::InsufficientSamples)` or
`Err(EnvelopeFault::DegenerateVariance)` the envelope remains in its
prior state; callers should **not** observe until calibration
succeeds.

---

## 2. `paper-lock` Binary Exit-Code Table

`src/main.rs` is the `paper-lock` CLI entry point.

| Exit code | Condition | Source |
|-----------|-----------|--------|
| `0` | Successful execution of the requested subcommand | default return |
| `1` | Unknown subcommand passed as `argv[1]` | `src/main.rs:63` |
| `1` | No subcommand provided (missing `argv[1]`) | `src/main.rs:67` |
| `2` | Subcommand executed but pipeline failed (HDF5 load failure, shape mismatch, or `run_stage_iii` error) | `src/main.rs:57` |
| `101` | Panic — should not occur under documented preconditions. Treat as a bug and open an issue. | Rust default panic handler |

Notes.

- The `paper-lock` binary is only built when
  `--features std,paper_lock,hdf5_loader` are enabled (see
  `Cargo.toml:51`).
- Exit codes `0`, `1`, and `2` are the **stable contract**. Exit
  code `101` is not a stable contract — it is a latent-bug
  indicator.

---

## 3. Example-Binary `.unwrap()` / `.expect()` Disposition

The `examples/` directory contains driver programs that exercise the
library on synthetic and real-data fixtures. As of v1.0.0 there are
99 `.unwrap()` / `.expect()` sites across the example bank. They
fall into three categories:

### 3.1  Calibration-asserted preconditions — **retained**

Sites where the invariant is asserted by upstream calibration and a
violation indicates a programmer error, not a recoverable fault.
These carry a descriptive `.expect("…")` message that names the
calibration step whose violation produced the panic.

**Example.**

```rust
let rho = envelope.rho().expect("envelope calibrated in Stage II — must have ρ");
```

### 3.2  Silent-default masking — **replaced in v1.0.1**

Sites that wrapped a legitimate `Result` or `Option` in
`.unwrap_or(default)` where the default silently corrupts downstream
figure data. These are replaced with structured error propagation
so the example binary exits with code 2 and a clean message instead
of emitting a figure that silently misrepresents the residual.

**Before.**

```rust
let value = read_field(&row, "snr").unwrap_or(0.0);   // masking
```

**After.**

```rust
let value = read_field(&row, "snr")
    .ok_or_else(|| format!("row {i}: missing required 'snr' column"))?;
```

### 3.3  Control-flow unwraps — **replaced in v1.0.1**

Sites where `.unwrap()` was used as a shortcut for control-flow on a
`Result` that has a user-recoverable path (e.g. the file does not
exist because the user hasn't downloaded the dataset). Replaced with
`?` propagation so the binary exits 2 with the error message
instead of a panic backtrace.

---

## 4. Error-Propagation Style Guide

For downstream contributors writing new examples:

1. **Use `?` at the outermost boundary** — return
   `Result<(), Box<dyn std::error::Error>>` from `fn main`.
2. **Wrap error messages with context.** Prefer
   `err.with_context(|| format!("…"))` or
   `map_err(|e| format!("{label}: {e}"))` over bare `?`.
3. **Reserve `.expect("…")` for calibrated preconditions.** The
   `.expect` message must name the invariant (e.g. *"envelope
   calibrated in Stage II — must have ρ"*), not the call site.
4. **Never use `.unwrap_or(default)` to mask a legitimate `Err`.**
   If the default is physical (e.g. noise floor of 0 dB), document
   it in the field loader and unit-test the default branch. If the
   default is a shortcut to avoid handling the error, write the
   handler.
5. **Panics are bugs in production paths.** The observer core
   (`src/engine.rs`, `src/grammar.rs`, `src/dsa.rs`,
   `src/envelope.rs`) is panic-free by Kani proof. Example binaries
   should exit cleanly on bad input; they should not panic.

---

## 5. Kani Proof Surface

The library invariants documented above are locked by six Kani
harnesses (`src/kani_proofs.rs`):

| Harness | Invariant |
|---|---|
| `grammar_panic_freedom` | `GrammarEvaluator::observe` never panics on any valid input |
| `severity_monotonicity` | Severity scores are monotone under fixed input sequences |
| `envelope_consistency` | Envelope judgement is consistent with ρ and ‖r‖ |
| `decimation_epoch_bound` | Decimation epoch counter remains within bounds |
| `fixed_point_resync_drift` | Q16.16 resync cannot drift beyond documented ulp |
| `q16_16_quantize_panic` | Q16.16 quantize is panic-free across all finite f32 inputs |

The `quality.yml` CI workflow runs these proofs on every PR; the
`qemu_timing.yml` workflow validates per-sample-latency on
Cortex-M4F, RISC-V 32-bit, and x86-64.
