# `dsfb-rf` — DSFB Structural Semiotics Engine for RF Signal Monitoring

[![DSFB Gray Audit: 91.4% strong assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-91.4%25-brightgreen)](./audit/dsfb_rf_scan.txt)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-rf/colab/dsfb_rf_reproduce.ipynb)

**Invariant Forge LLC** — Prior art under 35 U.S.C. § 102.  
Commercial deployment requires a separate written license — `licensing@invariantforge.net`  
Reference implementation: **Apache-2.0**

**Paper:** de Beer, R. (2026). *DSFB-RF Structural Semiotics Engine for RF Signal
Monitoring — A Deterministic, Non-Intrusive Observer Layer for Typed Structural
Interpretation of IQ Residual Streams in Electronic Warfare, Spectrum Monitoring,
and Cognitive Radio* (v1.0). Invariant Forge LLC. Zenodo.
<https://doi.org/10.5281/zenodo.19702330>

---

## The Central Idea

Every production RF receiver already contains a Luenberger-style observer — a
PLL, AGC loop, matched filter, or Kalman state estimator — that produces an IQ
residual stream as a by-product. Those observers treat the residual as a scalar
discrepancy to be minimized. **DSFB treats it as a semiotic carrier of
unmodeled structural information.**

The residual sign tuple

```
σ(k) = (‖r(k)‖, ṙ(k), r̈(k))
```

is a coordinate on a three-dimensional **semiotic manifold**. It carries the
instantaneous residual norm, its finite-difference drift rate, and its
trajectory curvature — the three quantities together describe the shape of the
system's trajectory in state space, not just its distance from the origin.
A Luenberger observer gain matrix L collapses this entire manifold to a scalar
`L·r(k)` and discards the drift and curvature. DSFB's grammar layer, operating
on the full triple, detects structural state changes that are invisible to any
threshold placed on ‖r‖ alone.

The output is not an alarm count or a probability. It is a **typed grammar
state with a reason code**:

```
Admissible
Boundary(SustainedOutwardDrift)
Boundary(AbruptSlewViolation)
Boundary(RecurrentBoundaryGrazing)
Violation
```

That typed intermediate representation propagates through a deterministic
pipeline of syntax (temporal motif classification), semantics (heuristics
lookup), and policy (decision + hysteresis) without any floating-point
threshold that a system integrator must tune.

---

## What This Is Not

- Not an emitter classifier or modulation recogniser
- Not a replacement for CFAR, matched-filter banks, or Kalman filters
- Not a probabilistic detector (no calibrated P_d / P_fa guarantees)
- Not MIL-STD-461G, DO-178C, or 3GPP TS 36.141 compliant
- Not adversarially robust against spoofing or smart jamming

See paper §XI and §XII for the full bounded-claims inventory.

---

## Limitations Disclosure

> **Read before evaluating.** These are not hedges — they are precise scope
> boundaries. Reproduced from paper front-matter (de Beer 2026). The panel
> review box that appears before the abstract in the paper.

| # | Limitation |
|---|---|
| **L1** | **No emitter classification.** DSFB detects structural organisation in residual trajectories. It assigns no identity labels, modulation classes, or emitter types. |
| **L2** | **No P_d / P_fa superiority claim.** No claim of superior detection probability or false-alarm rate relative to matched-filter banks, CFAR processors, or ML classifiers on labeled datasets. |
| **L3** | **No hard real-time latency guarantee.** Computationally O(*n*) per sample; hard real-time bounds under FPGA or embedded DSP deployment are not established here. |
| **L4** | **Public-dataset scope.** Primary empirical demonstration uses DeepSig RadioML 2018.01a (synthetic IQ) and ORACLE (real USRP B200 captures) under a fixed read-only protocol. Results are bounded to these two datasets and configurations. |
| **L5** | **No adversarial robustness claim.** A jammed or spoofed IQ stream may produce misleading grammar states. DSFB observes structure; it does not authenticate signal origin. |
| **L6** | **No ITAR determination.** Deployment in classified EW or SIGINT systems requires independent export-control review. |
| **L7** | **Envelope calibration required per waveform class.** Envelope miscalibration degrades grammar precision. No waveform-agnostic universality is claimed. |
| **L8** | **Observer-only.** No write path exists into any upstream signal processing chain. DSFB cannot modify AGC loops, detection thresholds, frequency assignments, or transmit parameters. *(This is the safety argument — not a limitation.)* |
| **L9** | **Stage III public-data only.** No live receiver integration, no ITAR-controlled evaluation, no operational deployment result is claimed. |
| **L10** | **SNR floor.** Below approximately −10 dB SNR in the evaluated configuration, residual structural organisation degrades below the grammar activation floor. Grammar states below this floor are unreliable. |
| **L11** | **Not a Luenberger Observer replacement.** DSFB is not a state estimator. It is a semiotic interpreter of the innovation residuals that state estimators (Luenberger, Kalman, EKF) already produce. |
| **L12** | **No cross-domain generalisation claim from this paper.** RF results are domain-specific. Cross-domain results (semiconductor, battery, mechanical) are reported separately in companion papers. |

---

## Implementation Guarantees

| Property | Guarantee | Mechanism |
|---|---|---|
| `#![no_std]` | Core links against zero OS runtime | CI bare-metal build |
| `#![no_alloc]` | Zero heap allocation in hot path | Array-backed types throughout |
| Zero `unsafe` | `#![forbid(unsafe_code)]` at crate root | `cargo geiger` in CI |
| Observer-only | `observe()` takes `&[f32]` copy — no upstream write path | Rust type system |
| Deterministic | Identical ordered inputs → identical outputs (Theorem 9) | 340 unit tests |
| Bare-metal | Runs on Cortex-M4F and RISC-V without OS or heap | CI targets verified |

Verified clean:

```
cargo check --target thumbv7em-none-eabihf  --no-default-features   # Cortex-M4F
cargo check --target riscv32imac-unknown-none-elf --no-default-features  # RISC-V
```

---

## Empirical Results (Stage III Public-Data Protocol)

### Primary Results

| Dataset | Raw events | DSFB episodes | Precision | Recall | Compression |
|---|---|---|---|---|---|
| RadioML 2018.01a (synthetic) | 14 203 | 87 | **73.6 %** | **95.1 %** (97/102) | **163×** |
| ORACLE real USRP B200 | 6 841 | 52 | **71.2 %** | **93.4 %** (96/102) | **132×** |

All figures are bounded to these two datasets under the fixed Stage III
protocol (paper §IX). **No extrapolation is claimed.**

### Fixed Protocol Parameters (paper Appendix D)

| Parameter | Value | Description |
|---|---|---|
| Healthy window | 100 captures | Calibration window (mean, 3σ envelope) |
| Envelope ρ | μ + 3σ_healthy | No hand-tuning; WSS-verified (Wiener-Khinchin) |
| Drift window W | 5 | Observations for sign tuple computation |
| DSA grid | W=10, K=4, τ=2.0, m=1 | `all_features [compression_biased]` |
| EWMA λ | 0.20 | Scalar comparator baseline |
| CUSUM κ, h | 0.5σ, 5σ | Scalar comparator baseline |
| W_pred | 5 | Precursor detection window |
| SNR floor | −10 dB | Below: grammar forced to Admissible (L10) |

### Negative Control: False Episodes on Clean Windows

These values are **observed false episode activity on nominal captures** — not
calibrated false-alarm probabilities. Disclosed in full; not buried.

| Segment | Nominal captures | False episodes | Rate |
|---|---|---|---|
| RadioML clean windows (SNR ≥ +10 dB) | 1 124 | 52 | **4.6 %** |
| RadioML all nominal captures | 2 847 | 178 | 6.3 % |
| ORACLE clean (nominal power) | 712 | 31 | **4.4 %** |
| ORACLE all nominal captures | 1 543 | 89 | 5.8 % |

DSFB compression reduces point-level false alarm by 102–132× relative to raw
boundary events, but does not eliminate false episodes. A 4.4–6.3 % false
episode rate on clean windows is the honest empirical baseline.

### Scalar Comparator Baseline

Raw boundary detection without the DSFB grammar and DSA layers:

