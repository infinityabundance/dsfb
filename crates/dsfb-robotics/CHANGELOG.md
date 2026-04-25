# Changelog

All notable changes to `dsfb-robotics` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 2 — Core API (in progress)

Foundational `no_std` + `no_alloc` modules are live:

- **`math.rs`** — `libm`-free f64 helpers: `abs_f64`, `sqrt_f64` via
  bounded Newton-Raphson (64-iteration upper bound, JPL Power-of-Ten
  Rule 2 compliant), `finite_mean`, `finite_variance`, `clamp_f64`.
- **`platform.rs`** — `RobotContext` enum:
  `ArmCommissioning | ArmOperating | LeggedStance | LeggedSwing |
  Maintenance`. `admissibility_multiplier()` returns `+∞` during
  suppressed contexts so violations cannot be raised during
  identification or maintenance.
- **`sign.rs`** — `SignTuple { norm, drift, slew }` with missingness-
  aware `SignWindow<const W: usize>` circular buffer.
- **`envelope.rs`** — `AdmissibilityEnvelope` with `ρ = μ + 3σ`
  calibration, `boundary_frac = 0.5` default, commissioning-suppressed
  violations via `f64::INFINITY` multiplier.
- **`grammar.rs`** — three-state FSM (`Admissible | Boundary[ReasonCode]
  | Violation`) with 2-confirmation hysteresis, `ReasonCode` qualifier
  (`SustainedOutwardDrift | AbruptSlewViolation |
  RecurrentBoundaryGrazing | EnvelopeViolation`), and `K`-long grazing
  history buffer.
- **`episode.rs`** — canonical `Episode` struct (byte-identical fields
  to `dsfb-semiconductor::Episode`).
- **`policy.rs`** — `PolicyDecision { Silent | Review | Escalate }` with
  deterministic grammar → decision mapping.
- **`calibration.rs`** — healthy-window calibration with
  coefficient-of-variation gate to reject fault-contaminated windows.
- **`stationarity.rs`** — two-half WSS check for calibration windows.
- **`uncertainty.rs`** — GUM JCGM 100:2008 Type A/B uncertainty budget
  with quadrature combination and expanded-uncertainty coverage factor.
- **`heuristics.rs`** — typed `RoboticsMotif` enum (StribeckGap,
  BacklashRing, BpfiGrowth, GrfDesync, MpcStanceLag, CoMDrift, Unknown).
- **`syntax.rs`** — minimal grammar-state + sign-tuple → motif
  classifier (full pattern recognition lands in Phase 3).
- **`kinematics.rs`** — shared `joint_residual_norm` and
  `tau_residual_norm` helpers for KUKA / Panda / DLR / UR10 adapters;
  `TorqueSensorSide { Link | Motor }` for the sensing-side contrast.
- **`balancing.rs`** — dual-channel combiner for MPC force residual
  `r_F` and centroidal-momentum residual `r_ξ`; `BalancingCombine`
  strategy enum (`SumOfSquares`, `WeightedSum`).
- **`engine.rs`** — streaming `DsfbRoboticsEngine<W, K>` const-generic
  orchestrator. `observe(&[f64], &mut [Episode], RobotContext) -> usize`
  is the no-alloc bulk API; `observe_one` is the per-sample form.
- **`lib.rs`** — top-level `observe(&[f64], &mut [Episode]) -> usize`
  convenience wrapper with Stage III calibration from the first 20 %
  of the input.
- **Acceptance:** 97 tests pass on `--no-default-features`,
  `--features std,serde`, and `--features std,paper_lock`; `cargo
  clippy -D warnings` clean; zero `.unwrap()` / `.expect()` /
  `panic!` / `todo!` / `unimplemented!` in production code; zero
  `unsafe` anywhere (enforced at crate root); no external-tool
  attributions in any file.

### Phase 1 — Crate scaffold (2026-04-24)

- Initialised crate at `crates/dsfb-robotics/` as a standalone Cargo
  project (empty `[workspace]` escape-hatch, mirroring `dsfb-rf`) so it
  can be published independently.
- Added `Cargo.toml` with feature matrix `alloc` / `std` / `serde` /
  `paper_lock` / `real_figures` / `experimental`, matching the
  `dsfb-rf` conventions. Direct dependency count is kept minimal
  (serde, serde_json, csv — all optional) to stay under the dsfb-gray
  "direct deps ≤ 10" threshold.
- Pinned toolchain to `rustc 1.85.1` via `rust-toolchain.toml` with
  cross-targets for `thumbv7em-none-eabihf`, `riscv32imac-unknown-none-elf`,
  and `x86_64-unknown-linux-gnu` so the no_std core is buildable for
  the two representative embedded platforms alongside host development.
- Added `deny.toml` with the same licence allowlist as sibling DSFB
  crates (Apache-2.0 / MIT / BSD-2-Clause / BSD-3-Clause / ISC /
  Unicode-DFS-2016) and explicit deny of the GPL / LGPL / AGPL family.
- Added `LICENSE` (Apache-2.0), `NOTICE` (Invariant Forge LLC IP notice
  plus dataset-attribution clause), and `CITATION.cff`.
- Added `.gitignore` with entries for LaTeX intermediates, the
  `data/raw/` and per-dataset full-corpus directories (never
  committed — only manifests + stratified slices are), Python venvs,
  and the local-only working-doc convention (`AGENTS.md` /
  `CONVENTIONS.md` excluded from the git tree per project rule).
- Initialised `src/lib.rs` with the full set of crate-root attributes
  — `#![no_std]`, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`,
  `#![deny(clippy::all)]` — and declared the canonical `Episode`
  struct with fields byte-identical to `dsfb-semiconductor`. The
  Phase 1 `observe` is a placeholder that returns zero episodes; the
  full grammar FSM lands in Phase 2.
- Added four Phase 1 smoke tests covering the empty-input case, the
  non-empty Phase 1 placeholder case, the `Episode::empty()` default
  invariants, and the zero-capacity output-buffer edge case. All four
  pass under `cargo test --no-default-features --lib`.
- Added `src/main.rs` as a feature-gated `paper-lock` binary stub so
  `cargo check --features std,paper_lock` succeeds. Full subcommand
  dispatch lands in Phase 4.
- Verified the feature matrix compiles cleanly on Phase 1 scaffold:
  `cargo check --no-default-features`, `--features alloc`,
  `--features std`, `--features std,serde`, and
  `--features std,paper_lock` all succeed.

## [0.1.0] — Upcoming

Initial public release. Version-bumping policy from here on is:

- `0.1.x` for Phase 1–3 (scaffold, core API, dataset adapters).
- `0.2.x` for Phase 4–5 (paper-lock + figures + Colab).
- `0.3.x` for Phase 6–7 (audit stack + docs completeness).
- `0.4.x` for Phase 8 (paper patches; no code changes).
- `1.0.0` for Phase 9 (dsfb-gray audit ≥95%, Miri + Kani + fuzz all
  clean, paper PDF rebuilt, ready for Zenodo deposit).

Breaking changes to the public `observe()` signature between `0.x`
releases will be flagged in this changelog and accompanied by a
migration note.
