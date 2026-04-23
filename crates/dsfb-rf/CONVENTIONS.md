# CONVENTIONS.md — Engineering Standards for DSFB-RF

**de Beer, R. (2026) · Invariant Forge LLC**
**Maintained from four authoritative perspectives:**
- **[SBIR]** Elite SBIR Phase I/II/III engineer
- **[RF]** Senior RF / EW systems engineer (VITA, SOSA, MORA)
- **[PAPER]** DSFB mathematical framework author
- **[IEEE]** Peer-review standards (IEEEtran, GUM, reproducibility)

Read AGENTS.md first for AI agent operating rules.

---

## Part I — Mathematical and Scientific Integrity
*Perspective: [PAPER] + [IEEE]*

### I.1 · The Central Object — Sign Tuple Invariant

Every contribution to this codebase must preserve the integrity of the sign
tuple as a semiotic manifold coordinate:

```
σ(k) = (‖r(k)‖, ṙ(k), r̈(k))
```

- `‖r(k)‖` — residual norm. What every SNR threshold sees.
- `ṙ(k)` — finite-difference drift rate. What thresholds discard.
- `r̈(k)` — trajectory curvature. What filters on ‖r‖ cannot see.

**No change to the sign tuple definition without updating Theorem 1, Lemma 3,
and the `sign.rs` module simultaneously.** The three must stay in sync.
Drift computation uses a causal window of width W only — never future samples.
Missing/sub-threshold samples contribute zero to drift and slew sums
(missingness-aware; not post-hoc masking).

### I.2 · Grammar State Assignments

The grammar FSM states must match the paper exactly at all times:

| State | Condition |
|---|---|
| `Violation` | `‖r(k)‖ > ρ_eff` |
| `Boundary` | `‖r(k)‖ > 0.5ρ_eff AND (ṙ > 0 OR \|r̈\| > δ_s)` OR recurrent near-boundary hits ≥ K in window W |
| `Admissible` | otherwise |

Hysteresis: **2 consecutive confirmations** required before any state
transition commits. This is Lemma 5 — do not remove or relax this without
updating the lemma. The reason code (`SustainedOutwardDrift`,
`AbruptSlewViolation`, `RecurrentBoundaryGrazing`, `EnvelopeViolation`)
must always accompany the `Boundary` variant.

### I.3 · GUM Uncertainty Budget — Non-Negotiable

The envelope radius ρ is derived from JCGM 100:2008 (GUM), not from an
informal "3σ rule". The budget structure is:

```
u_A  = σ_healthy / √N             (Type A: statistical, WSS-verified)
u_B  = √(Σᵢ u_B,i²)              (Type B: noise figure, ADC quantisation,
                                    thermal drift, LO phase noise, IQ imbalance)
u_c  = √(u_A² + u_B²)
U    = k · u_c   (k=3, 99.7%)
ρ    = μ_healthy + U
```

**WSS pre-condition must pass before the budget is computed.** A failed WSS
check invalidates the Type A term and must propagate a warning, not a
silent fallback. Do not add any path that bypasses the WSS check.

### I.4 · Bounded Claims — The Hard Ceiling

The following claims are established by the paper and must not be exceeded
in any context — README, code comments, example output, figures, or
any other artifact:

| Established claim | Source |
|---|---|
| RadioML precision: **73.6 %** | Table I, Stage III protocol |
| RadioML recall: **95.1 %** (97/102) | Table I |
| RadioML compression: **163×** (14 203 → 87) | Table I |
| ORACLE precision: **71.2 %** | Table I |
| ORACLE recall: **93.4 %** (96/102) | Table I |
| ORACLE compression: **132×** (6 841 → 52) | Table I |
| False episode rate on clean windows: **4.4–6.3 %** | §IX negative control |
| SNR floor: **−10 dB** | L10 |

**Do not claim numbers above these.** Do not state "near-perfect", "optimal",
or "best-in-class" anywhere. The false episode rate must never be hidden —
it is part of academic honesty. Every figure that shows precision/recall
must include the corresponding false-episode rate.

### I.5 · Limitations Disclosure (L1–L12) — Always Present

The twelve Limitations (L1–L12) from the paper front-matter are legally and
academically significant prior-art scope delineators. They must appear in:
- Paper abstract box (always)
- README Limitations Disclosure section (always)
- Any operator-facing documentation

They must never be softened, abbreviated, or removed. If a new claim is
added that partially conflicts with a limitation, add a new limitation —
do not modify the existing ones.