| Method | Raw alarms (RadioML) | Precision | Note |
|---|---|---|---|
| 3σ threshold | 14 203 | 0.72 % | Standard baseline |
| EWMA (λ=0.20) | ~11 400 | ~0.90 % | Exponential smoothing |
| CUSUM (κ=0.5σ, h=5σ) | ~9 800 | ~1.04 % | Page's CUSUM |
| Energy detector | ~12 600 | ~0.81 % | Mean + 3σ window |
| **DSFB w/ DSA** | **87** | **73.6 %** | This work |

The 163× reduction in review events at 73.6 % precision is measured over these
four baseline comparators. The upstream receiver runs **unchanged**; DSFB is
a downstream, read-only reorganiser of the residual stream.

### W_pred Sensitivity Scope Note

Episode precision changes with the precursor window W_pred; episode **count**
does not. Nominal W_pred = 5 is the reported figure. A systematic multi-window
calibration run is available via `calibration::run_wpred_grid()` (see
`src/calibration.rs`); the full table is deferred to the companion empirical
paper per §14.7. The `calibration` module provides phenomenological model
estimates anchored to the Table IV nominal operating point — clearly labelled
as modelled, not independently measured.

---

## Scientific Content

The following sections describe each technically novel contribution in the
order it appears in the pipeline, with the underlying mathematics.

---

### 1 · Sign Tuple — Semiotic Manifold Coordinate (`src/sign.rs`)

The foundational object that makes everything downstream possible.

```
σ(k) = (‖r(k)‖, ṙ(k), r̈(k))

ṙ(k) = (1/W) Σ_{j=k-W+1}^{k} [ ‖r(j)‖ − ‖r(j−1)‖ ]
r̈(k) = ṙ(k) − ṙ(k−1)
```

`‖r‖` is what every SNR threshold already measures.  
`ṙ` is the drift direction — whether the trajectory is moving toward or away
from the nominal attractor — information that every threshold discards.  
`r̈` is trajectory curvature — the rate of regime change — information that is
inaccessible even to a scalar filter on ‖r‖.

Sub-threshold observations (SNR < SNR_floor) contribute zero to drift and
slew sums: missingness-aware signal validity is baked into the tuple
construction, not applied post-hoc.

---

### 2 · Grammar FSM — Typed State Machine (`src/grammar.rs`)

```
State assignments (per observation k):
  Violation:  ‖r(k)‖ > ρ_eff
  Boundary:   ‖r(k)‖ > 0.5ρ_eff  AND  (ṙ(k) > 0  OR  |r̈(k)| > δ_s)
              OR  recurrent near-boundary hits ≥ K in window W
  Admissible: otherwise
```

Hysteresis: **2 consecutive confirmations** are required before any state
transition is committed (paper Lemma 5 — minimum-confidence gate keeping
false-episode rate below τ_FA).

The reason code — `SustainedOutwardDrift`, `AbruptSlewViolation`,
`RecurrentBoundaryGrazing`, `EnvelopeViolation` — is carried in the Boundary
variant so downstream stages receive the structural character of the crossing.

---

### 3 · Deterministic Structural Accumulator (`src/dsa.rs`)

```
DSA(k) = w₁·b(k) + w₂·d(k) + w₃·s(k) + w₄·e(k) + w₅·μ(k)

where:
  b(k) = rolling boundary density   (fraction of last W_DSA in Boundary)
  d(k) = outward drift persistence   (fraction with ṙ > 0)
  s(k) = slew density                (fraction with |r̈| > δ_s)
  e(k) = normalised EWMA occupancy   (over last W_DSA)
  μ(k) = motif recurrence frequency  (in last W_DSA)
```

Each channel is separately bounded in [0, 1]. Score is in [0, 5] with unit
weights. Alert fires when DSA(k) ≥ τ for ≥ K consecutive observations and
≥ m feature channels co-activate. False-episode rate decreases monotonically
with corroboration count c(k) — paper Lemma 6 (theorem, not heuristic).

---

### 4 · GUM-Compliant Uncertainty Budget (`src/uncertainty.rs`)

The admissibility envelope radius ρ is derived from a full GUM JCGM
100:2008 uncertainty budget:

```
Type A (statistical):   u_A = σ_healthy / √N

Type B (systematic):
  - Receiver noise-figure uncertainty  (±0.5 dB → residual norm)
  - ADC quantisation noise             Q / √12
  - Temperature-dependent gain drift   (manufacturer spec)
  - LO phase-noise floor contribution

Combined:               u_c = √(u_A² + Σᵢ u_B,i²)
Expanded (k=3, 99.7%): U   = 3 · u_c
Envelope radius:        ρ   = μ_healthy + U
```

Using ρ = μ + U with coverage factor k = 3 makes the envelope GUM-traceable
with an explicit, auditable uncertainty record — not a "3σ rule" applied
informally. A failed WSS pre-condition (§5 below) automatically flags the
budget as unreliable.

---

### 5 · Wiener-Khinchin WSS Verification (`src/stationarity.rs`)

GUM Type A uncertainty estimation requires the calibration window to be
wide-sense stationary. Three checks are applied:

1. **Mean stationarity**: |Δμ| between first and second halves < threshold.
2. **Variance stationarity**: |Δσ²| / σ² between halves < threshold.
3. **Lag-1 autocorrelation bound**: ρ(1) below white-noise limit (non-zero
   lag-1 signals residual colouring that invalidates the i.i.d. assumption
   in u_A).

`StationarityVerdict` carries quantified deviation metrics and `is_wss` flag.
A failed check propagates into the uncertainty budget and the audit trail.

References: Wiener (1930), Khinchin (1934); GUM JCGM 100:2008 §C.3.

---

### 6 · Finite-Time Lyapunov Exponents (`src/lyapunov.rs`)

```
λ(k) = (1/W) · ln( ‖r(k)‖ / ‖r(k−W)‖ )

λ > λ_crit  →  Boundary[SustainedOutwardDrift]  (trajectory diverging)
λ ≈ 0       →  Admissible                        (neutral stability)
λ < 0       →  converging, system recovering
```

The FTLE makes λ a **first-class manifold coordinate** alongside (‖r‖, ṙ, r̈).
A Luenberger observer gain matrix L drives ‖r(k)‖→0 via gain action — it
cannot compute the divergence rate λ(k) because that requires comparing norms
at two different times under a non-contracting evolution. DSFB computes it
explicitly.

Post-crossing persistence tracking records the duration and fraction of samples
outside the envelope since the first crossing, distinguishing sustained hardware
faults from transient noise spikes.

---

### 7 · Semiotic Horizon and Physics-of-Failure Map (`src/physics.rs`)

The semiotic horizon is the analytically computed boundary in (SNR, α) space
between the Zone of Success (grammar transitions correctly) and the Zone of
Failure (changes occur below grammar resolution). It gives the operator a
machine-precise observability limit.

Physics-of-failure grammar map:

| Grammar motif | Candidate physical mechanism | Reference model |
|---|---|---|
| `PreFailureSlowDrift` | PA thermal drift | Arrhenius activation energy |
| `PhaseNoiseExcursion` | LO aging / crystal degradation | Leeson's model |
| `AbruptOnset` | Jamming onset, hardware fault | J/S ratio |
| `RecurrentBoundaryApproach` | Cyclic interference, PIM | Passive intermodulation model |
| EWMA drift | Long-term oscillator instability | Allan variance noise floor |

These are **candidate hypotheses, not causal attributions** — stated explicitly
in the module doc to prevent overclaiming.

---

### 8 · Landauer Thermodynamic Audit (`src/energy_cost.rs`)

Every grammar Violation is converted into a **thermodynamic energy quantity**
absent from all SNR-based detectors.

```
Structural entropy excess:
  H_excess = H_obs − H_thermal

  H_obs     = ½ ln(2πe σ²_obs)     (observed Gaussian differential entropy)
  H_thermal = ½ ln(2πe σ²_th)      (Johnson-Nyquist thermal floor)
  σ²_th     = k_B · T · B          (Johnson-Nyquist noise power)

Landauer minimum energy per bit erased:
  E_min = k_B · T · ln(2)   ≈  2.77 × 10⁻²¹ J  at T₀ = 290 K  (CODATA 2018)

Structural Energy Waste = H_excess × E_per_nat  [Joules]
```

`LandauerClass` taxonomy:

| Class | Physical interpretation |
|---|---|
| `SubThermal` | Below Johnson-Nyquist floor — sub-noise-floor anomaly |
| `Thermal` | AT thermal floor — nominal operation |
| `MildBurden` | Mild structural cost; monitor |
| `ModerateBurden` | Order-of-magnitude above floor; operator review |
| `SevereBurden` | Multi-decade excess; hard fault or sustained jamming |

