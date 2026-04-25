# Architecture

## One-paragraph summary

`dsfb-robotics` is a deterministic `no_std` + `no_alloc` + zero-`unsafe`
observer that reads residual streams produced by incumbent robotics
pipelines — kinematic identification, whole-body control, prognostics
health monitoring — and structures them into a three-state grammar
(`Admissible` / `Boundary` / `Violation`) with typed episodes. The
core is a pure function: `observe(residuals: &[f64], out: &mut [Episode]) -> usize`.
The caller owns the output buffer, so the observer never allocates.
The upstream robotics pipeline is untouched by construction (the
input is a shared reference, not mutable).

## Module tree (target end-state, Phase 2+)

```
src/
├── lib.rs           crate root: #![no_std] #![forbid(unsafe_code)]; public observe()
├── main.rs          paper-lock binary (feature-gated on paper_lock)
├── math.rs          libm-free f64 helpers: sign, abs, hypot2 (squared), windowed mean/var
├── sign.rs          residual-sign tuple σ(k) = (‖r‖², ṙ, r̈) — the semiotic manifold coordinate
├── envelope.rs      AdmissibilityEnvelope { rho, W, K, tau, m, beta } — E(k) = { r : ‖r‖ ≤ ρ(k) }
├── grammar.rs       three-state FSM + ReasonCode: Admissible | Boundary[reason] | Violation
├── syntax.rs        classify sign tuples into named temporal motifs
├── heuristics.rs    robotics motifs: stribeck_gap, backlash_ring, bpfi_growth, grf_desync, mpc_stance_lag, com_drift
├── kinematics.rs    shared IdentificationResidual helper for KUKA / Panda / DLR / UR10 adapters
├── balancing.rs     shared BalancingResidual helper for Cheetah / iCub adapters
├── engine.rs        const-generic DsfbRoboticsEngine<W, K, M>; observe() orchestration
├── episode.rs       canonical Episode struct (see §Canonical API)
├── platform.rs      RobotContext: ArmCommissioning | ArmOperating | LeggedStance | LeggedSwing
├── calibration.rs   healthy-window calibration + envelope radius selection
├── stationarity.rs  WSS check (Wiener-Khinchin pre-condition on calibration window)
├── uncertainty.rs   GUM JCGM 100:2008 Type A/B uncertainty budget
├── policy.rs        Silent | Review | Escalate decision ranking
├── fixedpoint.rs    Q16.16 deterministic ingress (optional; for exact reproducibility)
├── standards.rs     mapping to ROS 2 diagnostic_msgs, OPC UA Robotics 40010, ISO 10218, ISO 13849, IEC 61508
├── audit.rs         continuous-rigor audit emitter (std-gated)
├── output.rs        JSON / traceability emitters (std-gated)
├── paper_lock.rs    headline-metric enforcement + tolerance gate
├── pipeline.rs      Stage III 3-pass ingest (calibration → observation → aggregation)
├── datasets/
│   ├── mod.rs
│   ├── cwru.rs                (PHM, bearing envelope-spectrum residual)
│   ├── ims.rs                 (PHM, run-to-failure HI trajectory)
│   ├── cmapss.rs              (regime drift, cross-domain analogue)
│   ├── kuka_lwr.rs            (kinematics, link-side ID torque residual)
│   ├── femto_st.rs            (PHM, accelerated-aging vibration HI)
│   ├── panda_gaz.rs           (kinematics, motor-side ID torque residual)
│   ├── dlr_justin.rs          (kinematics, link-side ID torque residual)
│   ├── ur10_kufieta.rs        (kinematics, motor-side ID torque residual)
│   ├── cheetah3.rs            (balancing, MPC force residual + CoM-obs residual)
│   └── icub_pushrecovery.rs   (balancing, contact-wrench + centroidal-momentum residual)
└── kani_proofs.rs   Kani harnesses: FSM totality, τ-bound monotonicity, buffer-bound safety
```

## Data flow

