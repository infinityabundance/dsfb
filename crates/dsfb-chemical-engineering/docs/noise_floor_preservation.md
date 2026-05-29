# Noise-floor preservation: preserved evidence, not detection

> **Prior-art disclosure.** This document states, as prior art in its own right, a falsifiable
> *preserved-evidence* property of the DSFB-Chemical-Engineering evidence contract: the sub-threshold
> noise floor that conventional process monitoring discards is sealed, bit-exact, into a hash-linked
> record. It is **explicitly not** a detection, sensitivity, or lead-time claim. The bound is the point.

## 1. What conventional monitoring discards

A control chart or alarm system keeps only what crosses a limit. A residual that stays below the alarm
threshold — the "noise floor" — leaves no persistent trace: it is not recorded, not replayable, and not
available to a later forensic review. Whatever structure was developing under the limit in the minutes
before an event is gone.

## 2. What the evidence contract preserves

The CUDA/CPU evidence factory (`crates/dsfb-chemical-engineering-cuda/src/evidence.rs`) seals, **per
sample**, a canonical 40-byte little-endian record into a per-lane SHA-256 digest:

```text
raw_bits(x) ‖ q ‖ e ‖ d ‖ s        (8 + 8 + 8 + 8 + 8 = 40 bytes)
```

- `raw_bits(x)` — the residual `x` reinterpreted as a `u64` (IEEE-754 bit pattern, **not** converted).
- `q` — fixed-point quantised residual (`round(x · 1e6)`); `e = max(0,q)`; `d` — causal windowed drift
  (integer ring buffer); `s = q[i] − q[i−1]`.

Because `raw_bits(x)` is hashed, **the lane digest changes even when two physically different residuals
round to the same fixed-point integer `q`.** Sub-quantisation information — including the noise floor an
alarm-limit comparison never sees — is therefore retained in the sealed record and re-emerges
byte-for-byte on replay. The digest chains into the lane digest → Merkle root → `evidence_root`, so the
preserved noise floor is anchored in the same tamper-evident hash chain as everything else in the court
record.

## 3. The bound — what this is **not**

This is a *preserved-evidence* property, not a detection capability. The distinction is mechanical, not
rhetorical:

| Claim | Status |
|---|---|
| The noise floor is recoverable, bit-exact, from the sealed record on replay | **Yes** (the disclosed property) |
| The inference path reads the raw bits to raise / grade / admit an episode | **No** — the DSFB grammar consumes only the residual triple; raw bits enter the SHA-256 digest and nothing else |
| Any sub-threshold sensitivity, earlier detection, or lead-time improvement | **No claim** — two runs whose residuals differ only below the quantisation grid produce identical episodes and badges; only their digests differ |
| "Sealing a sample makes it detectable" | **False** — sealing ≠ detecting |

The property is falsifiable in one direction only: it guarantees the noise floor *is in the record*, not
that anything diagnostic was learned from it.

## 4. Why preserve it

The value is forensic, not diagnostic. In a post-incident or regulated-record review the question is
often: *"what did the instrument actually report, bit for bit, in the minutes before the event?"* A
probabilistic confidence score or a thresholded alarm cannot answer it. A hash-linked record that sealed
the raw bits can — and can prove the record was not altered after the fact. This is the same
auditability argument as the rest of the Chemical Court Record, applied to the part of the signal that
monitoring normally throws away.

## 5. Determinism note

Quantisation is a single IEEE-754 double multiply plus round-half-away-from-zero (no fused
multiply–add); the CUDA translation unit is compiled with `--fmad=false` and the Rust reference performs
no implicit FMA, so the CPU and GPU evidence paths agree bit-for-bit. The frozen
`crates/dsfb-chemical-engineering-cuda/tests/golden_evidence.rs` gate pins the lane digest and
`evidence_root` of a bit-portable input, so any drift in this contract fails loudly even without a GPU.

## 6. Where this lives

Paper: `\section{Noise-floor preservation: preserved evidence, not detection}` (`sec:noisefloor`).
Code: the evidence contract in `evidence.rs` (`lane_evidence_cpu`, `CONTRACT_ID`) and the CUDA
`evidence_kernel`. README: the "Noise-floor preservation" section. Disclosed as prior art independent of
any specific deployment.