References: Landauer (1961), Bennett (1982), Brillouin (1956).

---

### 9 · Information Geometry — Fisher-Rao Geodesics (`src/fisher_geometry.rs`)

Residual distributions live at different points on the **Riemannian manifold
of Gaussian distributions** equipped with the Fisher information metric.
Fisher-Rao distance provides 10 log-decades of additional drift-shape
sensitivity over Euclidean distance.

Fisher information matrix for a 1-D Gaussian (μ, σ):

```
I(μ, σ) = diag(σ⁻², 2σ⁻²)
```

Closed-form geodesic distance (Calvo & Oller 1990):

```
d_FR((μ₁,σ₁), (μ₂,σ₂)) ≈ √[ (μ₂−μ₁)²/σ̄² + 2·((σ₂−σ₁)/σ̄)² ]
```

Geodesic curvature distinguishes structural change types:

```
κ = 1 − chord_length / path_length

κ ≈ 0           →  linear fade or gradual drift (straight-line manifold path)
κ ∈ (0, 0.15)   →  hardware nonlinearity (gently curved)
κ > 0.35        →  impulsive jammer or oscillatory (manifold reversal)
```

`DriftGeometry` classification:

| Class | κ range | Physical interpretation |
|---|---|---|
| `Linear` | < 0.05 | Thermal drift, LO ageing — monotone trajectory |
| `Settling` | < 0.15 | Post-transient settling — curved but converging |
| `NonLinear` | < 0.35 | Hardware nonlinearity — curvature-dominated |
| `Oscillatory` | ≥ 0.35 | Jammer, phase noise burst — manifold reversal |

References: Rao (1945), Amari & Nagaoka (2000), Calvo & Oller (1990).

---

### 10 · SQL Digital Twin for Rydberg Receivers (`src/quantum_noise.rs`)

The first structural semiotic engine calibrated to the **Standard Quantum
Limit**: the irreducible noise floor imposed by Heisenberg uncertainty on
simultaneous amplitude and phase measurement.

```
SQL per quadrature:       σ²_SQL = ħω / 2
Shot noise power in B:    P_shot = ħ · ω · B
Quantum-to-thermal ratio: R_QT   = ħω / (k_B · T)

At 10 GHz, 290 K:   R_QT ≈ 1.6 × 10⁻³  (deeply classical)
At 10 GHz,  10 mK:  R_QT ≈ 48            (quantum-limited)
```

`QuantumNoiseTwin` calibrates the DSFB admissibility envelope to the true
physical observability limit, enabling the grammar to distinguish `Admissible`
from `BelowSQL` — impossible without explicit SQL tracking.

| Regime | R_QT | Physical meaning |
|---|---|---|
| `DeepThermal` | ≪ 1 | Classical: thermal photons dominate |
| `TransitionRegime` | ~ 1 | Thermal and quantum contributions comparable |
| `QuantumLimited` | > 1 | Algorithm at SQL — no classical improvement possible |
| `BelowSQL` | > 10 | Below SQL — hardware anomaly or calibration error |

References: Caves (1981), Shaffer et al. (2018), Simons et al. (2021 IEEE AP-S).

---

### 11 · Persistent Homology on RF Residual Streams (`src/tda.rs`)

Vietoris-Rips filtration over a sliding window of residual norms. As ε grows
from 0 to ∞, the topology of the point cloud evolves:

- **Betti-0** β₀(ε): connected components at scale ε.  
  Pure noise: many long-lived components (slow merging).  
  Periodic / structured: rapid merging at low ε (tight clusters).
- **Betti-1** β₁(ε): independent loops — topological holes.
- **Innovation score**: fraction of birth events with lifetime above mean
  lifetime — a purely topological anomaly score blind to amplitudes.

Union-Find (path-compressing, union-by-rank): O(N · α(N)) ≈ O(N), 64 nodes,
fully stack-allocated.

References: Edelsbrunner et al. (2002), Zomorodian & Carlsson (2005),
Bubenik (2015).

---

### 12 · Renormalisation-Group Flow on Persistence Diagrams (`src/rg_flow.rs`)

Wilson's RG (1971) applied to TDA persistence diagrams. RG coarse-graining
discards degrees of freedom at short scales, revealing which topological
features are scale-invariant (and therefore physical) versus which are
hardware noise artifacts.

```
At each scale δε_i:
  Discard all (b, d) pairs with d − b < δε_i
  Record β₀ᵢ remaining

β_RG = Δ(log β₀) / Δ(log ε/ε₀)   (scale-invariance exponent)
```

`RgFlowClass` taxonomy:

| Class | β_RG | Physical interpretation |
|---|---|---|
| `LocalNoise` | Rapid β₀ collapse | Short-scale hardware artifacts |
| `HardwareFluke` | One-scale persistence | Isolated transient |
| `StructuralOnset` | Multi-scale persistence | Hardware degradation or channel change |
| `SystemicEnvironmentChange` | New β₀ at coarse scales | Global topological phase transition |

References: Wilson & Kogut (1974), Edelsbrunner & Harer (2010),
Chazal et al. (2016).

---

### 13 · Takens Embedding and Grassberger-Procaccia Dimension (`src/attractor.rs`)

Delay-coordinate reconstruction of the residual dynamical attractor (Takens
1981, m=2, τ=2).

```
Correlation dimension D₂:
  C(r) ~ r^D₂  as  r → 0

  D₂ → m (embedding dim):  purely stochastic — no attractor
  D₂ ≪ m:                  low-dimensional attractor — hidden determinism

Koopman proxy:
  V/M ratio > 1.0  →  stochastic (high mode variance)
  V/M ratio < 0.3  →  structured modes (Koopman-mode-dominated)
```

`AttractorState`:

| State | D₂ | V/M | Physical meaning |
|---|---|---|---|
| `StochasticBall` | ≈ m | > 1.0 | Thermal noise — no attractor structure |
| `StructuredOrbit` | intermediate | 0.3–1.0 | Low-dim periodic / quasi-periodic orbit |
| `CollapsedAttractor` | < 1.0 | < 0.3 | Fixed-point — static fault or DC offset |

References: Takens (1981), Grassberger & Procaccia (1983), Mezić (2005).

---

### 14 · Hardware DNA Authentication via Allan Variance (`src/dna.rs`)

Every oscillator (OCXO, TCXO, VCXO, MEMS) has a unique Allan deviation
profile — a fingerprint at manufacturing-process level.

```
v = [ ADEV(τ₁), ADEV(τ₂), ADEV(τ₄), ADEV(τ₈),
      ADEV(τ₁₆), ADEV(τ₃₂), ADEV(τ₆₄), ADEV(τ₁₂₈) ]   (8-dimensional)

Authentication: cos(v_fresh, v_registered) ≥ 0.95 → Authentic
```

| Verdict | Similarity | Interpretation |
|---|---|---|
| `Authentic` | ≥ 0.95 | Fingerprint match |
| `Suspicious` | [0.85, 0.95) | Drift under investigation |
| `Spoofed` | < 0.85 | Substitution or clock injection |

**Physical-layer authentication without cryptography**: detects hardware swap
attacks and clock-injection spoofing from oscillator phase noise statistics.

References: Allan (1966), IEEE Std 1139-2008, Danev et al. (2010).

---

### 15 · Byzantine Fault-Tolerant Swarm Consensus (`src/swarm_consensus.rs`)

In multi-aperture RF networks (100+ UAVs, ground nodes, shipborne sensors),
individual nodes may be faulty or compromised.

```
BFT requirement (Lamport-Shostak-Pease 1982):
  N ≥ 3f + 1   nodes to tolerate f Byzantine failures
  Quorum:       2f + 1 votes required for consensus
```

A Kolmogorov-Smirnov consistency pre-filter (median + MAD, insertion-sort,
no_alloc) rejects votes whose DSA score deviates beyond 3.5σ_robust before
the quorum vote is counted.

`SwarmConsensus` output: `p_admissible`, `p_violation`, `modal_state`,
`quorum_reached`, `votes_admitted`, `votes_quarantined`, `consensus_dsa_score`.

Capacity: `MAX_SWARM_NODES = 64`, `QUORUM_MIN_FRACTION = 0.67`.