```
  ┌────────────────────────────┐                     ┌──────────────────┐
  │  upstream robotics stack   │   residual stream   │   dsfb-robotics  │
  │  (ID, MPC, observer, ...)  │ ───────────────────▶│    observe()     │
  └────────────────────────────┘   &[f64]            └────────┬─────────┘
                                                              │
                                                              │ episodes
                                                              ▼
                                                ┌──────────────────────────────┐
                                                │  operator review surface     │
                                                │  (dashboard, audit log, etc.)│
                                                └──────────────────────────────┘
```

No arrow returns from the observer to the upstream stack. The relation
is one-directional by construction.

## Canonical API

```rust
pub struct Episode {
    pub index: usize,
    pub residual_norm_sq: f64,
    pub drift: f64,
    pub grammar: &'static str,   // "Admissible" | "Boundary" | "Violation"
    pub decision: &'static str,  // "Silent" | "Review" | "Escalate"
}

pub fn observe(residuals: &[f64], out: &mut [Episode]) -> usize;
```

Field shape is byte-identical to `dsfb-semiconductor::Episode` so the
DSFB episode stream can be consumed uniformly across crates (robotics,
semiconductor, RF, battery, oil-gas, turbine).

## Const-generic capacity (Phase 2)

The full engine type is `DsfbRoboticsEngine<const W: usize, const K: usize, const M: usize>`:

- `W` — drift-window length (samples over which `ṙ` is estimated).
- `K` — persistence threshold (samples in `Boundary` before `Violation`).
- `M` — heuristics-bank capacity (typed motif slots).

Per-scan episode output is bounded by `W + K + 2` (see the grammar FSM
in `dsfb-rf`'s `grammar.rs` for the identical bound). The caller's
output buffer is sized from this formula.

## Feature gating

| Feature | Adds | Depends on |
|---|---|---|
| *(none)* | Core FSM + envelope + heuristics + `observe()` | — |
| `alloc` | `Vec<Episode>`-returning `observe_vec` wrapper | — |
| `std` | `pipeline`, `output`, `audit` (host-side tooling) | `alloc` |
| `serde` | JSON / manifest emission | `std` |
| `paper_lock` | Headline-metric enforcement, tolerance gate | `std`, `serde` |
| `real_figures` | Real-dataset figure bank (CSV readers for slices) | `std`, `serde` |
| `experimental` | Exploratory extensions not in the paper | — |

## Cross-crate conventions

| Concept | Canonical source | Mirrored in robotics as |
|---|---|---|
| `Episode` struct | `dsfb-semiconductor` | `dsfb_robotics::Episode` |
| Grammar FSM | `dsfb-rf::grammar` | `dsfb_robotics::grammar` |
| `AdmissibilityEnvelope` | `dsfb-rf::envelope` | `dsfb_robotics::envelope` |
| Const-generic engine | `dsfb-rf::engine::DsfbRfEngine` | `dsfb_robotics::engine::DsfbRoboticsEngine` |
| Feature matrix | `dsfb-rf::Cargo.toml` | `dsfb-robotics::Cargo.toml` (1:1) |
| Paper-lock pattern | `dsfb-rf::main.rs` | `dsfb-robotics::main.rs` (Phase 4) |
| Miri audit layout | `dsfb-rf/audit/miri/` | `dsfb-robotics/audit/miri/` (Phase 6) |

## Phase roadmap

Implementation is organised into nine phases; see [`CHANGELOG.md`](CHANGELOG.md)
for the authoritative status log and per-phase acceptance criteria.
Current status: **Phase 1 (scaffold) complete**; **Phase 2 (core API)
in progress** — foundational modules (`math`, `platform`, `sign`,
`envelope`, `grammar`, `episode`, `policy`, `calibration`, `engine`)
are live and under test; residual-source helpers (`kinematics`,
`balancing`, `heuristics`, `syntax`, `stationarity`, `uncertainty`)
are minimal-viable and will be expanded alongside the dataset
adapters in Phase 3.