### I.6 · Theorem / Lemma Correspondence

| Theorem/Lemma | What it covers | Where in code |
|---|---|---|
| Theorem 1 | Sign tuple as semiotic manifold coordinate | `src/sign.rs` |
| Theorem 9 | Determinism: identical inputs → identical outputs | All modules |
| Lemma 5 | Hysteresis gate: false-episode bound | `src/grammar.rs` |
| Lemma 6 | Corroboration monotone: FP decreases with c | `src/dsa.rs` |
| Envelope Exit Theorem | Finite escape time from admissible set | `src/envelope.rs` |

If code contradicts any theorem, the code is wrong — not the theorem.

---

## Part II — RF Systems Engineering Standards
*Perspective: [RF]*

### II.1 · Non-Intrusion Contract — The Safety Argument

This is not a limitation. It is the primary safety and deployment argument.

**At every point in the call graph, the DSFB-RF engine must satisfy:**
1. Input: `residual_norm: f32` — a **copied** scalar, never a reference
   into upstream signal memory.
2. No write path exists into any upstream data structure.
3. `observe()` returns an advisory `ObserveResult` — it does not set flags,
   modify AGC gains, change detection thresholds, or emit control signals.
4. Removing DSFB requires zero reconfiguration of upstream systems.

This is enforced at the type level by `#![forbid(unsafe_code)]` and by the
`observe()` signature. **Never add a `&mut` reference to any upstream type.**

### II.2 · VITA 49.2 / SOSA / MORA Compatibility

Input interfaces must remain mappable to VITA 49.2 VRT context packets:
- `PlatformContext` fields correspond to VRT context extension packet fields
- `ResidualSource` trait maps to VITA 49.2 data packet payload (`&[f32]`)
- Zero-copy on receive path: reads from DMA buffers in SOSA deployments

When adding fields to `PlatformContext`:
- Document the VITA 49.2 field mapping in the doc comment
- Use `f32` or fixed-width integer types only (no `f64` in hot path)
- Maintain `#[repr(C)]` compatibility for FFI into GNU Radio blocks

The MORA Software Resource characterisation (non-intrusive Observer) must
remain accurate. Do not add any control-plane interface.

### II.3 · SNR Floor Enforcement

Below the SNR floor (nominally −10 dB), grammar must be forced to
`Admissible` regardless of residual norm. This is not optional.

```rust
if ctx.estimated_snr_db < self.snr_floor_db {
    return ObserveResult::below_floor(k);
}
```

This applies unconditionally. Do not allow grammar computation on
sub-floor observations even if the residual norm is high (interference
may be the cause, not a structural event).

### II.4 · Calibration Requirements

**Envelope calibration is required per waveform class (L7).** Code must
never present a "universal" threshold or default envelope. The calibration
API must:
- Require a healthy-window sample set as explicit input
- Run WSS verification before computing ρ
- Return a `CalibrationResult` with `is_reliable` flag based on WSS
- Log the full GUM budget in the audit trail

For embedded deployment, calibration runs once at startup from a stored
healthy window. Recalibration is required on waveform class change.

### II.5 · Hardware Targets

Must compile clean for all three targets at all times:

```sh
cargo check --target thumbv7em-none-eabihf  --no-default-features   # Cortex-M4F
cargo check --target riscv32imac-unknown-none-elf --no-default-features  # RISC-V
cargo check                                  --features std          # x86-64 host
```

Stack usage in `observe()` must remain below 512 bytes (Cortex-M4F budget).
The bench reports this — verify it stays within budget after any hot-path edit.

### II.6 · Latency Budget

The benchmark (`cycles_per_sample.rs`) measures cycles per sample.
At 2 MS/s, the budget is 500 ns/sample on an x86-64 host reference.
Target: < 5000 ns on Cortex-M4F, < 10 000 ns on RISC-V (QEMU).

**After any edit to the hot path (`observe()`, `sign.rs`, `grammar.rs`,
`dsa.rs`), run the bench and compare to the baseline:**
- x86-64 baseline: ~31 ns/sample (nominal), ~29 ns (drift)
- Cortex-M4F baseline: measured in QEMU CI

### II.7 · GNU Radio Integration (gr-dsfb)

The `gr-dsfb/` OOT module is a deployment artifact, not a core library.
It must:
- Load `libdsfb_rf.so` via ctypes with a pure-Python fallback
- Never import from `gr_dsfb` into `dsfb-rf` Rust crate (dependencies only flow one way)
- ZeroMQ PUSH socket endpoint is configurable — no hardcoded addresses
- GRC block YAML parameters must match `PlatformContext` fields exactly