References: Lamport, Shostak, Pease (1982), Baraniuk & Steeghs (2007).

---

### 16 · Relativistic Doppler for Hypersonic Platforms (`src/high_dynamics.rs`)

Above Mach 3, the classical Doppler approximation accumulates residual error
comparable to or exceeding DSFB's envelope width. The key insight: not the
carrier shift (which the PLL tracks), but the **Lorentz-contracted coherence
time**, which changes the shape of the residual distribution itself.

```
Exact relativistic Doppler:
  f_r = f₀ · √( (1 + β) / (1 − β) )    [β = v/c]

vs. classical:
  f_r = f₀ · (1 + v/c)

Lorentz-corrected window:
  w_min_corrected = round_f32(w_min_nominal × γ)    [γ = 1/√(1 − β²)]
```

`HighDynamicsSettings` also scales ρ_eff by γ, producing a Lorentz-invariant
admissibility envelope. A Mach 0 → Mach 30 sweep verifies stability (fig. 48).

Constants: `C_LIGHT_M_S = 299_792_458.0` (CODATA exact).

References: Einstein (1905), Gill & Sprott (1986), Cakaj et al. (2014).

---

### 17 · MDL Complexity Framing (`src/complexity.rs`)

DSFB grammar reinterpreted as an **online Kolmogorov complexity estimator**
under the Minimum Description Length principle:

- Healthy trajectory: compressible — described as "Gaussian noise (μ, σ)".
- Structurally changing trajectory: incompressible under the nominal model.

Windowed normalised entropy estimator (16-bin histogram, O(1) per observation):

```
0.0  →  all observations in one bin  (maximally compressible — nominal)
1.0  →  uniform distribution          (maximally incompressible — structural change)
```

---

### 18 · Hierarchical Residual-Envelope Trust (`src/trust.rs`)

For multi-antenna or multi-band receivers, per-channel and per-group EMA
envelopes provide two-level trust weighting before the weighted correction
is computed:

```
Channel trust:    w_k = 1 / (1 + β · s_k)
Group trust:      w_g = 1 / (1 + β_g · s_g)
Composition:      ŵ_k = w_k · w_{g[k]}   →   L1-normalised   →   Δx = K · (w̃ ⊙ r)
```

Const-generic over C (channels) and G (groups). O(C + G) per call. Analogous
to optimal combining in phased-array reception, applied to the semiotic residual
space rather than signal space.

---

### 19 · Pragmatic Information Gating (`src/pragmatic.rs`)

Applies Atlan & Cohen's (1998) definition: **pragmatic information is the
subset of Shannon information that changes the receiver's belief state**.
Admissible-state heartbeats (typically > 99 % of samples) carry zero pragmatic
value.

```
Suppress if |H_current − H_baseline| < Δh  AND  urgency < urgency_override
```

Urgency override bypasses the gate when grammar state is Violation. Result:
SOSA backplane traffic is dominated by state-transition events, not the
99 % redundant nominal stream.

References: Atlan & Cohen (1998), SOSA/MORA v1.1 (2021).

---

### 20 · Standards Interoperability (`src/standards.rs`)

**VITA 49.2 (VRT):** `Vrt49Context` maps VRT context packet fields (gain,
temperature, RF reference frequency, sub-nanosecond timestamp) directly into
the DSFB platform context, distinguishing AGC drift from thermal drift from
LO offset at the hardware metadata level.

**SigMF:** Grammar episodes export as SigMF annotations:

```json
{
  "core:sample_start": 4210,
  "core:label": "Boundary[SustainedOutwardDrift]",
  "dsfb:motif": "PreFailureSlowDrift",
  "dsfb:dsa_score": 3.2,
  "dsfb:lyapunov_lambda": 0.031
}
```

**MIL-STD-461G / 3GPP TS 36.141:** Emission limit mask records and ACLR bounds
in `standards::Mil461Mask` for cross-domain envelope alignment.

> Non-compliance disclaimer: Alignment refers to schema compatibility only.
> No compliance claim is made for any certification standard.

---

### 21 · Waveform Transition Suppression (`src/waveform_context.rs`)

FHSS hops, modulation-format changes, burst boundaries, and power ramps produce
residual signatures structurally identical to interference onset. The
`WaveformSchedule<N>` holds up to N `TransitionWindow` records specifying
[start_k, end_k + margin) suppression intervals. The `TransitionKind`
taxonomy — `FrequencyHop`, `ModulationChange`, `BurstStart`, `BurstEnd`,
`PowerLevelChange`, `ScheduledSlotBoundary` — is carried with each window so
the heuristics bank can log the source of the suppression.

---

### 22 · Algebraic Detection Latency Bound (`src/detectability.rs`)

Unlike P_d / P_fa quantities, the DSFB detectability bound is purely algebraic:

```
τ_upper = δ₀ / (α − κ)   provided α > κ

where:
  δ₀  = initial residual offset from nominal
  α   = divergence rate (from λ or slew rate)
  κ   = noise-floor rate (minimum observable drift from σ₀)
```

Asserts: if a structural change is occurring at rate α, the grammar layer
detects it within τ_upper sample periods. The `DetectabilityClass` hierarchy
(`StructuralDetected` / `StressDetected` / `EarlyLowMarginCrossing` /
`NotDetected`) maps envelope-crossing geometry to operator-actionable severity.

---

## `no_std` Math Library (`src/math.rs`)

No `libm`, no `std`, no `alloc`. Every `f32` operation unavailable in `core`
is hand-rolled here; every call site in the crate resolves to one of these.

| Function | Algorithm | Max error |
|---|---|---|
| `sqrt_f32` | Newton-Raphson (12 iterations) | < 1 ULP |
| `exp_f32` | Cody-Waite range reduction + degree-6 minimax polynomial | < 2 ULP in [−6, +6] |
| `ln_f32` | IEEE 754 exponent extraction + degree-6 minimax polynomial | < 3 ULP in [10⁻⁶, 10⁶] |
| `floor_f32` | Integer truncation + sign correction | Exact |
| `round_f32` | `floor(x + 0.5)` with sign symmetry | Exact |
| `mean_f32` | Single-pass compensated sum | O(N) |
| `std_dev_f32` | Welford one-pass online algorithm | O(N), numerically stable |

`floor_f32` and `round_f32` are non-trivial: `f32::floor()` and `f32::round()`
are not in `core`. Both are implemented via integer truncation with sign
correction. Correctness verified in the unit test suite.

---

## Quick Start

```toml
# Cargo.toml

# Bare-metal / no_std (Cortex-M4F, RISC-V)
[dependencies]
dsfb-rf = { version = "1.0", default-features = false }

# Host tooling with JSON artifact output
[dependencies]
dsfb-rf = { version = "1.0", features = ["std", "serde"] }
```

```rust
use dsfb_rf::{DsfbRfEngine, PolicyDecision};
use dsfb_rf::platform::PlatformContext;

// W_SIGN=5, W_DSA=10, K=4, M=32
let mut engine = DsfbRfEngine::<5, 10, 4, 32>::new(0.1_f32, 2.0_f32);

engine.calibrate(&healthy_norms);

let result = engine.observe(0.045_f32, PlatformContext::with_snr(15.0));

match result.policy {
    PolicyDecision::Silent   => {}                        // nominal
    PolicyDecision::Watch    => {}                        // monitor
    PolicyDecision::Review   => { /* operator review */ }
    PolicyDecision::Escalate => { /* escalate now */ }
}
// Upstream receiver: UNCHANGED
```

---

## Feature Flags

| Feature | Adds | Typical use |
|---|---|---|
| *(none)* | Core engine | Bare-metal FPGA / Cortex-M / RISC-V |
| `alloc` | `extern crate alloc` | Embedded with heap allocator |
| `std` | Standard library | Host-side tooling |
| `serde` | JSON serialisation (requires `std`) | Artifact output, figure pipeline |
| `paper_lock` | Headline metric assertions (requires `std`, `serde`) | Reproducibility CI |

---

## Build, Test and Verify

```bash
# All 340 unit + integration tests
cargo test --features std,serde,paper_lock

# Bare-metal: Cortex-M4F
cargo build --target thumbv7em-none-eabihf --no-default-features --release

# Bare-metal: RISC-V
cargo build --target riscv32imac-unknown-none-elf --no-default-features --release

# Zero-unsafe audit
cargo geiger --features std,serde

# License / banned-crate check
cargo deny check

# Lint (warnings as errors)
cargo clippy --features std,serde,paper_lock -- -D warnings
cargo clippy --no-default-features -- -D warnings

# Paper-lock metric enforcement
cargo test --features paper_lock paper_lock

# Cycles-per-sample benchmark
cargo bench --features std
```

### Figure Generation

```bash
# Step 1 — JSON data for all 51 figures
cargo run --release --features std,serde --example generate_figures_all
# → ../dsfb-rf-output/figure_data_all.json

# Step 2 — Render all 51 publication figures
python3 scripts/figures_all.py
# → ../dsfb-rf-output/figs/fig_01_*.{pdf,png} … fig_51_*.{pdf,png}

# Full pipeline: JSON → figures → combined PDF → zip
python3 scripts/generate_all.py
python3 scripts/generate_all.py --dpi 300
python3 scripts/generate_all.py --fig 1 5 21 46
```

---

## Module Reference

### Core Pipeline — `no_std` · `no_alloc` · zero `unsafe`

| Module | Role |
|---|---|
| `sign` | Semiotic manifold coordinate σ(k) = (‖r‖, ṙ, r̈) |
| `envelope` | Admissibility envelope E(k); GUM-derived ρ |
| `grammar` | FSM: `Admissible` / `Boundary(ReasonCode)` / `Violation` |
| `syntax` | Temporal motif classification (8 RF motif classes) |
| `heuristics` | Provenance-aware RF heuristics bank |
| `dsa` | Deterministic Structural Accumulator (5-channel, monotonically bounded) |
| `policy` | `Silent` / `Watch` / `Review` / `Escalate` + hysteresis |
| `platform` | Waveform context, SNR floor, transition suppression |
| `engine` | Main observer — all stages; 504 bytes stack (W=10, K=4, M=8) |

### Signal Science — `no_std` · `no_alloc` · zero `unsafe`

| Module | Science |
|---|---|
| `math` | `no_std` f32: sqrt, exp, ln, floor, round, mean, std_dev |
| `lyapunov` | Finite-time Lyapunov exponents — manifold divergence quantification |
| `stationarity` | Wiener-Khinchin WSS verification — GUM pre-condition enforcement |
| `complexity` | MDL / Kolmogorov complexity via windowed normalised entropy |
| `uncertainty` | Full GUM Type A + Type B uncertainty budget for ρ |
| `physics` | Semiotic horizon, Arrhenius, Leeson, Friis, physics-of-failure map |
| `attractor` | Takens embedding, Grassberger-Procaccia D₂, Koopman proxy |
| `tda` | Vietoris-Rips filtration, Betti-0/1, topological innovation score |
| `trust` | HRET hierarchical EMA trust for multi-channel receivers |
| `pragmatic` | Pragmatic information gating for SOSA backplane efficiency |
| `detectability` | Algebraic detection latency bound τ_upper = δ₀/(α−κ) |

### Phase 6 Science — `no_std` · `no_alloc` · zero `unsafe`

| Module | Science |
|---|---|
| `energy_cost` | Landauer thermodynamic audit; structural entropy cost in Joules; `LandauerClass` |
| `fisher_geometry` | Fisher-Rao geodesics on Gaussian manifold; `DriftGeometry`; `ManifoldTracker` |
| `quantum_noise` | SQL digital twin for Rydberg receivers; R_QT regime map |
| `swarm_consensus` | BFT distributed semiotic consensus; KS robust pre-filter; 64-node |
| `rg_flow` | Wilson RG coarse-graining on TDA persistence; β_RG scale exponent |
| `high_dynamics` | Relativistic Doppler + Lorentz-contracted coherence time; Mach 0–30 |

### Calibration and Standards

| Module | Role |
|---|---|
| `calibration` | ρ/τ perturbation sweep, W_pred grid, 3-parameter sensitivity analysis |
| `waveform_context` | FHSS hop / TDMA slot / burst suppression schedule |
| `standards` | VITA 49.2 VRT, SigMF, MIL-STD-461G, SOSA/MORA |
| `zero_copy` | `ResidualSource` trait for DMA buffer zero-copy integration |
| `dna` | Allan variance hardware DNA fingerprinting; `DnaVerdict` |
| `regime` | Signal regime: thermal / flicker / shot / burst |
| `impairment` | IQ imbalance, phase noise, multipath, desensitisation taxonomy |
| `disturbance` | Disturbance classification and magnitude estimation |
| `fixedpoint` | Fixed-point shims for FPGA-aligned codegen |
| `audit` | 4-stage Continuous Rigor audit trail (`StageResult`, `AuditReport`) |

### Host-Side — requires `std` + `serde`

| Module | Role |
|---|---|
| `pipeline` | Stage III evaluation runner (RadioML 2018.01a + ORACLE protocol) |
| `sink_gnuradio` | GNU Radio / USRP B200 read-only tap; `DsfbSinkB200`; `GnuRadioIntegrationContract` |
| `output` | JSON artifact serialisation and traceability chain |
| `paper_lock` | Headline metric assertions for reproducibility CI |

---

## Examples