---

## Part III — SBIR Deliverable Standards
*Perspective: [SBIR]*

### III.1 · TRL Tracking

Current TRL: **4** (component validation in laboratory environment).
Phase I exit requires TRL 4 with documented element performance.
Phase II threshold: TRL 5–6 (relevant environment).

Do not claim TRL 5+ until:
- Real hardware receiver integration is demonstrated (not QEMU)
- VITA 49.2 live stream tested on physical USRP B200
- Non-intrusion verified on live signal path (not synthetic data)

Code comments, README, and paper must be consistent on TRL level.

### III.2 · IP Protection — Prior Art Timestamps

Every substantive technical contribution must be:
1. Committed to the git repository with a meaningful commit message
2. Reflected in the paper (Zenodo DOI when registered)
3. Present in the output artifact zip (traceability)

The prior art date is the Zenodo deposit timestamp. The v1.0 DOI has been
minted: `https://doi.org/10.5281/zenodo.19702330`. All crate-surface
references (README, CITATION.cff, lib.rs rustdoc, docs/*, Colab notebook
§6) must carry the resolved DOI — never revert to the `XXXXXXXX`
placeholder. Every output ZIP continues to constitute a timestamped
delivery record on top of the Zenodo deposit.

**Never remove the "Prior art under 35 U.S.C. § 102" notice from any file.**

### III.3 · Deliverable Traceability

Every claim in the paper must trace to:
1. A function or module in `dsfb-rf/src/`
2. A figure in `dsfb-rf-output/dsfb-rf-<timestamp>/figs/`
3. A data value in `figure_data_all.json`

When adding a new paper claim, add the corresponding code path, figure, and
JSON key before the claim is considered complete.

### III.4 · Dependency Hygiene

The Rust crate must remain attractive to DoD/IC programme offices:
- No crates from unvetted authors without documented justification
- No `build.rs` scripts that phone home or require internet access
- `cargo deny` must pass clean (`deny.toml` is the authority)
- All dependencies Apache-2.0 or MIT licensed — no GPL in the crate graph
- `serde` and `serde_json` are `std`/`serde` feature-gated only

### III.5 · Documentation as Deliverable

The following files are SBIR deliverables — they must be kept current:
- `README.md` — operator-facing, limitations-honest, results-current
- `docs/non_intrusion_contract.md` — safety argument for S&T reviewer
- `docs/uncertainty_budget_gum.md` — measurement traceability for metrology review
- `docs/sosa_mora_alignment.md` — architecture alignment for acquisition
- `docs/radioml_oracle_protocol.md` — reproducibility protocol

**Do not modify these files to remove negative results.** A 4.4–6.3 % false
episode rate on clean windows must remain disclosed. SBIR technical
monitors will verify this against the data.

### III.6 · Output Artifact as Delivery Package

Every `dsfb-rf-<timestamp>-artifacts.zip` is a candidate delivery package.
It must be self-contained: a reviewer with no build environment must be
able to open the ZIP and see all figures, all data, and the combined PDF.

---

## Part IV — Code Quality Standards
*Perspective: [IEEE] + [SBIR]*

### IV.1 · Module Conventions

Each source file (`src/*.rs`) must have:
- A top-level `//!` module doc comment explaining its role in the pipeline
- A doc comment on every `pub` item with a `# Examples` section
- No `#[allow(dead_code)]` without a comment explaining why it is kept
- No `todo!()`, `unimplemented!()`, or `panic!()` in the hot path

### IV.2 · Test Coverage Standards

Every mathematical operation in a theorem or lemma must have a unit test
that verifies the stated bound or property explicitly — not just that
the function runs without panicking.

Specifically:
- Lemma 6: the test must verify FP rate decreases as corroboration increases
- Lemma 5: the test must verify no state transition occurs on a single event
- Theorem 9: the replay test must verify bit-identical output

### IV.3 · Naming Conventions

Use the paper's mathematical notation directly in code identifiers:

| Math | Code |
|---|---|
| σ(k) | `sign_tuple` |
| ṙ(k) | `drift_rate` |
| r̈(k) | `slew_rate` |
| ρ | `rho` or `envelope_radius` |
| λ | `lyapunov_exponent` |
| τ | `dsa_threshold` |
| W | `window_width` |
| K | `persistence_count` |

Do not use abbreviated or colloquial names that diverge from the paper.
Reviewers will check the code against the paper.

### IV.4 · Floating-Point Discipline

- Use `f32` throughout the hot path — `f64` is prohibited in `observe()`
- All FP operations must be deterministic under ordered replay (Theorem 9)
- No `f32::NAN` or `f32::INFINITY` permitted in any output field
- Sub-normal prevention: clamp `‖r‖` to `f32::MIN_POSITIVE` below 1e-38

### IV.5 · Feature Flags

| Feature flag | What it enables | Must be gated |
|---|---|---|
| `std` | `std::io`, `ZeroMQ`, `serde_json` output | Yes |
| `alloc` | `Vec`-backed structures | Yes |
| `serde` | `Serialize`/`Deserialize` derives | Yes |
| `paper_lock` | Compile-time assertions on paper constants | Recommended in CI |

No-feature default must compile for `thumbv7em-none-eabihf`.

### IV.6 · Commit Message Convention

```
<module>: <one-line summary>

[optional body with why, not what]
[ref: paper §X if changing a construct tied to a theorem]
```

Examples:
- `grammar: enforce 2-confirmation hysteresis on all transitions (Lemma 5)`
- `uncertainty: add IQ imbalance as Type B contributor`
- `bench: verify stack usage < 512B on cortex-m4f`

---

## Part V — Figure and Paper Output Standards
*Perspective: [IEEE] + [PAPER]*

### V.1 · Figure Requirements

Every figure must:
- Have axis labels with units
- Have a descriptive title matching its paper section reference
- Show measured data only — no model extrapolations without explicit labelling
- Show confidence intervals or error bars where applicable
- Use consistent colour palette across all 50 figures (defined in `figures_all.py`)

### V.2 · Figure Data Provenance

Every numeric value in every figure must:
- Trace to a named function in `generate_figures_all.rs`
- Be reproducible by running `cargo run --example generate_figures_all --features std,serde`
- Match a value in `figure_data_all.json` by key name

No hand-crafted numbers. No manually typed values in figure scripts.
If a value cannot be computed by the engine, it must be labelled as
"model estimate" and accompanied by a `†` footnote.

### V.3 · Combined PDF Contents

`dsfb-rf-all-figures.pdf` must contain figures in order fig_01 through fig_51.
Figure numbering in the file must match the paper's figure numbering.
The script `figures_all.py` handles this — do not manually reorder.

### V.4 · Paper LaTeX — Edit Protocol

Before editing `paper/dsfb_rf_v2.tex`:
1. Read the surrounding context (±50 lines)
2. Identify the theorem, lemma, or table being modified
3. Verify the corresponding code and figure are already updated
4. Do not change any equation without verifying dimensional consistency
5. Compile the PDF after every edit to catch broken references

After editing: verify `pdflatex` produces no undefined references.

---

## Part VI — What "Done" Means
*Perspective: [SBIR] + [IEEE]*

A feature or fix is **done** when all of the following are true:

- [ ] `cargo check --examples --features std` → zero warnings
- [ ] `cargo check --target thumbv7em-none-eabihf --no-default-features` → passes
- [ ] `cargo check --target riscv32imac-unknown-none-elf --no-default-features` → passes
- [ ] `cargo test --features std` → all tests pass
- [ ] Paper claim (if any) is updated to match
- [ ] Figure (if any) is regenerated and `figure_data_all.json` is current
- [ ] SBIR deliverable docs (if affected) are updated
- [ ] Output ZIP exists at `dsfb-rf-output/dsfb-rf-<timestamp>/`
- [ ] `paper/` folder was not written to

---

## Quick Reference Card

```
THE THREE RULES THAT MATTER MOST:

1. paper/ is READ-ONLY. Never write there.
2. Output goes to dsfb-rf-output/dsfb-rf-<timestamp>/.
3. cargo run --example generate_figures_all --features std,serde
   does EVERYTHING in one command.

THE FOUR CLAIMS THAT MUST NOT BE EXCEEDED:
  RadioML precision: 73.6%   ORACLE precision: 71.2%
  RadioML recall:   95.1%   ORACLE recall:   93.4%

THE TWO GUARANTEES THAT MUST NEVER BREAK:
  #![forbid(unsafe_code)]   — always
  observe() has no write path — always

THE ONE THING THAT DEFINES THE ENGINE:
  σ(k) = (‖r(k)‖, ṙ(k), r̈(k))  — the sign tuple
```