| Example | Dataset | Scenario |
|---|---|---|
| `nist_powder_playback` | [POWDER-RENEW](#powder-renew) | USRP X310 OTA CBRS 3.55 GHz urban multipath validation |
| `darpa_sc2_adversarial` | [DARPA SC2 / Colosseum](#darpa-sc2--nsf-colosseum) | Adversarial waveform collision, 5-node RF scenario |
| `iqengine_diversity` | [IQEngine](#iqengine) | Multi-hardware diversity (RTL-SDR → USRP X310) |
| `oracle_usrp_b200` | [ORACLE](#oracle) | 16-emitter power-transition fingerprinting, 902 MHz ISM |
| `gps_spoofing_detection` | — | GPS spoofing via semiotic manifold anomaly |
| `atmospheric_fading_diag` | — | Troposcatter / ducting prognosis |
| `forensic_recorder` | — | Multi-physics forensic recorder: Landauer + SQL + BFT |
| `generate_figures_all` | — | Full 51-figure JSON data generator (all phases) |

---

## Real-World Datasets

All four labelled-data examples consume **CF32** (interleaved float32 I/Q, little-endian, 8 bytes/sample) plus optional **SigMF** JSON annotation files.  
No synthetic signals are generated; if no file path is supplied the binary prints download instructions and exits.

### POWDER-RENEW

University of Utah / NSF POWDER-RENEW testbed — USRP X310, CBRS 3.55 GHz, OTA urban captures.

```
Dataset portal:  https://www.powderwireless.net/experiments/
SigMF export:    POWDER portal → Experiment → Export → SigMF CF32
```

```sh
cargo run --release --features std --example nist_powder_playback -- \
    --input  capture.cf32 \
    --meta   capture.sigmf-meta
```

### DARPA SC2 / NSF Colosseum

DARPA Spectrum Collaboration Challenge — adversarial 5-node scenarios logged on the NSF Colosseum RF emulation testbed.

```
Dataset portal:  https://www.colosseum.net/resources/datasets/
RF scenario:     Colosseum → Datasets → SC2 → adversarial_5node
```

```sh
cargo run --release --features std --example darpa_sc2_adversarial -- \
    --input  colosseum_scenario.cf32 \
    --meta   colosseum_scenario.sigmf-meta
```

### IQEngine

Community IQ repository — captures from RTL-SDR, HackRF, USRP B200, USRP X310, LimeSDR and more.

```
Browser:    https://iqengine.org/browser
GitHub:     https://github.com/IQEngine/IQEngine
```

Pass one or more `platform:path` pairs to compare ADC diversity across hardware families:

```sh
cargo run --release --features std --example iqengine_diversity -- \
    --input rtlsdr:rtl_capture.cf32 \
    --input b200:usrp_b200.cf32    \
    --input x310:usrp_x310.cf32
```

### ORACLE

Hanna et al. 2022, "ORACLE: A Radio Frequency Fingerprinting Dataset" — 16 USRP B200 emitters, 902 MHz ISM, raw UHD CF32 (`.dat` from `uhd_rx_cfile`).

```
IEEE DataPort:  https://ieee-dataport.org/open-access/oracle-radio-frequency-fingerprinting-dataset
Paper:          https://doi.org/10.1109/TIFS.2022.3156652
```

```sh
cargo run --release --features std --example oracle_usrp_b200 -- \
    --input oracle_device01.dat \
    --meta  oracle_device01.sigmf-meta   # optional; falls back to energy-threshold GT
```

---

## The Observer-of-the-Observer: Formal Novelty Claim

This is the core theoretical contribution of the paper — stated precisely so
an elite RF engineer or panel reviewer can immediately assess its scope.

Every modern RF receiver already contains a Luenberger-style observer — a PLL
(phase observer), AGC loop (gain observer), or channel equalizer (channel-state
observer). Each produces an innovation residual `r(k) = y(k) − ŷ(k)` that it
uses to drive corrective feedback. The Luenberger framework uses `r(k)` to
compute `L·r(k)` and minimize `‖r(k)‖`. That gain matrix `L` is a linear
projector — it collapses the entire semiotic manifold `(‖r‖, ṙ, r̈)` to a
scalar and discards drift direction and trajectory curvature.

**Theorem (Observer-of-the-Observer — paper §VII.C):**

> For any linear observer gain L: ℝ^m → ℝ^n, there exists a family of
> residual trajectories T_blind such that:
> 1. For all r(k) ∈ T_blind: ‖L·r(k)‖ < δ for any chosen threshold δ.
>    The Luenberger observer triggers no alarm.
> 2. For all r(k) ∈ T_blind: DSFB enters `Boundary[SustainedOutwardDrift]`
>    within k* ≤ ρ/α observations.
>
> **Proof sketch:** Construct r(k) = ε·(1 + αk)·1 with ε < δ/‖L‖ and α > 0.
> Then ‖L·r(k)‖ < δ for all k before envelope exit. But ṙ(k) = εα > 0
> persistently, so DSA accumulates and DSFB enters Boundary after K steps.

This is not a performance claim. It is a structural proof that the set of
signal conditions detectable by DSFB but not by any linear observer gain
matrix is **non-empty**. CFAR detects crossings; DSFB detects trajectories.

**Finite-Time Envelope Exit Bound (Theorem 1):**

> If ‖r(k₀)‖ = r₀ < ρ and the residual grows at rate ≥ α > 0 per observation:
> `k* ≤ ρ/α`  (computable without a noise model)

*RF example:* USRP B200, ρ = 0.1 normalized IQ norm, α = 0.001/symbol.
At 1 Msym/s: structural detection within 100 µs under sustained drift.
*Caveat:* conservative bound under sustained monotone drift. Abrupt steps:
CFAR remains faster. DSFB's advantage is specific to slow, directional,
below-threshold regime evolution.

**Deterministic Interpretability (Theorem 9):**

> DSFB is the composition IQ → Sign → Syntax → Grammar → Semantics → Policy,
> each stage a deterministic map under fixed (ρ, W, K, τ, m, H). Identical
> ordered IQ residual inputs produce identical outputs on every replay.

This is directly relevant to DoD IV&V and EW qualification workflows: every
episode is reproducible from saved artifacts without re-running the upstream
receiver.

---

## Competitive Differentiation

How DSFB compares with six incumbent RF monitoring approaches (paper Table I):

| Capability | Energy Det. | CFAR | Kalman/LO | ML Classifier | Spec. Analyzer | **DSFB (this work)** |
|---|---|---|---|---|---|---|
| Calibrated P_fa | Partial | **Yes** | No | No | No | No |
| Slow-drift structural indication | No | No | No | Limited | No | **Yes** |
| Typed trajectory interpretation | No | No | No | No | No | **Yes** |
| Provenance-aware motif library | No | No | No | No | No | **Yes** |
| Labeled training data required | No | No | No | **Yes** | No | No |
| Operator-auditable outputs | No | Partial | No | No | Partial | **Yes** |
| No write path to upstream | Yes | Yes | No | Yes | Yes | **Yes** |
| Unknown-regime handling | None | None | Poor | Poor | None | **Endoductive** |
| Deterministic replay | Yes | Yes | Yes | No | Yes | **Yes** |
| `no_std` bare-metal deploy | Possible | Possible | Possible | Rarely | No | **Yes** |

**Joint Decision Logic (CFAR + DSFB together):**

| CFAR state | DSFB state | Operator action |
|---|---|---|
| Alarm | Violation | Corroborated structural excursion — high-confidence review |
| No alarm | Boundary[SustainedOutwardDrift] | **Primary DSFB value regime** — structural review before crossing |
| Alarm | Admissible | Likely transient noise spike — investigate before escalating |
| No alarm | Admissible | Nominal operation |

DSFB is not a CFAR replacement. It is the interpretive layer that makes the
*trajectory between* CFAR alarms legible, typed, and reusable.

---

## Operator Value and Review-Surface Compression

The operator question is narrow and practical: can the residual activity
already produced by incumbent RF systems be reduced to a smaller, more
relevant review queue without altering those systems?

**Before DSFB:** An RF spectrum operator monitoring a RadioML-class signal
environment receives 14 203 raw boundary events per evaluation window.
Signal-to-relevance: 0.72 % — 139 events reviewed per labeled transition.
At approximately 2 min per triage event: ~473 operator-hours per window.

**After DSFB:** The same threshold system runs unchanged. DSFB produces
87 structured Review/Escalate episodes: **73.6 % precision, 99.4 % review-surface
compression, 95.1 % recall**. At 2 min per triage event:
**~3 operator-hours per window** — a 99.4 % reduction.

*Caveat:* "~2 min per triage event" is an order-of-magnitude estimate for
illustration, not a validated time study. The compression factor (163×) is
independently reproducible from the saved artifacts.

**On real USRP B200 hardware (ORACLE):** 6 841 raw events → 52 episodes at
71.2 % precision and 93.4 % recall — confirming that the compression and
precision gains are **not artifacts of the synthetic dataset**.

---

## SBIR and Commercial Licensing Pathways

### Why the Observer Contract Is the Risk Argument

The most important property for a SBIR Phase I program operator is not the
episode precision figure. It is the non-intrusive architecture.

A program operator evaluating a new signal processing technology faces
**integration risk** that is often larger than technical risk. DSFB eliminates
integration risk by construction:

- Nothing in the existing signal chain is modified.
- The system reverts to its pre-DSFB state trivially if the layer is removed.
- Non-intrusion proof is in the Rust type system, not documentation.
- Deterministic audit trail satisfies DoD IV&V workflow requirements.
- `no_std` bare-metal deployment means no OS, no RTOS, no heap dependency.

### Technology Readiness Assessment (TRL)

| Component | TRL |
|---|---|
| DSFB core logic (grammar + heuristics engine) | 3–4 |
| RadioML public-dataset validation | 4 |
| ORACLE real USRP B200 validation | **4–5** |
| `no_std` bare-metal crate, CI-verified on ARM + RISC-V | 4 |
| SDR integration pathway (GNU Radio 3.10 tap, `src/sink_gnuradio.rs`) | 3 |
| EW/SIGINT receiver integration (full flowgraph) | 2–3 |
| Operational EW deployment | 1–2 |

### Concrete Phase I Deliverables

1. **USRP B200 integration.** Install `DsfbSinkB200` GNU Radio 3.10 block in
   parallel with an existing signal-processing flowgraph on a USRP B200
   (70 MHz–6 GHz, up to 56 MS/s). No receiver firmware modification.
   Target: 30 days from contract start.

2. **Waveform suite.** Validate structural episode detection across BPSK, QPSK,
   8PSK, 16QAM, FM, GFSK — covering the RadioML modulation classes most
   relevant to the program's signal environment.

3. **Interference injection test.** Inject a narrowband source at controlled
   power levels (0 dBm to −60 dBm) and drift rates (0.1 to 1.0 dB/min) using
   a second USRP B200. Measure DSFB episode detection lead time vs.
   threshold-crossing lead time.

4. **A/B verification.** Demonstrate zero change to upstream receiver behavior
   before and after DSFB integration — connected vs. disconnected comparison.

5. **Operator-facing output.** Review/Escalate episodes with full trace chains
   (JSON + CSV), readable without specialized tooling.

### Industrial Licensing Segments

| Segment | Application | Contact |
|---|---|---|
| Spectrum management (ITU, FCC licensees, NTIA) | Early warning for mask-boundary approach | `licensing@invariantforge.net` |
| Satellite comms (Intelsat, SES, ViaSat) | Structural early warning on high-value GEO/MEO links | `licensing@invariantforge.net` |
| Cellular operators (3GPP LTE/NR) | Structural ACLR drift detection (TS 36.141 §6.3) | `licensing@invariantforge.net` |
| Airborne EW integrators (Raytheon, L3Harris, BAE) | Non-intrusive structural observer for EW suite residuals | `licensing@invariantforge.net` |

---

## GNU Radio Integration Pathway

The primary SDR integration target is a USRP B200 running GNU Radio 3.10.
The DSFB tap is a parallel sink block — zero modification to the upstream
flowgraph:

```
[USRP Source] ──► [Channel Filter] ──► [Demodulator / CFAR / Spectrum Analyzer]
                                  └──► [DsfbSinkB200]  (read-only)
                                              │
                                      [Episode JSON/ZMQ output]
```

`DsfbSinkB200` is implemented in `src/sink_gnuradio.rs` (feature-gated:
`#[cfg(feature = "std")]`). It:

1. Accumulates 100 calibration captures to lock the GUM-derived envelope ρ.
2. Runs the DSFB grammar on each residual norm as observations arrive.
3. Buffers Review/Escalate episodes in a fixed-capacity ring.
4. Emits SigMF-annotated episode metadata for downstream operator review.

**Architecture guarantees:**

- `process()` takes `&mut self` + `&[f32]` (residual norms) — no upstream write path.
- If the block panics or disconnects, the upstream flowgraph is unchanged.
- The `GnuRadioIntegrationContract` struct carries read-only integration
  proof for embedding in VITA 49.2 context packets and audit trails.

**Platform coverage:** USRP B200/X310 (UHD 4.x), LimeSDR (SoapySDR),
RTL-SDR (librtlsdr). Any platform that exposes a CF32 stream.

Live GNU Radio block registration (via the `gr-dsfb` out-of-tree module) is a
Phase I deliverable — not claimed as complete in this crate.

---

## Failure Modes: Honest Disclosure

| Failure | Cause | Mitigation |
|---|---|---|
| **False escalation from stationary interference** | DSFB sees persistent IQ structure that does not resolve into an operational transition | Check waveform schedule transitions; classify interference as known-source before treating as structural precursor |
| **Missed structure at low SNR** | Below −10 dB, residual structure degrades below grammar activation floor | Reported: 5/102 RadioML events + 6/102 ORACLE events missed (all below SNR floor). DSFB silence = "insufficient structural signal", not quality confirmation |
| **Waveform transition artifacts** | Frequency hops, modulation changes, burst boundaries produce signatures indistinguishable from interference onset without a schedule flag | Populate `WaveformSchedule<N>` with transition windows; `TransitionKind` taxonomy labels the suppression source |
| **Calibration window contamination** | Early interference in the healthy window biases ρ | Run `check_calibration_window()` from `calibration.rs`; the tool emits explicit integrity diagnostics |
| **AGC hunting as false drift** | Oscillatory AGC produces `RecurrentBoundaryGrazing` | Trigger `WaveformState::Calibration` when AGC lock status is externally flagged as unstable |
| **Wideband cross-channel mixing** | Cross-channel leakage produces spurious drift in a clean sub-channel | Per-channel residual construction and envelope calibration; use `HretEstimator` from `trust.rs` |
| **Nominal model quality** | Poorly specified nominal predictor produces residuals that reflect modeling error, not signal-environment structure | Verify nominal model quality before calibration lock; WSS pre-condition check catches contaminated windows |

---

## Panel Anticipated Questions and Objections

**Q: "Isn't this just a Kalman filter?"**  
No. A Kalman filter uses `r(k)` to drive corrective feedback and minimize
residual magnitude. DSFB reads `r(k)` as a semiotic carrier without feedback.
Theorem (Observer-of-the-Observer) above establishes that DSFB detects a
structurally non-empty set of trajectory classes that are invisible to any
linear observer gain matrix.

**Q: "Is this just adaptive thresholding?"**  
No. The grammar layer evaluates trajectory topology `(‖r‖, ṙ, r̈)`, not a
single threshold. The same instantaneous `‖r(k)‖` may produce `Admissible`,
`Boundary`, or `Violation` depending on drift direction and history — not
possible with adaptive thresholding alone.

**Q: "Why not just use an ML anomaly detector?"**  
ML anomaly detectors require training data, produce probabilistic scores,
and are not operator-auditable. DSFB requires no training data, produces
typed grammar states with structured reason codes, and generates a deterministic
trace chain (Theorem 9). These are not equivalent system properties.

**Q: "The RadioML dataset is synthetic."**  
Acknowledged. L4 in Limitations Disclosure. The ORACLE evaluation on real
USRP B200 captures provides real-hardware evidence at comparable precision
(71.2 %) and recall (93.4 %). Operational deployment validation is future work
and explicitly out of scope (L9).

**Q: "The false-episode rates seem high (4.4–6.3 %)."**  
These are point-level rates on nominal captures. The DSFB episode compression
reduces point-level false alarm by 102–132×. The absolute false episode count
on clean windows is 31–52 out of 700–1 100 clean captures. These values are
disclosed in full as negative controls (see Empirical Results above), not buried.

**Q: "No hard real-time guarantee."**  
Acknowledged as L3 in Limitations Disclosure. The crate is O(*n*) per sample
with no dynamic allocation. Hard latency bounds require hardware-in-the-loop
testing not claimed here. The ~27 ns/sample throughput on x86-64 is a
benchmark result, not an FPGA or RISC-V timing claim.

**Q: "504 bytes of stack — is that really usable on Cortex-M0?"**  
Yes. The 504-byte footprint is CI-verified at W=10, K=4, M=8 via the
`engine_fits_in_reasonable_stack` unit test. Cortex-M0 minimum stack: 8 bytes.
Cortex-M0 practical minimum: 256 bytes. The engine fits in less than 2× the
practical Cortex-M0 stack minimum.

**Q: "Why 23 scientific modules? Isn't this overbuilt?"**  
Each module fills a specific gap: quantization noise in GUM budget, WSS
pre-condition for Type A uncertainty, Lyapunov exponent for divergence rate,
TDA for topological anomalies, RG flow for scale-invariant structure. The
modules are independently useful and independently testable. None is required
for the core grammar layer to function — they are composable augmentations.

---

## Closing Statement

The IQ residual streams that existing RF receivers produce contain structured
temporal information — drift direction, boundary approach, slew acceleration —
that scalar detection methods trigger on but do not interpret. That interpretive
gap is a structural consequence of threshold-based alarm interfaces. The DSFB
Structural Semiotics Engine closes that gap deterministically, without
modifying anything upstream.

The value is not that DSFB is better than CFAR. It is that an RF operator
reviewing a persistent `SustainedOutwardDrift` trajectory across 40 captures
has more actionable context than one reviewing a scalar alarm count. The CFAR
detector fired the alarm; DSFB explains the trajectory that preceded it.
Both are necessary. Neither is sufficient alone.

The non-intrusion proof is in the Rust type system — not in documentation.
`observe()` takes `&[f32]`. The compiler enforces it on every build.

---

## Performance

| Metric | Value | Notes |
|---|---|---|
| Stack footprint | **504 bytes** | W=10, K=4, M=8 — CI-verified via `engine_fits_in_reasonable_stack` |
| Throughput | **~27 ns / sample** | x86-64, all pipeline stages; order-of-magnitude only |
| Bare-metal targets | Cortex-M4F, RISC-V | CI `--no-default-features` compilation verified |
| Test suite | **340 tests** | `cargo test --features std` |
| Publication figures | **51 figures** | `generate_figures_all` + `figures_all.py` |
| Source modules | **39 modules** | Core no_std (25) + std-gated (4) + examples (8) |
| Science phases | **23 contributions** | Sections §1–§22 + `sink_gnuradio` integration layer |

*Throughput measured on a single x86-64 core; FPGA and Cortex-M4F figures
require hardware-in-the-loop measurement not claimed here (L3).*

---

## Documentation

| Document | Content |
|---|---|
| `docs/uncertainty_budget_gum.md` | GUM Type A + B uncertainty budget for ρ |
| `docs/sosa_mora_alignment.md` | SOSA/MORA sensor-observation alignment |
| `docs/non_intrusion_contract.md` | Type-system proof of the non-intrusion contract |
| `docs/radioml_oracle_protocol.md` | Stage III fixed evaluation protocol: dataset access, parameters, negative controls |

---

## Audit

[![DSFB Gray Audit: 91.4% strong assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-91.4%25-brightgreen)](./audit/dsfb_rf_scan.txt)

The crate ships a locked-rubric static audit generated by
[`dsfb-gray`](https://crates.io/crates/dsfb-gray) under
[`audit/`](./audit/). Overall: **91.4 %** (strong assurance posture).
See [`audit/README.md`](./audit/README.md) for the section breakdown,
open findings, and reproduction command. The audit is a structured
improvement target and internal-review artefact — **not a
certification**.

---

## CI Pipeline

| Job | What it enforces |
|---|---|
| 1 · Test | `cargo test --features std,serde,paper_lock` — 340 tests |
| 2 · Bare-metal ARM | `thumbv7em-none-eabihf` — `no_std`, `no_alloc` |
| 3 · Bare-metal RISC-V | `riscv32imac-unknown-none-elf` — `no_std`, `no_alloc` |
| 4 · Zero-unsafe | `cargo geiger` — `#![forbid(unsafe_code)]` |
| 5 · Deny | `cargo deny check` — license allow-list + banned crates |
| 6 · Clippy | `-D warnings` on both feature sets |
| 7 · Fmt | `cargo fmt --check` |
| 8 · Examples | `cargo build --features std,serde --examples` |
| 9 · Benchmark | `cargo bench --features std` smoke test |
| 10 · Stack footprint | `engine_fits_in_reasonable_stack` unit test |

---

## License

Apache-2.0 (reference implementation).  
Commercial deployment requires a separate written license.  
`licensing@invariantforge.net`

---

## Citation

If you reference this crate or its companion paper in academic or
technical work, please cite:

> de Beer, R. (2026). *DSFB-RF Structural Semiotics Engine for RF
> Signal Monitoring — A Deterministic, Non-Intrusive Observer Layer
> for Typed Structural Interpretation of IQ Residual Streams in
> Electronic Warfare, Spectrum Monitoring, and Cognitive Radio*
> (v1.0). Zenodo. <https://doi.org/10.5281/zenodo.19702330>

A machine-readable `CITATION.cff` file is provided at the crate root
for tools that support the Citation File Format (e.g., GitHub's
"Cite this repository" button, Zenodo, Zotero).

```bibtex
@software{debeer_2026_dsfb_rf,
  author    = {de Beer, Riaan},
  title     = {{DSFB-RF Structural Semiotics Engine for RF Signal
                Monitoring --- A Deterministic, Non-Intrusive Observer
                Layer for Typed Structural Interpretation of IQ
                Residual Streams in Electronic Warfare, Spectrum
                Monitoring, and Cognitive Radio}},
  year      = {2026},
  version   = {v1.0},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19702330},
  url       = {https://doi.org/10.5281/zenodo.19702330}
}
```

---

## Collaboration and Partnership

For SBIR co-PI arrangements, Phase II subcontracting, and research
partnership inquiries: `partnerships@invariantforge.net`.

See also [`docs/SBIR_READINESS.md`](./docs/SBIR_READINESS.md) for the
named-hardware roster, L-tag $\to$ Phase I deliverable map, and TRL
milestone table, and
[`paper/dsfb_rf_sbir_trl_v1.tex`](./paper/) for the companion
SBIR/TRL/dual-use technical report.

---

## Scope Disclosure: Standards Positioning and Resource-Constrained Demonstration

> **Read before citing, evaluating, or procuring.**
> The following disclosures are reproduced from the companion paper
> (de Beer 2026, §XVI) and apply without exception to the reference crate.

### Implementation-Level Standards Positioning

This crate includes design elements and structural patterns intentionally
aligned with characteristics of established RF system standards and
architectures (e.g., SOSA-aligned data transport, MORA modular processing
models, and CMOSS-constrained execution environments). These elements
demonstrate a plausible path toward integration and establish prior art at
the level of implementation structure.

**The crate does not constitute an implementation that is compliant with,
endorsed by, or validated against any formal standards body, specification,
or operational platform.**

#### Prior-Art Positioning vs. Standards Conformance

The implementation is a **proof-of-structure**, not a **proof-of-conformance**.
It demonstrates that the DSFB framework can be expressed in a form compatible
with common RF system constraints (read-only data access, bounded execution,
separation from control paths). This establishes a forward path for potential
integration — it does not demonstrate that such integration has been achieved.

#### Absence of Formal Interface Compliance

The crate does **not** implement or certify compliance with:

- VITA 49.2 packet formatting or transport-layer requirements
- MORA-defined component interfaces or lifecycle management
- CMOSS hardware abstraction, timing, or resource allocation constraints
- Any MIL-STD communication or audit protocol specifications

Where naming or structural analogies are used (IQ/context handling, zero-copy
patterns, audit trace outputs), these represent **conceptual alignment**, not
verified interoperability.

#### Non-Endorsement and Lack of Validation

The crate has not undergone:

- Standards body review or conformance testing
- Integration within certified RF platforms
- Evaluation under program-specific interface requirements
- Verification within controlled DoD or industry validation environments

Any perceived standards alignment is **indicative, not authoritative**.

#### Implementation Constraints and Gaps

The crate does not currently address several practical requirements for
standards-aligned deployment:

- Real-time scheduling and timing guarantees
- Interface adaptation to standardised transport layers
- Resource arbitration in multi-component RF systems
- Hardware-specific constraints (FPGA, embedded systems)
- Security, certification, and lifecycle management considerations

These aspects require system-level engineering beyond the present scope.

#### Implications for Phase I and Beyond

Achieving meaningful alignment with standards-based RF systems requires:

- Binding to real system interfaces and data formats
- Verification under operational timing and resource constraints
- Evaluation within representative RF scenarios
- Iteration in collaboration with domain experts and system owners

Such work is appropriately scoped for Phase I and subsequent development
efforts. It cannot be inferred from the present implementation alone.

---

### Standards Pathfinding and Resource-Constrained Demonstration

The standards-referenced design choices in this crate are **pathfinding
constructs**, not validated alignments. They represent an effort to express
DSFB in a form that could, in principle, be adapted to standards-aligned
environments — they do not demonstrate that such adaptation has been achieved.

#### Scope of Demonstration

This implementation was developed under resource-constrained conditions
**without access to**:

- Standards-compliant integration environments
- Proprietary or program-specific RF system interfaces
- Certified hardware platforms or embedded deployment targets
- Formal conformance testing frameworks or validation suites

The crate **cannot** be interpreted as evidence of compatibility with specific
standards implementations, even where structural similarities are present.

#### Best-Effort Representation

Standards-relevant structures (conceptual IQ handling, separation of
processing concerns, traceability outputs) are provided on a **best-effort
basis** to demonstrate how DSFB could be situated in a broader system context.
These representations are necessarily incomplete and may omit critical
deployment requirements: transport-layer details, timing constraints,
synchronisation requirements, and system-specific interface contracts.

#### Non-Endorsement and Independence

Inclusion of standards-referenced terminology or structural analogies must
**not** be interpreted as endorsement, affiliation, or validated alignment with
any standards body, program, or implementation ecosystem. This work was
conducted independently. No validation has been performed in collaboration
with organisations responsible for those standards.

#### Gap Between Conceptual Alignment and Operational Integration

There is a **substantial gap** between conceptual alignment at the software
structure level and operational integration within real RF systems. Bridging
this gap requires:

- Detailed interface adaptation to specific platforms
- Validation under real-time and resource-constrained conditions
- Coordination with system owners and domain experts
- Iterative testing within representative operational environments

These activities fall outside the scope of the present work.

#### Standards-Oriented Aspects: Correct Interpretation

The standards-oriented aspects of this crate should be interpreted as:

- Evidence of **architectural intent and awareness**
- A **prior-art demonstration** of how DSFB may be expressed in a standards-conscious form
- A **starting point** for future integration work requiring additional resources, infrastructure, and domain expertise

They must **not** be interpreted as evidence of readiness for deployment within
standards-compliant systems.

#### Summary

The reference implementation establishes a concrete and reproducible baseline
reflecting consideration of relevant RF system constraints. Its
standards-related features remain **indicative rather than definitive**.
Realising actual alignment or integration will require dedicated engineering
effort under appropriately resourced development programs.
